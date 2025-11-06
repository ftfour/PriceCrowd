use axum::{extract::{Query, State}, http::StatusCode, response::IntoResponse, routing::get, Json, Router};
use once_cell::sync::Lazy;
use serde::Deserialize;
use std::collections::HashMap;

use crate::state::AppState;

static FNS_TOKEN: Lazy<String> = Lazy::new(|| {
    std::env::var("FNS_TOKEN").unwrap_or_default()
});

static FNS_BASE_URL: Lazy<String> = Lazy::new(|| {
    // Default to root domain to avoid TLS hostname mismatch on some setups
    std::env::var("FNS_BASE_URL").unwrap_or_else(|_| "https://proverkacheka.com/api/v1/check/get".to_string())
});

static FNS_PROMO_ID: Lazy<Option<i64>> = Lazy::new(|| {
    std::env::var("FNS_PROMO_ID").ok().and_then(|v| v.parse::<i64>().ok())
});

#[derive(Debug, Deserialize)]
pub struct CheckParams {
    pub qr: Option<String>,
    pub t: Option<String>,
    pub s: Option<String>,
    #[serde(rename = "fn")]
    pub fnc: Option<String>,
    pub i: Option<String>,
    pub fp: Option<String>,
    pub n: Option<String>,
}

pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/check", get(get_check))
        .with_state(state)
}

pub async fn get_check(Query(query): Query<CheckParams>, _state: State<AppState>) -> impl IntoResponse {
    if FNS_TOKEN.is_empty() {
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error":"FNS_TOKEN is not set"}))).into_response();
    }

    // Build parameters from either full qr string or individual params
    let mut params: HashMap<String, String> = HashMap::new();
    if let Some(qr) = query.qr.as_deref() {
        for pair in qr.trim().trim_start_matches('?').split('&') {
            if let Some((k, v)) = pair.split_once('=') {
                if !k.is_empty() && !v.is_empty() {
                    params.insert(k.to_string(), v.to_string());
                }
            }
        }
    }
    if let Some(v) = query.t { params.entry("t".into()).or_insert(v); }
    if let Some(v) = query.s { params.entry("s".into()).or_insert(v); }
    if let Some(v) = query.fnc { params.entry("fn".into()).or_insert(v); }
    if let Some(v) = query.i { params.entry("i".into()).or_insert(v); }
    if let Some(v) = query.fp { params.entry("fp".into()).or_insert(v); }
    if let Some(v) = query.n { params.entry("n".into()).or_insert(v); }

    // Minimal validation
    if !params.contains_key("t") || !params.contains_key("fn") || !params.contains_key("i") || !params.contains_key("fp") {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error":"missing required params"}))).into_response();
    }

    // Perform upstream request per spec (POST with token). Prefer qrraw when available.
    let client = reqwest::Client::new();
    let url = &*FNS_BASE_URL;

    // Build form body
    let mut form: Vec<(String, String)> = Vec::new();
    form.push(("token".into(), FNS_TOKEN.clone()));
    if let Some(promo) = *FNS_PROMO_ID {
        form.push(("promo_id".into(), promo.to_string()));
    }

    if let Some(qrraw) = query.qr {
        form.push(("qrraw".into(), qrraw));
    } else {
        // Map params: i -> fd (per provider spec)
        if let Some(v) = params.get("fn") { form.push(("fn".into(), v.clone())); }
        if let Some(v) = params.get("i").or_else(|| params.get("fd")) { form.push(("fd".into(), v.clone())); }
        if let Some(v) = params.get("fp") { form.push(("fp".into(), v.clone())); }
        if let Some(v) = params.get("t") { form.push(("t".into(), v.clone())); }
        if let Some(v) = params.get("n") { form.push(("n".into(), v.clone())); }
        if let Some(v) = params.get("s") { form.push(("s".into(), v.clone())); }
        // qr flag indicates QR scanning; optional
        form.push(("qr".into(), "1".into()));
    }

    let resp = client
        .post(url)
        .form(&form)
        .send()
        .await;

    let resp = match resp { Ok(r) => r, Err(e) => return (StatusCode::BAD_GATEWAY, Json(serde_json::json!({"error": e.to_string()}))).into_response() };
    let status = resp.status();
    let text = resp.text().await.unwrap_or_else(|_| "".into());
    if !status.is_success() {
        return (StatusCode::BAD_GATEWAY, Json(serde_json::json!({"upstream_status": status.as_u16(), "body": text}))).into_response();
    }
    match serde_json::from_str::<serde_json::Value>(&text) {
        Ok(v) => Json(v).into_response(),
        Err(_) => (StatusCode::BAD_GATEWAY, Json(serde_json::json!({"error":"invalid upstream json","body": text}))).into_response(),
    }
}

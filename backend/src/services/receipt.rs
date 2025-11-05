use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::{get, post}, Json, Router};
use chrono::{Utc, DateTime};
use futures::StreamExt;
use mongodb::{options::{FindOptions, IndexOptions}, IndexModel};
use serde::{Deserialize, Serialize};

use crate::{state::AppState, models::Receipt};

#[derive(Debug, Deserialize)]
pub struct QrData {
    pub qr: String,
    #[serde(default)]
    pub user: Option<String>,
    #[serde(default)]
    pub source: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct UploadResponse { pub status: &'static str }

pub fn routes(state: AppState) -> Router {
    // ensure index on timestamp desc for efficient listing
    let receipts = state.db.collection::<Receipt>("receipts");
    let _ = tokio::spawn(async move {
        // Index for listing
        let idx_ts = IndexModel::builder()
            .keys(bson::doc!{"timestamp": -1})
            .options(IndexOptions::builder().name(Some("ts_desc".to_string())).build())
            .build();
        let _ = receipts.create_index(idx_ts, None).await;
        // Unique index to prevent duplicate QR uploads
        let idx_qr = IndexModel::builder()
            .keys(bson::doc!{"qr": 1})
            .options(IndexOptions::builder().name(Some("qr_unique".to_string())).unique(true).build())
            .build();
        let _ = receipts.create_index(idx_qr, None).await;
    });

    Router::new()
        .route("/upload", post(upload_qr))
        .route("/list", get(list_receipts))
        .with_state(state)
}

pub async fn upload_qr(
    State(state): State<AppState>,
    Json(payload): Json<QrData>,
) -> impl IntoResponse {
    // basic validation: contains t= and fn= and minimal length
    let qr = payload.qr.trim();
    let ok_len = qr.len() >= 16;
    let pattern_ok = qr.contains("t=") && qr.contains("fn=");
    if !ok_len || !pattern_ok {
        return (StatusCode::BAD_REQUEST, Json(bson::doc!{"status": "invalid_qr"}));
    }

    let user = payload.user.unwrap_or_else(|| "anonymous".to_string());
    let source = payload.source.unwrap_or_else(|| "telegram_webapp".to_string());
    let now: DateTime<Utc> = Utc::now();

    let rec = Receipt {
        id: None,
        qr: qr.to_string(),
        timestamp: now,
        source,
        user,
    };

    let col = state.db.collection::<Receipt>("receipts");
    // Reject duplicates early
    if let Ok(Some(_)) = col.find_one(bson::doc!{"qr": &rec.qr}, None).await {
        return (StatusCode::OK, Json(bson::doc!{"status": "duplicate"}));
    }
    match col.insert_one(rec, None).await {
        Ok(_) => (StatusCode::OK, Json(bson::doc!{"status": "ok"})),
        Err(e) => {
            // If unique index violation, treat as duplicate
            let msg = e.to_string();
            if msg.contains("E11000") || msg.contains("duplicate key") {
                (StatusCode::OK, Json(bson::doc!{"status": "duplicate"}))
            } else {
                (StatusCode::INTERNAL_SERVER_ERROR, Json(bson::doc!{"status": "error"}))
            }
        }
    }
}

#[derive(Debug, Serialize)]
struct ReceiptOut {
    qr: String,
    timestamp: String,
    source: String,
    user: String,
}

pub async fn list_receipts(State(state): State<AppState>) -> impl IntoResponse {
    let col = state.db.collection::<Receipt>("receipts");
    let mut cur = match col
        .find(
            None,
            FindOptions::builder().sort(bson::doc! {"timestamp": -1}).limit(100).build(),
        )
        .await
    {
        Ok(c) => c,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(Vec::<ReceiptOut>::new())).into_response(),
    };

    let mut out = Vec::new();
    while let Some(next) = cur.next().await {
        match next {
            Ok(r) => out.push(ReceiptOut {
                qr: r.qr,
                timestamp: r.timestamp.to_rfc3339(),
                source: r.source,
                user: r.user,
            }),
            Err(_) => continue,
        }
    }
    Json(out).into_response()
}

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use bson::doc;
use tracing::error;

use crate::models::{TelegramSettingsDoc, TelegramSettingsUpdate};
use crate::state::AppState;

const TELEGRAM_KEY: &str = "telegram";

pub async fn get_telegram(State(state): State<AppState>) -> impl IntoResponse {
    match state.telegram_settings.find_one(doc!{"_id": TELEGRAM_KEY}, None).await {
        Ok(Some(doc)) => (StatusCode::OK, Json(doc)).into_response(),
        Ok(None) => {
            let default = TelegramSettingsDoc { key: TELEGRAM_KEY.to_string(), token: None, chat_id: None, webhook_url: None, enabled: false };
            (StatusCode::OK, Json(default)).into_response()
        }
        Err(e) => { error!(?e, "get telegram settings failed"); StatusCode::INTERNAL_SERVER_ERROR.into_response() }
    }
}

pub async fn put_telegram(State(state): State<AppState>, Json(patch): Json<TelegramSettingsUpdate>) -> impl IntoResponse {
    let mut set = doc!{};
    if let Some(v) = patch.token { set.insert("token", v); }
    if let Some(v) = patch.chat_id { set.insert("chat_id", v); }
    if let Some(v) = patch.webhook_url { set.insert("webhook_url", v); }
    if let Some(v) = patch.enabled { set.insert("enabled", v); }

    let filter = doc!{"_id": TELEGRAM_KEY};
    let update = doc!{"$set": set, "$setOnInsert": {"_id": TELEGRAM_KEY}};
    match state.telegram_settings.update_one(filter, update, mongodb::options::UpdateOptions::builder().upsert(true).build()).await {
        Ok(_) => match state.telegram_settings.find_one(doc!{"_id": TELEGRAM_KEY}, None).await {
            Ok(Some(doc)) => (StatusCode::OK, Json(doc)).into_response(),
            _ => StatusCode::OK.into_response(),
        },
        Err(e) => { error!(?e, "put telegram settings failed"); StatusCode::INTERNAL_SERVER_ERROR.into_response() }
    }
}


use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use bson::doc;
use serde::{Deserialize, Serialize};
use tracing::{error, info};

use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct TelegramUpdate {
    #[serde(default)]
    pub message: Option<TelegramMessage>,
}

#[derive(Debug, Deserialize)]
pub struct TelegramMessage {
    pub message_id: i64,
    pub chat: TelegramChat,
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default)]
    pub from: Option<TelegramUser>,
}

#[derive(Debug, Deserialize)]
pub struct TelegramChat { pub id: i64 }

#[derive(Debug, Deserialize)]
pub struct TelegramUser { #[allow(dead_code)] pub id: i64, #[allow(dead_code)] pub first_name: Option<String> }

#[derive(Serialize)]
struct SendMessagePayload<'a> { chat_id: i64, text: &'a str }

pub async fn webhook(State(state): State<AppState>, Json(update): Json<TelegramUpdate>) -> impl IntoResponse {
    // check if enabled
    let settings = match state.telegram_settings.find_one(doc!{"_id": "telegram"}, None).await {
        Ok(opt) => opt,
        Err(e) => { error!(?e, "load telegram settings failed"); return StatusCode::OK; }
    };
    let Some(settings) = settings else { return StatusCode::OK; };
    if !settings.enabled { return StatusCode::OK; }
    let Some(token) = settings.token.as_ref() else { return StatusCode::OK; };

    let Some(msg) = update.message else { return StatusCode::OK; };
    let chat_id = msg.chat.id;

    let url = format!("https://api.telegram.org/bot{}/sendMessage", token);
    let payload = SendMessagePayload { chat_id, text: "Привет!" };
    let client = reqwest::Client::new();
    match client.post(&url).json(&payload).send().await {
        Ok(resp) => { info!(status=?resp.status(), "sent telegram reply"); }
        Err(e) => { error!(?e, "telegram sendMessage failed"); }
    }
    StatusCode::OK
}


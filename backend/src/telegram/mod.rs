use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use bson::doc;
use serde::{Deserialize, Serialize};
use tokio::task::JoinHandle;
use tokio::time::{sleep, Duration};
use tracing::{error, info, warn};

use crate::state::AppState;

#[derive(Debug, Deserialize, Clone)]
pub struct TelegramUpdate {
    #[serde(default)]
    pub update_id: Option<i64>,
    #[serde(default)]
    pub message: Option<TelegramMessage>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TelegramMessage {
    pub message_id: i64,
    pub chat: TelegramChat,
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default)]
    pub from: Option<TelegramUser>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TelegramChat { pub id: i64 }

#[derive(Debug, Deserialize, Clone)]
pub struct TelegramUser { #[allow(dead_code)] pub id: i64, #[allow(dead_code)] pub first_name: Option<String> }

#[derive(Serialize)]
struct SendMessagePayload<'a> { chat_id: i64, text: &'a str }

// Webhook handler (оставим, но основной режим сейчас — polling)
pub async fn webhook(State(state): State<AppState>, Json(update): Json<TelegramUpdate>) -> impl IntoResponse {
    let settings = match state.telegram_settings.find_one(doc!{"_id": "telegram"}, None).await { Ok(opt)=>opt, Err(e)=> { error!(?e, "get settings"); None } };
    let Some(s) = settings else { return StatusCode::OK; };
    if !(s.enabled && s.webhook_enabled) { return StatusCode::OK; }
    let Some(token) = s.token.as_ref() else { return StatusCode::OK; };
    if let Some(msg) = update.message {
        let _ = send_message(token, msg.chat.id, "Привет!").await;
    }
    StatusCode::OK
}

pub fn spawn_poller(state: AppState) -> JoinHandle<()> {
    tokio::spawn(async move {
        let client = reqwest::Client::new();
        let mut offset: i64 = 0;
        let mut last_token: Option<String> = None;
        loop {
            // Читаем настройки каждый цикл
            let settings = match state.telegram_settings.find_one(doc!{"_id": "telegram"}, None).await {
                Ok(opt) => opt,
                Err(e) => { error!(?e, "poller: load settings failed"); None }
            };
            match settings {
                Some(s) if s.enabled && s.token.is_some() => {
                    let token = s.token.clone().unwrap();
                    // Сброс offset при смене токена
                    if last_token.as_deref() != Some(token.as_str()) {
                        offset = 0;
                        last_token = Some(token.clone());
                        info!("telegram poller: token changed, reset offset");
                    }

                    // Long polling
                    match get_updates(&client, &token, offset).await {
                        Ok((updates, new_offset)) => {
                            for u in &updates {
                                if let Some(ref msg) = u.message {
                                    let _ = send_message(&token, msg.chat.id, "Привет!").await;
                                }
                            }
                            offset = new_offset;
                        }
                        Err(e) => {
                            warn!(?e, "telegram poller: getUpdates error");
                            sleep(Duration::from_secs(3)).await;
                        }
                    }
                }
                _ => {
                    // Не включено — подождать и снова проверить
                    sleep(Duration::from_secs(5)).await;
                }
            }
        }
    })
}

async fn get_updates(client: &reqwest::Client, token: &str, offset: i64) -> anyhow::Result<(Vec<TelegramUpdate>, i64)> {
    #[derive(Deserialize)]
    struct UpdatesResp { ok: bool, #[serde(default)] result: Vec<TelegramUpdate> }
    let url = format!("https://api.telegram.org/bot{}/getUpdates", token);
    let body = serde_json::json!({ "offset": if offset>0 { Some(offset) } else { None::<i64> }, "timeout": 20 });
    let resp = client.post(&url).json(&body).send().await?;
    let st = resp.status();
    let data: UpdatesResp = resp.json().await?;
    if !st.is_success() || !data.ok { anyhow::bail!("getUpdates failed: status {:?}", st); }
    let mut new_offset = offset;
    for u in &data.result { if let Some(id) = u.update_id { if id >= new_offset { new_offset = id + 1; } } }
    Ok((data.result, new_offset))
}

async fn send_message(token: &str, chat_id: i64, text: &str) -> anyhow::Result<()> {
    let url = format!("https://api.telegram.org/bot{}/sendMessage", token);
    let payload = SendMessagePayload { chat_id, text };
    let client = reqwest::Client::new();
    let resp = client.post(&url).json(&payload).send().await?;
    if !resp.status().is_success() { anyhow::bail!("sendMessage failed: status {:?}", resp.status()); }
    Ok(())
}

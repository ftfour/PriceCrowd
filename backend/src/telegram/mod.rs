use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use bson::doc;
use serde::{Deserialize, Serialize};
use tokio::task::JoinHandle;
use tokio::time::{sleep, Duration};
use tracing::{error, info, warn};
use std::sync::atomic::{AtomicI64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use once_cell::sync::Lazy;
use tokio::sync::Mutex;

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
pub struct TelegramUser { #[allow(dead_code)] pub id: i64, #[allow(dead_code)] pub first_name: Option<String>, #[serde(default)] pub username: Option<String> }

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
                            let count = updates.len();
                            if count > 0 { push_log("info", &format!("получено обновлений: {}", count)).await; }
                            for u in &updates {
                                if let Some(ref msg) = u.message {
                                    let text = msg.text.clone().unwrap_or_default();
                                    if let Some(code) = parse_link_code(&text) {
                                        let tg_username = msg.from.as_ref().and_then(|u| u.username.clone());
                                        match link_account(&state, &code, msg.chat.id, tg_username.as_deref()).await {
                                            Ok(true) => { let _ = send_message(&token, msg.chat.id, "Аккаунт привязан ✅").await; push_log("info", &format!("linked code {}", code)).await; }
                                            Ok(false) => { let _ = send_message(&token, msg.chat.id, "Код недействителен или истёк ❌").await; push_log("warn", &format!("invalid code {}", code)).await; }
                                            Err(e) => { let _ = send_message(&token, msg.chat.id, "Ошибка привязки").await; push_log("error", &format!("link error: {}", e)).await; }
                                        }
                                    } else {
                                        let _ = send_message(&token, msg.chat.id, "Привет!").await;
                                    }
                                }
                            }
                            offset = new_offset;
                            LAST_POLL_MS.store(now_ms(), Ordering::Relaxed);
                        }
                        Err(e) => {
                            warn!(?e, "telegram poller: getUpdates error");
                            push_log("warn", &format!("ошибка getUpdates: {}", e)).await;
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

#[derive(Serialize, Clone)]
pub struct LogEntry { ts_ms: i64, level: String, message: String }

static LAST_POLL_MS: AtomicI64 = AtomicI64::new(0);
static LOGS: Lazy<Mutex<Vec<LogEntry>>> = Lazy::new(|| Mutex::new(Vec::with_capacity(200)));
const MAX_LOGS: usize = 200;

fn now_ms() -> i64 { SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as i64 }

async fn push_log(level: &str, message: &str) {
    let mut guard = LOGS.lock().await;
    guard.push(LogEntry { ts_ms: now_ms(), level: level.to_string(), message: message.to_string() });
    if guard.len() > MAX_LOGS { let overflow = guard.len() - MAX_LOGS; guard.drain(0..overflow); }
}

#[derive(Serialize)]
pub struct BotStatus { enabled: bool, webhook_enabled: bool, polling: bool, last_poll_ms: Option<i64> }

pub async fn status(State(state): State<AppState>) -> impl IntoResponse {
    let settings = state.telegram_settings.find_one(doc!{"_id": "telegram"}, None).await.ok().flatten();
    let (enabled, webhook_enabled) = settings.map(|s| (s.enabled, s.webhook_enabled)).unwrap_or((false,false));
    let last = LAST_POLL_MS.load(Ordering::Relaxed);
    let last_poll_ms = if last > 0 { Some(last) } else { None };
    let polling = enabled; // признак: воркер активен, если включен бот
    let status = BotStatus { enabled, webhook_enabled, polling, last_poll_ms };
    let logs = LOGS.lock().await.clone();
    let body = serde_json::json!({ "status": status, "logs": logs });
    (StatusCode::OK, Json(body)).into_response()
}

fn parse_link_code(text: &str) -> Option<String> {
    let t = text.trim();
    if t.starts_with("/link ") {
        return Some(t[6..].trim().to_uppercase());
    }
    // Accept raw 6-char code
    if t.len()==6 && t.chars().all(|c| c.is_ascii_alphanumeric()) {
        return Some(t.to_uppercase());
    }
    None
}

async fn link_account(state: &AppState, code: &str, chat_id: i64, username: Option<&str>) -> anyhow::Result<bool> {
    // find code
    let now = now_ms();
    let coll = &state.telegram_links;
    let filter = doc!{"code": code, "used": false, "exp_ms": {"$gt": now}};
    if let Some(link) = coll.find_one(filter, None).await? {
        // set user.telegram_id
        let users = state.db.collection::<crate::models::User>("users");
        let mut set = doc!{"telegram_id": chat_id};
        if let Some(u) = username { set.insert("telegram_username", u); }
        users.update_one(doc!{"username": &link.username}, doc!{"$set": set}, None).await?;
        // mark used
        if let Some(id) = link.id { let _ = state.telegram_links.update_one(doc!{"_id": id}, doc!{"$set": {"used": true}}, None).await; }
        return Ok(true);
    }
    Ok(false)
}

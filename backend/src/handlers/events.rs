use axum::{extract::State, response::IntoResponse, http::StatusCode, Json};
use futures::stream::StreamExt;
use bson::doc;
use tracing::error;

use crate::{state::AppState, models::EventDoc};

#[derive(serde::Serialize)]
struct PublicEvent { ts_ms: i64, kind: String, message: String, user: Option<String> }

pub async fn list_events(State(state): State<AppState>) -> impl IntoResponse {
    let col = state.db.collection::<EventDoc>("events");
    let mut cur = match col.find(None, mongodb::options::FindOptions::builder().sort(doc!{"ts_ms": -1}).limit(100).build()).await { Ok(c)=>c, Err(e)=> { error!(?e, "events.find failed"); return StatusCode::INTERNAL_SERVER_ERROR.into_response(); } };
    let mut out: Vec<PublicEvent> = Vec::new();
    while let Some(item) = cur.next().await { match item {
        Ok(ev) => out.push(PublicEvent { ts_ms: ev.ts_ms, kind: ev.kind, message: ev.message, user: ev.user }),
        Err(e) => { error!(?e, "events cursor error"); return StatusCode::INTERNAL_SERVER_ERROR.into_response(); }
    }}
    Json(out).into_response()
}

pub async fn log_event(state: &AppState, kind: &str, message: &str, user: Option<String>) {
    let col = state.db.collection::<EventDoc>("events");
    let _ = col.insert_one(EventDoc { id: None, ts_ms: chrono::Utc::now().timestamp_millis(), kind: kind.to_string(), message: message.to_string(), user }, None).await;
}


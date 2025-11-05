use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use bson::doc;
use futures::stream::StreamExt;
use tracing::error;

use crate::models::User;
use crate::state::AppState;

#[derive(serde::Serialize)]
pub struct RatingUser { pub name: String, pub points: i64 }

pub async fn list_user_ratings(State(state): State<AppState>) -> impl IntoResponse {
    let coll = state.db.collection::<User>("users");
    let mut cur = match coll.find(None, None).await { Ok(c)=>c, Err(e)=> { error!(?e, "users.find failed"); return StatusCode::INTERNAL_SERVER_ERROR.into_response(); } };
    let mut out: Vec<RatingUser> = Vec::new();
    while let Some(item) = cur.next().await { match item {
        Ok(u) => {
            let name = u.telegram_username.clone().unwrap_or(u.username);
            let points = u.points.unwrap_or(0); // пока очков нет
            out.push(RatingUser { name, points });
        }
        Err(e) => { error!(?e, "users cursor error"); return StatusCode::INTERNAL_SERVER_ERROR.into_response(); }
    }}
    // сортируем по points (все 0 — останется исходный порядок)
    out.sort_by(|a,b| b.points.cmp(&a.points));
    Json(out).into_response()
}


#[derive(serde::Deserialize)]
pub struct AwardPoints { pub username: String, #[serde(default = "default_delta")] pub delta: i64 }

fn default_delta() -> i64 { 1 }

pub async fn award_points(State(state): State<AppState>, Json(body): Json<AwardPoints>) -> impl IntoResponse {
    if body.username.trim().is_empty() { return StatusCode::BAD_REQUEST.into_response(); }
    let coll = state.db.collection::<User>("users");
    let filter = doc!{"username": &body.username };
    let update = doc!{"$inc": {"points": body.delta }};
    match coll.update_one(filter, update, None).await {
        Ok(r) if r.matched_count > 0 => { let _ = crate::handlers::events::log_event(&state, "receipt_verified", "??? ???????????", Some(body.username)).await; StatusCode::NO_CONTENT.into_response() },
        Ok(_) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => { error!(?e, "award_points failed"); StatusCode::INTERNAL_SERVER_ERROR.into_response() }
    }
}



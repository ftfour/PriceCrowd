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
            let points = 0i64; // пока очков нет
            out.push(RatingUser { name, points });
        }
        Err(e) => { error!(?e, "users cursor error"); return StatusCode::INTERNAL_SERVER_ERROR.into_response(); }
    }}
    // сортируем по points (все 0 — останется исходный порядок)
    out.sort_by(|a,b| b.points.cmp(&a.points));
    Json(out).into_response()
}


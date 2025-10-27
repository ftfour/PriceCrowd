use std::str::FromStr;

use axum::{extract::{Path, State}, http::StatusCode, response::IntoResponse, Json};
use bson::{doc, oid::ObjectId};
use futures::stream::StreamExt;
use tracing::error;

use crate::models::StoreActivity;
use crate::state::AppState;

pub async fn list_store_activities(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    let Ok(store_oid) = ObjectId::from_str(&id) else { return StatusCode::BAD_REQUEST.into_response(); };
    let mut cursor = match state.store_activities.find(doc!{"store_id": store_oid}, mongodb::options::FindOptions::builder().sort(doc!{"ts_ms": -1}).limit(50).build()).await {
        Ok(c)=>c,
        Err(e)=> { error!(?e, "query activities failed"); return StatusCode::INTERNAL_SERVER_ERROR.into_response(); }
    };
    let mut items: Vec<StoreActivity> = Vec::new();
    while let Some(res) = cursor.next().await { match res { Ok(doc)=> items.push(doc), Err(e)=> { error!(?e, "cursor error"); return StatusCode::INTERNAL_SERVER_ERROR.into_response(); } } }
    (StatusCode::OK, Json(items)).into_response()
}

pub async fn list_all_activities(State(state): State<AppState>) -> impl IntoResponse {
    let mut cursor = match state.store_activities.find(None, mongodb::options::FindOptions::builder().sort(doc!{"ts_ms": -1}).limit(50).build()).await {
        Ok(c)=>c,
        Err(e)=> { error!(?e, "query activities failed"); return StatusCode::INTERNAL_SERVER_ERROR.into_response(); }
    };
    let mut items: Vec<StoreActivity> = Vec::new();
    while let Some(res) = cursor.next().await { match res { Ok(doc)=> items.push(doc), Err(e)=> { error!(?e, "cursor error"); return StatusCode::INTERNAL_SERVER_ERROR.into_response(); } } }
    (StatusCode::OK, Json(items)).into_response()
}


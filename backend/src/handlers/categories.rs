use std::str::FromStr;

use axum::{extract::{Path, State}, http::StatusCode, response::IntoResponse, Json};
use bson::{doc, oid::ObjectId, Bson};
use futures::stream::StreamExt;
use tracing::error;

use crate::models::{Category, CategoryCreate, CategoryUpdate};
use crate::state::AppState;

pub async fn list_categories(State(state): State<AppState>) -> impl IntoResponse {
    let mut cursor = match state.categories.find(None, None).await { Ok(c)=>c, Err(e)=> { error!(?e, "query categories failed"); return StatusCode::INTERNAL_SERVER_ERROR.into_response(); } };
    let mut items: Vec<Category> = Vec::new();
    while let Some(res) = cursor.next().await { match res { Ok(doc)=> items.push(doc), Err(e)=> { error!(?e, "cursor error"); return StatusCode::INTERNAL_SERVER_ERROR.into_response(); } } }
    (StatusCode::OK, Json(items)).into_response()
}

pub async fn get_category(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    let Ok(oid) = ObjectId::from_str(&id) else { return StatusCode::BAD_REQUEST.into_response(); };
    let filter = doc! {"_id": oid};
    match state.categories.find_one(filter, None).await {
        Ok(Some(doc)) => (StatusCode::OK, Json(doc)).into_response(),
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => { error!(?e, "find category failed"); StatusCode::INTERNAL_SERVER_ERROR.into_response() }
    }
}

pub async fn create_category(State(state): State<AppState>, Json(payload): Json<CategoryCreate>) -> impl IntoResponse {
    let mut parents: Vec<ObjectId> = Vec::new();
    for s in payload.parent_ids.iter() { if let Ok(oid) = ObjectId::from_str(s) { parents.push(oid); } }
    let cat = Category { id: None, name: payload.name, desc: payload.desc, parent_ids: parents };
    match state.categories.insert_one(cat, None).await {
        Ok(result) => {
            let id = match result.inserted_id { Bson::ObjectId(oid) => oid, _ => ObjectId::new() };
            let filter = doc! {"_id": id};
            match state.categories.find_one(filter, None).await { Ok(Some(created)) => (StatusCode::CREATED, Json(created)).into_response(), _ => StatusCode::INTERNAL_SERVER_ERROR.into_response() }
        }
        Err(e) => { error!(?e, "insert category failed"); StatusCode::INTERNAL_SERVER_ERROR.into_response() }
    }
}

pub async fn update_category(State(state): State<AppState>, Path(id): Path<String>, Json(patch): Json<CategoryUpdate>) -> impl IntoResponse {
    let Ok(oid) = ObjectId::from_str(&id) else { return StatusCode::BAD_REQUEST.into_response(); };
    let mut set = doc! {};
    if let Some(n) = patch.name { set.insert("name", n); }
    if let Some(d) = patch.desc { set.insert("desc", d); }
    if let Some(pids) = patch.parent_ids { let mut v: Vec<ObjectId> = Vec::new(); for s in pids.iter(){ if let Ok(oo)=ObjectId::from_str(s){ v.push(oo);} } set.insert("parent_ids", v);}    
    if set.is_empty() { return StatusCode::BAD_REQUEST.into_response(); }
    let filter = doc! {"_id": oid};
    let update = doc! {"$set": set};
    match state.categories.find_one_and_update(filter, update, None).await {
        Ok(Some(updated)) => (StatusCode::OK, Json(updated)).into_response(),
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => { error!(?e, "update category failed"); StatusCode::INTERNAL_SERVER_ERROR.into_response() }
    }
}

pub async fn delete_category(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    let Ok(oid) = ObjectId::from_str(&id) else { return StatusCode::BAD_REQUEST.into_response(); };
    let filter = doc! {"_id": oid};
    match state.categories.delete_one(filter, None).await {
        Ok(res) if res.deleted_count == 1 => StatusCode::NO_CONTENT.into_response(),
        Ok(_) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => { error!(?e, "delete category failed"); StatusCode::INTERNAL_SERVER_ERROR.into_response() }
    }
}


use std::str::FromStr;

use axum::{extract::{Path, State}, http::StatusCode, response::IntoResponse, Json};
use bson::{doc, oid::ObjectId, Bson};
use futures::stream::StreamExt;
use tracing::error;

use crate::models::{Product, ProductCreate, ProductUpdate};
use crate::state::AppState;

pub async fn list_products(State(state): State<AppState>) -> impl IntoResponse {
    let mut cursor = match state.products.find(None, None).await {
        Ok(c) => c,
        Err(e) => {
            error!(?e, "failed to query products");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let mut items: Vec<Product> = Vec::new();
    while let Some(res) = cursor.next().await {
        match res {
            Ok(doc) => items.push(doc),
            Err(e) => {
                error!(?e, "cursor error");
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
        }
    }
    (StatusCode::OK, Json(items)).into_response()
}

pub async fn get_product(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    let Ok(oid) = ObjectId::from_str(&id) else { return StatusCode::BAD_REQUEST.into_response(); };
    let filter = doc! {"_id": oid};
    match state.products.find_one(filter, None).await {
        Ok(Some(doc)) => (StatusCode::OK, Json(doc)).into_response(),
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            error!(?e, "find_one failed");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

pub async fn create_product(State(state): State<AppState>, Json(payload): Json<ProductCreate>) -> impl IntoResponse {
    // map category ids
    let mut cat_ids: Vec<ObjectId> = Vec::new();
    for s in payload.category_ids.iter() {
        if let Ok(oid) = ObjectId::from_str(s) { cat_ids.push(oid); }
    }
    let product = Product {
        id: None,
        title: payload.title,
        desc: payload.desc,
        image_url: payload.image_url,
        category_ids: cat_ids,
    };

    match state.products.insert_one(product, None).await {
        Ok(result) => {
            let id = match result.inserted_id {
                Bson::ObjectId(oid) => oid,
                _ => ObjectId::new(),
            };
            let filter = doc! {"_id": id};
            match state.products.find_one(filter, None).await {
                Ok(Some(created)) => (StatusCode::CREATED, Json(created)).into_response(),
                _ => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            }
        }
        Err(e) => {
            error!(?e, "insert failed");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

pub async fn update_product(State(state): State<AppState>, Path(id): Path<String>, Json(patch): Json<ProductUpdate>) -> impl IntoResponse {
    let Ok(oid) = ObjectId::from_str(&id) else { return StatusCode::BAD_REQUEST.into_response(); };
    let mut set = doc! {};
    if let Some(t) = patch.title { set.insert("title", t); }
    if let Some(d) = patch.desc { set.insert("desc", d); }
    if let Some(iu) = patch.image_url { set.insert("image_url", iu); }
    if let Some(cats) = patch.category_ids {
        let mut list: Vec<ObjectId> = Vec::new();
        for s in cats.iter() { if let Ok(oid) = ObjectId::from_str(s) { list.push(oid); } }
        set.insert("category_ids", list);
    }
    if set.is_empty() { return StatusCode::BAD_REQUEST.into_response(); }

    let filter = doc! {"_id": oid};
    let update = doc! {"$set": set};
    match state.products.find_one_and_update(filter.clone(), update, None).await {
        Ok(Some(updated)) => (StatusCode::OK, Json(updated)).into_response(),
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            error!(?e, "update failed");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

pub async fn delete_product(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    let Ok(oid) = ObjectId::from_str(&id) else { return StatusCode::BAD_REQUEST.into_response(); };
    let filter = doc! {"_id": oid};
    match state.products.delete_one(filter, None).await {
        Ok(res) if res.deleted_count == 1 => StatusCode::NO_CONTENT.into_response(),
        Ok(_) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            error!(?e, "delete failed");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

pub async fn add_product_category(State(state): State<AppState>, Path((id, cat_id)): Path<(String, String)>) -> impl IntoResponse {
    let (Ok(oid), Ok(cid)) = (ObjectId::from_str(&id), ObjectId::from_str(&cat_id)) else { return StatusCode::BAD_REQUEST.into_response(); };
    let filter = doc! {"_id": oid};
    let update = doc! {"$addToSet": {"category_ids": cid}};
    match state.products.update_one(filter, update, None).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => { error!(?e, "add category failed"); StatusCode::INTERNAL_SERVER_ERROR.into_response() }
    }
}

pub async fn remove_product_category(State(state): State<AppState>, Path((id, cat_id)): Path<(String, String)>) -> impl IntoResponse {
    let (Ok(oid), Ok(cid)) = (ObjectId::from_str(&id), ObjectId::from_str(&cat_id)) else { return StatusCode::BAD_REQUEST.into_response(); };
    let filter = doc! {"_id": oid};
    let update = doc! {"$pull": {"category_ids": cid}};
    match state.products.update_one(filter, update, None).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => { error!(?e, "remove category failed"); StatusCode::INTERNAL_SERVER_ERROR.into_response() }
    }
}


use std::str::FromStr;
use std::collections::HashMap as StdHashMap;

use axum::{extract::{Path, State}, http::StatusCode, response::IntoResponse, Json};
use bson::{doc, oid::ObjectId};
use futures::stream::StreamExt;
use tracing::error;

use crate::models::{Store, StoreCreate, StoreUpdate, StoreItem, StoreItemCreate, StoreItemUpdate, StoreActivity};
use crate::state::AppState;

pub async fn list_stores(State(state): State<AppState>) -> impl IntoResponse {
    let mut cursor = match state.stores.find(None, None).await {
        Ok(c) => c,
        Err(e) => {
            error!(?e, "failed to query stores");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    let mut items: Vec<Store> = Vec::new();
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

pub async fn get_store(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    let Ok(oid) = ObjectId::from_str(&id) else { return StatusCode::BAD_REQUEST.into_response(); };
    let filter = doc! {"_id": oid};
    match state.stores.find_one(filter, None).await {
        Ok(Some(doc)) => (StatusCode::OK, Json(doc)).into_response(),
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            error!(?e, "find_one store failed");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

pub async fn create_store(State(state): State<AppState>, Json(payload): Json<StoreCreate>) -> impl IntoResponse {
    let store = Store {
        id: None,
        name: payload.name,
        addr: payload.addr,
        desc: payload.desc,
        image_url: payload.image_url,
    };
    match state.stores.insert_one(store, None).await {
        Ok(result) => {
            let id = match result.inserted_id { bson::Bson::ObjectId(oid) => oid, _ => ObjectId::new() };
            let filter = doc! {"_id": id};
            match state.stores.find_one(filter, None).await {
                Ok(Some(created)) => (StatusCode::CREATED, Json(created)).into_response(),
                _ => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            }
        }
        Err(e) => {
            error!(?e, "insert store failed");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

pub async fn update_store(State(state): State<AppState>, Path(id): Path<String>, Json(patch): Json<StoreUpdate>) -> impl IntoResponse {
    let Ok(oid) = ObjectId::from_str(&id) else { return StatusCode::BAD_REQUEST.into_response(); };
    let mut set = doc! {};
    if let Some(v) = patch.name { set.insert("name", v); }
    if let Some(v) = patch.addr { set.insert("addr", v); }
    if let Some(v) = patch.desc { set.insert("desc", v); }
    if let Some(v) = patch.image_url { set.insert("image_url", v); }
    if set.is_empty() { return StatusCode::BAD_REQUEST.into_response(); }
    let filter = doc! {"_id": oid};
    let update = doc! {"$set": set};
    match state.stores.find_one_and_update(filter, update, None).await {
        Ok(Some(updated)) => (StatusCode::OK, Json(updated)).into_response(),
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            error!(?e, "update store failed");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

pub async fn delete_store(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    let Ok(oid) = ObjectId::from_str(&id) else { return StatusCode::BAD_REQUEST.into_response(); };
    let filter = doc! {"_id": oid};
    match state.stores.delete_one(filter, None).await {
        Ok(res) if res.deleted_count == 1 => StatusCode::NO_CONTENT.into_response(),
        Ok(_) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            error!(?e, "delete store failed");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

pub async fn list_store_products(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    let Ok(store_oid) = ObjectId::from_str(&id) else { return StatusCode::BAD_REQUEST.into_response(); };
    let mut cursor = match state.store_items.find(doc!{"store_id": store_oid}, None).await { Ok(c)=>c, Err(e)=> { error!(?e, "query store_items failed"); return StatusCode::INTERNAL_SERVER_ERROR.into_response(); } };
    let mut items: Vec<StoreItem> = Vec::new();
    while let Some(res) = cursor.next().await { match res { Ok(doc)=> items.push(doc), Err(e)=> { error!(?e, "cursor error"); return StatusCode::INTERNAL_SERVER_ERROR.into_response(); } } }
    // collect product ids
    let pids: Vec<ObjectId> = items.iter().map(|it| it.product_id.clone()).collect();
    let mut products_map = StdHashMap::new();
    if !pids.is_empty() {
        let mut pcursor = match state.products.find(doc!{"_id": {"$in": &pids}}, None).await { Ok(c)=>c, Err(e)=> { error!(?e, "query products for store items failed"); return StatusCode::INTERNAL_SERVER_ERROR.into_response(); } };
        while let Some(res) = pcursor.next().await { match res { Ok(p)=> { if let Some(id) = p.id { products_map.insert(id, p); } }, Err(e)=> { error!(?e, "products cursor error"); return StatusCode::INTERNAL_SERVER_ERROR.into_response(); } } }
    }
    let payload: Vec<serde_json::Value> = items.into_iter().map(|it| {
        let prod = products_map.get(&it.product_id);
        serde_json::json!({
            "_id": it.id,
            "product_id": it.product_id,
            "store_id": it.store_id,
            "price": it.price,
            "product": prod,
        })
    }).collect();
    (StatusCode::OK, Json(payload)).into_response()
}

pub async fn add_store_product(State(state): State<AppState>, Path(id): Path<String>, Json(body): Json<StoreItemCreate>) -> impl IntoResponse {
    let Ok(store_oid) = ObjectId::from_str(&id) else { return StatusCode::BAD_REQUEST.into_response(); };
    let Ok(product_oid) = ObjectId::from_str(&body.product_id) else { return StatusCode::BAD_REQUEST.into_response(); };
    // upsert
    let filter = doc!{"store_id": store_oid, "product_id": product_oid};
    let update = doc!{"$set": {"price": body.price}, "$setOnInsert": {"store_id": store_oid.clone(), "product_id": product_oid.clone()}};
    match state.store_items.update_one(filter, update, mongodb::options::UpdateOptions::builder().upsert(true).build()).await {
        Ok(_) => {
            let now_ms = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis() as i64;
            // fetch product name for activity
            let product_name = match state.products.find_one(doc!{"_id": product_oid}, None).await { Ok(opt)=> opt.map(|p| p.title), Err(_)=> None };
            let store_name = match state.stores.find_one(doc!{"_id": store_oid}, None).await { Ok(opt)=> opt.map(|s| s.name), Err(_)=> None };
            let activity = StoreActivity { id: None, store_id: store_oid, product_id: Some(product_oid), kind: "item_added".to_string(), ts_ms: now_ms, price: Some(body.price), product_name, store_name };
            let _ = state.store_activities.insert_one(activity, None).await;
            // return new doc
            match state.store_items.find_one(doc!{"store_id": store_oid, "product_id": product_oid}, None).await {
                Ok(Some(doc)) => {
                    let resp = serde_json::json!({
                        "_id": doc.id,
                        "product_id": doc.product_id,
                        "store_id": doc.store_id,
                        "price": doc.price,
                    });
                    (StatusCode::CREATED, Json(resp)).into_response()
                }
                Ok(None) => StatusCode::CREATED.into_response(),
                Err(_) => StatusCode::CREATED.into_response(),
            }
        }
        Err(e) => { error!(?e, "upsert store item failed"); StatusCode::INTERNAL_SERVER_ERROR.into_response() }
    }
}

pub async fn update_store_product(State(state): State<AppState>, Path((id, product_id)): Path<(String, String)>, Json(body): Json<StoreItemUpdate>) -> impl IntoResponse {
    let (Ok(store_oid), Ok(product_oid)) = (ObjectId::from_str(&id), ObjectId::from_str(&product_id)) else { return StatusCode::BAD_REQUEST.into_response(); };
    let filter = doc!{"store_id": store_oid, "product_id": product_oid};
    let update = doc!{"$set": {"price": body.price}};
    match state.store_items.update_one(filter, update, None).await {
        Ok(res) if res.matched_count > 0 => {
            let now_ms = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis() as i64;
            // fetch product name for activity
            let product_name = match state.products.find_one(doc!{"_id": product_oid}, None).await { Ok(opt)=> opt.map(|p| p.title), Err(_)=> None };
            let store_name = match state.stores.find_one(doc!{"_id": store_oid}, None).await { Ok(opt)=> opt.map(|s| s.name), Err(_)=> None };
            let activity = StoreActivity { id: None, store_id: store_oid, product_id: Some(product_oid), kind: "price_updated".to_string(), ts_ms: now_ms, price: Some(body.price), product_name, store_name };
            let _ = state.store_activities.insert_one(activity, None).await;
            StatusCode::NO_CONTENT.into_response()
        },
        Ok(_) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => { error!(?e, "update store product failed"); StatusCode::INTERNAL_SERVER_ERROR.into_response() }
    }
}

pub async fn remove_store_product(State(state): State<AppState>, Path((id, product_id)): Path<(String, String)>) -> impl IntoResponse {
    let (Ok(store_oid), Ok(product_oid)) = (ObjectId::from_str(&id), ObjectId::from_str(&product_id)) else { return StatusCode::BAD_REQUEST.into_response(); };
    let filter = doc!{"store_id": store_oid, "product_id": product_oid};
    match state.store_items.delete_one(filter, None).await {
        Ok(res) if res.deleted_count == 1 => {
            let now_ms = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis() as i64;
            let product_name = match state.products.find_one(doc!{"_id": product_oid}, None).await { Ok(opt)=> opt.map(|p| p.title), Err(_)=> None };
            let store_name = match state.stores.find_one(doc!{"_id": store_oid}, None).await { Ok(opt)=> opt.map(|s| s.name), Err(_)=> None };
            let activity = StoreActivity { id: None, store_id: store_oid, product_id: Some(product_oid), kind: "item_removed".to_string(), ts_ms: now_ms, price: None, product_name, store_name };
            let _ = state.store_activities.insert_one(activity, None).await;
            StatusCode::NO_CONTENT.into_response()
        },
        Ok(_) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => { error!(?e, "remove store product failed"); StatusCode::INTERNAL_SERVER_ERROR.into_response() }
    }
}

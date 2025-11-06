use axum::{extract::{State, Path}, http::StatusCode, response::IntoResponse, Json};
use chrono::{DateTime, Utc};
use bson::{doc, oid::ObjectId};
use futures::stream::StreamExt;
use tracing::error;

use crate::{state::AppState, models::{Operation, OperationItem}};

#[derive(serde::Deserialize)]
pub struct CreateOperationBody {
    pub date: String,
    pub seller: String,
    pub amount: f64,
    pub items: Vec<OperationItem>,
    pub qr: Option<String>,
    pub uploaded_by: Option<String>,
    #[serde(default)]
    pub raw: Option<serde_json::Value>,
}

pub async fn create_operation(State(state): State<AppState>, Json(body): Json<CreateOperationBody>) -> impl IntoResponse {
    let col = state.db.collection::<Operation>("operations");
    if let Some(user) = &body.uploaded_by {
        // мягкое ограничение: блокируем только если есть черновик (draft)
        match col.find_one(doc!{"uploaded_by": user, "status": "draft"}, None).await {
            Ok(Some(_)) => return (StatusCode::CONFLICT, Json(serde_json::json!({"error": "user_has_operation"}))).into_response(),
            Ok(None) => {}
            Err(e) => { error!(?e, "find existing op failed"); return StatusCode::INTERNAL_SERVER_ERROR.into_response(); }
        }
    }
    if let Some(qr) = &body.qr {
        match col.find_one(doc!{"qr": qr}, None).await { Ok(Some(_)) => return (StatusCode::CONFLICT, Json(serde_json::json!({"error": "qr_used"}))).into_response(), _=>{} }
    }
    let op = Operation {
        id: None,
        date: body.date,
        seller: body.seller,
        amount: body.amount,
        items: body.items,
        status: "draft".into(),
        store_id: None,
        qr: body.qr,
        uploaded_by: body.uploaded_by,
        // store raw as string to avoid BSON large integer issues
        raw: body.raw.map(|v| serde_json::to_string(&v).unwrap_or_else(|_| String::new())),
    };
    match col.insert_one(op, None).await {
        Ok(res) => {
            let id = match res.inserted_id { bson::Bson::ObjectId(oid)=> oid, _=> ObjectId::new() };
            (StatusCode::OK, Json(serde_json::json!({"_id": id}))).into_response()
        }
        Err(e) => { error!(?e, "insert op failed"); StatusCode::INTERNAL_SERVER_ERROR.into_response() }
    }
}

pub async fn list_operations(State(state): State<AppState>) -> impl IntoResponse {
    let col = state.db.collection::<Operation>("operations");
    let opts = mongodb::options::FindOptions::builder().sort(doc!{"date": -1}).limit(200).build();
    let mut cur = match col.find(None, opts).await { Ok(c)=>c, Err(e)=> { error!(?e, "ops.find failed"); return StatusCode::INTERNAL_SERVER_ERROR.into_response(); } };
    let mut out = Vec::new();
    while let Some(n) = cur.next().await { match n { Ok(op)=> out.push(op), Err(e)=> { error!(?e, "ops cursor"); return StatusCode::INTERNAL_SERVER_ERROR.into_response(); } } }
    Json(out).into_response()
}

pub async fn get_operation(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    let Ok(oid) = ObjectId::parse_str(&id) else { return StatusCode::BAD_REQUEST.into_response(); };
    let col = state.db.collection::<Operation>("operations");
    match col.find_one(doc!{"_id": oid}, None).await {
        Ok(Some(op)) => Json(op).into_response(),
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => { error!(?e, "get operation failed"); StatusCode::INTERNAL_SERVER_ERROR.into_response() }
    }
}

#[derive(serde::Deserialize)]
pub struct UpdateStatus { pub status: String }

pub async fn update_status(State(state): State<AppState>, Path(id): Path<String>, Json(body): Json<UpdateStatus>) -> impl IntoResponse {
    let Ok(oid) = ObjectId::parse_str(&id) else { return StatusCode::BAD_REQUEST.into_response(); };
    let col = state.db.collection::<Operation>("operations");
    // Load existing op
    let existing = match col.find_one(doc!{"_id": oid}, None).await {
        Ok(Some(op)) => op,
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(e) => { error!(?e, "get operation before status change failed"); return StatusCode::INTERNAL_SERVER_ERROR.into_response(); }
    };

    // If transitioning to posted, apply prices to store items and record activities.
    if body.status == "posted" && existing.status != "posted" {
        let Some(store_oid) = existing.store_id.clone() else { return (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error":"missing_store"}))).into_response() };

        // Parse timestamp from operation date
        let ts_ms = match DateTime::parse_from_rfc3339(&existing.date) {
            Ok(dt) => dt.timestamp_millis(),
            Err(_) => Utc::now().timestamp_millis(),
        };

        // For robust product_id extraction, read raw document as well
        let raw_col = state.db.collection::<bson::Document>("operations");
        let raw_doc = raw_col.find_one(doc!{"_id": oid}, None).await.ok().flatten();

        for (idx, it) in existing.items.iter().enumerate() {
            // Determine product_id: prefer typed, otherwise from raw document
            let mut prod_opt: Option<ObjectId> = it.product_id.clone();
            if prod_opt.is_none() {
                if let Some(doc) = &raw_doc {
                    if let Some(items) = doc.get_array("items").ok() {
                        if let Some(bson::Bson::Document(d)) = items.get(idx) {
                            if let Some(val) = d.get("product_id") {
                                match val {
                                    bson::Bson::ObjectId(oid) => { prod_opt = Some(*oid); },
                                    bson::Bson::String(s) => { if let Ok(p) = ObjectId::parse_str(s) { prod_opt = Some(p); } },
                                    _ => {}
                                }
                            }
                        }
                    }
                }
            }

            let Some(product_oid) = prod_opt else { continue };
            let price = it.price;
            // Upsert store item price
            let filter = doc!{"store_id": &store_oid, "product_id": &product_oid};
            let update = doc!{"$set": {"price": price}, "$setOnInsert": {"store_id": &store_oid, "product_id": &product_oid}};
            let upsert_opts = mongodb::options::UpdateOptions::builder().upsert(true).build();
            let _ = state.store_items.update_one(filter, update, upsert_opts).await;

            // Fetch names for activity
            let product_name = match state.products.find_one(doc!{"_id": &product_oid}, None).await { Ok(opt)=> opt.map(|p| p.title), Err(_)=> None };
            let store_name = match state.stores.find_one(doc!{"_id": &store_oid}, None).await { Ok(opt)=> opt.map(|s| s.name), Err(_)=> None };

            // Record activity with receipt timestamp
            let act = crate::models::StoreActivity { id: None, store_id: store_oid.clone(), product_id: Some(product_oid.clone()), kind: "price_set".into(), ts_ms, price: Some(price), product_name, store_name };
            let _ = state.store_activities.insert_one(act, None).await;
        }
    }

    match col.update_one(doc!{"_id": oid}, doc!{"$set": {"status": body.status}}, None).await {
        Ok(r) if r.matched_count>0 => StatusCode::NO_CONTENT.into_response(),
        Ok(_) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => { error!(?e, "update status failed"); StatusCode::INTERNAL_SERVER_ERROR.into_response() }
    }
}

#[derive(serde::Deserialize)]
pub struct OperationUpdateBody {
    #[serde(default)]
    pub store_id: Option<String>,
    #[serde(default)]
    pub items: Option<Vec<OperationItem>>,
}

pub async fn update_operation(State(state): State<AppState>, Path(id): Path<String>, Json(body): Json<OperationUpdateBody>) -> impl IntoResponse {
    let Ok(oid) = ObjectId::parse_str(&id) else { return StatusCode::BAD_REQUEST.into_response(); };
    let mut set = doc!{};
    if let Some(sid) = body.store_id {
        if !sid.is_empty() {
            if let Ok(soid) = ObjectId::parse_str(&sid) { set.insert("store_id", soid); }
        } else {
            set.insert("store_id", bson::Bson::Null);
        }
    }
    if let Some(items) = body.items { set.insert("items", bson::to_bson(&items).unwrap_or(bson::Bson::Null)); }
    if set.is_empty() { return StatusCode::BAD_REQUEST.into_response(); }
    let col = state.db.collection::<Operation>("operations");
    match col.update_one(doc!{"_id": oid}, doc!{"$set": set}, None).await {
        Ok(r) if r.matched_count>0 => StatusCode::NO_CONTENT.into_response(),
        Ok(_) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => { error!(?e, "update operation failed"); StatusCode::INTERNAL_SERVER_ERROR.into_response() }
    }
}

pub async fn delete_operation(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    let Ok(oid) = ObjectId::parse_str(&id) else { return StatusCode::BAD_REQUEST.into_response(); };
    let col = state.db.collection::<Operation>("operations");
    match col.delete_one(doc!{"_id": oid}, None).await {
        Ok(r) if r.deleted_count == 1 => StatusCode::NO_CONTENT.into_response(),
        Ok(_) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => { error!(?e, "delete operation failed"); StatusCode::INTERNAL_SERVER_ERROR.into_response() }
    }
}

use axum::{extract::{State, Path}, http::StatusCode, response::IntoResponse, Json};
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
        match col.find_one(doc!{"uploaded_by": user, "status": {"$ne": "deleted"}}, None).await {
            Ok(Some(_)) => return (StatusCode::CONFLICT, Json(serde_json::json!({"error": "user_has_operation"}))).into_response(),
            Ok(None) => {}
            Err(e) => { error!(?e, "find existing op failed"); return StatusCode::INTERNAL_SERVER_ERROR.into_response(); }
        }
    }
    if let Some(qr) = &body.qr {
        match col.find_one(doc!{"qr": qr}, None).await { Ok(Some(_)) => return (StatusCode::CONFLICT, Json(serde_json::json!({"error": "qr_used"}))).into_response(), _=>{} }
    }
    let op = Operation { id: None, date: body.date, seller: body.seller, amount: body.amount, items: body.items, status: "draft".into(), store_id: None, qr: body.qr, uploaded_by: body.uploaded_by, raw: body.raw.map(|v| bson::to_bson(&v).unwrap_or(bson::Bson::Null)) };
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

#[derive(serde::Deserialize)]
pub struct UpdateStatus { pub status: String }

pub async fn update_status(State(state): State<AppState>, Path(id): Path<String>, Json(body): Json<UpdateStatus>) -> impl IntoResponse {
    let Ok(oid) = ObjectId::parse_str(&id) else { return StatusCode::BAD_REQUEST.into_response(); };
    let col = state.db.collection::<Operation>("operations");
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

use axum::{extract::{Path, State}, http::StatusCode, response::IntoResponse, Json};
use bson::{doc, oid::ObjectId};
use futures::stream::StreamExt;
use tracing::error;
use argon2::{Argon2, password_hash::{PasswordHasher, SaltString}};

use crate::models::User;
use crate::state::AppState;

#[derive(serde::Deserialize)]
pub struct CreateUser { pub username: String, pub password: String }

#[derive(serde::Deserialize)]
pub struct UpdateUser { pub username: Option<String>, pub password: Option<String> }

#[derive(serde::Serialize)]
pub struct PublicUser { pub _id: ObjectId, pub username: String, pub role: String }

pub async fn list_users(State(state): State<AppState>) -> impl IntoResponse {
    let coll = state.db.collection::<User>("users");
    let mut cur = match coll.find(None, None).await { Ok(c)=>c, Err(e)=> { error!(?e, "users.find failed"); return StatusCode::INTERNAL_SERVER_ERROR.into_response(); } };
    let mut out: Vec<PublicUser> = Vec::new();
    while let Some(item) = cur.next().await { match item {
        Ok(u) => { if let Some(id)=u.id { out.push(PublicUser{ _id:id, username:u.username, role:u.role }) } },
        Err(e) => { error!(?e, "users cursor error"); return StatusCode::INTERNAL_SERVER_ERROR.into_response(); }
    }}
    Json(out).into_response()
}

pub async fn create_user(State(state): State<AppState>, Json(body): Json<CreateUser>) -> impl IntoResponse {
    if body.username.trim().is_empty() || body.password.len() < 4 { return StatusCode::BAD_REQUEST.into_response(); }
    let salt = SaltString::generate(&mut rand::thread_rng());
    let hash = match Argon2::default().hash_password(body.password.as_bytes(), &salt) { Ok(h)=>h.to_string(), Err(e)=> { error!(?e, "hash_password failed"); return StatusCode::INTERNAL_SERVER_ERROR.into_response(); } };
    let user = User { id: None, username: body.username, password_hash: hash, role: "user".into() };
    let coll = state.db.collection::<User>("users");
    match coll.insert_one(user, None).await {
        Ok(res) => {
            let id = match res.inserted_id { bson::Bson::ObjectId(oid)=> oid, _=> ObjectId::new() };
            Json(serde_json::json!({"_id": id})).into_response()
        }
        Err(e) => { error!(?e, "insert user failed"); StatusCode::CONFLICT.into_response() }
    }
}

pub async fn update_user(State(state): State<AppState>, Path(id): Path<String>, Json(body): Json<UpdateUser>) -> impl IntoResponse {
    let Ok(oid) = ObjectId::parse_str(&id) else { return StatusCode::BAD_REQUEST.into_response(); };
    let mut set = doc!{};
    if let Some(u) = body.username { set.insert("username", u); }
    if let Some(p) = body.password { 
        let salt = SaltString::generate(&mut rand::thread_rng());
        let hash = match Argon2::default().hash_password(p.as_bytes(), &salt) { Ok(h)=>h.to_string(), Err(e)=> { error!(?e, "hash_password failed"); return StatusCode::INTERNAL_SERVER_ERROR.into_response(); } };
        set.insert("password_hash", hash);
    }
    if set.is_empty() { return StatusCode::BAD_REQUEST.into_response(); }
    let coll = state.db.collection::<User>("users");
    match coll.update_one(doc!{"_id": oid}, doc!{"$set": set}, None).await {
        Ok(r) if r.matched_count>0 => StatusCode::NO_CONTENT.into_response(),
        Ok(_) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => { error!(?e, "update user failed"); StatusCode::INTERNAL_SERVER_ERROR.into_response() }
    }
}

pub async fn delete_user(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    let Ok(oid) = ObjectId::parse_str(&id) else { return StatusCode::BAD_REQUEST.into_response(); };
    let coll = state.db.collection::<User>("users");
    match coll.delete_one(doc!{"_id": oid}, None).await {
        Ok(r) if r.deleted_count==1 => StatusCode::NO_CONTENT.into_response(),
        Ok(_) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => { error!(?e, "delete user failed"); StatusCode::INTERNAL_SERVER_ERROR.into_response() }
    }
}


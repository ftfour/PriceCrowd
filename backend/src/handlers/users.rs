use axum::{extract::{Path, State}, http::StatusCode, response::IntoResponse, Json};
use bson::{doc, oid::ObjectId};
use futures::stream::StreamExt;
use tracing::error;
use argon2::{Argon2, password_hash::{PasswordHasher, SaltString}};

use crate::models::User;
use crate::state::AppState;
use axum::http::HeaderMap;
use jsonwebtoken::{DecodingKey, Validation, decode};
use chrono::Utc;
use crate::models::{Claims, TelegramLink};
use rand::{distributions::Alphanumeric, Rng};

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
    let user = User { id: None, username: body.username, password_hash: hash, role: "user".into(), telegram_id: None, telegram_username: None };
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

fn extract_username_from_auth(headers: &HeaderMap, secret: &str) -> Option<String> {
    let auth = headers.get(axum::http::header::AUTHORIZATION)?.to_str().ok()?;
    let token = auth.strip_prefix("Bearer ")?;
    let data = decode::<Claims>(token, &DecodingKey::from_secret(secret.as_bytes()), &Validation::default()).ok()?;
    Some(data.claims.sub)
}

#[derive(serde::Serialize)]
pub struct LinkStartResponse { pub code: String, pub exp_ms: i64 }

pub async fn start_telegram_link(State(state): State<AppState>, headers: HeaderMap) -> impl IntoResponse {
    let Some(username) = extract_username_from_auth(&headers, &state.jwt_secret) else { return StatusCode::UNAUTHORIZED.into_response(); };
    // generate code
    let code: String = rand::thread_rng().sample_iter(&Alphanumeric).take(6).map(char::from).collect::<String>().to_uppercase();
    let exp_ms = (Utc::now() + chrono::Duration::minutes(15)).timestamp_millis();
    let link = TelegramLink { id: None, code: code.clone(), username, exp_ms, used: false };
    match state.telegram_links.insert_one(link, None).await {
        Ok(_) => Json(LinkStartResponse { code, exp_ms }).into_response(),
        Err(e) => { error!(?e, "insert link failed"); StatusCode::INTERNAL_SERVER_ERROR.into_response() }
    }
}

#[derive(serde::Serialize)]
pub struct LinkStatus { pub linked: bool, pub telegram_id: Option<i64> }

pub async fn telegram_link_status(State(state): State<AppState>, headers: HeaderMap) -> impl IntoResponse {
    let Some(username) = extract_username_from_auth(&headers, &state.jwt_secret) else { return StatusCode::UNAUTHORIZED.into_response(); };
    let coll = state.db.collection::<User>("users");
    match coll.find_one(doc!{"username": &username}, None).await {
        Ok(Some(u)) => Json(LinkStatus { linked: u.telegram_id.is_some(), telegram_id: u.telegram_id }).into_response(),
        _ => StatusCode::NOT_FOUND.into_response(),
    }
}

pub async fn unlink_telegram(State(state): State<AppState>, headers: HeaderMap) -> impl IntoResponse {
    let Some(username) = extract_username_from_auth(&headers, &state.jwt_secret) else { return StatusCode::UNAUTHORIZED.into_response(); };
    let coll = state.db.collection::<User>("users");
    match coll.update_one(doc!{"username": &username}, doc!{"$unset": {"telegram_id": ""}}, None).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => { error!(?e, "unlink failed"); StatusCode::INTERNAL_SERVER_ERROR.into_response() }
    }
}

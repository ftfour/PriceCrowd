use axum::{extract::{State, Request as AxumRequest}, http::StatusCode, response::IntoResponse, Json};
use axum::middleware::Next;
use bson::doc;
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, Header, EncodingKey, decode, DecodingKey, Validation};
use tracing::error;
use argon2::{Argon2, password_hash::{PasswordHash, PasswordVerifier}};

use crate::models::{LoginRequest, LoginResponse, User, Claims};
use crate::state::AppState;

pub async fn login(State(state): State<AppState>, Json(body): Json<LoginRequest>) -> impl IntoResponse {
    let users = state.db.collection::<User>("users");
    let Some(user) = users.find_one(doc!{"username": &body.username}, None).await.unwrap_or(None) else {
        return StatusCode::UNAUTHORIZED.into_response();
    };
    let parsed = PasswordHash::new(&user.password_hash).ok();
    if parsed.is_none() { return StatusCode::UNAUTHORIZED.into_response(); }
    let ok = Argon2::default().verify_password(body.password.as_bytes(), &parsed.unwrap()).is_ok();
    if !ok { return StatusCode::UNAUTHORIZED.into_response(); }

    let exp = (Utc::now() + Duration::hours(24)).timestamp();
    let claims = Claims { sub: user.username.clone(), role: user.role.clone(), exp };
    let token = match encode(&Header::default(), &claims, &EncodingKey::from_secret(state.jwt_secret.as_bytes())) {
        Ok(t) => t,
        Err(e) => { error!(?e, "jwt encode error"); return StatusCode::INTERNAL_SERVER_ERROR.into_response(); }
    };
    Json(LoginResponse { token, username: user.username, role: user.role }).into_response()
}

pub async fn require_admin(State(state): State<AppState>, req: AxumRequest, next: Next) -> impl IntoResponse {
    let Some(auth) = req.headers().get(axum::http::header::AUTHORIZATION).and_then(|v| v.to_str().ok()) else {
        return StatusCode::UNAUTHORIZED.into_response();
    };
    let Some(token) = auth.strip_prefix("Bearer ") else { return StatusCode::UNAUTHORIZED.into_response(); };
    let validation = Validation::default();
    let decoded = decode::<Claims>(token, &DecodingKey::from_secret(state.jwt_secret.as_bytes()), &validation);
    let Ok(data) = decoded else { return StatusCode::UNAUTHORIZED.into_response(); };
    if data.claims.role != "admin" { return StatusCode::FORBIDDEN.into_response(); }
    next.run(req).await
}

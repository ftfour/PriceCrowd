use axum::{Router, routing::{get, post, put}};
use tower_http::cors::{Any, CorsLayer};
use axum::http::HeaderValue;
use tower_http::services::ServeDir;

use crate::handlers;
use crate::state::AppState;

pub fn build(state: AppState, uploads_dir: String) -> Router {
    let cors = match std::env::var("CORS_ALLOWED_ORIGINS") {
        Ok(list) => {
            let origins: Vec<HeaderValue> = list
                .split(',')
                .filter_map(|s| HeaderValue::from_str(s.trim()).ok())
                .collect();
            if origins.is_empty() { CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any) } else { CorsLayer::new().allow_origin(origins).allow_methods(Any).allow_headers(Any) }
        }
        Err(_) => CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any),
    };

    let static_service = ServeDir::new(&uploads_dir);

    let admin_guard = axum::middleware::from_fn_with_state(state.clone(), handlers::auth::require_admin);

    let public = Router::new()
        .route("/healthz", get(handlers::health::health))
        .route("/auth/login", post(handlers::auth::login))
        .route("/telegram/webhook", post(crate::telegram::webhook))
        .route("/users/link_telegram/start", post(handlers::users::start_telegram_link))
        .route("/users/link_telegram/status", get(handlers::users::telegram_link_status))
        .route("/users/link_telegram/unlink", post(handlers::users::unlink_telegram))
        .route("/products", get(handlers::products::list_products))
        .route("/products/:id", get(handlers::products::get_product))
        .route("/products/:id/insights", get(handlers::insights::list_product_insights))
        .route("/categories", get(handlers::categories::list_categories))
        .route("/categories/:id", get(handlers::categories::get_category))
        .route("/stores", get(handlers::stores::list_stores))
        .route("/stores/:id", get(handlers::stores::get_store))
        .route("/stores/:id/products", get(handlers::stores::list_store_products))
        .route("/stores/:id/products/insights", get(handlers::insights::list_store_product_insights))
        .route("/stores/:id/activities", get(handlers::activities::list_store_activities))
        .route("/activities", get(handlers::activities::list_all_activities))
        .route("/events", get(handlers::events::list_events))
        .route("/ratings/users", get(handlers::ratings::list_user_ratings))
        .nest_service("/uploads", static_service)
        .with_state(state.clone());

    let admin = Router::new()
        .route("/products", post(handlers::products::create_product))
        .route("/products/:id", put(handlers::products::update_product).delete(handlers::products::delete_product))
        .route("/products/:id/categories/:cat_id", post(handlers::products::add_product_category).delete(handlers::products::remove_product_category))
        .route("/categories", post(handlers::categories::create_category))
        .route("/categories/:id", put(handlers::categories::update_category).delete(handlers::categories::delete_category))
        .route("/stores", post(handlers::stores::create_store))
        .route("/stores/:id", put(handlers::stores::update_store).delete(handlers::stores::delete_store))
        .route("/stores/:id/products", post(handlers::stores::add_store_product))
        .route("/stores/:id/products/:product_id", put(handlers::stores::update_store_product).delete(handlers::stores::remove_store_product))
        .route("/settings/telegram", get(handlers::settings::get_telegram).put(handlers::settings::put_telegram))
        .route("/users", get(handlers::users::list_users).post(handlers::users::create_user))
        .route("/users/:id", put(handlers::users::update_user).delete(handlers::users::delete_user))
        .route("/settings/telegram/status", get(crate::telegram::status))
        .route("/dev/clear", post(handlers::dev::clear_all))
        .route("/ratings/grant", post(handlers::ratings::award_points))
        .route("/upload", post(handlers::uploads::upload_file))
        .with_state(state.clone())
        .layer(admin_guard);

    public.merge(admin).layer(cors)
}

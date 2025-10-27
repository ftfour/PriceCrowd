use axum::{Router, routing::{get, post, put}};
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;

use crate::handlers;
use crate::state::AppState;

pub fn build(state: AppState, uploads_dir: String) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let static_service = ServeDir::new(&uploads_dir);

    Router::new()
        .route("/healthz", get(handlers::health::health))
        .route("/products", get(handlers::products::list_products).post(handlers::products::create_product))
        .route("/products/:id", get(handlers::products::get_product).put(handlers::products::update_product).delete(handlers::products::delete_product))
        .route("/products/:id/insights", get(handlers::insights::list_product_insights))
        .route("/products/:id/categories/:cat_id", post(handlers::products::add_product_category).delete(handlers::products::remove_product_category))
        .route("/categories", get(handlers::categories::list_categories).post(handlers::categories::create_category))
        .route("/categories/:id", get(handlers::categories::get_category).put(handlers::categories::update_category).delete(handlers::categories::delete_category))
        .route("/stores", get(handlers::stores::list_stores).post(handlers::stores::create_store))
        .route("/stores/:id", get(handlers::stores::get_store).put(handlers::stores::update_store).delete(handlers::stores::delete_store))
        .route("/stores/:id/products", get(handlers::stores::list_store_products).post(handlers::stores::add_store_product))
        .route("/stores/:id/products/:product_id", put(handlers::stores::update_store_product).delete(handlers::stores::remove_store_product))
        .route("/stores/:id/products/insights", get(handlers::insights::list_store_product_insights))
        .route("/stores/:id/activities", get(handlers::activities::list_store_activities))
        .route("/upload", post(handlers::uploads::upload_file))
        .route("/activities", get(handlers::activities::list_all_activities))
        .nest_service("/uploads", static_service)
        .with_state(state)
        .layer(cors)
}


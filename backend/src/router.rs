use axum::Router;
use tower_http::cors::{Any, CorsLayer};
use axum::http::HeaderValue;
use axum::http::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE};

use crate::state::AppState;
use crate::services::{self, auth, receipt, price, telegram, fns};

// Централизованная сборка маршрутов. Сохраняем существующие пути (legacy)
// и добавляем логические namespaces (nest), которые пока не регистрируют
// новых путей, но задают архитектурные границы.
pub fn build_app(state: AppState, uploads_dir: String) -> Router {
    // Build a CORS layer for new-style routers (services/*)
    let cors = match std::env::var("CORS_ALLOWED_ORIGINS") {
        Ok(list) => {
            let origins: Vec<HeaderValue> = list
                .split(',')
                .filter_map(|s| HeaderValue::from_str(s.trim()).ok())
                .collect();
            if origins.is_empty() {
                CorsLayer::new()
                    .allow_origin(Any)
                    .allow_methods(Any)
                    .allow_headers([AUTHORIZATION, CONTENT_TYPE, ACCEPT])
            } else {
                CorsLayer::new()
                    .allow_origin(origins)
                    .allow_methods(Any)
                    .allow_headers([AUTHORIZATION, CONTENT_TYPE, ACCEPT])
            }
        }
        Err(_) => CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers([AUTHORIZATION, CONTENT_TYPE, ACCEPT]),
    };
    // Логические namespaces (пока пустые, чтобы не дублировать маршруты):
    let app = Router::new()
        .nest("/auth", auth::routes(state.clone()))
        .nest("/receipts", receipt::routes(state.clone()))
        .nest("/prices", price::routes(state.clone()))
        .nest("/telegram", telegram::routes(state.clone()))
        .nest("/fns", fns::routes(state.clone()))
        .layer(cors);

    // Сохраняем полный legacy-роутер — все текущие endpoints и функциональность
    // остаются без изменений (включая раздачу статики и CORS/guard'ы)
    let legacy = crate::routes::build(state, uploads_dir);

    app.merge(legacy)
}

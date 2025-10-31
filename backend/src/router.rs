use axum::Router;

use crate::state::AppState;
use crate::services::{self, auth, receipt, price, telegram};

// Централизованная сборка маршрутов. Сохраняем существующие пути (legacy)
// и добавляем логические namespaces (nest), которые пока не регистрируют
// новых путей, но задают архитектурные границы.
pub fn build_app(state: AppState, uploads_dir: String) -> Router {
    // Логические namespaces (пока пустые, чтобы не дублировать маршруты):
    let app = Router::new()
        .nest("/auth", auth::routes(state.clone()))
        .nest("/receipts", receipt::routes(state.clone()))
        .nest("/prices", price::routes(state.clone()))
        .nest("/telegram", telegram::routes(state.clone()));

    // Сохраняем полный legacy-роутер — все текущие endpoints и функциональность
    // остаются без изменений (включая раздачу статики и CORS/guard'ы)
    let legacy = crate::routes::build(state, uploads_dir);

    app.merge(legacy)
}

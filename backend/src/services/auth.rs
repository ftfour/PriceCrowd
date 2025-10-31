use axum::Router;

use crate::state::AppState;

// Публичный API сервиса авторизации
// Оставляем маршруты пустыми, чтобы не дублировать текущие endpoints (/auth/login)
// — они уже регистрируются в legacy-роутере. Этот модуль даёт точку расширения.
pub fn routes(_state: AppState) -> Router {
    // TODO: вынести в отдельный сервис позже (Auth/JWT)
    Router::new()
}

// Пробрасываем существующие обработчики для удобства использования из слоя сервисов
pub use crate::handlers::auth::{login, require_admin};

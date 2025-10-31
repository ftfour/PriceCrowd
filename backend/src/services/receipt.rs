use axum::Router;

use crate::state::AppState;

// Сервис работы с чеками/ФНС (пока заглушка)
// TODO: вынести в отдельный сервис позже (интеграция с ФНС)
pub fn routes(_state: AppState) -> Router {
    Router::new()
}

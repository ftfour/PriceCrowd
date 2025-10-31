use axum::Router;

use crate::state::AppState;

// Telegram сервис: webhook/polling/линковка
// Не дублируем маршруты, чтобы сохранить текущие пути из legacy-роутера.
pub fn routes(_state: AppState) -> Router {
    // TODO: вынести в отдельный сервис позже (интеграция Telegram)
    Router::new()
}

// Пробрасываем существующие функции для использования из main/router
pub use crate::telegram::{spawn_poller, webhook, status};

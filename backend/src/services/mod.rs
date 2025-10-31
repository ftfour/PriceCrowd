// Слой сервисов: логическая микро-декомпозиция поверх существующих handlers
// Ничего не ломаем — текущие пути и обработчики остаются. Здесь добавляем
// точки расширения и маршруты-обертки, которые можно вынести в отдельные сервисы позже.

pub mod common;
pub mod auth;
pub mod receipt;
pub mod price;
pub mod telegram;

// Экспорт подмодулей для удобного импорта в main/router
pub use auth::*;
pub use receipt::*;
pub use price::*;
pub use telegram::*;

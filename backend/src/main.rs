use std::net::SocketAddr;

use axum::Router;
use dotenvy::dotenv;
use tracing::info;

mod models;
mod state;
mod handlers;
mod routes; // legacy routes (сохраняем для обратной совместимости)
mod telegram; // legacy telegram module

mod db; // новый модуль для БД (логическая декомпозиция)
mod services; // слой сервисов
mod router; // новый сборщик маршрутов

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_target(false)
        .compact()
        .init();

    let port: u16 = std::env::var("PORT").ok().and_then(|s| s.parse().ok()).unwrap_or(8080);
    let uploads_dir = std::env::var("UPLOADS_DIR").unwrap_or_else(|_| "uploads".into());

    let state = state::init_from_env().await?;

    // Start Telegram poller (runs only when enabled in settings)
    // TODO: вынести в отдельный сервис позже (оркестрация интеграций)
    let _tg_handle = services::telegram::spawn_poller(state.clone());

    // Новый способ сборки маршрутов с логическими "микросервисными" неймспейсами
    // и сохранением всех текущих endpoint-ов без изменений путей
    let app: Router = router::build_app(state, uploads_dir);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("listening on http://{}", addr);
    axum::serve(tokio::net::TcpListener::bind(addr).await?, app).await?;
    Ok(())
}

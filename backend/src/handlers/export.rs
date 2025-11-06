use axum::{extract::State, response::IntoResponse, http::{HeaderMap, HeaderValue, header}};
use futures::stream::StreamExt;
use serde::Serialize;

use crate::{state::AppState, models::{Product, Store, Category, StoreItem, StoreActivity, User, Receipt, EventDoc, Operation}};

async fn collect_all<T: Unpin + Send + for<'de> serde::Deserialize<'de> + Serialize + Clone>(
    col: &mongodb::Collection<T>,
) -> Vec<T> {
    let mut out = Vec::new();
    let mut cur = match col.find(None, None).await { Ok(c)=> c, Err(_)=> return out };
    while let Some(n) = cur.next().await {
        if let Ok(doc) = n { out.push(doc); }
    }
    out
}

pub async fn export_all(State(state): State<AppState>) -> impl IntoResponse {
    // typed collections present in AppState
    let products = collect_all(&state.products).await;
    let stores = collect_all(&state.stores).await;
    let categories = collect_all(&state.categories).await;
    let store_items = collect_all(&state.store_items).await;
    let store_activities = collect_all(&state.store_activities).await;
    let settings: Vec<crate::models::TelegramSettingsDoc> = collect_all(&state.telegram_settings).await;
    let telegram_links: Vec<crate::models::TelegramLink> = collect_all(&state.telegram_links).await;

    // other collections loaded via db directly
    let users_col = state.db.collection::<User>("users");
    let users = collect_all(&users_col).await;
    let receipts_col = state.db.collection::<Receipt>("receipts");
    let receipts = collect_all(&receipts_col).await;
    let events_col = state.db.collection::<EventDoc>("events");
    let events = collect_all(&events_col).await;
    let operations_col = state.db.collection::<Operation>("operations");
    let operations = collect_all(&operations_col).await;

    let payload = serde_json::json!({
        "meta": {
            "generated_at": chrono::Utc::now().to_rfc3339(),
            "collections": {
                "products": products.len(),
                "stores": stores.len(),
                "categories": categories.len(),
                "store_items": store_items.len(),
                "store_activities": store_activities.len(),
                "settings": settings.len(),
                "telegram_links": telegram_links.len(),
                "users": users.len(),
                "receipts": receipts.len(),
                "events": events.len(),
                "operations": operations.len(),
            }
        },
        "products": products,
        "stores": stores,
        "categories": categories,
        "store_items": store_items,
        "store_activities": store_activities,
        "settings": settings,
        "telegram_links": telegram_links,
        "users": users,
        "receipts": receipts,
        "events": events,
        "operations": operations,
    });

    // Suggest download with filename
    let filename = format!("pricecrowd_export_{}.json", chrono::Utc::now().format("%Y%m%d_%H%M%S"));
    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, HeaderValue::from_static("application/json"));
    if let Ok(v) = HeaderValue::from_str(&format!("attachment; filename=\"{}\"", filename)) {
        headers.insert(header::CONTENT_DISPOSITION, v);
    }
    (headers, axum::Json(payload))
}


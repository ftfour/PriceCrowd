use axum::{extract::State, response::IntoResponse, http::{HeaderMap, HeaderValue, header}, Json};
use futures::stream::StreamExt;
use serde::Serialize;
use mongodb::bson::{doc, Document};

use crate::{state::AppState, models::{User, Receipt, EventDoc, Operation}};

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

#[derive(serde::Deserialize)]
pub struct ImportDump {
    #[serde(default)] products: Vec<Document>,
    #[serde(default)] stores: Vec<Document>,
    #[serde(default)] categories: Vec<Document>,
    #[serde(default)] store_items: Vec<Document>,
    #[serde(default)] store_activities: Vec<Document>,
    #[serde(default)] settings: Vec<Document>,
    #[serde(default)] telegram_links: Vec<Document>,
    #[serde(default)] users: Vec<Document>,
    #[serde(default)] receipts: Vec<Document>,
    #[serde(default)] events: Vec<Document>,
    #[serde(default)] operations: Vec<Document>,
}

async fn replace_collection(col: &mongodb::Collection<Document>, docs: &[Document]) -> mongodb::error::Result<u64> {
    let _ = col.delete_many(doc!{}, None).await?;
    if docs.is_empty() { return Ok(0); }
    let res = col.insert_many(docs.to_vec(), None).await?;
    Ok(res.inserted_ids.len() as u64)
}

pub async fn import_all(State(state): State<AppState>, Json(payload): Json<ImportDump>) -> impl IntoResponse {
    // Use raw Document to preserve _id/ObjectId data; assumes export format
    let products_col = state.db.collection::<Document>("products");
    let stores_col = state.db.collection::<Document>("stores");
    let categories_col = state.db.collection::<Document>("categories");
    let store_items_col = state.db.collection::<Document>("store_items");
    let store_activities_col = state.db.collection::<Document>("store_activities");
    let settings_col = state.db.collection::<Document>("settings");
    let telegram_links_col = state.db.collection::<Document>("telegram_links");
    let users_col = state.db.collection::<Document>("users");
    let receipts_col = state.db.collection::<Document>("receipts");
    let events_col = state.db.collection::<Document>("events");
    let operations_col = state.db.collection::<Document>("operations");

    let mut applied: Vec<(&str, u64)> = Vec::new();
    macro_rules! apply {
        ($label:literal, $col:expr, $docs:expr) => {
            match replace_collection(&$col, &$docs).await {
                Ok(cnt) => applied.push(($label, cnt)),
                Err(e) => return axum::response::Json(doc!{"status": "error", "message": format!("{}: {}", $label, e)}).into_response(),
            }
        };
    }

    apply!("products", products_col, payload.products);
    apply!("stores", stores_col, payload.stores);
    apply!("categories", categories_col, payload.categories);
    apply!("store_items", store_items_col, payload.store_items);
    apply!("store_activities", store_activities_col, payload.store_activities);
    apply!("settings", settings_col, payload.settings);
    apply!("telegram_links", telegram_links_col, payload.telegram_links);
    apply!("users", users_col, payload.users);
    apply!("receipts", receipts_col, payload.receipts);
    apply!("events", events_col, payload.events);
    apply!("operations", operations_col, payload.operations);

    axum::response::Json(doc!{
        "status": "ok",
        "imported": applied.iter().map(|(k,v)| doc!{"name": k, "count": (*v as i64)}).collect::<Vec<Document>>()
    }).into_response()
}

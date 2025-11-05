use axum::{extract::State, http::StatusCode, response::IntoResponse};
use bson::doc;
use tracing::error;

use crate::{state::AppState, models::{Receipt}};

pub async fn clear_all(State(state): State<AppState>) -> impl IntoResponse {
    // Danger: admin-guarded in routes.rs
    let db = &state.db;
    let mut ok = true;

    macro_rules! wipe_coll {
        ($name:literal) => {{
            let col = db.collection::<bson::Document>($name);
            if let Err(e) = col.delete_many(doc!{}, None).await { error!(?e, "clear {} failed", $name); ok = false; }
        }};
    }

    // Core domain
    wipe_coll!("products");
    wipe_coll!("categories");
    wipe_coll!("stores");
    wipe_coll!("store_items");
    wipe_coll!("store_activities");
    // Receipts and events
    let receipts = db.collection::<Receipt>("receipts");
    let _ = receipts.delete_many(doc!{}, None).await.map_err(|e| { error!(?e, "clear receipts failed"); ok = false; e });
    wipe_coll!("events");
    // Do NOT wipe users/settings by default

    if ok { StatusCode::NO_CONTENT } else { StatusCode::INTERNAL_SERVER_ERROR }
}


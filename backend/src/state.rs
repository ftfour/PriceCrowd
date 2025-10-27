use anyhow::Result;
use mongodb::{options::ClientOptions, Client, Collection};

use crate::models::{Product, Store, Category, StoreItem, StoreActivity};

#[derive(Clone)]
pub struct AppState {
    pub products: Collection<Product>,
    pub stores: Collection<Store>,
    pub categories: Collection<Category>,
    pub store_items: Collection<StoreItem>,
    pub store_activities: Collection<StoreActivity>,
}

pub async fn init_from_env() -> Result<AppState> {
    let mongo_uri = std::env::var("MONGODB_URI").unwrap_or_else(|_| "mongodb://localhost:27017".into());
    let db_name = std::env::var("DATABASE_NAME").unwrap_or_else(|_| "pricecrowd".into());

    let mut opts = ClientOptions::parse(mongo_uri).await?;
    opts.app_name = Some("pricecrowd-backend".into());
    let client = Client::with_options(opts)?;

    let db = client.database(&db_name);
    let products: Collection<Product> = db.collection("products");
    let stores: Collection<Store> = db.collection("stores");
    let categories: Collection<Category> = db.collection("categories");
    let store_items: Collection<StoreItem> = db.collection("store_items");
    let store_activities: Collection<StoreActivity> = db.collection("store_activities");

    Ok(AppState { products, stores, categories, store_items, store_activities })
}


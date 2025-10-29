use anyhow::Result;
use mongodb::{options::ClientOptions, Client, Collection, Database};
use tracing::info;
use argon2::{Argon2, password_hash::{PasswordHasher, SaltString}};

use crate::models::{Product, Store, Category, StoreItem, StoreActivity, TelegramSettingsDoc, TelegramLink};

#[derive(Clone)]
pub struct AppState {
    pub products: Collection<Product>,
    pub stores: Collection<Store>,
    pub categories: Collection<Category>,
    pub store_items: Collection<StoreItem>,
    pub store_activities: Collection<StoreActivity>,
    pub telegram_settings: Collection<TelegramSettingsDoc>,
    pub telegram_links: Collection<TelegramLink>,
    pub jwt_secret: String,
    pub db: Database,
}

pub async fn init_from_env() -> Result<AppState> {
    let mongo_uri = std::env::var("MONGODB_URI").unwrap_or_else(|_| "mongodb://localhost:27017".into());
    let db_name = std::env::var("DATABASE_NAME").unwrap_or_else(|_| "pricecrowd".into());
    let jwt_secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "dev-secret-change-me".into());

    let mut opts = ClientOptions::parse(mongo_uri).await?;
    opts.app_name = Some("pricecrowd-backend".into());
    let client = Client::with_options(opts)?;

    let db = client.database(&db_name);
    let products: Collection<Product> = db.collection("products");
    let stores: Collection<Store> = db.collection("stores");
    let categories: Collection<Category> = db.collection("categories");
    let store_items: Collection<StoreItem> = db.collection("store_items");
    let store_activities: Collection<StoreActivity> = db.collection("store_activities");
    let telegram_settings: Collection<TelegramSettingsDoc> = db.collection("settings");
    let telegram_links: Collection<TelegramLink> = db.collection("telegram_links");

    // seed admin if configured
    seed_admin(&db, &jwt_secret).await?;

    Ok(AppState { products, stores, categories, store_items, store_activities, telegram_settings, telegram_links, jwt_secret, db })
}

async fn seed_admin(db: &mongodb::Database, _jwt_secret: &str) -> Result<()> {
    use crate::models::User;
    let users: Collection<User> = db.collection("users");
    let username = match std::env::var("ADMIN_USERNAME") { Ok(v) => v, Err(_) => return Ok(()) };
    let password = match std::env::var("ADMIN_PASSWORD") { Ok(v) => v, Err(_) => return Ok(()) };
    // if exists, skip
    if users.find_one(bson::doc!{"username": &username}, None).await?.is_some() {
        return Ok(());
    }
    let salt = SaltString::generate(&mut rand::thread_rng());
    let hash = Argon2::default().hash_password(password.as_bytes(), &salt)
        .map_err(|e| anyhow::anyhow!(e.to_string()))?
        .to_string();
    let user = User { id: None, username, password_hash: hash, role: "admin".into(), telegram_id: None, telegram_username: None };
    users.insert_one(user, None).await?;
    info!("seeded admin user");
    Ok(())
}

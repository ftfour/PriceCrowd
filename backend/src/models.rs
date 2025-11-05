use serde::{Deserialize, Serialize};
use bson::oid::ObjectId;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Product {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub title: String,
    #[serde(default)]
    pub desc: String,
    #[serde(default)]
    pub image_url: Option<String>,
    #[serde(default)]
    pub category_ids: Vec<ObjectId>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProductCreate {
    pub title: String,
    pub desc: String,
    #[serde(default)]
    pub image_url: Option<String>,
    #[serde(default)]
    pub category_ids: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ProductUpdate {
    pub title: Option<String>,
    pub desc: Option<String>,
    pub image_url: Option<String>,
    pub category_ids: Option<Vec<String>>, // replace full set if provided
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Store {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub name: String,
    pub addr: String,
    #[serde(default)]
    pub desc: String,
    #[serde(default)]
    pub image_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StoreCreate {
    pub name: String,
    pub addr: String,
    #[serde(default)]
    pub desc: String,
    #[serde(default)]
    pub image_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct StoreUpdate {
    pub name: Option<String>,
    pub addr: Option<String>,
    pub desc: Option<String>,
    pub image_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Category {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub name: String,
    #[serde(default)]
    pub desc: String,
    #[serde(default)]
    pub parent_ids: Vec<ObjectId>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CategoryCreate {
    pub name: String,
    #[serde(default)]
    pub desc: String,
    #[serde(default)]
    pub parent_ids: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct CategoryUpdate {
    pub name: Option<String>,
    pub desc: Option<String>,
    pub parent_ids: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StoreItem {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub store_id: ObjectId,
    pub product_id: ObjectId,
    pub price: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StoreActivity {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub store_id: ObjectId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub product_id: Option<ObjectId>,
    pub kind: String, // item_added | price_set | price_updated | item_removed
    pub ts_ms: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub product_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub store_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StoreItemCreate { pub product_id: String, pub price: f64 }

#[derive(Debug, Serialize, Deserialize)]
pub struct StoreItemUpdate { pub price: f64 }

// Auth
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub username: String,
    pub password_hash: String,
    pub role: String, // "admin" | "user"
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub telegram_id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub telegram_username: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub points: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest { pub username: String, pub password: String }

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse { pub token: String, pub username: String, pub role: String }

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub role: String,
    pub exp: i64,
}

// Telegram link codes
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TelegramLink {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub code: String,
    pub username: String,
    pub exp_ms: i64,
    #[serde(default)]
    pub used: bool,
}

// Admin Settings
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TelegramSettingsDoc {
    #[serde(rename = "_id")]
    pub key: String, // fixed key: "telegram"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chat_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook_url: Option<String>,
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub webhook_enabled: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TelegramSettingsUpdate {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook_enabled: Option<bool>,
}

// Receipts (QR uploads)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Receipt {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub qr: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub source: String,
    pub user: String,
}

// Generic app events
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EventDoc {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub ts_ms: i64,
    pub kind: String, // receipt_uploaded | receipt_verified | user_registered | other
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
}

// Operations (created from receipts)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OperationItem {
    pub name: String,
    pub price: f64,
    pub quantity: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub product_id: Option<ObjectId>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Operation {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub date: String,
    pub seller: String,
    pub amount: f64,
    pub items: Vec<OperationItem>,
    pub status: String, // draft | posted | deleted
    #[serde(skip_serializing_if = "Option::is_none")]
    pub store_id: Option<ObjectId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub qr: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uploaded_by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw: Option<bson::Bson>,
}

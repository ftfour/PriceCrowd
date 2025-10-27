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


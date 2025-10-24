use std::{net::SocketAddr, str::FromStr};

use axum::{
    extract::{Multipart, Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post, put},
    Json, Router,
};
use bson::{doc, oid::ObjectId, Bson};
use dotenvy::dotenv;
use mongodb::{options::ClientOptions, Client, Collection};
use serde::{Deserialize, Serialize};
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;
use tracing::{error, info};
use futures::stream::StreamExt;
use uuid::Uuid;
use std::path::PathBuf;

#[derive(Clone)]
struct AppState {
    products: Collection<Product>,
    stores: Collection<Store>,
    categories: Collection<Category>,
    store_items: Collection<StoreItem>,
    store_activities: Collection<StoreActivity>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Product {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    id: Option<ObjectId>,
    title: String,
    desc: String,
    #[serde(default)]
    image_url: Option<String>,
    #[serde(default)]
    category_ids: Vec<ObjectId>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ProductCreate {
    title: String,
    desc: String,
    #[serde(default)]
    image_url: Option<String>,
    #[serde(default)]
    category_ids: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct ProductUpdate {
    title: Option<String>,
    desc: Option<String>,
    image_url: Option<String>,
    category_ids: Option<Vec<String>>, // replace full set if provided
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Store {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    id: Option<ObjectId>,
    name: String,
    addr: String,
    #[serde(default)]
    desc: String,
    #[serde(default)]
    image_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct StoreCreate {
    name: String,
    addr: String,
    #[serde(default)]
    desc: String,
    #[serde(default)]
    image_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct StoreUpdate {
    name: Option<String>,
    addr: Option<String>,
    desc: Option<String>,
    image_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Category {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    id: Option<ObjectId>,
    name: String,
    #[serde(default)]
    desc: String,
    #[serde(default)]
    parent_ids: Vec<ObjectId>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CategoryCreate {
    name: String,
    #[serde(default)]
    desc: String,
    #[serde(default)]
    parent_ids: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct CategoryUpdate {
    name: Option<String>,
    desc: Option<String>,
    parent_ids: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct StoreItem {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    id: Option<ObjectId>,
    store_id: ObjectId,
    product_id: ObjectId,
    price: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct StoreActivity {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    id: Option<ObjectId>,
    store_id: ObjectId,
    #[serde(skip_serializing_if = "Option::is_none")]
    product_id: Option<ObjectId>,
    kind: String, // item_added | price_set | price_updated | item_removed
    ts_ms: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    price: Option<f64>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_target(false)
        .compact()
        .init();

    let mongo_uri = std::env::var("MONGODB_URI").unwrap_or_else(|_| "mongodb://localhost:27017".into());
    let db_name = std::env::var("DATABASE_NAME").unwrap_or_else(|_| "pricecrowd".into());
    let port: u16 = std::env::var("PORT").ok().and_then(|s| s.parse().ok()).unwrap_or(8080);

    // MongoDB client
    let mut opts = ClientOptions::parse(mongo_uri).await?;
    // reasonable defaults
    opts.app_name = Some("pricecrowd-backend".into());
    let client = Client::with_options(opts)?;
    let db = client.database(&db_name);
    let products: Collection<Product> = db.collection("products");
    let stores: Collection<Store> = db.collection("stores");
    let categories: Collection<Category> = db.collection("categories");
    let store_items: Collection<StoreItem> = db.collection("store_items");
    let store_activities: Collection<StoreActivity> = db.collection("store_activities");

    let state = AppState { products, stores, categories, store_items, store_activities };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let uploads_dir = std::env::var("UPLOADS_DIR").unwrap_or_else(|_| "uploads".into());
    let static_service = ServeDir::new(&uploads_dir);

    let app = Router::new()
        .route("/healthz", get(health))
        .route("/products", get(list_products).post(create_product))
        .route("/products/:id", get(get_product).put(update_product).delete(delete_product))
        .route("/products/:id/categories/:cat_id", post(add_product_category).delete(remove_product_category))
        .route("/categories", get(list_categories).post(create_category))
        .route("/categories/:id", get(get_category).put(update_category).delete(delete_category))
        .route("/stores", get(list_stores).post(create_store))
        .route("/stores/:id", get(get_store).put(update_store).delete(delete_store))
        .route("/stores/:id/products", get(list_store_products).post(add_store_product))
        .route("/stores/:id/products/:product_id", put(update_store_product).delete(remove_store_product))
        .route("/stores/:id/activities", get(list_store_activities))
        .route("/upload", post(upload_file))
        .nest_service("/uploads", static_service)
        .with_state(state)
        .layer(cors);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("listening on http://{}", addr);
    axum::serve(tokio::net::TcpListener::bind(addr).await?, app).await?;
    Ok(())
}

async fn health() -> impl IntoResponse { (StatusCode::OK, "ok") }

async fn list_products(State(state): State<AppState>) -> impl IntoResponse {
    let mut cursor = match state.products.find(None, None).await {
        Ok(c) => c,
        Err(e) => {
            error!(?e, "failed to query products");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let mut items: Vec<Product> = Vec::new();
    while let Some(res) = cursor.next().await {
        match res {
            Ok(doc) => items.push(doc),
            Err(e) => {
                error!(?e, "cursor error");
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
        }
    }
    (StatusCode::OK, Json(items)).into_response()
}

async fn get_product(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    let Ok(oid) = ObjectId::from_str(&id) else { return StatusCode::BAD_REQUEST.into_response(); };
    let filter = doc! {"_id": oid};
    match state.products.find_one(filter, None).await {
        Ok(Some(doc)) => (StatusCode::OK, Json(doc)).into_response(),
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            error!(?e, "find_one failed");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

async fn create_product(State(state): State<AppState>, Json(payload): Json<ProductCreate>) -> impl IntoResponse {
    // map category ids
    let mut cat_ids: Vec<ObjectId> = Vec::new();
    for s in payload.category_ids.iter() {
        if let Ok(oid) = ObjectId::from_str(s) { cat_ids.push(oid); }
    }
    let product = Product {
        id: None,
        title: payload.title,
        desc: payload.desc,
        image_url: payload.image_url,
        category_ids: cat_ids,
    };

    match state.products.insert_one(product, None).await {
        Ok(result) => {
            let id = match result.inserted_id {
                Bson::ObjectId(oid) => oid,
                _ => ObjectId::new(),
            };
            let filter = doc! {"_id": id};
            match state.products.find_one(filter, None).await {
                Ok(Some(created)) => (StatusCode::CREATED, Json(created)).into_response(),
                _ => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            }
        }
        Err(e) => {
            error!(?e, "insert failed");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

async fn update_product(State(state): State<AppState>, Path(id): Path<String>, Json(patch): Json<ProductUpdate>) -> impl IntoResponse {
    let Ok(oid) = ObjectId::from_str(&id) else { return StatusCode::BAD_REQUEST.into_response(); };
    let mut set = doc! {};
    if let Some(t) = patch.title { set.insert("title", t); }
    if let Some(d) = patch.desc { set.insert("desc", d); }
    if let Some(iu) = patch.image_url { set.insert("image_url", iu); }
    if let Some(cats) = patch.category_ids {
        let mut list: Vec<ObjectId> = Vec::new();
        for s in cats.iter() { if let Ok(oid) = ObjectId::from_str(s) { list.push(oid); } }
        set.insert("category_ids", list);
    }
    if set.is_empty() { return StatusCode::BAD_REQUEST.into_response(); }

    let filter = doc! {"_id": oid};
    let update = doc! {"$set": set};
    match state.products.find_one_and_update(filter.clone(), update, None).await {
        Ok(Some(updated)) => (StatusCode::OK, Json(updated)).into_response(),
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            error!(?e, "update failed");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

async fn delete_product(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    let Ok(oid) = ObjectId::from_str(&id) else { return StatusCode::BAD_REQUEST.into_response(); };
    let filter = doc! {"_id": oid};
    match state.products.delete_one(filter, None).await {
        Ok(res) if res.deleted_count == 1 => StatusCode::NO_CONTENT.into_response(),
        Ok(_) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            error!(?e, "delete failed");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

// Manage product categories
async fn add_product_category(State(state): State<AppState>, Path((id, cat_id)): Path<(String, String)>) -> impl IntoResponse {
    let (Ok(oid), Ok(cid)) = (ObjectId::from_str(&id), ObjectId::from_str(&cat_id)) else { return StatusCode::BAD_REQUEST.into_response(); };
    let filter = doc! {"_id": oid};
    let update = doc! {"$addToSet": {"category_ids": cid}};
    match state.products.update_one(filter, update, None).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => { error!(?e, "add category failed"); StatusCode::INTERNAL_SERVER_ERROR.into_response() }
    }
}

async fn remove_product_category(State(state): State<AppState>, Path((id, cat_id)): Path<(String, String)>) -> impl IntoResponse {
    let (Ok(oid), Ok(cid)) = (ObjectId::from_str(&id), ObjectId::from_str(&cat_id)) else { return StatusCode::BAD_REQUEST.into_response(); };
    let filter = doc! {"_id": oid};
    let update = doc! {"$pull": {"category_ids": cid}};
    match state.products.update_one(filter, update, None).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => { error!(?e, "remove category failed"); StatusCode::INTERNAL_SERVER_ERROR.into_response() }
    }
}

// Categories CRUD
async fn list_categories(State(state): State<AppState>) -> impl IntoResponse {
    let mut cursor = match state.categories.find(None, None).await { Ok(c)=>c, Err(e)=> { error!(?e, "query categories failed"); return StatusCode::INTERNAL_SERVER_ERROR.into_response(); } };
    let mut items: Vec<Category> = Vec::new();
    while let Some(res) = cursor.next().await { match res { Ok(doc)=> items.push(doc), Err(e)=> { error!(?e, "cursor error"); return StatusCode::INTERNAL_SERVER_ERROR.into_response(); } } }
    (StatusCode::OK, Json(items)).into_response()
}

async fn get_category(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    let Ok(oid) = ObjectId::from_str(&id) else { return StatusCode::BAD_REQUEST.into_response(); };
    let filter = doc! {"_id": oid};
    match state.categories.find_one(filter, None).await {
        Ok(Some(doc)) => (StatusCode::OK, Json(doc)).into_response(),
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => { error!(?e, "find category failed"); StatusCode::INTERNAL_SERVER_ERROR.into_response() }
    }
}

async fn create_category(State(state): State<AppState>, Json(payload): Json<CategoryCreate>) -> impl IntoResponse {
    let mut parents: Vec<ObjectId> = Vec::new();
    for s in payload.parent_ids.iter() { if let Ok(oid) = ObjectId::from_str(s) { parents.push(oid); } }
    let cat = Category { id: None, name: payload.name, desc: payload.desc, parent_ids: parents };
    match state.categories.insert_one(cat, None).await {
        Ok(result) => {
            let id = match result.inserted_id { Bson::ObjectId(oid) => oid, _ => ObjectId::new() };
            let filter = doc! {"_id": id};
            match state.categories.find_one(filter, None).await { Ok(Some(created)) => (StatusCode::CREATED, Json(created)).into_response(), _ => StatusCode::INTERNAL_SERVER_ERROR.into_response() }
        }
        Err(e) => { error!(?e, "insert category failed"); StatusCode::INTERNAL_SERVER_ERROR.into_response() }
    }
}

async fn update_category(State(state): State<AppState>, Path(id): Path<String>, Json(patch): Json<CategoryUpdate>) -> impl IntoResponse {
    let Ok(oid) = ObjectId::from_str(&id) else { return StatusCode::BAD_REQUEST.into_response(); };
    let mut set = doc! {};
    if let Some(n) = patch.name { set.insert("name", n); }
    if let Some(d) = patch.desc { set.insert("desc", d); }
    if let Some(pids) = patch.parent_ids { let mut v: Vec<ObjectId> = Vec::new(); for s in pids.iter(){ if let Ok(oo)=ObjectId::from_str(s){ v.push(oo);} } set.insert("parent_ids", v);}    
    if set.is_empty() { return StatusCode::BAD_REQUEST.into_response(); }
    let filter = doc! {"_id": oid};
    let update = doc! {"$set": set};
    match state.categories.find_one_and_update(filter, update, None).await {
        Ok(Some(updated)) => (StatusCode::OK, Json(updated)).into_response(),
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => { error!(?e, "update category failed"); StatusCode::INTERNAL_SERVER_ERROR.into_response() }
    }
}

async fn delete_category(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    let Ok(oid) = ObjectId::from_str(&id) else { return StatusCode::BAD_REQUEST.into_response(); };
    let filter = doc! {"_id": oid};
    match state.categories.delete_one(filter, None).await {
        Ok(res) if res.deleted_count == 1 => StatusCode::NO_CONTENT.into_response(),
        Ok(_) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => { error!(?e, "delete category failed"); StatusCode::INTERNAL_SERVER_ERROR.into_response() }
    }
}

async fn list_stores(State(state): State<AppState>) -> impl IntoResponse {
    let mut cursor = match state.stores.find(None, None).await {
        Ok(c) => c,
        Err(e) => {
            error!(?e, "failed to query stores");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    let mut items: Vec<Store> = Vec::new();
    while let Some(res) = cursor.next().await {
        match res {
            Ok(doc) => items.push(doc),
            Err(e) => {
                error!(?e, "cursor error");
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
        }
    }
    (StatusCode::OK, Json(items)).into_response()
}

async fn get_store(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    let Ok(oid) = ObjectId::from_str(&id) else { return StatusCode::BAD_REQUEST.into_response(); };
    let filter = doc! {"_id": oid};
    match state.stores.find_one(filter, None).await {
        Ok(Some(doc)) => (StatusCode::OK, Json(doc)).into_response(),
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            error!(?e, "find_one store failed");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

async fn create_store(State(state): State<AppState>, Json(payload): Json<StoreCreate>) -> impl IntoResponse {
    let store = Store {
        id: None,
        name: payload.name,
        addr: payload.addr,
        desc: payload.desc,
        image_url: payload.image_url,
    };
    match state.stores.insert_one(store, None).await {
        Ok(result) => {
            let id = match result.inserted_id { Bson::ObjectId(oid) => oid, _ => ObjectId::new() };
            let filter = doc! {"_id": id};
            match state.stores.find_one(filter, None).await {
                Ok(Some(created)) => (StatusCode::CREATED, Json(created)).into_response(),
                _ => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            }
        }
        Err(e) => {
            error!(?e, "insert store failed");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

async fn update_store(State(state): State<AppState>, Path(id): Path<String>, Json(patch): Json<StoreUpdate>) -> impl IntoResponse {
    let Ok(oid) = ObjectId::from_str(&id) else { return StatusCode::BAD_REQUEST.into_response(); };
    let mut set = doc! {};
    if let Some(v) = patch.name { set.insert("name", v); }
    if let Some(v) = patch.addr { set.insert("addr", v); }
    if let Some(v) = patch.desc { set.insert("desc", v); }
    if let Some(v) = patch.image_url { set.insert("image_url", v); }
    if set.is_empty() { return StatusCode::BAD_REQUEST.into_response(); }
    let filter = doc! {"_id": oid};
    let update = doc! {"$set": set};
    match state.stores.find_one_and_update(filter, update, None).await {
        Ok(Some(updated)) => (StatusCode::OK, Json(updated)).into_response(),
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            error!(?e, "update store failed");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

async fn delete_store(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    let Ok(oid) = ObjectId::from_str(&id) else { return StatusCode::BAD_REQUEST.into_response(); };
    let filter = doc! {"_id": oid};
    match state.stores.delete_one(filter, None).await {
        Ok(res) if res.deleted_count == 1 => StatusCode::NO_CONTENT.into_response(),
        Ok(_) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            error!(?e, "delete store failed");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

// Store items (per-store product price)
async fn list_store_products(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    let Ok(store_oid) = ObjectId::from_str(&id) else { return StatusCode::BAD_REQUEST.into_response(); };
    let mut cursor = match state.store_items.find(doc!{"store_id": store_oid}, None).await { Ok(c)=>c, Err(e)=> { error!(?e, "query store_items failed"); return StatusCode::INTERNAL_SERVER_ERROR.into_response(); } };
    let mut items: Vec<StoreItem> = Vec::new();
    while let Some(res) = cursor.next().await { match res { Ok(doc)=> items.push(doc), Err(e)=> { error!(?e, "cursor error"); return StatusCode::INTERNAL_SERVER_ERROR.into_response(); } } }
    // collect product ids
    let pids: Vec<ObjectId> = items.iter().map(|it| it.product_id.clone()).collect();
    let mut products_map = std::collections::HashMap::new();
    if !pids.is_empty() {
        let mut pcursor = match state.products.find(doc!{"_id": {"$in": &pids}}, None).await { Ok(c)=>c, Err(e)=> { error!(?e, "query products for store items failed"); return StatusCode::INTERNAL_SERVER_ERROR.into_response(); } };
        while let Some(res) = pcursor.next().await { match res { Ok(p)=> { if let Some(id) = p.id { products_map.insert(id, p); } }, Err(e)=> { error!(?e, "products cursor error"); return StatusCode::INTERNAL_SERVER_ERROR.into_response(); } } }
    }
    let payload: Vec<serde_json::Value> = items.into_iter().map(|it| {
        let prod = products_map.get(&it.product_id);
        serde_json::json!({
            "_id": it.id,
            "product_id": it.product_id,
            "price": it.price,
            "product": prod,
        })
    }).collect();
    (StatusCode::OK, Json(payload)).into_response()
}

#[derive(Debug, Serialize, Deserialize)]
struct StoreItemCreate { product_id: String, price: f64 }

async fn add_store_product(State(state): State<AppState>, Path(id): Path<String>, Json(payload): Json<StoreItemCreate>) -> impl IntoResponse {
    let (Ok(store_oid), Ok(product_oid)) = (ObjectId::from_str(&id), ObjectId::from_str(&payload.product_id)) else { return StatusCode::BAD_REQUEST.into_response(); };
    // Upsert: if item exists, update price; otherwise insert
    let filter = doc!{"store_id": store_oid, "product_id": product_oid};
    let update = doc!{"$set": {"store_id": store_oid, "product_id": product_oid, "price": payload.price}};
    match state.store_items.update_one(filter.clone(), update, mongodb::options::UpdateOptions::builder().upsert(true).build()).await {
        Ok(_) => {
            let now_ms = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis() as i64;
            let activity = StoreActivity { id: None, store_id: store_oid, product_id: Some(product_oid), kind: "price_set".to_string(), ts_ms: now_ms, price: Some(payload.price) };
            let _ = state.store_activities.insert_one(activity, None).await;
            // return the joined item
            match state.products.find_one(doc!{"_id": product_oid}, None).await {
                Ok(prod_opt) => {
                    let resp = serde_json::json!({"product_id": product_oid, "price": payload.price, "product": prod_opt});
                    (StatusCode::CREATED, Json(resp)).into_response()
                }
                Err(_) => StatusCode::CREATED.into_response(),
            }
        }
        Err(e) => { error!(?e, "upsert store item failed"); StatusCode::INTERNAL_SERVER_ERROR.into_response() }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct StoreItemUpdate { price: f64 }

async fn update_store_product(State(state): State<AppState>, Path((id, product_id)): Path<(String, String)>, Json(body): Json<StoreItemUpdate>) -> impl IntoResponse {
    let (Ok(store_oid), Ok(product_oid)) = (ObjectId::from_str(&id), ObjectId::from_str(&product_id)) else { return StatusCode::BAD_REQUEST.into_response(); };
    let filter = doc!{"store_id": store_oid, "product_id": product_oid};
    let update = doc!{"$set": {"price": body.price}};
    match state.store_items.update_one(filter, update, None).await {
        Ok(res) if res.matched_count > 0 => {
            let now_ms = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis() as i64;
            let activity = StoreActivity { id: None, store_id: store_oid, product_id: Some(product_oid), kind: "price_updated".to_string(), ts_ms: now_ms, price: Some(body.price) };
            let _ = state.store_activities.insert_one(activity, None).await;
            StatusCode::NO_CONTENT.into_response()
        },
        Ok(_) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => { error!(?e, "update store product failed"); StatusCode::INTERNAL_SERVER_ERROR.into_response() }
    }
}

async fn remove_store_product(State(state): State<AppState>, Path((id, product_id)): Path<(String, String)>) -> impl IntoResponse {
    let (Ok(store_oid), Ok(product_oid)) = (ObjectId::from_str(&id), ObjectId::from_str(&product_id)) else { return StatusCode::BAD_REQUEST.into_response(); };
    let filter = doc!{"store_id": store_oid, "product_id": product_oid};
    match state.store_items.delete_one(filter, None).await {
        Ok(res) if res.deleted_count == 1 => {
            let now_ms = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis() as i64;
            let activity = StoreActivity { id: None, store_id: store_oid, product_id: Some(product_oid), kind: "item_removed".to_string(), ts_ms: now_ms, price: None };
            let _ = state.store_activities.insert_one(activity, None).await;
            StatusCode::NO_CONTENT.into_response()
        },
        Ok(_) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => { error!(?e, "remove store product failed"); StatusCode::INTERNAL_SERVER_ERROR.into_response() }
    }
}

// List store activities
async fn list_store_activities(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    let Ok(store_oid) = ObjectId::from_str(&id) else { return StatusCode::BAD_REQUEST.into_response(); };
    // optional query param limit
    // Axum 0.7 extract query easily; for brevity, use default 20
    let mut cursor = match state.store_activities.find(doc!{"store_id": store_oid}, mongodb::options::FindOptions::builder().sort(doc!{"ts_ms": -1}).limit(50).build()).await {
        Ok(c)=>c,
        Err(e)=> { error!(?e, "query activities failed"); return StatusCode::INTERNAL_SERVER_ERROR.into_response(); }
    };
    let mut items: Vec<StoreActivity> = Vec::new();
    while let Some(res) = cursor.next().await { match res { Ok(doc)=> items.push(doc), Err(e)=> { error!(?e, "cursor error"); return StatusCode::INTERNAL_SERVER_ERROR.into_response(); } } }
    (StatusCode::OK, Json(items)).into_response()
}

async fn upload_file(mut multipart: Multipart) -> impl IntoResponse {
    // Expect a single field named "file"
    while let Some(field) = multipart.next_field().await.unwrap_or(None) {
        let name = field.name().map(|s| s.to_string());
        if name.as_deref() != Some("file") {
            continue;
        }
        let file_name = field.file_name().map(|s| s.to_string()).unwrap_or_else(|| "upload.bin".to_string());
        let bytes = match field.bytes().await {
            Ok(b) => b,
            Err(e) => {
                error!(?e, "read multipart bytes failed");
                return StatusCode::BAD_REQUEST.into_response();
            }
        };

        // Ensure uploads dir exists
        let uploads_dir = std::env::var("UPLOADS_DIR").unwrap_or_else(|_| "uploads".into());
        let _ = tokio::fs::create_dir_all(&uploads_dir).await;

        let ext = std::path::Path::new(&file_name).extension().and_then(|s| s.to_str()).unwrap_or("");
        let unique = format!("{}{}{}",
            Uuid::new_v4(),
            if ext.is_empty() { "" } else { "." },
            ext
        );
        let mut path = PathBuf::from(&uploads_dir);
        path.push(&unique);
        match tokio::fs::write(&path, &bytes).await {
            Ok(_) => {
                let url = format!("/uploads/{}", unique);
                let body = serde_json::json!({"url": url});
                return (StatusCode::OK, Json(body)).into_response();
            }
            Err(e) => {
                error!(?e, "write upload failed");
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
        }
    }
    StatusCode::BAD_REQUEST.into_response()
}

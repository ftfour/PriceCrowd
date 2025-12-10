use axum::{extract::State, http::StatusCode, response::IntoResponse};
use bson::{doc, oid::ObjectId};
use rand::Rng;
use tracing::{error, info};

use crate::{
    models::{Category, Product, Store},
    state::AppState,
};

pub async fn clear_test_data(State(state): State<AppState>) -> impl IntoResponse {
    // Danger: admin-guarded in routes.rs
    let db = &state.db;
    let mut ok = true;
    let filter = doc! { "is_test": true };

    macro_rules! wipe_coll {
        ($name:literal) => {{
            let col = db.collection::<bson::Document>($name);
            if let Err(e) = col.delete_many(filter.clone(), None).await {
                error!(?e, "clear test data in {} failed", $name);
                ok = false;
            }
        }};
    }

    info!("Clearing test data");
    // Core domain
    wipe_coll!("products");
    wipe_coll!("categories");
    wipe_coll!("stores");
    wipe_coll!("store_items");
    wipe_coll!("store_activities");
    // Do NOT wipe users/settings/receipts/events by default
    info!("Finished clearing test data");

    if ok {
        StatusCode::NO_CONTENT
    } else {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

pub async fn seed(State(state): State<AppState>) -> impl IntoResponse {
    tokio::spawn(async move {
        info!("Started seeding test data");
        let db = &state.db;

        // Create test categories
        let cat_coll = db.collection::<bson::Document>("categories");
        let mut category_ids = vec![];
        for i in 1..=20 {
            let mut doc = bson::to_document(&Category {
                id: None,
                name: format!("Тест-категория {}", i),
                desc: "".to_string(),
                parent_ids: vec![],
            })
            .unwrap();
            doc.insert("is_test", true);
            if let Ok(res) = cat_coll.insert_one(doc, None).await {
                if let Some(id) = res.inserted_id.as_object_id() {
                    category_ids.push(id.clone());
                }
            }
        }
        info!("Created {} test categories", category_ids.len());

        // Create test stores
        let store_coll = db.collection::<bson::Document>("stores");
        let mut store_ids = vec![];
        for i in 1..=10 {
            let mut doc = bson::to_document(&Store {
                id: None,
                name: format!("Тест-магазин {}", i),
                addr: format!("Тестовый город, ул. Тестовая {}", i),
                desc: "".to_string(),
                image_url: None,
            })
            .unwrap();
            doc.insert("is_test", true);
            if let Ok(res) = store_coll.insert_one(doc, None).await {
                if let Some(id) = res.inserted_id.as_object_id() {
                    store_ids.push(id.clone());
                }
            }
        }
        info!("Created {} test stores", store_ids.len());

        // Create test products
        let prod_coll = db.collection::<bson::Document>("products");
        let mut product_ids = vec![];
        for i in 1..=500 {
            let cat_id = if !category_ids.is_empty() {
                let mut rng = rand::thread_rng();
                let idx = rng.gen_range(0..category_ids.len());
                vec![category_ids[idx].clone()]
            } else {
                vec![]
            };
            let mut doc = bson::to_document(&Product {
                id: None,
                title: format!("Тест-товар {}", i),
                desc: "Описание тестового товара".to_string(),
                image_url: None,
                category_ids: cat_id,
            })
            .unwrap();
            doc.insert("is_test", true);
            if let Ok(res) = prod_coll.insert_one(doc, None).await {
                if let Some(id) = res.inserted_id.as_object_id() {
                    product_ids.push(id.clone());
                }
            }
        }
        info!("Created {} test products", product_ids.len());

        // Create store items
        let store_item_coll = db.collection::<bson::Document>("store_items");
        let mut count = 0;
        if !store_ids.is_empty() && !product_ids.is_empty() {
            for store_id in &store_ids {
                // Not all products in all stores
                let products_in_store: Vec<&ObjectId> = product_ids.iter().filter(|_| rand::thread_rng().gen_bool(0.7)).collect();
                for product_id in products_in_store {
                    let price = {
                        let mut rng = rand::thread_rng();
                        let price_float: f64 = rng.gen_range(50.0..1000.0);
                        price_float.trunc() / 100.0 * 100.0 - 0.1 // xx.9
                    };
                    let doc = doc! {
                        "store_id": store_id,
                        "product_id": product_id,
                        "price": price,
                        "is_test": true,
                    };
                    if store_item_coll.insert_one(doc, None).await.is_ok() {
                        count += 1;
                    }
                }
            }
        }
        info!("Created {} test store items", count);
        info!("Finished seeding test data");
    });

    StatusCode::ACCEPTED
}

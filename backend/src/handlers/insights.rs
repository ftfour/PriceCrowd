use std::collections::{HashMap, HashSet};
use std::str::FromStr;

use axum::{extract::{Path, State}, http::StatusCode, response::IntoResponse, Json};
use bson::{doc, oid::ObjectId};
use futures::stream::StreamExt;
use tracing::error;

use crate::models::StoreItem;
use crate::state::AppState;

// Product-centric insights: list stores carrying the product with current price and per-store price history; also city stats
pub async fn list_product_insights(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    let Ok(pid) = ObjectId::from_str(&id) else { return StatusCode::BAD_REQUEST.into_response(); };
    // current prices in stores
    let mut cursor = match state.store_items.find(doc!{"product_id": pid.clone()}, None).await { Ok(c)=>c, Err(e)=> { error!(?e, "query store_items by product failed"); return StatusCode::INTERNAL_SERVER_ERROR.into_response(); } };
    let mut stores_prices: Vec<(ObjectId, f64)> = Vec::new();
    let mut store_ids: HashSet<ObjectId> = HashSet::new();
    while let Some(res) = cursor.next().await {
        match res {
            Ok(it) => { stores_prices.push((it.store_id, it.price)); store_ids.insert(it.store_id); }
            Err(e) => { error!(?e, "cursor error"); return StatusCode::INTERNAL_SERVER_ERROR.into_response(); }
        }
    }
    // store names
    let mut stores_map: HashMap<ObjectId, String> = HashMap::new();
    if !store_ids.is_empty() {
        let mut sc = match state.stores.find(doc!{"_id": {"$in": store_ids.iter().collect::<Vec<_>>() }}, None).await { Ok(c)=>c, Err(e)=> { error!(?e, "query stores failed"); return StatusCode::INTERNAL_SERVER_ERROR.into_response(); } };
        while let Some(res) = sc.next().await { match res { Ok(s)=> { if let Some(sid)=s.id { stores_map.insert(sid, s.name); } }, Err(e)=> { error!(?e, "stores cursor error"); return StatusCode::INTERNAL_SERVER_ERROR.into_response(); } } }
    }
    // per-store history from activities
    let mut acts_cursor = match state.store_activities.find(doc!{"product_id": pid.clone()}, mongodb::options::FindOptions::builder().sort(doc!{"ts_ms": 1}).build()).await { Ok(c)=>c, Err(e)=> { error!(?e, "query activities failed"); return StatusCode::INTERNAL_SERVER_ERROR.into_response(); } };
    let mut history_by_store: HashMap<ObjectId, Vec<(i64,f64)>> = HashMap::new();
    while let Some(res) = acts_cursor.next().await {
        match res {
            Ok(a) => {
                if let (Some(sid), Some(price)) = (Some(a.store_id), a.price) { history_by_store.entry(sid).or_default().push((a.ts_ms, price)); }
            }
            Err(e) => { error!(?e, "activities cursor error"); return StatusCode::INTERNAL_SERVER_ERROR.into_response(); }
        }
    }
    // city stats
    let count = stores_prices.len() as u64;
    let sum: f64 = stores_prices.iter().map(|(_,p)| *p).sum();
    let city_avg = if count>0 { Some(sum / count as f64) } else { None };
    let min_store = stores_prices.iter().min_by(|a,b| a.1.partial_cmp(&b.1).unwrap()).cloned();
    let max_store = stores_prices.iter().max_by(|a,b| a.1.partial_cmp(&b.1).unwrap()).cloned();

    let min_json = min_store.map(|(sid,price)| serde_json::json!({"store_id": sid, "store_name": stores_map.get(&sid), "price": price}));
    let max_json = max_store.map(|(sid,price)| serde_json::json!({"store_id": sid, "store_name": stores_map.get(&sid), "price": price}));

    let stores_out: Vec<serde_json::Value> = stores_prices.into_iter().map(|(sid, price)| {
        let hist = history_by_store.get(&sid).cloned().unwrap_or_default();
        serde_json::json!({
            "store_id": sid,
            "store_name": stores_map.get(&sid),
            "price": price,
            "history": hist.iter().map(|(t,p)| serde_json::json!({"ts_ms": t, "price": p})).collect::<Vec<_>>()
        })
    }).collect();

    let out = serde_json::json!({
        "stores": stores_out,
        "city_avg": city_avg,
        "min": min_json,
        "max": max_json,
    });
    (StatusCode::OK, Json(out)).into_response()
}

// Batch insights for products in a store: current store price, city average, cheapest store, and price history in this store
pub async fn list_store_product_insights(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    let Ok(store_oid) = ObjectId::from_str(&id) else { return StatusCode::BAD_REQUEST.into_response(); };
    // store items for this store
    let mut cursor = match state.store_items.find(doc!{"store_id": store_oid}, None).await { Ok(c)=>c, Err(e)=> { error!(?e, "query store_items failed"); return StatusCode::INTERNAL_SERVER_ERROR.into_response(); } };
    let mut store_items_vec: Vec<StoreItem> = Vec::new();
    let mut product_ids: Vec<ObjectId> = Vec::new();
    let mut store_prices: HashMap<ObjectId, f64> = HashMap::new();
    while let Some(res) = cursor.next().await {
        match res {
            Ok(it) => {
                store_prices.insert(it.product_id.clone(), it.price);
                product_ids.push(it.product_id.clone());
                store_items_vec.push(it);
            }
            Err(e) => { error!(?e, "cursor error"); return StatusCode::INTERNAL_SERVER_ERROR.into_response(); }
        }
    }
    if product_ids.is_empty() {
        return (StatusCode::OK, Json(Vec::<serde_json::Value>::new())).into_response();
    }
    // fetch product docs
    let mut products_map: HashMap<ObjectId, (String, Option<String>)> = HashMap::new();
    let mut pcursor = match state.products.find(doc!{"_id": {"$in": &product_ids}}, None).await { Ok(c)=>c, Err(e)=> { error!(?e, "query products failed"); return StatusCode::INTERNAL_SERVER_ERROR.into_response(); } };
    while let Some(res) = pcursor.next().await {
        match res {
            Ok(p) => {
                if let Some(pid) = p.id.clone() { products_map.insert(pid, (p.title, p.image_url)); }
            }
            Err(e) => { error!(?e, "products cursor error"); return StatusCode::INTERNAL_SERVER_ERROR.into_response(); }
        }
    }
    // city stats from all store_items for these product_ids
    let mut all_items_cursor = match state.store_items.find(doc!{"product_id": {"$in": &product_ids}}, None).await { Ok(c)=>c, Err(e)=> { error!(?e, "query city items failed"); return StatusCode::INTERNAL_SERVER_ERROR.into_response(); } };
    let mut sum: HashMap<ObjectId, f64> = HashMap::new();
    let mut cnt: HashMap<ObjectId, u64> = HashMap::new();
    let mut cheapest_price: HashMap<ObjectId, (f64, ObjectId)> = HashMap::new();
    while let Some(res) = all_items_cursor.next().await {
        match res {
            Ok(it) => {
                *sum.entry(it.product_id.clone()).or_insert(0.0) += it.price;
                *cnt.entry(it.product_id.clone()).or_insert(0) += 1;
                match cheapest_price.get(&it.product_id) {
                    Some((p, _sid)) if *p <= it.price => {}
                    _ => { cheapest_price.insert(it.product_id.clone(), (it.price, it.store_id.clone())); }
                }
            }
            Err(e) => { error!(?e, "city items cursor error"); return StatusCode::INTERNAL_SERVER_ERROR.into_response(); }
        }
    }
    // fetch store names for cheapest
    let cheapest_store_ids: HashSet<ObjectId> = cheapest_price.values().map(|(_, sid)| sid.clone()).collect();
    let mut stores_map: HashMap<ObjectId, String> = HashMap::new();
    if !cheapest_store_ids.is_empty() {
        let mut scursor = match state.stores.find(doc!{"_id": {"$in": cheapest_store_ids.iter().collect::<Vec<_>>() }}, None).await { Ok(c)=>c, Err(e)=> { error!(?e, "query stores failed"); return StatusCode::INTERNAL_SERVER_ERROR.into_response(); } };
        while let Some(res) = scursor.next().await {
            match res {
                Ok(s) => { if let Some(sid) = s.id { stores_map.insert(sid, s.name); } }
                Err(e) => { error!(?e, "stores cursor error"); return StatusCode::INTERNAL_SERVER_ERROR.into_response(); }
            }
        }
    }
    // activities history for these products in this store
    let mut acts_cursor = match state.store_activities.find(
        doc!{"store_id": store_oid, "product_id": {"$in": &product_ids}},
        mongodb::options::FindOptions::builder().sort(doc!{"ts_ms": 1}).build()
    ).await { Ok(c)=>c, Err(e)=> { error!(?e, "query activities failed"); return StatusCode::INTERNAL_SERVER_ERROR.into_response(); } };
    let mut history: HashMap<ObjectId, Vec<(i64,f64)>> = HashMap::new();
    while let Some(res) = acts_cursor.next().await {
        match res {
            Ok(a) => {
                if let (Some(pid), Some(price)) = (a.product_id, a.price) {
                    history.entry(pid).or_default().push((a.ts_ms, price));
                }
            }
            Err(e) => { error!(?e, "activities cursor error"); return StatusCode::INTERNAL_SERVER_ERROR.into_response(); }
        }
    }

    // build payload
    let mut out: Vec<serde_json::Value> = Vec::new();
    for pid in product_ids {
        let (title, image_url) = products_map.get(&pid).cloned().unwrap_or((pid.to_hex(), None));
        let sp = store_prices.get(&pid).cloned();
        let cavg = match (sum.get(&pid), cnt.get(&pid)) {
            (Some(s), Some(c)) if *c > 0 => Some(*s / *c as f64),
            _ => None,
        };
        let cheap = cheapest_price.get(&pid).map(|(p, sid)| {
            let name = stores_map.get(sid).cloned();
            serde_json::json!({"store_id": sid, "store_name": name, "price": p})
        });
        let hist = history.get(&pid).cloned().unwrap_or_default();
        out.push(serde_json::json!({
            "product_id": pid,
            "product_title": title,
            "product_image_url": image_url,
            "store_price": sp,
            "city_avg": cavg,
            "cheapest": cheap,
            "history": hist.iter().map(|(t,p)| serde_json::json!({"ts_ms": t, "price": p})).collect::<Vec<_>>()
        }));
    }

    (StatusCode::OK, Json(out)).into_response()
}

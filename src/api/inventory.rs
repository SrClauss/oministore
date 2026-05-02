use axum::{extract::{Path, Query}, response::Json, routing::get, Router};
use mongodb::bson::{self, doc, Document};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::services::mongo;

#[derive(Deserialize, Serialize)]
pub struct InventoryFilter {
    pub product_id: Option<String>,
    pub warehouse_id: Option<String>,
    pub sell_without_stock: Option<bool>,
}

#[derive(Deserialize, Serialize)]
pub struct InventoryPayload {
    pub product_id: String,
    pub quantity: i64,
    pub reserved: i64,
    pub warehouse_id: String,
    pub sell_without_stock: bool,
}

#[derive(Deserialize, Serialize)]
pub struct InventoryUpdate {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub product_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quantity: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reserved: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warehouse_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sell_without_stock: Option<bool>,
}

pub fn router() -> Router {
    Router::new()
        .route("/", get(get_inventorys).post(create_inventory))
        .route("/:id", get(get_inventory).put(update_inventory).delete(delete_inventory))
}

async fn get_inventorys(Query(filter): Query<InventoryFilter>) -> Json<Value> {
    let mut query = Document::new();
    if let Some(value) = filter.product_id {
        query.insert("product_id", value);
    }
    if let Some(value) = filter.warehouse_id {
        query.insert("warehouse_id", value);
    }
    if let Some(value) = filter.sell_without_stock {
        query.insert("sell_without_stock", value);
    }
    match mongo::find_all("inventorys", query).await {
        Ok(items) => Json(json!({"data": items})),
        Err(error) => Json(json!({"error": error})),
    }
}

async fn get_inventory(Path(id): Path<String>) -> Json<Value> {
    match mongo::find_one("inventorys", &id).await {
        Ok(Some(item)) => Json(json!({"data": item})),
        Ok(None) => Json(json!({"error": "not found"})),
        Err(error) => Json(json!({"error": error})),
    }
}

async fn create_inventory(Json(payload): Json<InventoryPayload>) -> Json<Value> {
    let document = bson::to_document(&payload).unwrap_or_default();
    match mongo::insert_one("inventorys", document).await {
        Ok(result) => Json(json!({"data": result})),
        Err(error) => Json(json!({"error": error})),
    }
}

async fn update_inventory(Path(id): Path<String>, Json(payload): Json<InventoryUpdate>) -> Json<Value> {
    let updates = bson::to_document(&payload).unwrap_or_default();
    if updates.is_empty() {
        return Json(json!({"error": "no fields to update"}));
    }
    match mongo::update_one("inventorys", &id, updates).await {
        Ok(Some(item)) => Json(json!({"data": item})),
        Ok(None) => Json(json!({"error": "not found"})),
        Err(error) => Json(json!({"error": error})),
    }
}

async fn delete_inventory(Path(id): Path<String>) -> Json<Value> {
    match mongo::delete_one("inventorys", &id).await {
        Ok(count) => Json(json!({"deleted_count": count})),
        Err(error) => Json(json!({"error": error})),
    }
}

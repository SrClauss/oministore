use axum::{extract::{Path, Query}, response::Json, routing::get, Router};
use mongodb::bson::{self, doc, Document};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::services::mongo;

#[derive(Deserialize, Serialize)]
pub struct CartItemFilter {
    pub cart_id: Option<String>,
    pub product_id: Option<String>,
}

#[derive(Deserialize, Serialize)]
pub struct CartItemPayload {
    pub cart_id: String,
    pub product_id: String,
    pub quantity: i64,
    pub price: f64,
}

#[derive(Deserialize, Serialize)]
pub struct CartItemUpdate {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cart_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub product_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quantity: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<f64>,
}

pub fn router() -> Router {
    Router::new()
        .route("/", get(get_cart_items).post(create_cart_item))
        .route("/:id", get(get_cart_item).put(update_cart_item).delete(delete_cart_item))
}

async fn get_cart_items(Query(filter): Query<CartItemFilter>) -> Json<Value> {
    let mut query = Document::new();
    if let Some(value) = filter.cart_id {
        query.insert("cart_id", value);
    }
    if let Some(value) = filter.product_id {
        query.insert("product_id", value);
    }
    match mongo::find_all("cart_items", query).await {
        Ok(items) => Json(json!({"data": items})),
        Err(error) => Json(json!({"error": error})),
    }
}

async fn get_cart_item(Path(id): Path<String>) -> Json<Value> {
    match mongo::find_one("cart_items", &id).await {
        Ok(Some(item)) => Json(json!({"data": item})),
        Ok(None) => Json(json!({"error": "not found"})),
        Err(error) => Json(json!({"error": error})),
    }
}

async fn create_cart_item(Json(payload): Json<CartItemPayload>) -> Json<Value> {
    let document = bson::to_document(&payload).unwrap_or_default();
    match mongo::insert_one("cart_items", document).await {
        Ok(result) => Json(json!({"data": result})),
        Err(error) => Json(json!({"error": error})),
    }
}

async fn update_cart_item(Path(id): Path<String>, Json(payload): Json<CartItemUpdate>) -> Json<Value> {
    let updates = bson::to_document(&payload).unwrap_or_default();
    if updates.is_empty() {
        return Json(json!({"error": "no fields to update"}));
    }
    match mongo::update_one("cart_items", &id, updates).await {
        Ok(Some(item)) => Json(json!({"data": item})),
        Ok(None) => Json(json!({"error": "not found"})),
        Err(error) => Json(json!({"error": error})),
    }
}

async fn delete_cart_item(Path(id): Path<String>) -> Json<Value> {
    match mongo::delete_one("cart_items", &id).await {
        Ok(count) => Json(json!({"deleted_count": count})),
        Err(error) => Json(json!({"error": error})),
    }
}

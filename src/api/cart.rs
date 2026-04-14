use axum::{extract::{Path, Query}, response::Json, routing::get, Router};
use mongodb::bson::{self, doc, Document};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::services::mongo;

#[derive(Deserialize, Serialize)]
pub struct CartFilter {
    pub user_id: Option<String>,
    pub status: Option<String>,
}

#[derive(Deserialize, Serialize)]
pub struct CartPayload {
    pub user_id: String,
    pub items: Vec<Value>,
    pub shipping: Option<Value>,
    pub coupon: Option<Value>,
    pub subtotal: f64,
    pub discount_total: f64,
    pub shipping_total: f64,
    pub total: f64,
    pub status: String,
}

#[derive(Deserialize, Serialize)]
pub struct CartUpdate {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<Vec<Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shipping: Option<Option<Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub coupon: Option<Option<Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtotal: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discount_total: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shipping_total: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}

pub fn router() -> Router {
    Router::new()
        .route("/", get(get_carts).post(create_cart))
        .route("/:id", get(get_cart).put(update_cart).delete(delete_cart))
}

async fn get_carts(Query(filter): Query<CartFilter>) -> Json<Value> {
    let mut query = Document::new();
    if let Some(value) = filter.user_id {
        query.insert("user_id", value);
    }
    if let Some(value) = filter.status {
        query.insert("status", value);
    }
    match mongo::find_all("carts", query).await {
        Ok(items) => Json(json!({"data": items})),
        Err(error) => Json(json!({"error": error})),
    }
}

async fn get_cart(Path(id): Path<String>) -> Json<Value> {
    match mongo::find_one("carts", &id).await {
        Ok(Some(item)) => Json(json!({"data": item})),
        Ok(None) => Json(json!({"error": "not found"})),
        Err(error) => Json(json!({"error": error})),
    }
}

async fn create_cart(Json(payload): Json<CartPayload>) -> Json<Value> {
    let document = bson::to_document(&payload).unwrap_or_default();
    match mongo::insert_one("carts", document).await {
        Ok(result) => Json(json!({"data": result})),
        Err(error) => Json(json!({"error": error})),
    }
}

async fn update_cart(Path(id): Path<String>, Json(payload): Json<CartUpdate>) -> Json<Value> {
    let updates = bson::to_document(&payload).unwrap_or_default();
    if updates.is_empty() {
        return Json(json!({"error": "no fields to update"}));
    }
    match mongo::update_one("carts", &id, updates).await {
        Ok(Some(item)) => Json(json!({"data": item})),
        Ok(None) => Json(json!({"error": "not found"})),
        Err(error) => Json(json!({"error": error})),
    }
}

async fn delete_cart(Path(id): Path<String>) -> Json<Value> {
    match mongo::delete_one("carts", &id).await {
        Ok(count) => Json(json!({"deleted_count": count})),
        Err(error) => Json(json!({"error": error})),
    }
}

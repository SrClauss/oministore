use axum::{extract::{Path, Query}, response::Json, routing::get, Router};
use mongodb::bson::{self, doc, Document};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::services::mongo;

#[derive(Deserialize, Serialize)]
pub struct ShippingFilter {
    pub order_id: Option<String>,
    pub status: Option<String>,
}

#[derive(Deserialize, Serialize)]
pub struct ShippingPayload {
    pub order_id: String,
    pub address: Value,
    pub method: String,
    pub cost: f64,
    pub status: String,
}

#[derive(Deserialize, Serialize)]
pub struct ShippingUpdate {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cost: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}

pub fn router() -> Router {
    Router::new()
        .route("/", get(get_shippings).post(create_shipping))
        .route("/:id", get(get_shipping).put(update_shipping).delete(delete_shipping))
}

async fn get_shippings(Query(filter): Query<ShippingFilter>) -> Json<Value> {
    let mut query = Document::new();
    if let Some(value) = filter.order_id {
        query.insert("order_id", value);
    }
    if let Some(value) = filter.status {
        query.insert("status", value);
    }
    match mongo::find_all("shippings", query).await {
        Ok(items) => Json(json!({"data": items})),
        Err(error) => Json(json!({"error": error})),
    }
}

async fn get_shipping(Path(id): Path<String>) -> Json<Value> {
    match mongo::find_one("shippings", &id).await {
        Ok(Some(item)) => Json(json!({"data": item})),
        Ok(None) => Json(json!({"error": "not found"})),
        Err(error) => Json(json!({"error": error})),
    }
}

async fn create_shipping(Json(payload): Json<ShippingPayload>) -> Json<Value> {
    let document = bson::to_document(&payload).unwrap_or_default();
    match mongo::insert_one("shippings", document).await {
        Ok(result) => Json(json!({"data": result})),
        Err(error) => Json(json!({"error": error})),
    }
}

async fn update_shipping(Path(id): Path<String>, Json(payload): Json<ShippingUpdate>) -> Json<Value> {
    let updates = bson::to_document(&payload).unwrap_or_default();
    if updates.is_empty() {
        return Json(json!({"error": "no fields to update"}));
    }
    match mongo::update_one("shippings", &id, updates).await {
        Ok(Some(item)) => Json(json!({"data": item})),
        Ok(None) => Json(json!({"error": "not found"})),
        Err(error) => Json(json!({"error": error})),
    }
}

async fn delete_shipping(Path(id): Path<String>) -> Json<Value> {
    match mongo::delete_one("shippings", &id).await {
        Ok(count) => Json(json!({"deleted_count": count})),
        Err(error) => Json(json!({"error": error})),
    }
}

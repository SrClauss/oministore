use axum::{extract::{Path, Query}, response::Json, routing::get, Router};
use mongodb::bson::{self, doc, Document};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::services::mongo;

#[derive(Deserialize, Serialize)]
pub struct WarehouseFilter {
    pub name: Option<String>,
    pub location: Option<String>,
}

#[derive(Deserialize, Serialize)]
pub struct WarehousePayload {
    pub name: String,
    pub location: String,
    pub capacity: i64,
    pub active: bool,
}

#[derive(Deserialize, Serialize)]
pub struct WarehouseUpdate {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capacity: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active: Option<bool>,
}

pub fn router() -> Router {
    Router::new()
        .route("/", get(get_warehouses).post(create_warehouse))
        .route("/:id", get(get_warehouse).put(update_warehouse).delete(delete_warehouse))
}

async fn get_warehouses(Query(filter): Query<WarehouseFilter>) -> Json<Value> {
    let mut query = Document::new();
    if let Some(value) = filter.name {
        query.insert("name", value);
    }
    if let Some(value) = filter.location {
        query.insert("location", value);
    }
    match mongo::find_all("warehouses", query).await {
        Ok(items) => Json(json!({"data": items})),
        Err(error) => Json(json!({"error": error})),
    }
}

async fn get_warehouse(Path(id): Path<String>) -> Json<Value> {
    match mongo::find_one("warehouses", &id).await {
        Ok(Some(item)) => Json(json!({"data": item})),
        Ok(None) => Json(json!({"error": "not found"})),
        Err(error) => Json(json!({"error": error})),
    }
}

async fn create_warehouse(Json(payload): Json<WarehousePayload>) -> Json<Value> {
    let document = bson::to_document(&payload).unwrap_or_default();
    match mongo::insert_one("warehouses", document).await {
        Ok(result) => Json(json!({"data": result})),
        Err(error) => Json(json!({"error": error})),
    }
}

async fn update_warehouse(Path(id): Path<String>, Json(payload): Json<WarehouseUpdate>) -> Json<Value> {
    let updates = bson::to_document(&payload).unwrap_or_default();
    if updates.is_empty() {
        return Json(json!({"error": "no fields to update"}));
    }
    match mongo::update_one("warehouses", &id, updates).await {
        Ok(Some(item)) => Json(json!({"data": item})),
        Ok(None) => Json(json!({"error": "not found"})),
        Err(error) => Json(json!({"error": error})),
    }
}

async fn delete_warehouse(Path(id): Path<String>) -> Json<Value> {
    match mongo::delete_one("warehouses", &id).await {
        Ok(count) => Json(json!({"deleted_count": count})),
        Err(error) => Json(json!({"error": error})),
    }
}

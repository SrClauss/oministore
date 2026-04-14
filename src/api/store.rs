use axum::{extract::{Path, Query}, response::Json, routing::get, Router};
use mongodb::bson::{self, doc, Document};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::services::mongo;

#[derive(Deserialize, Serialize)]
pub struct StoreFilter {
    pub name: Option<String>,
    pub active: Option<bool>,
}

#[derive(Deserialize, Serialize)]
pub struct StorePayload {
    pub name: String,
    pub url: String,
    pub currency: String,
    pub active: bool,
}

#[derive(Deserialize, Serialize)]
pub struct StoreUpdate {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active: Option<bool>,
}

pub fn router() -> Router {
    Router::new()
        .route("/", get(get_stores).post(create_store))
        .route("/:id", get(get_store).put(update_store).delete(delete_store))
}

async fn get_stores(Query(filter): Query<StoreFilter>) -> Json<Value> {
    let mut query = Document::new();
    if let Some(value) = filter.name {
        query.insert("name", value);
    }
    if let Some(value) = filter.active {
        query.insert("active", value);
    }
    match mongo::find_all("stores", query).await {
        Ok(items) => Json(json!({"data": items})),
        Err(error) => Json(json!({"error": error})),
    }
}

async fn get_store(Path(id): Path<String>) -> Json<Value> {
    match mongo::find_one("stores", &id).await {
        Ok(Some(item)) => Json(json!({"data": item})),
        Ok(None) => Json(json!({"error": "not found"})),
        Err(error) => Json(json!({"error": error})),
    }
}

async fn create_store(Json(payload): Json<StorePayload>) -> Json<Value> {
    let document = bson::to_document(&payload).unwrap_or_default();
    match mongo::insert_one("stores", document).await {
        Ok(result) => Json(json!({"data": result})),
        Err(error) => Json(json!({"error": error})),
    }
}

async fn update_store(Path(id): Path<String>, Json(payload): Json<StoreUpdate>) -> Json<Value> {
    let updates = bson::to_document(&payload).unwrap_or_default();
    if updates.is_empty() {
        return Json(json!({"error": "no fields to update"}));
    }
    match mongo::update_one("stores", &id, updates).await {
        Ok(Some(item)) => Json(json!({"data": item})),
        Ok(None) => Json(json!({"error": "not found"})),
        Err(error) => Json(json!({"error": error})),
    }
}

async fn delete_store(Path(id): Path<String>) -> Json<Value> {
    match mongo::delete_one("stores", &id).await {
        Ok(count) => Json(json!({"deleted_count": count})),
        Err(error) => Json(json!({"error": error})),
    }
}

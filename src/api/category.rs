use axum::{extract::{Path, Query}, response::Json, routing::get, Router};
use mongodb::bson::{self, doc, Document};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::services::mongo;

#[derive(Deserialize, Serialize)]
pub struct CategoryFilter {
    pub name: Option<String>,
}

#[derive(Deserialize, Serialize)]
pub struct CategoryPayload {
    pub name: String,
    pub description: Option<String>,
    pub active: bool,
}

#[derive(Deserialize, Serialize)]
pub struct CategoryUpdate {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<Option<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active: Option<bool>,
}

pub fn router() -> Router {
    Router::new()
        .route("/", get(get_categorys).post(create_category))
        .route("/:id", get(get_category).put(update_category).delete(delete_category))
}

async fn get_categorys(Query(filter): Query<CategoryFilter>) -> Json<Value> {
    let mut query = Document::new();
    if let Some(value) = filter.name {
        query.insert("name", value);
    }
    match mongo::find_all("categorys", query).await {
        Ok(items) => Json(json!({"data": items})),
        Err(error) => Json(json!({"error": error})),
    }
}

async fn get_category(Path(id): Path<String>) -> Json<Value> {
    match mongo::find_one("categorys", &id).await {
        Ok(Some(item)) => Json(json!({"data": item})),
        Ok(None) => Json(json!({"error": "not found"})),
        Err(error) => Json(json!({"error": error})),
    }
}

async fn create_category(Json(payload): Json<CategoryPayload>) -> Json<Value> {
    let document = bson::to_document(&payload).unwrap_or_default();
    match mongo::insert_one("categorys", document).await {
        Ok(result) => Json(json!({"data": result})),
        Err(error) => Json(json!({"error": error})),
    }
}

async fn update_category(Path(id): Path<String>, Json(payload): Json<CategoryUpdate>) -> Json<Value> {
    let updates = bson::to_document(&payload).unwrap_or_default();
    if updates.is_empty() {
        return Json(json!({"error": "no fields to update"}));
    }
    match mongo::update_one("categorys", &id, updates).await {
        Ok(Some(item)) => Json(json!({"data": item})),
        Ok(None) => Json(json!({"error": "not found"})),
        Err(error) => Json(json!({"error": error})),
    }
}

async fn delete_category(Path(id): Path<String>) -> Json<Value> {
    match mongo::delete_one("categorys", &id).await {
        Ok(count) => Json(json!({"deleted_count": count})),
        Err(error) => Json(json!({"error": error})),
    }
}

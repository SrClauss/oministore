use axum::{extract::{Path, Query}, response::Json, routing::get, Router};
use mongodb::bson::{self, doc, Document};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::services::mongo;

#[derive(Deserialize, Serialize)]
pub struct CustomerFilter {
    pub name: Option<String>,
    pub email: Option<String>,
}

#[derive(Deserialize, Serialize)]
pub struct CustomerPayload {
    pub name: String,
    pub email: String,
    pub phone: Option<String>,
    pub address: Option<Value>,
    pub active: bool,
}

#[derive(Deserialize, Serialize)]
pub struct CustomerUpdate {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone: Option<Option<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<Option<Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active: Option<bool>,
}

pub fn router() -> Router {
    Router::new()
        .route("/", get(get_customers).post(create_customer))
        .route("/:id", get(get_customer).put(update_customer).delete(delete_customer))
}

async fn get_customers(Query(filter): Query<CustomerFilter>) -> Json<Value> {
    let mut query = Document::new();
    if let Some(value) = filter.name {
        query.insert("name", value);
    }
    if let Some(value) = filter.email {
        query.insert("email", value);
    }
    match mongo::find_all("customers", query).await {
        Ok(items) => Json(json!({"data": items})),
        Err(error) => Json(json!({"error": error})),
    }
}

async fn get_customer(Path(id): Path<String>) -> Json<Value> {
    match mongo::find_one("customers", &id).await {
        Ok(Some(item)) => Json(json!({"data": item})),
        Ok(None) => Json(json!({"error": "not found"})),
        Err(error) => Json(json!({"error": error})),
    }
}

async fn create_customer(Json(payload): Json<CustomerPayload>) -> Json<Value> {
    let document = bson::to_document(&payload).unwrap_or_default();
    match mongo::insert_one("customers", document).await {
        Ok(result) => Json(json!({"data": result})),
        Err(error) => Json(json!({"error": error})),
    }
}

async fn update_customer(Path(id): Path<String>, Json(payload): Json<CustomerUpdate>) -> Json<Value> {
    let updates = bson::to_document(&payload).unwrap_or_default();
    if updates.is_empty() {
        return Json(json!({"error": "no fields to update"}));
    }
    match mongo::update_one("customers", &id, updates).await {
        Ok(Some(item)) => Json(json!({"data": item})),
        Ok(None) => Json(json!({"error": "not found"})),
        Err(error) => Json(json!({"error": error})),
    }
}

async fn delete_customer(Path(id): Path<String>) -> Json<Value> {
    match mongo::delete_one("customers", &id).await {
        Ok(count) => Json(json!({"deleted_count": count})),
        Err(error) => Json(json!({"error": error})),
    }
}

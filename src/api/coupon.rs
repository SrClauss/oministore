use axum::{extract::{Path, Query}, response::Json, routing::get, Router};
use mongodb::bson::{self, doc, Document};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::services::mongo;

#[derive(Deserialize, Serialize)]
pub struct CouponFilter {
    pub code: Option<String>,
    pub active: Option<bool>,
}

#[derive(Deserialize, Serialize)]
pub struct CouponPayload {
    pub code: String,
    pub discount_type: String,
    pub discount_value: f64,
    pub active: bool,
}

#[derive(Deserialize, Serialize)]
pub struct CouponUpdate {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discount_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discount_value: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active: Option<bool>,
}

pub fn router() -> Router {
    Router::new()
        .route("/", get(get_coupons).post(create_coupon))
        .route("/:id", get(get_coupon).put(update_coupon).delete(delete_coupon))
}

async fn get_coupons(Query(filter): Query<CouponFilter>) -> Json<Value> {
    let mut query = Document::new();
    if let Some(value) = filter.code {
        query.insert("code", value);
    }
    if let Some(value) = filter.active {
        query.insert("active", value);
    }
    match mongo::find_all("coupons", query).await {
        Ok(items) => Json(json!({"data": items})),
        Err(error) => Json(json!({"error": error})),
    }
}

async fn get_coupon(Path(id): Path<String>) -> Json<Value> {
    match mongo::find_one("coupons", &id).await {
        Ok(Some(item)) => Json(json!({"data": item})),
        Ok(None) => Json(json!({"error": "not found"})),
        Err(error) => Json(json!({"error": error})),
    }
}

async fn create_coupon(Json(payload): Json<CouponPayload>) -> Json<Value> {
    let document = bson::to_document(&payload).unwrap_or_default();
    match mongo::insert_one("coupons", document).await {
        Ok(result) => Json(json!({"data": result})),
        Err(error) => Json(json!({"error": error})),
    }
}

async fn update_coupon(Path(id): Path<String>, Json(payload): Json<CouponUpdate>) -> Json<Value> {
    let updates = bson::to_document(&payload).unwrap_or_default();
    if updates.is_empty() {
        return Json(json!({"error": "no fields to update"}));
    }
    match mongo::update_one("coupons", &id, updates).await {
        Ok(Some(item)) => Json(json!({"data": item})),
        Ok(None) => Json(json!({"error": "not found"})),
        Err(error) => Json(json!({"error": error})),
    }
}

async fn delete_coupon(Path(id): Path<String>) -> Json<Value> {
    match mongo::delete_one("coupons", &id).await {
        Ok(count) => Json(json!({"deleted_count": count})),
        Err(error) => Json(json!({"error": error})),
    }
}

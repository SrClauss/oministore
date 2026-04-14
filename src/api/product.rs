use axum::{extract::{Path, Query}, response::Json, routing::get, Router};
use mongodb::bson::{self, doc, Document};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::services::{cache, mongo};

#[derive(Deserialize, Serialize)]
pub struct ProductFilter {
    pub name: Option<String>,
    pub sku: Option<String>,
    pub category_id: Option<String>,
    pub page: Option<u64>,
    pub limit: Option<u64>,
}

#[derive(Deserialize, Serialize)]
pub struct ProductPayload {
    pub name: String,
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
    pub category_ids: Option<Vec<String>>,
    pub photos: Option<Vec<String>>,
    pub media: Option<Vec<String>>,
    pub sku: String,
    pub color: Option<String>,
    pub attributes: Option<Vec<String>>,
    pub related_product_ids: Option<Vec<String>>,
    pub variant_product_ids: Option<Vec<String>>,
    pub active: bool,
}

#[derive(Deserialize, Serialize)]
pub struct ProductUpdate {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<Option<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub photos: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sku: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<Option<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attributes: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub related_product_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variant_product_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active: Option<bool>,
}

pub fn router() -> Router {
    Router::new()
        .route("/", get(get_products).post(create_product))
        .route("/:id", get(get_product).put(update_product).delete(delete_product))
}

async fn get_products(Query(filter): Query<ProductFilter>) -> Json<Value> {
    let page = filter.page.unwrap_or(1).max(1);
    let limit = filter.limit.unwrap_or(20).clamp(1, 100);

    let mut query = Document::new();
    if let Some(value) = &filter.name {
        query.insert("name", value.clone());
    }
    if let Some(value) = &filter.sku {
        query.insert("sku", value.clone());
    }

    // Cache apenas quando filtra por category_id (leituras frequentes)
    if let Some(ref category_id) = filter.category_id {
        query.insert("category_ids", category_id.clone());

        let cache_key = format!("category:{category_id}:products:p{page}:l{limit}");

        if let Some(cached) = cache::get(&cache_key).await {
            return Json(cached);
        }

        return match mongo::find_paginated("products", query, page, limit).await {
            Ok((items, total)) => {
                let result = json!({
                    "data": items,
                    "total": total,
                    "page": page,
                    "limit": limit,
                    "pages": (total as f64 / limit as f64).ceil() as u64
                });
                cache::set(&cache_key, &result).await;
                Json(result)
            }
            Err(error) => Json(json!({"error": error})),
        };
    }

    // Listagem geral sem cache (pode ter filtros variados)
    match mongo::find_paginated("products", query, page, limit).await {
        Ok((items, total)) => Json(json!({
            "data": items,
            "total": total,
            "page": page,
            "limit": limit,
            "pages": (total as f64 / limit as f64).ceil() as u64
        })),
        Err(error) => Json(json!({"error": error})),
    }
}

async fn get_product(Path(id): Path<String>) -> Json<Value> {
    match mongo::find_one("products", &id).await {
        Ok(Some(item)) => Json(json!({"data": item})),
        Ok(None) => Json(json!({"error": "not found"})),
        Err(error) => Json(json!({"error": error})),
    }
}

async fn create_product(Json(payload): Json<ProductPayload>) -> Json<Value> {
    let document = bson::to_document(&payload).unwrap_or_default();
    match mongo::insert_one("products", document).await {
        Ok(result) => Json(json!({"data": result})),
        Err(error) => Json(json!({"error": error})),
    }
}

async fn update_product(Path(id): Path<String>, Json(payload): Json<ProductUpdate>) -> Json<Value> {
    let updates = bson::to_document(&payload).unwrap_or_default();
    if updates.is_empty() {
        return Json(json!({"error": "no fields to update"}));
    }
    match mongo::update_one("products", &id, updates).await {
        Ok(Some(item)) => {
            cache::del_pattern("category:*:products:*").await;
            cache::del_pattern("collection:*:products:*").await;
            Json(json!({"data": item}))
        }
        Ok(None) => Json(json!({"error": "not found"})),
        Err(error) => Json(json!({"error": error})),
    }
}

async fn delete_product(Path(id): Path<String>) -> Json<Value> {
    match mongo::delete_one("products", &id).await {
        Ok(count) => {
            cache::del_pattern("category:*:products:*").await;
            cache::del_pattern("collection:*:products:*").await;
            Json(json!({"deleted_count": count}))
        }
        Err(error) => Json(json!({"error": error})),
    }
}

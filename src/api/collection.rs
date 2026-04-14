use axum::{extract::{Path, Query}, response::Json, routing::get, Router};
use mongodb::bson::{self, doc, oid::ObjectId, Document};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::services::{cache, mongo};

#[derive(Deserialize, Serialize)]
pub struct CollectionFilter {
    pub name: Option<String>,
    pub product_id: Option<String>,
}

#[derive(Deserialize, Serialize)]
pub struct CollectionPayload {
    pub name: String,
    pub description: Option<String>,
    pub product_ids: Option<Vec<String>>,
    pub active: bool,
}

#[derive(Deserialize, Serialize)]
pub struct CollectionUpdate {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<Option<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub product_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active: Option<bool>,
}

#[derive(Deserialize)]
pub struct Pagination {
    pub page: Option<u64>,
    pub limit: Option<u64>,
}

pub fn router() -> Router {
    Router::new()
        .route("/", get(get_collections).post(create_collection))
        .route("/:id", get(get_collection).put(update_collection).delete(delete_collection))
        .route("/:id/products", get(get_collection_products))
}

async fn get_collections(Query(filter): Query<CollectionFilter>) -> Json<Value> {
    let mut query = Document::new();
    if let Some(value) = filter.name {
        query.insert("name", value);
    }
    if let Some(value) = filter.product_id {
        query.insert("product_id", value);
    }
    match mongo::find_all("collections", query).await {
        Ok(items) => Json(json!({"data": items})),
        Err(error) => Json(json!({"error": error})),
    }
}

async fn get_collection(Path(id): Path<String>) -> Json<Value> {
    match mongo::find_one("collections", &id).await {
        Ok(Some(item)) => Json(json!({"data": item})),
        Ok(None) => Json(json!({"error": "not found"})),
        Err(error) => Json(json!({"error": error})),
    }
}

async fn create_collection(Json(payload): Json<CollectionPayload>) -> Json<Value> {
    let document = bson::to_document(&payload).unwrap_or_default();
    match mongo::insert_one("collections", document).await {
        Ok(result) => Json(json!({"data": result})),
        Err(error) => Json(json!({"error": error})),
    }
}

async fn update_collection(Path(id): Path<String>, Json(payload): Json<CollectionUpdate>) -> Json<Value> {
    let updates = bson::to_document(&payload).unwrap_or_default();
    if updates.is_empty() {
        return Json(json!({"error": "no fields to update"}));
    }
    match mongo::update_one("collections", &id, updates).await {
        Ok(Some(item)) => Json(json!({"data": item})),
        Ok(None) => Json(json!({"error": "not found"})),
        Err(error) => Json(json!({"error": error})),
    }
}

async fn delete_collection(Path(id): Path<String>) -> Json<Value> {
    match mongo::delete_one("collections", &id).await {
        Ok(count) => {
            cache::del_pattern(&format!("collection:{id}:products:*")).await;
            Json(json!({"deleted_count": count}))
        }
        Err(error) => Json(json!({"error": error})),
    }
}

/// GET /collections/:id/products?page=1&limit=20
/// Busca produtos da coleção com paginação e cache Redis (TTL 5 min)
async fn get_collection_products(
    Path(id): Path<String>,
    Query(pagination): Query<Pagination>,
) -> Json<Value> {
    let page = pagination.page.unwrap_or(1).max(1);
    let limit = pagination.limit.unwrap_or(20).clamp(1, 100);
    let cache_key = format!("collection:{id}:products:p{page}:l{limit}");

    // Cache hit
    if let Some(cached) = cache::get(&cache_key).await {
        return Json(cached);
    }

    // Buscar a coleção para obter product_ids
    let collection_doc = match mongo::find_one("collections", &id).await {
        Ok(Some(doc)) => doc,
        Ok(None) => return Json(json!({"error": "collection not found"})),
        Err(e) => return Json(json!({"error": e})),
    };

    let product_ids: Vec<ObjectId> = collection_doc
        .get("product_ids")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str())
                .filter_map(|s| ObjectId::parse_str(s).ok())
                .collect()
        })
        .unwrap_or_default();

    if product_ids.is_empty() {
        let result = json!({"data": [], "total": 0, "page": page, "limit": limit});
        cache::set(&cache_key, &result).await;
        return Json(result);
    }

    match mongo::find_by_ids_paginated("products", product_ids, page, limit).await {
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
    }
}

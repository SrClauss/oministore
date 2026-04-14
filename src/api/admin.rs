use axum::{
    async_trait,
    extract::{FromRequestParts, Query},
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Json, Response},
    routing::get,
    Router,
};
use mongodb::bson::Document;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::services::mongo;

pub fn router() -> Router {
    Router::new()
        .route("/dashboard", get(dashboard))
        .route("/orders", get(list_orders))
        .route("/products", get(list_products))
        .route("/customers", get(list_customers))
        .route("/coupons", get(list_coupons))
}

/// Extractor that validates the X-Admin-Token header against the ADMIN_TOKEN env var.
pub struct AdminAuth;

#[async_trait]
impl<S: Send + Sync> FromRequestParts<S> for AdminAuth {
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let expected = match std::env::var("ADMIN_TOKEN") {
            Ok(t) if !t.is_empty() => t,
            _ => {
                return Err(
                    (
                        StatusCode::UNAUTHORIZED,
                        Json(json!({"error": "unauthorized"})),
                    )
                        .into_response(),
                );
            }
        };
        let token = parts
            .headers
            .get("x-admin-token")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        if token == expected {
            Ok(AdminAuth)
        } else {
            Err(
                (
                    StatusCode::UNAUTHORIZED,
                    Json(json!({"error": "unauthorized"})),
                )
                    .into_response(),
            )
        }
    }
}

#[derive(Deserialize)]
struct PaginationParams {
    page: Option<u64>,
    limit: Option<u64>,
    payment_status: Option<String>,
}

async fn dashboard(_auth: AdminAuth) -> Json<Value> {
    let all_orders = mongo::find_all("orders", Document::new()).await.unwrap_or_default();
    let total_orders = all_orders.len() as u64;

    let paid_orders: Vec<&Value> = all_orders
        .iter()
        .filter(|o| o.get("payment_status").and_then(Value::as_str) == Some("paid"))
        .collect();

    let paid_orders_count = paid_orders.len() as u64;

    let pending_orders = all_orders
        .iter()
        .filter(|o| o.get("payment_status").and_then(Value::as_str) == Some("pending"))
        .count() as u64;

    let total_customers = mongo::find_all("customers", Document::new())
        .await
        .unwrap_or_default()
        .len() as u64;

    let total_products = mongo::find_all("products", Document::new())
        .await
        .unwrap_or_default()
        .len() as u64;

    let total_revenue: f64 = paid_orders
        .iter()
        .filter_map(|item| item.get("total").and_then(Value::as_f64))
        .sum();

    Json(json!({
        "total_orders": total_orders,
        "paid_orders": paid_orders_count,
        "pending_orders": pending_orders,
        "total_customers": total_customers,
        "total_products": total_products,
        "total_revenue": total_revenue,
    }))
}

async fn list_orders(_auth: AdminAuth, Query(params): Query<PaginationParams>) -> Json<Value> {
    let page = params.page.unwrap_or(1);
    let limit = params.limit.unwrap_or(20);
    let mut filter = Document::new();
    if let Some(status) = params.payment_status {
        filter.insert("payment_status", status);
    }
    match mongo::find_paginated("orders", filter, page, limit).await {
        Ok((data, total)) => Json(json!({"data": data, "total": total, "page": page, "limit": limit})),
        Err(e) => Json(json!({"error": e})),
    }
}

async fn list_products(_auth: AdminAuth, Query(params): Query<PaginationParams>) -> Json<Value> {
    let page = params.page.unwrap_or(1);
    let limit = params.limit.unwrap_or(20);
    match mongo::find_paginated("products", Document::new(), page, limit).await {
        Ok((data, total)) => Json(json!({"data": data, "total": total, "page": page, "limit": limit})),
        Err(e) => Json(json!({"error": e})),
    }
}

async fn list_customers(_auth: AdminAuth, Query(params): Query<PaginationParams>) -> Json<Value> {
    let page = params.page.unwrap_or(1);
    let limit = params.limit.unwrap_or(20);
    match mongo::find_paginated("customers", Document::new(), page, limit).await {
        Ok((data, total)) => Json(json!({"data": data, "total": total, "page": page, "limit": limit})),
        Err(e) => Json(json!({"error": e})),
    }
}

async fn list_coupons(_auth: AdminAuth, Query(params): Query<PaginationParams>) -> Json<Value> {
    let page = params.page.unwrap_or(1);
    let limit = params.limit.unwrap_or(20);
    match mongo::find_paginated("coupons", Document::new(), page, limit).await {
        Ok((data, total)) => Json(json!({"data": data, "total": total, "page": page, "limit": limit})),
        Err(e) => Json(json!({"error": e})),
    }
}

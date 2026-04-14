use axum::{extract::{Path, Query}, response::Json, routing::get, Router};
use mongodb::bson::{self, doc, Document};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::services::mongo;

#[derive(Deserialize, Serialize)]
pub struct AuditLogFilter {
    pub entity: Option<String>,
    pub action: Option<String>,
}

#[derive(Deserialize, Serialize)]
pub struct AuditLogPayload {
    pub entity: String,
    pub action: String,
    pub user_id: Option<String>,
    pub changes: Option<Value>,
}

#[derive(Deserialize, Serialize)]
pub struct AuditLogUpdate {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entity: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<Option<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub changes: Option<Option<Value>>,
}

pub fn router() -> Router {
    Router::new()
        .route("/", get(get_audit_logs).post(create_audit_log))
        .route("/:id", get(get_audit_log).put(update_audit_log).delete(delete_audit_log))
}

async fn get_audit_logs(Query(filter): Query<AuditLogFilter>) -> Json<Value> {
    let mut query = Document::new();
    if let Some(value) = filter.entity {
        query.insert("entity", value);
    }
    if let Some(value) = filter.action {
        query.insert("action", value);
    }
    match mongo::find_all("audit_logs", query).await {
        Ok(items) => Json(json!({"data": items})),
        Err(error) => Json(json!({"error": error})),
    }
}

async fn get_audit_log(Path(id): Path<String>) -> Json<Value> {
    match mongo::find_one("audit_logs", &id).await {
        Ok(Some(item)) => Json(json!({"data": item})),
        Ok(None) => Json(json!({"error": "not found"})),
        Err(error) => Json(json!({"error": error})),
    }
}

async fn create_audit_log(Json(payload): Json<AuditLogPayload>) -> Json<Value> {
    let document = bson::to_document(&payload).unwrap_or_default();
    match mongo::insert_one("audit_logs", document).await {
        Ok(result) => Json(json!({"data": result})),
        Err(error) => Json(json!({"error": error})),
    }
}

async fn update_audit_log(Path(id): Path<String>, Json(payload): Json<AuditLogUpdate>) -> Json<Value> {
    let updates = bson::to_document(&payload).unwrap_or_default();
    if updates.is_empty() {
        return Json(json!({"error": "no fields to update"}));
    }
    match mongo::update_one("audit_logs", &id, updates).await {
        Ok(Some(item)) => Json(json!({"data": item})),
        Ok(None) => Json(json!({"error": "not found"})),
        Err(error) => Json(json!({"error": error})),
    }
}

async fn delete_audit_log(Path(id): Path<String>) -> Json<Value> {
    match mongo::delete_one("audit_logs", &id).await {
        Ok(count) => Json(json!({"deleted_count": count})),
        Err(error) => Json(json!({"error": error})),
    }
}

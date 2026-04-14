use axum::{response::Json, routing::{delete, post}, Router};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::services::storage;

pub fn router() -> Router {
    Router::new()
        .route("/presign", post(presign_upload))
        .route("/confirm", post(confirm_upload))
        .route("/", delete(delete_upload))
        .route("/bulk-delete", post(bulk_delete))
}

/// POST /api/uploads/presign
/// Solicita uma Presigned URL para o frontend enviar a imagem diretamente ao MinIO.
///
/// Body: { "filename": "foto.jpg", "folder": "products" }
/// Returns: { "key": "temp/uuid-foto.jpg", "url": "https://...", "expires_in": 900 }
#[derive(Deserialize)]
struct PresignRequest {
    filename: String,
}

async fn presign_upload(Json(payload): Json<PresignRequest>) -> Json<Value> {
    match storage::presigned_upload_url(&payload.filename).await {
        Ok((key, url)) => Json(json!({
            "key": key,
            "url": url,
            "expires_in": 900,
            "instructions": "Faça um PUT para 'url' com o arquivo binário no body. Guarde 'key' para confirmar depois."
        })),
        Err(e) => Json(json!({"error": e})),
    }
}

/// POST /api/uploads/confirm
/// Move a imagem de temp/ para a pasta definitiva após confirmação do cadastro.
///
/// Body: { "temp_key": "temp/uuid-foto.jpg", "dest_folder": "products" }
/// Returns: { "key": "products/foto.jpg", "url": "http://minio:9000/omnistore/products/foto.jpg" }
#[derive(Deserialize)]
struct ConfirmRequest {
    temp_key: String,
    dest_folder: String,
}

async fn confirm_upload(Json(payload): Json<ConfirmRequest>) -> Json<Value> {
    if !payload.temp_key.starts_with("temp/") {
        return Json(json!({"error": "chave deve estar na pasta temp/"}));
    }
    match storage::confirm_upload(&payload.temp_key, &payload.dest_folder).await {
        Ok(key) => Json(json!({
            "key": key,
            "url": storage::public_url(&key)
        })),
        Err(e) => Json(json!({"error": e})),
    }
}

/// DELETE /api/uploads
/// Deleta um objeto permanente do bucket (ex: trocar foto de produto).
///
/// Body: { "key": "products/foto.jpg" }
#[derive(Deserialize)]
struct DeleteRequest {
    key: String,
}

async fn delete_upload(Json(payload): Json<DeleteRequest>) -> Json<Value> {
    if payload.key.starts_with("temp/") {
        return Json(json!({"error": "use este endpoint apenas para arquivos permanentes"}));
    }
    match storage::delete_object(&payload.key).await {
        Ok(()) => Json(json!({"deleted": true, "key": payload.key})),
        Err(e) => Json(json!({"error": e})),
    }
}

/// POST /api/uploads/bulk-delete
/// Deleta múltiplos objetos de uma vez (ex: remover produto com todas as fotos).
///
/// Body: { "keys": ["products/a.jpg", "products/b.jpg"] }
#[derive(Deserialize, Serialize)]
struct BulkDeleteRequest {
    keys: Vec<String>,
}

async fn bulk_delete(Json(payload): Json<BulkDeleteRequest>) -> Json<Value> {
    let safe_keys: Vec<String> = payload
        .keys
        .into_iter()
        .filter(|k| !k.starts_with("temp/"))
        .collect();

    if safe_keys.is_empty() {
        return Json(json!({"error": "nenhuma chave válida fornecida"}));
    }

    match storage::delete_objects(safe_keys).await {
        Ok(count) => Json(json!({"deleted_count": count})),
        Err(e) => Json(json!({"error": e})),
    }
}

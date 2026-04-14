use axum::{response::Json, routing::get, Router};
use mongodb::bson::{self, doc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::services::mongo;

const STORE_COLLECTION: &str = "store_config";

#[derive(Deserialize, Serialize)]
pub struct StoreConfigUpdate {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logo_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub theme_colors: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gateway_keys: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Value>,
}

pub fn router() -> Router {
    Router::new()
        .route("/", get(get_store_config).put(update_store_config))
}

/// GET /api/store - Retorna a configuração única da loja
async fn get_store_config() -> Json<Value> {
    // Busca o primeiro (e único) documento de configuração
    match mongo::find_all(STORE_COLLECTION, doc! {}).await {
        Ok(items) if !items.is_empty() => Json(json!({"data": items[0].clone()})),
        Ok(_) => {
            // Se não existe, retorna config padrão
            Json(json!({
                "data": {
                    "name": "Minha Loja",
                    "logo_url": null,
                    "theme_colors": {},
                    "gateway_keys": {},
                    "metadata": {},
                    "created_at": null,
                    "updated_at": null
                },
                "message": "Using default config. Update to save your settings."
            }))
        }
        Err(error) => Json(json!({"error": error})),
    }
}

/// PUT /api/store - Atualiza a configuração única da loja
async fn update_store_config(Json(payload): Json<StoreConfigUpdate>) -> Json<Value> {
    let updates = bson::to_document(&payload).unwrap_or_default();
    if updates.is_empty() {
        return Json(json!({"error": "no fields to update"}));
    }

    // Verifica se já existe uma config
    match mongo::find_all(STORE_COLLECTION, doc! {}).await {
        Ok(items) if !items.is_empty() => {
            // Atualiza a config existente
            if let Some(id_value) = items[0].get("_id") {
                if let Some(id_str) = id_value.as_str() {
                    match mongo::update_one(STORE_COLLECTION, id_str, updates).await {
                        Ok(Some(item)) => Json(json!({"data": item})),
                        Ok(None) => Json(json!({"error": "not found"})),
                        Err(error) => Json(json!({"error": error})),
                    }
                } else {
                    Json(json!({"error": "invalid id format"}))
                }
            } else {
                Json(json!({"error": "no id found"}))
            }
        }
        Ok(_) => {
            // Cria nova config se não existe
            match mongo::insert_one(STORE_COLLECTION, updates).await {
                Ok(result) => Json(json!({"data": result, "message": "Store config created"})),
                Err(error) => Json(json!({"error": error})),
            }
        }
        Err(error) => Json(json!({"error": error})),
    }
}

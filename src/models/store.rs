use mongodb::bson::{DateTime, oid::ObjectId};
use serde::{Deserialize, Serialize};
use super::field::Metadata;

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
pub struct Store {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub name: String,
    pub domain: Option<String>,
    pub logo_url: Option<String>,
    pub theme_colors: Option<Metadata>,
    pub firebase_config: Option<Metadata>,
    pub gateway_keys: Option<Metadata>,
    #[serde(default)]
    pub metadata: Metadata,
    pub active: bool,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

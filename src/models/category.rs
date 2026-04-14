use mongodb::bson::{DateTime, oid::ObjectId};
use serde::{Deserialize, Serialize};
use super::field::Metadata;

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
pub struct Category {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub name: String,
    pub slug: String,
    pub parent_id: Option<ObjectId>,
    pub description: Option<String>,
    #[serde(default)]
    pub metadata: Metadata,
    pub active: bool,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

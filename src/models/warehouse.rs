use mongodb::bson::{oid::ObjectId, DateTime};
use serde::{Deserialize, Serialize};
use super::field::Metadata;

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
pub struct Warehouse {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,

    pub name: String,
    pub location: String,

    #[serde(default)]
    pub metadata: Metadata,

    pub active: bool,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

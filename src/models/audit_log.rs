use mongodb::bson::{DateTime, oid::ObjectId};
use serde::{Deserialize, Serialize};
use super::field::Metadata;

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
pub struct AuditLog {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub user_id: Option<ObjectId>,
    pub action: String,
    pub entity: String,
    pub entity_id: Option<ObjectId>,
    pub summary: Option<String>,
    #[serde(default)]
    pub metadata: Metadata,
    pub created_at: DateTime,
}

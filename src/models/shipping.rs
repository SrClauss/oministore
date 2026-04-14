use mongodb::bson::{DateTime, oid::ObjectId};
use serde::{Deserialize, Serialize};
use super::field::Metadata;

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
pub struct Shipping {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,

    pub carrier: String,
    pub service_level: String,
    pub cost: f64,
    pub address: String,
    pub estimated_delivery_days: Option<i32>,
    pub tracking_number: Option<String>,
    pub active: bool,
    pub created_at: DateTime,
    pub updated_at: DateTime,

    #[serde(default)]
    pub metadata: Metadata,
}

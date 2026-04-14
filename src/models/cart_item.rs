use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use super::field::Metadata;

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
pub struct CartItem {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub product_id: ObjectId,
    pub product_name: String,
    pub variant_id: Option<ObjectId>,
    pub quantity: i64,
    pub unit_price: f64,
    pub total_price: f64,
    #[serde(default)]
    pub metadata: Metadata,
}

use mongodb::bson::{DateTime, oid::ObjectId};
use serde::{Deserialize, Serialize};
use super::field::Metadata;
use super::inventory::Inventory;

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
pub struct Product {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,

    pub name: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub category_ids: Vec<ObjectId>,
    pub photos: Vec<String>,
    pub media: Vec<String>,
    pub sku: String,
    pub color: Option<String>,
    pub attributes: Vec<String>,
    pub related_product_ids: Vec<ObjectId>,
    pub inventory: Inventory,
    pub variant_product_ids: Vec<ObjectId>,

    #[serde(default)]
    pub metadata: Metadata,

    pub active: bool,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

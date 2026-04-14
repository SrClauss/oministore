use mongodb::bson::DateTime;
use serde::{Deserialize, Serialize};
use super::field::Metadata;
use super::warehouse::Warehouse;

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
pub struct Inventory {
    pub quantity: i64,
    pub reserved: i64,
    pub warehouse: Warehouse,
    pub sell_without_stock: bool,

    #[serde(default)]
    pub metadata: Metadata,

    pub updated_at: DateTime,
}

use mongodb::bson::DateTime;
use serde::{Deserialize, Serialize};
use super::field::Metadata;

/// Estoque simplificado (single-warehouse)
#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
pub struct Inventory {
    /// Quantidade disponível em estoque
    pub quantity: i64,
    
    /// Quantidade reservada (em carrinhos)
    pub reserved: i64,
    
    /// Permite vender mesmo sem estoque
    pub sell_without_stock: bool,

    #[serde(default)]
    pub metadata: Metadata,

    pub updated_at: DateTime,
}

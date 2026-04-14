use mongodb::bson::{DateTime, oid::ObjectId};
use serde::{Deserialize, Serialize};
use super::field::Metadata;

/// Configuração única da loja (single-tenant)
#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
pub struct StoreConfig {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    
    /// Nome da loja
    pub name: String,
    
    /// URL do logo
    pub logo_url: Option<String>,
    
    /// Cores do tema (hex colors)
    pub theme_colors: Option<Metadata>,
    
    /// Chaves dos gateways de pagamento (Mercado Pago, Asaas)
    pub gateway_keys: Option<Metadata>,
    
    /// Metadados adicionais
    #[serde(default)]
    pub metadata: Metadata,
    
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

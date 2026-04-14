use mongodb::bson::{oid::ObjectId, DateTime};
use serde::{Deserialize, Serialize};
use super::field::Metadata;

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
pub struct Address {
    pub street: String,
    pub city: String,
    pub state: String,
    pub postal_code: String,
    pub country: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Metadata>,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
pub struct Customer {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub name: String,
    pub email: String,
    /// Firebase UID linking this customer to a Firebase Auth account.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub firebase_uid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<Address>,
    #[serde(default)]
    pub metadata: Metadata,
    pub active: bool,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

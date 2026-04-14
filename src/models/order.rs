use mongodb::bson::{DateTime, oid::ObjectId};
use serde::{Deserialize, Serialize};
use super::field::Metadata;

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
pub enum PaymentStatus {
    Pending,
    Paid,
    Failed,
    Refunded,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
pub struct BillingDetails {
    pub name: String,
    pub email: String,
    pub document: Option<String>,
    pub address: String,
    pub phone: Option<String>,
    #[serde(default)]
    pub metadata: Metadata,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
pub struct OrderItem {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub product_id: ObjectId,
    pub product_name: String,
    pub sku: String,
    pub quantity: i64,
    pub unit_price: f64,
    pub total_price: f64,
    #[serde(default)]
    pub metadata: Metadata,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
pub struct Order {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub store_id: ObjectId,
    pub customer_id: ObjectId,
    pub items: Vec<OrderItem>,
    pub payment_status: PaymentStatus,
    pub shipping_address: String,
    pub billing_details: BillingDetails,
    pub subtotal: f64,
    pub discount_total: f64,
    pub shipping_total: f64,
    pub total: f64,
    #[serde(default)]
    pub metadata: Metadata,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

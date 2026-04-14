use mongodb::bson::{DateTime, oid::ObjectId};
use serde::{Deserialize, Serialize};
use super::cart_item::CartItem;
use super::coupon::Coupon;
use super::field::Metadata;
use super::shipping::Shipping;

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
pub enum CartStatus {
    Draft,
    Pending,
    Completed,
    Abandoned,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
pub struct Cart {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,

    pub user_id: ObjectId,
    pub items: Vec<CartItem>,
    pub shipping: Option<Shipping>,
    pub coupon: Option<Coupon>,

    pub subtotal: f64,
    pub discount_total: f64,
    pub shipping_total: f64,
    pub total: f64,
    pub status: CartStatus,

    #[serde(default)]
    pub metadata: Metadata,

    pub created_at: DateTime,
    pub updated_at: DateTime,
}

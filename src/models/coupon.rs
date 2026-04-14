use mongodb::bson::{DateTime, oid::ObjectId};
use serde::{Deserialize, Serialize};
use super::field::Metadata;

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
pub enum DiscountType {
    Percent,
    Fixed,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
pub struct Coupon {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,

    pub code: String,
    pub discount_type: DiscountType,
    pub discount_value: f64,
    pub min_cart_total: Option<f64>,
    pub valid_until: Option<DateTime>,
    pub active: bool,

    #[serde(default)]
    pub metadata: Metadata,
}

use axum::{extract::{Path, Query}, response::Json, routing::{get, post}, Router};
use mongodb::bson::{self, doc, Document};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::services::mongo;

#[derive(Deserialize, Serialize)]
pub struct CouponFilter {
    pub code: Option<String>,
    pub active: Option<bool>,
}

#[derive(Deserialize, Serialize)]
pub struct CouponPayload {
    pub code: String,
    pub discount_type: String,
    pub discount_value: f64,
    pub active: bool,
}

#[derive(Deserialize, Serialize)]
pub struct CouponUpdate {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discount_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discount_value: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active: Option<bool>,
}

#[derive(Deserialize)]
pub struct CouponValidateRequest {
    pub code: String,
    pub cart_total: Option<f64>,
}

pub fn router() -> Router {
    Router::new()
        .route("/", get(get_coupons).post(create_coupon))
        .route("/validate", post(validate_coupon))
        .route("/:id", get(get_coupon).put(update_coupon).delete(delete_coupon))
}

/// POST /api/coupons/validate
///
/// Recebe `{ code, cart_total }` e retorna se o cupom é válido, junto com o
/// tipo e valor de desconto. Valida: existe, está ativo, não expirou,
/// ainda tem usos disponíveis, e o total do carrinho atinge o mínimo exigido.
async fn validate_coupon(Json(payload): Json<CouponValidateRequest>) -> Json<Value> {
    let filter = doc! { "code": &payload.code };
    let coupon = match mongo::find_one_by_filter("coupons", filter).await {
        Ok(Some(c)) => c,
        Ok(None) => return Json(json!({ "valid": false, "reason": "coupon not found" })),
        Err(e) => return Json(json!({ "valid": false, "reason": e })),
    };

    // Verifica se está ativo
    if !coupon.get("active").and_then(Value::as_bool).unwrap_or(false) {
        return Json(json!({ "valid": false, "reason": "coupon is inactive" }));
    }

    // Verifica validade (valid_until como milissegundos epoch ou string ISO 8601)
    if let Some(valid_until) = coupon.get("valid_until") {
        let expired = if let Some(ms_str) = valid_until
            .get("$date")
            .and_then(|d| d.get("$numberLong").and_then(Value::as_str))
        {
            // Formato BSON ISODate armazenado como milliseconds since epoch
            ms_str
                .parse::<i64>()
                .map(|ms| {
                    use std::time::{SystemTime, UNIX_EPOCH};
                    let now_ms = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .map(|d| d.as_millis() as i64)
                        .unwrap_or(i64::MAX);
                    ms < now_ms
                })
                .unwrap_or(false)
        } else if let Some(s) = valid_until.as_str() {
            // Formato string ISO 8601 (ex: "2025-12-31T23:59:59Z")
            // ISO 8601 strings are ASCII and sort lexicographically, so byte-slicing is safe.
            is_iso_date_past(s)
        } else {
            false
        };

        if expired {
            return Json(json!({ "valid": false, "reason": "coupon has expired" }));
        }
    }

    // Verifica max_uses / usage_count
    if let Some(max_uses) = coupon.get("max_uses").and_then(Value::as_i64) {
        let usage_count = coupon.get("usage_count").and_then(Value::as_i64).unwrap_or(0);
        if usage_count >= max_uses {
            return Json(json!({ "valid": false, "reason": "coupon usage limit reached" }));
        }
    }

    // Verifica mínimo de carrinho
    if let Some(min_total) = coupon.get("min_cart_total").and_then(Value::as_f64) {
        let cart_total = payload.cart_total.unwrap_or(0.0);
        if cart_total < min_total {
            return Json(json!({
                "valid": false,
                "reason": format!("cart total {cart_total} is below minimum {min_total}")
            }));
        }
    }

    let discount_type = coupon
        .get("discount_type")
        .and_then(Value::as_str)
        .unwrap_or("fixed");
    let discount_value = coupon.get("discount_value").and_then(Value::as_f64).unwrap_or(0.0);

    Json(json!({
        "valid": true,
        "discount_type": discount_type,
        "discount_value": discount_value,
    }))
}

/// Retorna true se a data ISO 8601 já passou.
///
/// Compara os primeiros 19 bytes da string ("YYYY-MM-DDTHH:MM:SS") com a data/hora
/// atual no mesmo formato. ISO 8601 strings são ASCII puro, portanto o slicing em bytes
/// é seguro.
fn is_iso_date_past(date_str: &str) -> bool {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now_secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let now_iso = secs_to_iso(now_secs);
    let len = date_str.len().min(19);
    // ISO 8601 strings are ASCII; byte slicing is safe.
    let date_prefix = &date_str.as_bytes()[..len];
    let now_prefix = &now_iso.as_bytes()[..now_iso.len().min(19)];
    date_prefix < now_prefix
}

/// Converte Unix timestamp (segundos) em string ISO 8601 "YYYY-MM-DDTHH:MM:SS".
/// Usa o algoritmo civil_from_days de Howard Hinnant.
fn secs_to_iso(secs: u64) -> String {
    let days_total = secs / 86400;
    let rem = secs % 86400;
    let hour = rem / 3600;
    let min = (rem % 3600) / 60;
    let sec = rem % 60;

    let z = days_total as i64 + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };

    format!("{y:04}-{m:02}-{d:02}T{hour:02}:{min:02}:{sec:02}")
}

async fn get_coupons(Query(filter): Query<CouponFilter>) -> Json<Value> {
    let mut query = Document::new();
    if let Some(value) = filter.code {
        query.insert("code", value);
    }
    if let Some(value) = filter.active {
        query.insert("active", value);
    }
    match mongo::find_all("coupons", query).await {
        Ok(items) => Json(json!({"data": items})),
        Err(error) => Json(json!({"error": error})),
    }
}

async fn get_coupon(Path(id): Path<String>) -> Json<Value> {
    match mongo::find_one("coupons", &id).await {
        Ok(Some(item)) => Json(json!({"data": item})),
        Ok(None) => Json(json!({"error": "not found"})),
        Err(error) => Json(json!({"error": error})),
    }
}

async fn create_coupon(Json(payload): Json<CouponPayload>) -> Json<Value> {
    let document = bson::to_document(&payload).unwrap_or_default();
    match mongo::insert_one("coupons", document).await {
        Ok(result) => Json(json!({"data": result})),
        Err(error) => Json(json!({"error": error})),
    }
}

async fn update_coupon(Path(id): Path<String>, Json(payload): Json<CouponUpdate>) -> Json<Value> {
    let updates = bson::to_document(&payload).unwrap_or_default();
    if updates.is_empty() {
        return Json(json!({"error": "no fields to update"}));
    }
    match mongo::update_one("coupons", &id, updates).await {
        Ok(Some(item)) => Json(json!({"data": item})),
        Ok(None) => Json(json!({"error": "not found"})),
        Err(error) => Json(json!({"error": error})),
    }
}

async fn delete_coupon(Path(id): Path<String>) -> Json<Value> {
    match mongo::delete_one("coupons", &id).await {
        Ok(count) => Json(json!({"deleted_count": count})),
        Err(error) => Json(json!({"error": error})),
    }
}

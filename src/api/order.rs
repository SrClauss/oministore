use axum::{extract::{Path, Query}, response::Json, routing::{get, post}, Router};
use mongodb::bson::{self, doc, Document};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::env;

use crate::services::mongo;

#[derive(Deserialize, Serialize)]
pub struct OrderFilter {
    pub customer_id: Option<String>,
    pub store_id: Option<String>,
    pub payment_status: Option<String>,
}

#[derive(Deserialize, Serialize)]
pub struct OrderPayload {
    pub customer_id: String,
    pub store_id: String,
    pub items: Vec<Value>,
    pub payment_status: String,
    pub shipping_address: String,
    pub billing_details: Value,
    pub subtotal: f64,
    pub discount_total: f64,
    pub shipping_total: f64,
    pub total: f64,
}

#[derive(Deserialize, Serialize)]
pub struct OrderUpdate {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payment_status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shipping_address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub billing_details: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtotal: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discount_total: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shipping_total: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<f64>,
}

pub fn router() -> Router {
    Router::new()
        .route("/", get(get_orders).post(create_order))
        .route("/checkout", post(create_order_checkout))
        .route("/checkout/asaas", post(create_order_checkout_asaas))
        .route("/:id", get(get_order).put(update_order).delete(delete_order))
}

#[derive(Deserialize, Serialize)]
pub struct AsaasCustomer {
    pub name: String,
    #[serde(rename = "cpfCnpj")]
    pub cpf_cnpj: String,
    pub email: String,
    pub phone: Option<String>,
}

#[derive(Deserialize, Serialize)]
pub struct AsaasCheckoutRequest {
    pub cart_id: String,
    pub store_id: Option<String>,
    pub customer: AsaasCustomer,
    pub billing_type: Option<String>,
    pub due_date: Option<String>,
    pub description: Option<String>,
    pub notification_url: Option<String>,
}

#[derive(Deserialize, Serialize)]
pub struct CheckoutRequest {
    pub cart_id: String,
    pub store_id: Option<String>,
    pub back_urls: Option<CheckoutBackUrls>,
    pub notification_url: Option<String>,
    pub payer: Option<CheckoutPayer>,
    pub external_reference: Option<String>,
    pub auto_return: Option<String>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct CheckoutBackUrls {
    pub success: String,
    pub failure: String,
    pub pending: String,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct CheckoutPayer {
    pub email: String,
}

#[derive(Deserialize, Serialize)]
struct MercadoPagoPreferenceResponse {
    pub id: String,
    #[serde(rename = "init_point")]
    pub init_point: Option<String>,
    #[serde(rename = "sandbox_init_point")]
    pub sandbox_init_point: Option<String>,
}

#[derive(Serialize)]
struct CheckoutResponse {
    pub order_id: String,
    pub preference_id: String,
    pub checkout_url: String,
}

async fn create_order_checkout(Json(payload): Json<CheckoutRequest>) -> Json<Value> {
    let cart = match mongo::find_one("carts", &payload.cart_id).await {
        Ok(Some(item)) => item,
        Ok(None) => return Json(json!({"error": "cart not found"})),
        Err(error) => return Json(json!({"error": error})),
    };

    // Compute external_reference before calling preference so we can store it in the order
    let external_reference = payload
        .external_reference
        .clone()
        .unwrap_or_else(|| {
            cart.get("_id")
                .and_then(|v| v.get("$oid"))
                .and_then(Value::as_str)
                .map(|s| s.to_string())
                .unwrap_or_else(|| "cart_checkout".to_string())
        });

    let preference = match create_mercado_pago_preference(&cart, &payload, &external_reference).await {
        Ok(pref) => pref,
        Err(error) => return Json(json!({"error": error})),
    };

    let checkout_url = preference
        .sandbox_init_point
        .as_ref()
        .or(preference.init_point.as_ref())
        .cloned()
        .unwrap_or_else(|| "".to_string());

    if checkout_url.is_empty() {
        return Json(json!({"error": "failed to resolve Mercado Pago checkout URL"}));
    }

    let order_doc = build_order_from_cart(&cart, &payload, &preference, &external_reference);
    let order_insert = match mongo::insert_one("orders", order_doc).await {
        Ok(inserted) => inserted,
        Err(error) => return Json(json!({"error": error})),
    };

    let _ = mongo::update_one("carts", &payload.cart_id, doc! {"status": "ordered"}).await;

    Json(json!({
        "data": CheckoutResponse {
            order_id: order_insert["inserted_id"].to_string(),
            preference_id: preference.id,
            checkout_url,
        }
    }))
}

async fn create_mercado_pago_preference(
    cart: &Value,
    payload: &CheckoutRequest,
    external_reference: &str,
) -> Result<MercadoPagoPreferenceResponse, String> {
    let access_token = env::var("MERCADO_PAGO_ACCESS_TOKEN")
        .map_err(|_| "MERCADO_PAGO_ACCESS_TOKEN is required".to_string())?;
    let base_url = env::var("MERCADO_PAGO_BASE_URL")
        .unwrap_or_else(|_| "https://api.mercadopago.com".to_string());

    let back_urls = payload.back_urls.clone().unwrap_or(CheckoutBackUrls {
        success: env::var("CHECKOUT_SUCCESS_URL")
            .unwrap_or_else(|_| "https://api.arkana.fun/checkout/success".to_string()),
        failure: env::var("CHECKOUT_FAILURE_URL")
            .unwrap_or_else(|_| "https://api.arkana.fun/checkout/failure".to_string()),
        pending: env::var("CHECKOUT_PENDING_URL")
            .unwrap_or_else(|_| "https://api.arkana.fun/checkout/pending".to_string()),
    });

    let notification_url = payload.notification_url.clone().unwrap_or_else(|| {
        env::var("MERCADO_PAGO_NOTIFICATION_URL")
            .unwrap_or_else(|_| "https://api.arkana.fun/api/webhooks/mercadopago".to_string())
    });

    let payer = payload.payer.clone().unwrap_or(CheckoutPayer {
        email: cart
            .get("user_id")
            .and_then(Value::as_str)
            .unwrap_or("payer@example.com")
            .to_string(),
    });

    let items = cart.get("items").cloned().unwrap_or_else(|| json!([]));

    let request_body = json!({
        "items": items,
        "payer": {
            "email": payer.email,
        },
        "back_urls": {
            "success": back_urls.success,
            "failure": back_urls.failure,
            "pending": back_urls.pending,
        },
        "notification_url": notification_url,
        "external_reference": external_reference,
        "auto_return": payload.auto_return.clone().unwrap_or_else(|| "approved".to_string()),
    });

    let client = reqwest::Client::new();
    let response = client
        .post(format!("{}/checkout/preferences", base_url))
        .bearer_auth(access_token)
        .json(&request_body)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let status = response.status();
    let text = response.text().await.map_err(|e| e.to_string())?;

    if !status.is_success() {
        return Err(format!("Mercado Pago API failure {}: {}", status, text));
    }

    serde_json::from_str::<MercadoPagoPreferenceResponse>(&text).map_err(|e| e.to_string())
}

fn build_order_from_cart(
    cart: &Value,
    payload: &CheckoutRequest,
    preference: &MercadoPagoPreferenceResponse,
    external_reference: &str,
) -> Document {
    let shipping_address = cart
        .get("shipping")
        .and_then(|v| v.get("address"))
        .and_then(Value::as_str)
        .unwrap_or("not provided")
        .to_string();

    let billing_details = json!({
        "payer": payload
            .payer
            .as_ref()
            .map(|p| json!({"email": p.email.clone()}))
            .unwrap_or_else(|| json!({
                "email": cart
                    .get("user_id")
                    .and_then(Value::as_str)
                    .unwrap_or("payer@example.com")
            })),
        "preference_id": preference.id,
    });

    let order_value = json!({
        "customer_id": cart
            .get("user_id")
            .cloned()
            .unwrap_or_else(|| json!(null)),
        "store_id": payload
            .store_id
            .clone()
            .unwrap_or_else(|| "default-store".to_string()),
        "items": cart.get("items").cloned().unwrap_or_else(|| json!([])),
        "payment_status": "pending",
        "payment_provider": "mercadopago",
        "external_reference": external_reference,
        "shipping_address": shipping_address,
        "billing_details": billing_details,
        "subtotal": cart.get("subtotal").and_then(Value::as_f64).unwrap_or(0.0),
        "discount_total": cart.get("discount_total").and_then(Value::as_f64).unwrap_or(0.0),
        "shipping_total": cart.get("shipping_total").and_then(Value::as_f64).unwrap_or(0.0),
        "total": cart.get("total").and_then(Value::as_f64).unwrap_or(0.0),
    });

    bson::to_document(&order_value).unwrap_or_default()
}

async fn create_order_checkout_asaas(Json(payload): Json<AsaasCheckoutRequest>) -> Json<Value> {
    let cart = match mongo::find_one("carts", &payload.cart_id).await {
        Ok(Some(item)) => item,
        Ok(None) => return Json(json!({"error": "cart not found"})),
        Err(error) => return Json(json!({"error": error})),
    };

    let sandbox = env::var("ASAAS_SANDBOX").unwrap_or_else(|_| "true".to_string());
    let base_url = if sandbox == "true" {
        env::var("ASAAS_SANDBOX_URL").unwrap_or_else(|_| "https://sandbox.asaas.com".to_string())
    } else {
        env::var("ASAAS_URL").unwrap_or_else(|_| "https://www.asaas.com".to_string())
    };

    let api_key = match env::var("ASAAS_API_KEY") {
        Ok(k) => k,
        Err(_) => return Json(json!({"error": "ASAAS_API_KEY is required"})),
    };

    let client = reqwest::Client::new();

    // Upsert customer in Asaas
    let customer_body = json!({
        "name": payload.customer.name,
        "cpfCnpj": payload.customer.cpf_cnpj,
        "email": payload.customer.email,
        "mobilePhone": payload.customer.phone.clone().unwrap_or_default(),
    });

    let customer_resp = match client
        .post(format!("{}/api/v3/customers", base_url))
        .header("access_token", &api_key)
        .json(&customer_body)
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => return Json(json!({"error": format!("asaas customer request failed: {e}")})),
    };

    let customer_json: Value = match customer_resp.json().await {
        Ok(v) => v,
        Err(e) => return Json(json!({"error": format!("asaas customer parse failed: {e}")})),
    };

    let asaas_customer_id = match customer_json.get("id").and_then(Value::as_str) {
        Some(id) => id.to_string(),
        None => return Json(json!({"error": "asaas did not return customer id", "details": customer_json})),
    };

    // Determine cart external_reference (cart _id)
    let external_reference = cart
        .get("_id")
        .and_then(|v| v.get("$oid"))
        .and_then(Value::as_str)
        .map(|s| s.to_string())
        .unwrap_or_else(|| payload.cart_id.clone());

    let billing_type = payload.billing_type.clone().unwrap_or_else(|| "PIX".to_string());
    let due_date = payload.due_date.clone().unwrap_or_else(|| "2025-12-31".to_string());
    let description = payload.description.clone().unwrap_or_else(|| "Pedido omnistore".to_string());
    let total = cart.get("total").and_then(Value::as_f64).unwrap_or(0.0);

    let payment_body = json!({
        "customer": asaas_customer_id,
        "billingType": billing_type,
        "value": total,
        "dueDate": due_date,
        "description": description,
        "externalReference": external_reference,
        "notificationEnabled": true,
    });

    let payment_resp = match client
        .post(format!("{}/api/v3/payments", base_url))
        .header("access_token", &api_key)
        .json(&payment_body)
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => return Json(json!({"error": format!("asaas payment request failed: {e}")})),
    };

    let payment_json: Value = match payment_resp.json().await {
        Ok(v) => v,
        Err(e) => return Json(json!({"error": format!("asaas payment parse failed: {e}")})),
    };

    let asaas_payment_id = match payment_json.get("id").and_then(Value::as_str) {
        Some(id) => id.to_string(),
        None => return Json(json!({"error": "asaas did not return payment id", "details": payment_json})),
    };

    let invoice_url = payment_json.get("invoiceUrl").and_then(Value::as_str).unwrap_or("").to_string();
    let bank_slip_url = payment_json.get("bankSlipUrl").and_then(Value::as_str).unwrap_or("").to_string();
    let pix_payload = payment_json
        .get("pixQrCode")
        .and_then(|v| v.get("payload"))
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let pix_encoded_image = payment_json
        .get("pixQrCode")
        .and_then(|v| v.get("encodedImage"))
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();

    // Build order document
    let shipping_address = cart
        .get("shipping")
        .and_then(|v| v.get("address"))
        .and_then(Value::as_str)
        .unwrap_or("not provided")
        .to_string();

    let order_value = json!({
        "customer_id": cart.get("user_id").cloned().unwrap_or_else(|| json!(null)),
        "store_id": payload.store_id.clone().unwrap_or_else(|| "default-store".to_string()),
        "items": cart.get("items").cloned().unwrap_or_else(|| json!([])),
        "payment_status": "pending",
        "payment_provider": "asaas",
        "external_reference": external_reference,
        "asaas_payment_id": asaas_payment_id,
        "shipping_address": shipping_address,
        "billing_details": {
            "customer": payload.customer.name,
            "cpf_cnpj": payload.customer.cpf_cnpj,
            "email": payload.customer.email,
            "billing_type": billing_type,
            "due_date": due_date,
            "asaas_customer_id": asaas_customer_id,
        },
        "subtotal": cart.get("subtotal").and_then(Value::as_f64).unwrap_or(0.0),
        "discount_total": cart.get("discount_total").and_then(Value::as_f64).unwrap_or(0.0),
        "shipping_total": cart.get("shipping_total").and_then(Value::as_f64).unwrap_or(0.0),
        "total": total,
    });

    let order_doc = bson::to_document(&order_value).unwrap_or_default();
    let order_insert = match mongo::insert_one("orders", order_doc).await {
        Ok(inserted) => inserted,
        Err(error) => return Json(json!({"error": error})),
    };

    let _ = mongo::update_one("carts", &payload.cart_id, doc! {"status": "ordered"}).await;

    Json(json!({
        "data": {
            "order_id": order_insert["inserted_id"].to_string(),
            "payment_id": asaas_payment_id,
            "invoice_url": invoice_url,
            "bank_slip_url": bank_slip_url,
            "pix_payload": pix_payload,
            "pix_encoded_image": pix_encoded_image,
        }
    }))
}

async fn get_orders(Query(filter): Query<OrderFilter>) -> Json<Value> {
    let mut query = Document::new();
    if let Some(value) = filter.customer_id {
        query.insert("customer_id", value);
    }
    if let Some(value) = filter.store_id {
        query.insert("store_id", value);
    }
    if let Some(value) = filter.payment_status {
        query.insert("payment_status", value);
    }
    match mongo::find_all("orders", query).await {
        Ok(items) => Json(json!({"data": items})),
        Err(error) => Json(json!({"error": error})),
    }
}

async fn get_order(Path(id): Path<String>) -> Json<Value> {
    match mongo::find_one("orders", &id).await {
        Ok(Some(item)) => Json(json!({"data": item})),
        Ok(None) => Json(json!({"error": "not found"})),
        Err(error) => Json(json!({"error": error})),
    }
}

async fn create_order(Json(payload): Json<OrderPayload>) -> Json<Value> {
    let document = bson::to_document(&payload).unwrap_or_default();
    match mongo::insert_one("orders", document).await {
        Ok(result) => Json(json!({"data": result})),
        Err(error) => Json(json!({"error": error})),
    }
}

async fn update_order(Path(id): Path<String>, Json(payload): Json<OrderUpdate>) -> Json<Value> {
    let updates = bson::to_document(&payload).unwrap_or_default();
    if updates.is_empty() {
        return Json(json!({"error": "no fields to update"}));
    }
    match mongo::update_one("orders", &id, updates).await {
        Ok(Some(item)) => Json(json!({"data": item})),
        Ok(None) => Json(json!({"error": "not found"})),
        Err(error) => Json(json!({"error": error})),
    }
}

async fn delete_order(Path(id): Path<String>) -> Json<Value> {
    match mongo::delete_one("orders", &id).await {
        Ok(count) => Json(json!({"deleted_count": count})),
        Err(error) => Json(json!({"error": error})),
    }
}

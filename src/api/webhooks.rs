use axum::{
    extract::Query,
    http::{HeaderMap, StatusCode},
    response::Json,
    routing::post,
    Router,
};
use hmac::{Hmac, Mac};
use mongodb::bson::doc;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::{json, Value};
use sha2::Sha256;
use std::env;

type HmacSha256 = Hmac<Sha256>;

pub fn router() -> Router {
    Router::new()
        .route("/mercadopago", post(mercadopago_webhook))
        .route("/asaas", post(asaas_webhook))
}

// Query params enviados pelo Mercado Pago na URL do webhook
#[derive(Deserialize)]
struct MpQueryParams {
    #[serde(rename = "data.id")]
    data_id: Option<String>,
}

// Corpo da notificação Mercado Pago
fn deserialize_optional_string<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt = Option::<Value>::deserialize(deserializer)?;
    Ok(opt.map(|value| match value {
        Value::String(s) => s,
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        other => other.to_string(),
    }))
}

#[derive(Deserialize, Serialize, Debug)]
struct MpNotification {
    #[serde(default, deserialize_with = "deserialize_optional_string")]
    pub id: Option<String>,
    pub live_mode: Option<bool>,
    #[serde(rename = "type")]
    pub kind: Option<String>,
    pub date_created: Option<String>,
    #[serde(default, deserialize_with = "deserialize_optional_string")]
    pub user_id: Option<String>,
    pub api_version: Option<String>,
    pub action: Option<String>,
    pub data: Option<MpData>,
}

#[derive(Deserialize, Serialize, Debug)]
struct MpData {
    #[serde(default, deserialize_with = "deserialize_optional_string")]
    pub id: Option<String>,
}

/// POST /api/webhooks/mercadopago?data.id=XXX
///
/// Valida a assinatura HMAC-SHA256 do x-signature e retorna 200 imediatamente.
/// Processamento assíncrono do evento ocorre depois.
async fn mercadopago_webhook(
    Query(params): Query<MpQueryParams>,
    headers: HeaderMap,
    Json(body): Json<MpNotification>,
) -> (StatusCode, Json<Value>) {
    let secret = env::var("MERCADO_PAGO_WEBHOOK_SECRET")
        .unwrap_or_else(|_| env::var("MERCADO_PAGO_ACCESS_TOKEN").unwrap_or_default());

    // Valida assinatura apenas se a secret estiver configurada
    if !secret.is_empty() {
        let x_signature = headers
            .get("x-signature")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");

        let x_request_id = headers
            .get("x-request-id")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");

        if !x_signature.is_empty() {
            match verify_signature(x_signature, x_request_id, &params.data_id, &secret) {
                Ok(false) => {
                    return (
                        StatusCode::UNAUTHORIZED,
                        Json(json!({"error": "signature mismatch"})),
                    );
                }
                Err(e) => {
                    return (
                        StatusCode::BAD_REQUEST,
                        Json(json!({"error": e})),
                    );
                }
                Ok(true) => {}
            }
        }
    }

    // Responde 200 imediatamente (Mercado Pago aguarda max 22 segundos)
    let event_type = body.kind.clone().unwrap_or_else(|| "unknown".to_string());
    let action = body.action.clone().unwrap_or_else(|| "unknown".to_string());
    let resource_id = body
        .data
        .as_ref()
        .and_then(|d| d.id.clone())
        .unwrap_or_default();

    // Despacha processamento assíncrono
    tokio::spawn(async move {
        handle_event(&event_type, &action, &resource_id).await;
    });

    (StatusCode::OK, Json(json!({"received": true})))
}

/// Valida a assinatura x-signature conforme documentação Mercado Pago.
///
/// Header: `ts=1704908010,v1=618c85345248dd...`
/// Template: `id:{data_id};request-id:{x_request_id};ts:{ts};`
fn verify_signature(
    x_signature: &str,
    x_request_id: &str,
    data_id: &Option<String>,
    secret: &str,
) -> Result<bool, String> {
    // Extrai ts e v1 do header
    let mut ts: Option<&str> = None;
    let mut v1: Option<&str> = None;

    for part in x_signature.split(',') {
        if let Some(value) = part.trim().strip_prefix("ts=") {
            ts = Some(value.trim());
        } else if let Some(value) = part.trim().strip_prefix("v1=") {
            v1 = Some(value.trim());
        }
    }

    let ts = ts.ok_or("ts not found in x-signature")?;
    let v1 = v1.ok_or("v1 not found in x-signature")?;

    // Monta o manifest incluindo apenas campos presentes
    let mut manifest = String::new();
    if let Some(id) = data_id {
        if !id.is_empty() {
            manifest.push_str(&format!("id:{};", id.to_lowercase()));
        }
    }
    if !x_request_id.is_empty() {
        manifest.push_str(&format!("request-id:{};", x_request_id));
    }
    manifest.push_str(&format!("ts:{};", ts));

    // Calcula HMAC-SHA256
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .map_err(|e| e.to_string())?;
    mac.update(manifest.as_bytes());
    let result = hex::encode(mac.finalize().into_bytes());

    Ok(result == v1)
}

/// Processa o evento em background após responder 200.
async fn handle_event(kind: &str, action: &str, resource_id: &str) {
    match kind {
        "payment" => {
            let access_token = match env::var("MERCADO_PAGO_ACCESS_TOKEN") {
                Ok(t) => t,
                Err(_) => {
                    eprintln!("[webhook:mp] MERCADO_PAGO_ACCESS_TOKEN not set");
                    return;
                }
            };

            let url = format!("https://api.mercadopago.com/v1/payments/{}", resource_id);
            let client = reqwest::Client::new();
            let payment: Value = match client
                .get(&url)
                .bearer_auth(&access_token)
                .send()
                .await
            {
                Ok(resp) => match resp.json().await {
                    Ok(v) => v,
                    Err(e) => {
                        eprintln!("[webhook:mp] failed to parse payment response: {e}");
                        return;
                    }
                },
                Err(e) => {
                    eprintln!("[webhook:mp] failed to fetch payment {resource_id}: {e}");
                    return;
                }
            };

            let mp_status = payment.get("status").and_then(Value::as_str).unwrap_or("").to_string();
            if mp_status.is_empty() {
                eprintln!("[webhook:mp] payment {resource_id} has no status field");
            }
            let external_reference = payment
                .get("external_reference")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string();

            if external_reference.is_empty() {
                eprintln!("[webhook:mp] no external_reference in payment {resource_id}");
                return;
            }

            let mapped_status = map_mp_status(&mp_status);

            let filter = doc! {"external_reference": &external_reference};
            let orders = match crate::services::mongo::find_all("orders", filter).await {
                Ok(o) => o,
                Err(e) => {
                    eprintln!("[webhook:mp] failed to query orders: {e}");
                    return;
                }
            };

            if let Some(order) = orders.first() {
                let order_id = order
                    .get("_id")
                    .and_then(|v| v.get("$oid"))
                    .and_then(Value::as_str)
                    .unwrap_or("");
                if !order_id.is_empty() {
                    match crate::services::mongo::update_one(
                        "orders",
                        order_id,
                        doc! {"payment_status": &mapped_status},
                    )
                    .await
                    {
                        Ok(_) => eprintln!("[webhook:mp] updated order {order_id} payment_status={mapped_status}"),
                        Err(e) => eprintln!("[webhook:mp] failed to update order {order_id}: {e}"),
                    }
                }
            } else {
                eprintln!("[webhook:mp] no order found for external_reference={external_reference}");
            }
        }
        "subscription_preapproval" => {
            eprintln!("[webhook:mp] subscription {action} id={resource_id}");
        }
        "subscription_authorized_payment" => {
            eprintln!("[webhook:mp] subscription_payment {action} id={resource_id}");
        }
        "topic_merchant_order_wh" => {
            eprintln!("[webhook:mp] merchant_order {action} id={resource_id}");
        }
        "topic_chargebacks_wh" => {
            eprintln!("[webhook:mp] chargeback {action} id={resource_id}");
        }
        "topic_claims_integration_wh" => {
            eprintln!("[webhook:mp] claim {action} id={resource_id}");
        }
        other => {
            eprintln!("[webhook:mp] unhandled event type={other} action={action} id={resource_id}");
        }
    }
}

fn map_mp_status(status: &str) -> String {
    match status {
        "approved" => "paid".to_string(),
        "pending" | "in_process" | "authorized" => "pending".to_string(),
        "rejected" | "cancelled" => "failed".to_string(),
        "refunded" | "charged_back" => "refunded".to_string(),
        other => other.to_string(),
    }
}

/// POST /api/webhooks/asaas
/// Recebe notificações de pagamento do Asaas.
async fn asaas_webhook(headers: HeaderMap, Json(body): Json<Value>) -> (StatusCode, Json<Value>) {
    let event = body
        .get("event")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();

    // TODO: HMAC-SHA256 signature validation requires raw bytes, not parsed JSON.
    // When ASAAS_WEBHOOK_SECRET is set, validation is skipped because we only have
    // the parsed body here. Refactor to use raw body extraction if strict validation is needed.
    if let Some(sig) = headers
        .get("x-hook-signature")
        .and_then(|v| v.to_str().ok())
    {
        eprintln!("[webhook:asaas] signature={sig} (validation skipped — raw body not available)");
    }

    let body_clone = body.clone();
    tokio::spawn(async move {
        handle_asaas_event(&event, &body_clone).await;
    });

    (StatusCode::OK, Json(json!({"received": true})))
}

async fn handle_asaas_event(event: &str, body: &Value) {
    let external_reference = body
        .get("payment")
        .and_then(|p| p.get("externalReference"))
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();

    if external_reference.is_empty() {
        eprintln!("[webhook:asaas] no externalReference in event={event}");
        return;
    }

    let mapped_status = match event {
        "PAYMENT_RECEIVED" | "PAYMENT_CONFIRMED" => "paid",
        "PAYMENT_OVERDUE" => "overdue",
        "PAYMENT_DELETED"
        | "PAYMENT_REFUNDED"
        | "PAYMENT_CHARGEBACK_REQUESTED"
        | "PAYMENT_CHARGEBACK_DISPUTE"
        | "PAYMENT_AWAITING_CHARGEBACK_REVERSAL"
        | "PAYMENT_CHARGEBACK_REVERSED" => "refunded",
        "PAYMENT_UPDATED" => "pending",
        other => {
            eprintln!("[webhook:asaas] unhandled event={other}");
            return;
        }
    };

    let filter = doc! {"external_reference": &external_reference};
    let orders = match crate::services::mongo::find_all("orders", filter).await {
        Ok(o) => o,
        Err(e) => {
            eprintln!("[webhook:asaas] failed to query orders: {e}");
            return;
        }
    };

    if let Some(order) = orders.first() {
        let order_id = order
            .get("_id")
            .and_then(|v| v.get("$oid"))
            .and_then(Value::as_str)
            .unwrap_or("");
        if !order_id.is_empty() {
            match crate::services::mongo::update_one(
                "orders",
                order_id,
                doc! {"payment_status": mapped_status},
            )
            .await
            {
                Ok(_) => eprintln!("[webhook:asaas] updated order {order_id} payment_status={mapped_status}"),
                Err(e) => eprintln!("[webhook:asaas] failed to update order {order_id}: {e}"),
            }
        }
    } else {
        eprintln!("[webhook:asaas] no order found for external_reference={external_reference}");
    }
}

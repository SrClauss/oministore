use axum::{
    body::Body,
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Json},
    routing::get,
    Router,
};
use axum_server::bind;
use serde_json::json;
use std::net::SocketAddr;
use tower_http::cors::{AllowHeaders, AllowMethods, AllowOrigin, CorsLayer};
use tracing_subscriber::{fmt, EnvFilter};
mod api;
mod models;
mod services;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with_target(false)
        .with_span_events(fmt::format::FmtSpan::CLOSE)
        .init();

    // Inicializa o bucket MinIO e aplica lifecycle policies
    services::storage::ensure_bucket().await;

    let cors = build_cors_layer();

    let app = Router::new()
        .route("/", get(root))
        .route("/hello", get(hello))
        .route("/health", get(health))
        .nest("/api", api::router())
        .layer(axum::middleware::from_fn(log_request))
        .layer(cors);
    let addr: SocketAddr = "0.0.0.0:8080".parse().unwrap();

    bind(addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

/// Constrói o CorsLayer a partir da variável de ambiente CORS_ORIGINS.
/// Em development (APP_ENV != "production"), permite qualquer origem.
/// Em production, restringe às origens listadas em CORS_ORIGINS (separadas por vírgula).
fn build_cors_layer() -> CorsLayer {
    let env = std::env::var("APP_ENV").unwrap_or_else(|_| "development".to_string());
    let origins_var = std::env::var("CORS_ORIGINS").unwrap_or_default();

    if env != "production" || origins_var.is_empty() {
        return CorsLayer::new()
            .allow_origin(AllowOrigin::any())
            .allow_methods(AllowMethods::any())
            .allow_headers(AllowHeaders::any());
    }

    let origins: Vec<axum::http::HeaderValue> = origins_var
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .filter_map(|s| {
            s.parse().map_err(|_| {
                tracing::warn!(origin = %s, "CORS_ORIGINS: invalid origin skipped");
            }).ok()
        })
        .collect();

    if origins.is_empty() {
        return CorsLayer::new()
            .allow_origin(AllowOrigin::any())
            .allow_methods(AllowMethods::any())
            .allow_headers(AllowHeaders::any());
    }

    CorsLayer::new()
        .allow_origin(origins)
        .allow_methods(AllowMethods::any())
        .allow_headers(AllowHeaders::any())
}

async fn root() -> Json<serde_json::Value> {
    Json(json!({ "message": "Hello, world!" }))
}

async fn hello() -> Json<serde_json::Value> {
    Json(json!({ "message": "Hello from api.arkana.fun!" }))
}

async fn health() -> (StatusCode, Json<serde_json::Value>) {
    let mongo_status = if services::mongo::ping().await { "ok" } else { "error" };
    let redis_status = if services::cache::ping().await { "ok" } else { "error" };

    let overall = if mongo_status == "ok" && redis_status == "ok" { "ok" } else { "degraded" };
    let status_code = if overall == "ok" { StatusCode::OK } else { StatusCode::SERVICE_UNAVAILABLE };

    (
        status_code,
        Json(json!({
            "status": overall,
            "mongo": mongo_status,
            "redis": redis_status,
        })),
    )
}

async fn log_request(req: Request<Body>, next: Next) -> impl IntoResponse {
    tracing::info!(method = %req.method(), path = %req.uri().path(), "request start");
    let response = next.run(req).await;
    tracing::info!(status = %response.status(), "request complete");
    response
}

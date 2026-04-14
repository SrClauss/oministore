use axum::{
    body::Body,
    http::Request,
    middleware::Next,
    response::{IntoResponse, Json},
    routing::get,
    Router,
};
use axum_server::bind;
use serde_json::json;
use std::net::SocketAddr;
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

    let app = Router::new()
        .route("/", get(root))
        .route("/hello", get(hello))
        .nest("/api", api::router())
        .layer(axum::middleware::from_fn(log_request));
    let addr: SocketAddr = "0.0.0.0:8080".parse().unwrap();

    bind(addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn root() -> Json<serde_json::Value> {
    Json(json!({ "message": "Hello, world!" }))
}

async fn hello() -> Json<serde_json::Value> {
    Json(json!({ "message": "Hello from api.arkana.fun!" }))
}

async fn log_request(req: Request<Body>, next: Next) -> impl IntoResponse {
    tracing::info!(method = %req.method(), path = %req.uri().path(), "request start");
    let response = next.run(req).await;
    tracing::info!(status = %response.status(), "request complete");
    response
}

//! Example: How to properly configure tracing to display trace_id
//!
//! This example demonstrates how to correctly set up tracing subscriber to display trace_id in logs

use axum::{routing::get, Router};
use trace_id::{TraceId, TraceIdLayer};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

#[tokio::main]
async fn main() {
    // Initialize tracing subscriber - this is the key part
    // Must properly configure subscriber to see trace_id in logs
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();

    let app = Router::new()
        .route("/", get(handler))
        .route("/test", get(test_handler))
        .layer(TraceIdLayer::new());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    tracing::info!("Starting server on 0.0.0.0:3000");
    axum::serve(listener, app).await.unwrap();
}

async fn handler(trace_id: TraceId) -> String {
    // These logs will display trace_id because we set the trace_id field in the span
    tracing::info!("Handling request");
    tracing::debug!("Debug information");
    format!("Hello! Your trace ID is: {trace_id}")
}

async fn test_handler() -> &'static str {
    // These logs will also display trace_id
    tracing::info!("Test handler called");
    tracing::warn!("This is a warning");
    "This is a test"
}

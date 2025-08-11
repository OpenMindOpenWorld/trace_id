//! Different tracing configuration examples
//!
//! This example demonstrates various tracing configuration approaches and how they affect trace_id display

use axum::{routing::get, Router};
use trace_id::{TraceId, TraceIdLayer};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

#[tokio::main]
async fn main() {
    // Method 1: Minimal configuration (may not display span fields)
    // tracing_subscriber::fmt::init();

    // Method 2: Configuration with environment variable filter
    // tracing_subscriber::fmt()
    //     .with_env_filter(EnvFilter::from_default_env())
    //     .init();

    // Method 3: Configuration with explicit log level (recommended)
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .init();

    // Method 4: If using custom formatter, ensure span context display is enabled
    /*
    tracing_subscriber::fmt()
        .with_span_events(fmt::format::FmtSpan::FULL)
        .with_env_filter(EnvFilter::from_default_env())
        .init();
    */

    let app = Router::new()
        .route("/", get(handler))
        .route("/test", get(test_handler))
        .layer(TraceIdLayer::new());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    tracing::info!("Starting server on 0.0.0.0:3000");
    axum::serve(listener, app).await.unwrap();
}

async fn handler(trace_id: TraceId) -> String {
    tracing::info!("Handling request in handler");
    tracing::debug!("Debug information in handler");
    call_service().await;
    format!("Hello! Your trace ID is: {trace_id}")
}

async fn test_handler() -> &'static str {
    tracing::info!("Test handler called");
    tracing::warn!("This is a warning in test handler");
    "This is a test"
}

async fn call_service() {
    tracing::info!("Calling external service");
    tracing::debug!("Debug info when calling service");
}

//! # High-Performance Distributed Tracing Module
//!
//! Focuses on trace_id generation, propagation, and management, providing W3C TraceContext compliant
//! tracing ID solutions. Core functionality is web framework agnostic, with out-of-the-box
//! middleware support for Axum.
//!
//! ## Core Features
//!
//! - **High Performance**: Uses timestamp + atomic counter + machine ID combination, avoiding UUID generation overhead
//! - **W3C Compliant**: Generates 128-bit trace-id compliant with W3C TraceContext specification
//! - **Async Friendly**: Context management based on tokio::task_local, supporting ID propagation between async tasks
//! - **Framework Agnostic**: Core functionality doesn't depend on specific web frameworks
//! - **Axum Integration**: Provides out-of-the-box middleware and extractors
//!
//! ## Basic Usage
//!
//! ### Generating and Using TraceId
//! ```
//! use trace_id::TraceId;
//!
//! // Generate new trace ID (W3C TraceContext compliant)
//! let trace_id = TraceId::new();
//! println!("Generated trace ID: {}", trace_id);
//!
//! // Create from string (with validation)
//! let valid_id = "0af7651916cd43dd8448eb211c80319c";
//! if let Some(trace_id) = TraceId::from_string_validated(valid_id) {
//!     println!("Valid trace ID: {}", trace_id);
//! }
//! ```
//!
//! ### Context Management
//! ```
//! use trace_id::{TraceId, with_trace_id, get_trace_id};
//!
//! async fn some_async_function() {
//!     // Get current trace ID in async context
//!     let current_id = get_trace_id();
//!     println!("Current trace ID: {}", current_id);
//! }
//!
//! #[tokio::main]
//! async fn main() {
//!     let trace_id = TraceId::new();
//!     
//!     // Execute async operations within specified trace context
//!     with_trace_id(trace_id, async {
//!         some_async_function().await;
//!     }).await;
//! }
//! ```
//!
//! ## Axum Integration
//!
//! After enabling the `axum` feature, you can use built-in middleware and extractors:
//!
//! ```ignore
//! use axum::{routing::get, Router};
//! use trace_id::{TraceId, TraceIdLayer};
//!
//! async fn handler(trace_id: TraceId) -> String {
//!     // Get TraceId directly in function signature
//!     format!("Hello! Your trace ID is: {}", trace_id)
//! }
//!
//! #[tokio::main]
//! async fn main() {
//!     let app = Router::new()
//!         .route("/", get(handler))
//!         .layer(TraceIdLayer::new()); // Automatically handle x-trace-id header
//!
//!     let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
//!     axum::serve(listener, app).await.unwrap();
//! }
//! ```
//!
//! ### Advanced Configuration
//! ```ignore
//! use trace_id::{TraceIdLayer, TraceIdConfig};
//!
//! let config = TraceIdConfig {
//!     enable_span: true,           // Enable tracing span
//!     enable_response_header: true, // Include trace ID in response
//! };
//!
//! let layer = TraceIdLayer::with_config(config)
//!     .with_generator(|| uuid::Uuid::new_v4().to_string()); // Custom generator
//! ```

// ================================================================================================
// Module Declarations
// ================================================================================================

/// Trace ID context management module
///
/// Provides async context management functionality based on tokio::task_local
mod context;

/// Trace ID core struct module
///
/// Contains TraceId struct definition and related implementations
mod trace_id;

// ================================================================================================
// Public API Exports
// ================================================================================================

/// Re-export context management functions
///
/// - `get_trace_id()`: Get the trace ID of the current async task
/// - `with_trace_id()`: Execute async operations within specified trace context
pub use context::{get_trace_id, with_trace_id};

/// Re-export core trace ID struct
pub use trace_id::TraceId;

/// Trace ID field name in HTTP headers
///
/// Follows common tracing system conventions, used for passing trace ID in HTTP requests/responses
pub const TRACE_ID_HEADER: &str = "x-trace-id";

// ================================================================================================
// Axum Framework Integration (Optional Feature)
// ================================================================================================

/// Axum framework integration module
///
/// Only available when "axum" feature is enabled
#[cfg(feature = "axum")]
mod integrations;

/// Re-export Axum middleware layer
///
/// Provides out-of-the-box trace ID middleware, supporting:
/// - Automatically extract trace ID from request headers
/// - Generate new trace ID (if not present in request)
/// - Add trace ID to response headers
/// - Create tracing span for log correlation
#[cfg(feature = "axum")]
pub use integrations::axum::{TraceIdConfig, TraceIdLayer};

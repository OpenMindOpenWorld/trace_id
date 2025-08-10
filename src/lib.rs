//! 简化的全链路追踪模块
//!
//! 专注于trace_id的生成、传递和管理。
//! 核心功能与Web框架无关，并为Axum提供了开箱即用的中间件支持。
//!
//! ## Usage
//!
//! ### 基础用法：生成和使用 TraceId
//! ```
//! use trace_id::TraceId;
//!
//! // 生成新的 trace ID
//! let trace_id = TraceId::new();
//! println!("Generated trace ID: {}", trace_id);
//!
//! // 从字符串创建（带验证）
//! let valid_id = "0af7651916cd43dd8448eb211c80319c";
//! if let Some(trace_id) = TraceId::from_string_validated(valid_id) {
//!     println!("Valid trace ID: {}", trace_id);
//! }
//! ```
//!
//! ### Axum 集成（需要启用 axum feature）
//! ```ignore
//! use axum::{routing::get, Router};
//! use trace_id::{TraceId, TraceIdLayer};
//!
//! async fn handler(trace_id: TraceId) -> String {
//!     // 直接在函数签名中获取 TraceId
//!     format!("Hello! Your trace ID is: {}", trace_id)
//! }
//!
//! #[tokio::main]
//! async fn main() {
//!     let app = Router::new()
//!         .route("/", get(handler))
//!         .layer(TraceIdLayer::new());
//!
//!     let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
//!     axum::serve(listener, app).await.unwrap();
//! }
//! ```

mod context;
mod trace_id;

pub use context::{get_trace_id, with_trace_id};
pub use trace_id::TraceId;

/// HTTP 头部中的追踪ID字段名
pub const TRACE_ID_HEADER: &str = "x-trace-id";

// -- axum feature --
#[cfg(feature = "axum")]
mod integrations;
#[cfg(feature = "axum")]
pub use integrations::axum::TraceIdLayer;

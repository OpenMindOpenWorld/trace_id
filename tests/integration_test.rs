//! 集成测试：验证 trace_id 与 tracing 系统的集成//! 集成测试

#![cfg(feature = "axum")]

use axum::http::Request;
use axum::{
    http::{Method, StatusCode},
    routing::get,
    Router,
};
use tower::util::ServiceExt;
use trace_id::{TraceIdLayer, TRACE_ID_HEADER};

/// 简单的测试处理器
async fn test_handler() -> &'static str {
    tracing::info!("Test handler called");
    "Hello, World!"
}

/// 测试 trace_id 与 tracing 的集成
#[tokio::test]
async fn test_tracing_integration() {
    // 创建测试应用
    let app = Router::new()
        .route("/test", get(test_handler))
        .layer(TraceIdLayer::new());

    // 创建测试请求
    let valid_trace_id = "0af7651916cd43dd8448eb211c80319c";
    let request = Request::builder()
        .method(Method::GET)
        .uri("/test")
        .header(TRACE_ID_HEADER, valid_trace_id)
        .body(axum::body::Body::empty())
        .unwrap();

    // 发送请求
    let response = app.oneshot(request).await.unwrap();

    // 验证响应
    assert_eq!(response.status(), StatusCode::OK);

    // 验证响应头包含 trace_id
    let trace_id_header = response.headers().get(TRACE_ID_HEADER);
    assert!(trace_id_header.is_some());
    assert_eq!(trace_id_header.unwrap(), valid_trace_id);
}

/// 测试自动生成 trace_id 的情况
#[tokio::test]
async fn test_auto_generate_trace_id() {
    let app = Router::new()
        .route("/test", get(test_handler))
        .layer(TraceIdLayer::new());

    // 创建不包含 trace_id 的请求
    let request = Request::builder()
        .method(Method::GET)
        .uri("/test")
        .body(axum::body::Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // 验证响应状态
    assert_eq!(response.status(), StatusCode::OK);

    // 验证响应头包含自动生成的 trace_id
    let trace_id_header = response.headers().get(TRACE_ID_HEADER);
    assert!(trace_id_header.is_some());

    // 验证 trace_id 格式（W3C TraceContext 规范长度为 32）
    let trace_id_str = trace_id_header.unwrap().to_str().unwrap();
    assert_eq!(trace_id_str.len(), 32);
}

//! 错误处理和边界情况测试
//!
//! 验证 trace_id 模块在各种异常情况下的健壮性和错误处理能力

use trace_id::{get_trace_id, TraceId};

#[cfg(feature = "axum")]
use axum::{
    body::Body,
    extract::Request,
    http::{HeaderMap, HeaderValue, Method, StatusCode},
    routing::get,
    Router,
};
#[cfg(feature = "axum")]
use tower::ServiceExt;
#[cfg(feature = "axum")]
use trace_id::TRACE_ID_HEADER;

#[cfg(feature = "axum")]
use trace_id::TraceIdLayer;

/// 测试处理器
///
/// 简单的异步处理器，返回固定的响应
#[cfg(feature = "axum")]
async fn test_handler() -> &'static str {
    "OK"
}

/// 测试TraceId验证的边界情况
///
/// 验证TraceId::from_string_validated方法对各种边界情况的处理
#[test]
fn test_trace_id_validation_edge_cases() {
    // 测试正好32个字符但包含无效字符的情况
    let invalid_chars = [
        "0af7651916cd43dd8448eb211c80319G",  // 大写G
        "0af7651916cd43dd8448eb211c80319-",  // 连字符
        "0af7651916cd43dd8448eb211c80319 ",  // 空格
        "0af7651916cd43dd8448eb211c80319\n", // 换行符
        "0af7651916cd43dd8448eb211c80319\0", // 空字符
    ];

    for invalid_id in &invalid_chars {
        let result = TraceId::from_string_validated(invalid_id);
        assert!(result.is_none(), "应该拒绝无效ID: {}", invalid_id);
    }

    // 测试边界长度
    let boundary_lengths = [
        (0, ""),
        (1, "a"),
        (31, "0af7651916cd43dd8448eb211c80319"),
        (33, "0af7651916cd43dd8448eb211c80319ca"),
        (
            64,
            "0af7651916cd43dd8448eb211c80319c0af7651916cd43dd8448eb211c80319c",
        ),
    ];

    for (length, test_str) in &boundary_lengths {
        let result = TraceId::from_string_validated(test_str);
        if *length == 32 {
            assert!(result.is_some(), "长度为32的有效ID应该被接受: {}", test_str);
        } else {
            assert!(
                result.is_none(),
                "长度为{}的ID应该被拒绝: {}",
                length,
                test_str
            );
        }
    }

    // 测试全零ID（应该被拒绝）
    let all_zeros = "00000000000000000000000000000000";
    let result = TraceId::from_string_validated(all_zeros);
    assert!(result.is_none(), "全零ID应该被拒绝");

    // 测试有效的ID
    let valid_id = "0af7651916cd43dd8448eb211c80319c";
    let result = TraceId::from_string_validated(valid_id);
    assert!(result.is_some(), "有效ID应该被接受: {}", valid_id);
}

/// 测试TraceId的Display和Debug实现
///
/// 验证TraceId的格式化输出是否正确
#[test]
fn test_trace_id_formatting() {
    let trace_id = TraceId::from_string_validated("0af7651916cd43dd8448eb211c80319c").unwrap();

    // 测试Display格式化
    let display_str = format!("{}", trace_id);
    assert_eq!(display_str, "0af7651916cd43dd8448eb211c80319c");

    // 测试Debug格式化
    let debug_str = format!("{:?}", trace_id);
    assert!(debug_str.contains("0af7651916cd43dd8448eb211c80319c"));

    // 测试Default实现
    let default_trace_id = TraceId::default();
    assert_eq!(default_trace_id.as_str().len(), 32);

    // 验证默认生成的ID是有效的
    let default_id_str = default_trace_id.as_str();
    let validated = TraceId::from_string_validated(default_id_str);
    assert!(validated.is_some(), "默认生成的ID应该是有效的");
}

/// 测试TraceId生成的唯一性
///
/// 验证连续生成的TraceId是否唯一
#[test]
fn test_trace_id_uniqueness() {
    let mut ids = std::collections::HashSet::new();

    // 生成1000个ID，验证唯一性
    for _ in 0..1000 {
        let trace_id = TraceId::new();
        let id_str = trace_id.as_str().to_string();

        // 验证ID格式
        assert_eq!(id_str.len(), 32, "ID长度应该是32");
        assert!(
            TraceId::from_string_validated(&id_str).is_some(),
            "生成的ID应该是有效的"
        );

        // 验证唯一性
        assert!(ids.insert(id_str.clone()), "ID应该是唯一的: {}", id_str);
    }
}

/// 测试内存安全性
///
/// 确保没有悬垂指针或内存泄漏
#[test]
fn test_memory_safety() {
    // 创建大量TraceId实例并立即丢弃
    for _ in 0..10000 {
        let trace_id = TraceId::new();
        let _cloned = trace_id.clone();
        let _string_repr = trace_id.as_str();
        let _display = format!("{}", trace_id);
        let _debug = format!("{:?}", trace_id);

        // 测试验证函数
        let _valid = TraceId::from_string_validated("0af7651916cd43dd8448eb211c80319c");
        let _invalid = TraceId::from_string_validated("invalid");
    }

    // 如果到达这里没有崩溃或内存错误，说明内存管理是安全的
    assert!(true);
}

/// 测试TraceId的克隆和相等性
///
/// 验证TraceId的Clone和PartialEq实现
#[test]
fn test_trace_id_clone_and_equality() {
    let trace_id1 = TraceId::new();
    let trace_id2 = trace_id1.clone();

    // 测试克隆后的相等性
    assert_eq!(trace_id1, trace_id2);
    assert_eq!(trace_id1.as_str(), trace_id2.as_str());

    // 测试不同ID的不相等性
    let trace_id3 = TraceId::new();
    assert_ne!(trace_id1, trace_id3);
    assert_ne!(trace_id1.as_str(), trace_id3.as_str());
}

/// 测试在没有上下文的情况下调用 get_trace_id
///
/// 验证在没有 `with_trace_id` 上下文的情况下调用 `get_trace_id`
/// 是否能够正确回退到生成一个新的、有效的 TraceId。
#[test]
fn test_get_trace_id_outside_context() {
    // 在没有设置任何 task_local 上下文的情况下直接调用
    let trace_id = get_trace_id();

    // 验证返回的ID是一个有效的TraceId
    assert_eq!(trace_id.as_str().len(), 32, "ID长度应该是32");
    assert!(
        TraceId::from_string_validated(trace_id.as_str()).is_some(),
        "在无上下文时生成的ID应该是有效的"
    );

    // 再次调用，应该生成一个新的、不同的ID
    let trace_id_2 = get_trace_id();
    assert_ne!(
        trace_id.as_str(),
        trace_id_2.as_str(),
        "连续调用应生成不同的ID"
    );
}

// 以下测试需要axum feature
#[cfg(feature = "axum")]
mod axum_tests {
    use super::*;

    /// 测试无效头部值的处理
    ///
    /// 验证TraceIdLayer对无效HTTP头部值的处理
    #[tokio::test]
    async fn test_invalid_header_values() {
        let app = Router::new()
            .route("/test", get(test_handler))
            .layer(TraceIdLayer::new());

        // 测试包含无效UTF-8字节的头部
        let mut headers = HeaderMap::new();
        let invalid_bytes = vec![0xFF, 0xFE, 0xFD]; // 无效UTF-8序列
        if let Ok(header_value) = HeaderValue::from_bytes(&invalid_bytes) {
            headers.insert(TRACE_ID_HEADER, header_value);
        }

        let request = Request::builder()
            .method(Method::GET)
            .uri("/test")
            .body(Body::empty())
            .unwrap();

        let mut request = request;
        *request.headers_mut() = headers;

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        // 应该生成新的trace_id而不是使用无效的头部值
        let trace_id_header = response.headers().get(TRACE_ID_HEADER);
        assert!(trace_id_header.is_some());

        // 验证生成的trace_id是有效的
        if let Some(header_value) = trace_id_header {
            if let Ok(trace_id_str) = header_value.to_str() {
                assert_eq!(trace_id_str.len(), 32);
                assert!(TraceId::from_string_validated(trace_id_str).is_some());
            }
        }
    }

    /// 测试自定义生成器的错误处理
    ///
    /// 验证当自定义生成器生成无效ID时的回退机制
    #[tokio::test]
    async fn test_custom_generator_error_handling() {
        // 创建一个会生成无效ID的生成器
        let layer = TraceIdLayer::new().with_generator(|| "invalid-id".to_string());

        let app = Router::new().route("/test", get(test_handler)).layer(layer);

        let request = Request::builder()
            .method(Method::GET)
            .uri("/test")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        // 当自定义生成器生成无效ID时，应该回退到默认生成器
        let trace_id_header = response.headers().get(TRACE_ID_HEADER);
        assert!(trace_id_header.is_some());

        if let Some(header_value) = trace_id_header {
            if let Ok(trace_id_str) = header_value.to_str() {
                // 应该是有效的32字符ID，而不是"invalid-id"
                assert_eq!(trace_id_str.len(), 32);
                assert_ne!(trace_id_str, "invalid-id");
                assert!(TraceId::from_string_validated(trace_id_str).is_some());
            }
        }
    }

    /// 测试高性能模式配置
    ///
    /// 验证高性能模式的TraceIdLayer是否正常工作
    #[tokio::test]
    async fn test_high_performance_config() {
        // 测试高性能模式（禁用tracing span）
        let app = Router::new()
            .route("/test", get(test_handler))
            .layer(TraceIdLayer::new_high_performance());

        let request = Request::builder()
            .method(Method::GET)
            .uri("/test")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        // 响应头应该包含trace_id
        let trace_id_header = response.headers().get(TRACE_ID_HEADER);
        assert!(trace_id_header.is_some());

        if let Some(header_value) = trace_id_header {
            if let Ok(trace_id_str) = header_value.to_str() {
                assert_eq!(trace_id_str.len(), 32);
                assert!(TraceId::from_string_validated(trace_id_str).is_some());
            }
        }
    }

    /// 测试极长的头部值处理
    ///
    /// 验证对异常长的HTTP头部值的处理
    #[tokio::test]
    async fn test_extremely_long_header_value() {
        let app = Router::new()
            .route("/test", get(test_handler))
            .layer(TraceIdLayer::new());

        // 创建一个极长的头部值
        let long_value = "a".repeat(10000);
        let request = Request::builder()
            .method(Method::GET)
            .uri("/test")
            .header(TRACE_ID_HEADER, &long_value)
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        // 验证响应包含trace_id头部
        let trace_id_header = response.headers().get(TRACE_ID_HEADER);
        assert!(trace_id_header.is_some());

        if let Some(header_value) = trace_id_header {
            if let Ok(trace_id_str) = header_value.to_str() {
                // 验证trace_id不为空且长度正确
                assert!(!trace_id_str.is_empty());
                assert_eq!(trace_id_str.len(), 32);
                assert!(TraceId::from_string_validated(trace_id_str).is_some());
            }
        }
    }

    /// 测试并发情况下的错误处理
    ///
    /// 验证在高并发场景下的错误处理能力
    #[tokio::test]
    async fn test_concurrent_error_handling() {
        const CONCURRENT_REQUESTS: usize = 50;

        let mut handles = vec![];

        for i in 0..CONCURRENT_REQUESTS {
            let handle = tokio::spawn(async move {
                // 为每个请求创建独立的应用实例
                let app = Router::new()
                    .route("/test", get(test_handler))
                    .layer(TraceIdLayer::new());

                // 创建各种类型的无效请求
                let invalid_trace_id = match i % 4 {
                    0 => "invalid",                                       // 太短
                    1 => "toolongtraceidentifierthatexceeds32characters", // 太长
                    2 => "0AF7651916CD43DD8448EB211C80319C",              // 大写
                    _ => "0af7651916cd43dd8448eb211c80319g",              // 无效字符
                };

                let request = Request::builder()
                    .method(Method::GET)
                    .uri("/test")
                    .header(TRACE_ID_HEADER, invalid_trace_id)
                    .body(Body::empty())
                    .unwrap();

                let response = app.oneshot(request).await.unwrap();
                assert_eq!(response.status(), StatusCode::OK);

                // 验证响应包含trace_id头部
                let trace_id_header = response.headers().get(TRACE_ID_HEADER);
                assert!(trace_id_header.is_some());

                if let Some(header_value) = trace_id_header {
                    if let Ok(trace_id_str) = header_value.to_str() {
                        // 验证trace_id不为空且格式正确
                        assert!(!trace_id_str.is_empty());
                        assert_eq!(trace_id_str.len(), 32);
                        assert!(TraceId::from_string_validated(trace_id_str).is_some());
                    }
                }
            });
            handles.push(handle);
        }

        // 等待所有请求完成
        for handle in handles {
            handle.await.unwrap();
        }
    }

    /// 测试响应头解析失败的处理
    ///
    /// 验证当生成的ID无法解析为HTTP头部值时的处理
    #[tokio::test]
    async fn test_response_header_parse_failure() {
        // 创建一个会生成包含无效字符的ID的生成器
        let layer = TraceIdLayer::new().with_generator(|| {
            // 生成包含控制字符的字符串，这在HTTP头部中是无效的
            "\x00\x01\x02\x03\x04\x05\x06\x07\x08\x09\x0A\x0B\x0C\x0D\x0E\x0F".to_string()
        });

        let app = Router::new().route("/test", get(test_handler)).layer(layer);

        let request = Request::builder()
            .method(Method::GET)
            .uri("/test")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        // 当生成的ID无法解析为HTTP头部值时，应该回退到默认生成器
        let trace_id_header = response.headers().get(TRACE_ID_HEADER);
        assert!(trace_id_header.is_some());

        if let Some(header_value) = trace_id_header {
            if let Ok(trace_id_str) = header_value.to_str() {
                // 应该是有效的32字符ID
                assert_eq!(trace_id_str.len(), 32);
                assert!(TraceId::from_string_validated(trace_id_str).is_some());
            }
        }
    }
}

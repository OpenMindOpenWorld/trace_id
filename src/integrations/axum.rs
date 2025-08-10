//! Axum框架的追踪ID中间件

use crate::{TRACE_ID_HEADER, context, trace_id::TraceId};
use axum::{
    extract::{FromRequestParts, Request},
    http::{HeaderMap, request::Parts},
    response::Response,
};
use std::convert::Infallible;
use std::{
    sync::Arc,
    task::{Context, Poll},
};
use tower::{Layer, Service};
use tracing::Instrument;

/// 用于生成追踪ID的函数签名
type Generator = Arc<dyn Fn() -> String + Send + Sync>;

/// 追踪ID中间件配置选项
#[derive(Clone, Debug)]
pub struct TraceIdConfig {
    /// 是否启用 tracing span（默认启用）
    pub enable_span: bool,
    /// 是否启用响应头（默认启用）
    pub enable_response_header: bool,
}

impl Default for TraceIdConfig {
    fn default() -> Self {
        Self {
            enable_span: true,
            enable_response_header: true,
        }
    }
}

/// 高性能追踪中间件层
///
/// 支持性能优化配置，只负责trace_id的提取、生成和传递
#[derive(Clone)]
pub struct TraceIdLayer {
    generator: Option<Generator>,
    config: TraceIdConfig,
}

impl TraceIdLayer {
    /// 创建新的追踪ID层，使用默认配置和高性能生成器
    pub fn new() -> Self {
        Self { 
            generator: None,
            config: TraceIdConfig::default(),
        }
    }
    
    /// 创建高性能模式的追踪ID层
    /// 
    /// 禁用 tracing span 以获得最佳性能
    pub fn new_high_performance() -> Self {
        Self {
            generator: None,
            config: TraceIdConfig {
                enable_span: false,
                enable_response_header: true,
            },
        }
    }
    
    /// 使用自定义配置创建追踪ID层
    pub fn with_config(config: TraceIdConfig) -> Self {
        Self {
            generator: None,
            config,
        }
    }

    /// 使用自定义的生成器创建追踪ID层
    ///
    /// # 参数
    /// * `generator` - 一个返回String的函数，用于生成ID
    ///
    /// # 示例
    /// ```
    /// use trace_id::TraceIdLayer;
    ///
    /// // 使用nanoid作为生成器
    /// // let layer = TraceIdLayer::new().with_generator(|| nanoid::nanoid!());
    /// ```
    pub fn with_generator<F>(mut self, generator: F) -> Self
    where
        F: Fn() -> String + Send + Sync + 'static,
    {
        self.generator = Some(Arc::new(generator));
        self
    }
}

impl Default for TraceIdLayer {
    fn default() -> Self {
        Self::new()
    }
}

impl<S> Layer<S> for TraceIdLayer {
    type Service = TraceIdService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        TraceIdService {
            inner,
            generator: self.generator.clone(),
            config: self.config.clone(),
        }
    }
}

/// 高性能追踪ID服务
#[derive(Clone)]
pub struct TraceIdService<S> {
    inner: S,
    generator: Option<Generator>,
    config: TraceIdConfig,
}

impl<S> Service<Request> for TraceIdService<S>
where
    S: Service<Request, Response = Response> + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>> + Send>,
    >;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: Request) -> Self::Future {
        // 从请求头中获取或生成追踪ID
        let trace_id = extract_or_generate_trace_id(req.headers(), self.generator.as_deref());

        // 提取请求信息用于span（在req被移动之前）
        let method = req.method().clone();
        let uri = req.uri().clone();

        // 将追踪ID添加到请求扩展中（用于向后兼容）
        req.extensions_mut().insert(trace_id.clone());

        let future = self.inner.call(req);

        let config = self.config.clone();
        
        Box::pin(async move {
            // 根据配置决定是否创建 span
            if config.enable_span {
                let span = tracing::info_span!(
                    "request",
                    trace_id = %trace_id.as_str(),
                    method = %method,
                    uri = %uri
                );
                
                // 在span和task_local上下文中执行请求处理
                async move {
                    context::with_trace_id(trace_id.clone(), async move {
                        let mut response = future.await?;
                        
                        // 根据配置决定是否添加响应头
                        if config.enable_response_header {
                            if let Ok(header_value) = trace_id.as_str().parse() {
                                response.headers_mut().insert(TRACE_ID_HEADER, header_value);
                            }
                        }
                        
                        Ok(response)
                    }).await
                }
                .instrument(span)
                .await
            } else {
                // 高性能模式：跳过 span 创建
                context::with_trace_id(trace_id.clone(), async move {
                    let mut response = future.await?;
                    
                    // 根据配置决定是否添加响应头
                    if config.enable_response_header {
                        if let Ok(header_value) = trace_id.as_str().parse() {
                            response.headers_mut().insert(TRACE_ID_HEADER, header_value);
                        }
                    }
                    
                    Ok(response)
                }).await
            }
        })
    }
}

/// 从请求头中提取或生成新的追踪ID（高性能版本）
fn extract_or_generate_trace_id(
    headers: &HeaderMap,
    generator: Option<&(dyn Fn() -> String + Send + Sync)>,
) -> TraceId {
    // 快速路径：直接从头部提取
    if let Some(header_value) = headers.get(TRACE_ID_HEADER) {
        if let Ok(id_str) = header_value.to_str() {
            // 使用快速验证提升性能
            if is_valid_trace_id_fast(id_str) {
                return TraceId::from_string_unchecked(id_str);
            } else if let Some(trace_id) = TraceId::from_string_validated(id_str) {
                return trace_id;
            }
        }
    }
    
    // 生成新的追踪ID
    if let Some(generator_fn) = generator {
        let generated_id = generator_fn();
        TraceId::from_string_validated(&generated_id).unwrap_or_else(|| TraceId::new())
    } else {
        TraceId::new()
    }
}

/// 快速验证追踪ID格式（避免详细检查）
/// 
/// 只接受符合 W3C TraceContext 规范的格式，其他格式需要完整验证
fn is_valid_trace_id_fast(id: &str) -> bool {
    // W3C TraceContext 规范：恰好32个字符的小写十六进制
    id.len() == 32 && id.bytes().all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
}

// -- TraceId Extractor --

/// Axum 提取器，用于在 handler 函数签名中直接获取 TraceId
///
/// # 示例
/// ```no_run
/// use axum::{routing::get, Router};
/// use trace_id::{TraceId, TraceIdLayer};
///
/// async fn my_handler(trace_id: TraceId) -> String {
///     tracing::info!(trace_id = %trace_id, "Handler started");
///     format!("Hello! Your trace ID is: {}", trace_id)
/// }
///
/// let app: Router = Router::new()
///     .route("/", get(my_handler))
///     .layer(TraceIdLayer::new());
/// ```
impl<S> FromRequestParts<S> for TraceId
where
    S: Send + Sync,
{
    type Rejection = Infallible;

    /// 从请求中提取 TraceId
    ///
    /// 这个提取器会调用 `context::get_trace_id()` 来获取当前请求的追踪ID。
    /// 由于 TraceIdLayer 中间件已经设置了追踪上下文，这个提取器永远不会失败。
    async fn from_request_parts(_parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        Ok(context::get_trace_id())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{HeaderValue, Request, StatusCode},
        routing::get,
        Router,
    };
    use tower::ServiceExt;

    // --- 辅助函数测试 ---
    mod id_extraction {
        use super::*;

        fn default_generator() -> Option<&'static (dyn Fn() -> String + Send + Sync)> {
            None
        }

        #[test]
        fn test_extract_trace_id_from_headers() {
            let mut headers = HeaderMap::new();
            let valid_trace_id = "0af7651916cd43dd8448eb211c80319c";
            headers.insert(
                TRACE_ID_HEADER,
                HeaderValue::from_static(valid_trace_id),
            );

            let trace_id = extract_or_generate_trace_id(&headers, default_generator());
            assert_eq!(trace_id.as_str(), valid_trace_id);
        }

        #[test]
        fn test_generate_trace_id_when_missing() {
            let headers = HeaderMap::new();
            let trace_id = extract_or_generate_trace_id(&headers, default_generator());
            assert_eq!(trace_id.as_str().len(), 32);
            assert!(TraceId::from_string_validated(trace_id.as_str()).is_some());
        }

        #[test]
        fn test_extract_with_invalid_header() {
            let mut headers = HeaderMap::new();
            headers.insert(TRACE_ID_HEADER, HeaderValue::from_static(""));
            let trace_id = extract_or_generate_trace_id(&headers, default_generator());
            assert_ne!(trace_id.as_str(), "");
            assert_eq!(trace_id.as_str().len(), 32);

            let mut headers = HeaderMap::new();
            let long_id = "a".repeat(129);
            headers.insert(TRACE_ID_HEADER, HeaderValue::from_str(&long_id).unwrap());
            let trace_id = extract_or_generate_trace_id(&headers, default_generator());
            assert_ne!(trace_id.as_str(), long_id);
        }

        #[test]
        fn test_with_custom_generator() {
            let headers = HeaderMap::new();
            let custom_id = "0af7651916cd43dd8448eb211c80319c";
            let generator = || custom_id.to_string();
            let trace_id = extract_or_generate_trace_id(&headers, Some(&generator));
            assert_eq!(trace_id.as_str(), custom_id);
        }

        #[test]
        fn test_custom_generator_fallback() {
            let headers = HeaderMap::new();
            let invalid_id = "this-is-not-a-valid-id";
            let generator = || invalid_id.to_string();
            let trace_id = extract_or_generate_trace_id(&headers, Some(&generator));
            assert_ne!(trace_id.as_str(), invalid_id);
            assert_eq!(trace_id.as_str().len(), 32);
        }
    }

    // --- 提取器测试 ---
    #[tokio::test]
    async fn test_trace_id_extractor() {
        let (mut parts, _body) = Request::builder().uri("/test").body(()).unwrap().into_parts();
        let test_trace_id = TraceId::new();

        crate::context::with_trace_id(test_trace_id.clone(), async move {
            let extracted_trace_id = TraceId::from_request_parts(&mut parts, &())
                .await
                .expect("TraceId extraction should never fail");
            assert_eq!(extracted_trace_id, test_trace_id);
        })
        .await;
    }

    // --- 中间件/服务测试 ---
    mod layer_behavior {
        use super::*;

        async fn handler(trace_id: TraceId) -> String {
            trace_id.to_string()
        }

        #[tokio::test]
        async fn test_end_to_end_flow() {
            let app = Router::new().route("/", get(handler)).layer(TraceIdLayer::new());

            // 场景1: 提供有效ID
            let valid_id = "0af7651916cd43dd8448eb211c80319c";
            let request = Request::builder()
                .uri("/")
                .header(TRACE_ID_HEADER, valid_id)
                .body(Body::empty())
                .unwrap();
            let response = app.clone().oneshot(request).await.unwrap();
            assert_eq!(response.status(), StatusCode::OK);
            assert_eq!(response.headers().get(TRACE_ID_HEADER).unwrap(), valid_id);
            let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
            assert_eq!(&body[..], valid_id.as_bytes());

            // 场景2: 不提供ID
            let request = Request::builder().uri("/").body(Body::empty()).unwrap();
            let response = app.clone().oneshot(request).await.unwrap();
            assert_eq!(response.status(), StatusCode::OK);
            let header_id = response.headers().get(TRACE_ID_HEADER).unwrap().to_str().unwrap().to_owned();
            assert_eq!(header_id.len(), 32);
            let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
            assert_eq!(body, header_id.as_bytes());
        }

        #[tokio::test]
        async fn test_high_performance_mode() {
            let app = Router::new().route("/", get(handler)).layer(TraceIdLayer::new_high_performance());
            let valid_id = "1234567890abcdef1234567890abcdef";
            let request = Request::builder()
                .uri("/")
                .header(TRACE_ID_HEADER, valid_id)
                .body(Body::empty())
                .unwrap();
            let response = app.oneshot(request).await.unwrap();
            assert_eq!(response.status(), StatusCode::OK);
            assert_eq!(response.headers().get(TRACE_ID_HEADER).unwrap(), valid_id);
        }

        #[tokio::test]
        async fn test_disable_response_header() {
            let config = TraceIdConfig {
                enable_span: true,
                enable_response_header: false,
            };
            let app = Router::new().route("/", get(handler)).layer(TraceIdLayer::with_config(config));
            let request = Request::builder().uri("/").body(Body::empty()).unwrap();
            let response = app.oneshot(request).await.unwrap();
            assert_eq!(response.status(), StatusCode::OK);
            assert!(response.headers().get(TRACE_ID_HEADER).is_none());
        }
    }
}

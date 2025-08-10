# `trace_id` 模块

这是一个为 `axum` 应用设计的轻量级追踪ID模块，专注于在**单个服务内部**实现日志关联（Log Correlation）。它被设计为与 `axum` 和 `tracing` 生态系统无缝集成，作为构建清晰、可调试服务的基石。

## 解决了什么问题？

本模块的本质是一个**日志关联（Log Correlation）方案**，其核心目标是：“**将一个请求的所有日志用同一个ID串起来**”，从而极大地简化调试和问题排查。

在现代Web应用中，一个用户的请求可能会流经多个处理环节。这带来了两大挑战：

1.  **日志混乱**：在并发环境下，来自不同请求的日志会混杂在一起，使得追踪单个请求的处理流程变得极其困难。
2.  **故障定位缓慢**：当生产环境出现问题时，快速确定哪个环节出错是至关重要的。没有统一的标识，这个过程就像大海捞针。

`trace_id` 模块通过为每一个进入系统的请求分配一个唯一的ID，并利用 `tracing` 的上下文机制确保该ID贯穿整个请求处理链，从而完美地解决了这些问题。

### 主要优势

-   **开发阶段**：
    -   **清晰的调试**：通过`trace_id`过滤日志，可以清晰地看到单个请求从入口到结束的完整执行路径，不受其他并发请求的干扰。
    -   **高效协作**：在团队间沟通问题时，只需提供`trace_id`，对方就能快速定位到相关的处理逻辑，极大提升了协作效率。

-   **生产阶段**：
    -   **快速故障定位**：当用户报告问题或监控系统告警时，运维人员可以利用`trace_id`在日志聚合系统（如ELK, Loki）中一键查询，立即获取与该请求相关的所有日志，将故障排查时间从数小时缩短到几分钟。
    -   **性能瓶颈分析**：`trace_id`是APM（应用性能监控）系统的核心，它可以串联起整个分布式调用链，帮助开发者直观地发现系统性能瓶颈。

## 核心功能

-   **ID生成与传递**：自动从HTTP请求头 `x-trace-id` 中提取ID。如果不存在，则会生成一个新的UUIDv4作为ID，并将其添加回答复的响应头中。
-   **Axum中间件集成**：提供一个即插即用的`TraceIdLayer`，可以轻松地集成到`axum`应用中。
-   **可靠的上下文管理**：使用`tokio::task_local!`来确保`trace_id`在整个异步调用栈中都可用。
-   **与`tracing`库集成**：自动将`trace_id`附加到请求的`tracing::Span`中，使其出现在所有相关的日志记录里。

## 使用示例

### 1. 在Axum中添加中间件

```rust
use axum::{routing::get, Router};
use trace_id::TraceIdLayer;

async fn handler() -> &'static str {
    // 该日志会自动关联上 trace_id
    tracing::info!("Handler executed");
    "Hello, World!"
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(handler))
        .layer(TraceIdLayer::new());

    // ... 启动服务器
}
```

### 2. 使用 Axum 提取器（推荐方式）

最符合 Axum 人体工程学的方式是在 handler 函数签名中直接获取 `TraceId`：

```rust
use axum::{routing::get, Router};
use trace_id::{TraceId, TraceIdLayer};

async fn my_handler(trace_id: TraceId) -> String {
    // 直接使用 trace_id，无需调用 get_trace_id()
    tracing::info!(trace_id = %trace_id, "Handler started");
    format!("Hello! Your trace ID is: {}", trace_id)
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(my_handler))
        .layer(TraceIdLayer::new());
    
    // ... 启动服务器
}
```

### 3. 在业务逻辑中获取ID（传统方式）

在任何处于`TraceIdLayer`保护下的异步函数中，你都可以轻松获取到当前的`trace_id`。

**注意**：通常你不需要在 `tracing::info!` 等日志宏中手动添加 `trace_id`，因为日志系统会自动从上下文中关联。`get_trace_id()` 更常用于需要将ID传递给外部系统（如错误监控平台）或在API响应体中返回等场景。

```rust
use trace_id::get_trace_id;

// 一个将 trace_id 用于错误上报的示例
pub async fn some_business_logic(payload: &str) {
    if payload.is_empty() {
        let trace_id = get_trace_id();
        // 当你需要将追踪ID与外部系统关联时，手动获取它就非常有用
        tracing::error!(
            "Payload is empty! This error will be associated with trace_id: {}",
            trace_id
        );
        // report_error_to_external_system("Payload is empty!", trace_id);
    }
    // ...
}
```

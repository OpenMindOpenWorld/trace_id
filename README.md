# trace_id

[![Crates.io](https://img.shields.io/crates/v/trace_id.svg)](https://crates.io/crates/trace_id)
[![Documentation](https://docs.rs/trace_id/badge.svg)](https://docs.rs/trace_id)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE)
[![Build Status](https://github.com/OpenMindOpenWorld/trace_id/workflows/CI/badge.svg)](https://github.com/OpenMindOpenWorld/trace_id/actions)

A lightweight, high-performance trace ID library for Rust applications, designed for seamless integration with web frameworks and the tracing ecosystem. Currently supports Axum with Actix Web support planned for future releases.

## 🚀 Features

- **Zero-overhead trace ID generation** - Optimized for high-performance applications
- **Web framework integration** - Currently supports Axum, with Actix Web planned
- **Seamless Axum integration** - Drop-in middleware with extractor support
- **Automatic request correlation** - Links all logs within a request lifecycle
- **Header-based propagation** - Supports `x-trace-id` header for distributed tracing
- **Thread-safe context management** - Built on `tokio::task_local!` for async safety
- **Tracing ecosystem integration** - Automatic span attachment for structured logging
- **Framework-agnostic core** - Extensible architecture for future framework support

## 📦 Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
trace_id = "0.1"

# For Axum integration
trace_id = { version = "0.1", features = ["axum"] }
```

## 🎯 Quick Start

### Basic Axum Integration

```rust
use axum::{routing::get, Router};
use trace_id::{TraceId, TraceIdLayer};
use tracing::info;

async fn handler(trace_id: TraceId) -> String {
    info!("Processing request"); // Automatically includes trace_id
    format!("Hello! Trace ID: {}", trace_id)
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(handler))
        .layer(TraceIdLayer::new());
    
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();
    
    axum::serve(listener, app).await.unwrap();
}
```

### Manual Trace ID Access

```rust
use trace_id::get_trace_id;

async fn business_logic() {
    let trace_id = get_trace_id();
    
    // Use trace_id for external system integration
    external_api_call(&trace_id).await;
    
    tracing::info!("Business logic completed");
}
```

## 📚 Documentation

### Core Components

#### `TraceId`

A lightweight wrapper around a UUID v4 that represents a unique trace identifier.

```rust
use trace_id::TraceId;

// Generate a new trace ID
let trace_id = TraceId::new();

// Parse from string (with validation)
let trace_id = TraceId::from_string_validated("550e8400-e29b-41d4-a716-446655440000")?;

// Convert to string
let id_string = trace_id.to_string();
```

#### `TraceIdLayer`

Axum middleware that automatically manages trace ID lifecycle:

- Extracts trace ID from `x-trace-id` header
- Generates new ID if header is missing
- Adds trace ID to response headers
- Sets up tracing context for the request

```rust
use axum::Router;
use trace_id::TraceIdLayer;

let app = Router::new()
    .route("/api/users", get(get_users))
    .layer(TraceIdLayer::new());
```

#### Context Management

Access the current trace ID from anywhere in your async call stack:

```rust
use trace_id::get_trace_id;

async fn deep_function() {
    let trace_id = get_trace_id();
    tracing::info!(trace_id = %trace_id, "Deep in the call stack");
}
```

## 🔧 Advanced Usage

### Custom Header Names

```rust
use trace_id::TraceIdLayer;

// Use custom header name (future feature)
let layer = TraceIdLayer::with_header("x-request-id");
```

### Error Handling

```rust
use trace_id::{TraceId, TraceIdError};

match TraceId::from_string_validated("invalid-uuid") {
    Ok(trace_id) => println!("Valid: {}", trace_id),
    Err(TraceIdError::InvalidFormat) => println!("Invalid UUID format"),
    Err(TraceIdError::AllZeros) => println!("UUID cannot be all zeros"),
}
```

## 🏗️ Architecture

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   HTTP Request  │───▶│  TraceIdLayer    │───▶│   Your Handler  │
│  (x-trace-id)   │    │  - Extract/Gen   │    │  (TraceId)      │
└─────────────────┘    │  - Set Context   │    └─────────────────┘
                       │  - Add to Span   │
                       └──────────────────┘
                                │
                                ▼
                       ┌──────────────────┐
                       │  Tracing Context │
                       │  - All logs      │
                       │  - Automatic ID  │
                       └──────────────────┘
```

## 🚦 Examples

Check out the [examples](examples/) directory for more comprehensive usage patterns:

- [Basic Axum Integration](examples/basic.rs)
- [Error Handling](examples/error_handling.rs)
- [Custom Middleware](examples/custom_middleware.rs)
- [Distributed Tracing](examples/distributed.rs)

## 🧪 Testing

```bash
# Run all tests
cargo test

# Run with Axum features
cargo test --features axum

# Run benchmarks
cargo bench

# Check code quality
cargo clippy --all-features -- -D warnings
cargo fmt --check
```

## 📊 Performance

Optimized for high-throughput applications:

- **ID Generation**: ~50ns per ID
- **String Validation**: ~100ns per validation
- **Context Access**: ~10ns per access
- **Memory Overhead**: 16 bytes per trace ID

Run benchmarks with `cargo bench` to see performance on your system.

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

### Development Setup

```bash
git clone https://github.com/OpenMindOpenWorld/trace_id.git
cd trace_id
cargo test --all-features
```

## 📄 License

This project is licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## 🔗 Related Projects

- [tracing](https://github.com/tokio-rs/tracing) - Structured logging and diagnostics
- [axum](https://github.com/tokio-rs/axum) - Ergonomic web framework (currently supported)
- [actix-web](https://github.com/actix/actix-web) - Powerful web framework (planned support)

---

<div align="center">
  <strong>Built with ❤️ for the Rust community</strong>
</div>

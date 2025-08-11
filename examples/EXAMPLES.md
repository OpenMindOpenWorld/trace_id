# trace_id Examples

This directory contains usage examples for the `trace_id` library, demonstrating how to integrate distributed tracing functionality in Axum applications.

## Running Examples

Since the examples use Axum integration features, you need to enable the `axum` feature to compile and run them:

### Basic Tracing Example

```bash
# Run the basic example demonstrating TraceId usage
cargo run --example tracing_example --features axum
```

Visit `http://localhost:3000/` or `http://localhost:3000/test` to test the functionality.

### Tracing Configuration Example

```bash
# Run the configuration example showing different tracing setup methods
cargo run --example tracing_configurations --features axum
```

## Example Descriptions

### tracing_example.rs
- Shows how to properly configure `tracing_subscriber` to display trace_id
- Demonstrates `TraceId` usage as an Axum extractor
- Includes basic logging examples

### tracing_configurations.rs
- Shows multiple `tracing_subscriber` configuration approaches
- Demonstrates environment variable configuration and default settings
- Includes more complex logging configuration examples

## Testing Examples

After starting the server, you can test with curl:

```bash
# Test homepage
curl -v http://localhost:3000/

# Test other routes
curl -v http://localhost:3000/test

# Request with custom trace-id
curl -H "x-trace-id: 0af7651916cd43dd8448eb211c80319c" -v http://localhost:3000/
```

Note the `x-trace-id` field in the response headers - this is the automatically generated or passed trace ID.

## Common Issues

### Compilation Error: TraceIdLayer Not Found

Make sure to use the `--features axum` parameter when running examples:

```bash
cargo run --example tracing_example --features axum
```

**Note**: Starting from version 0.1.0, examples are configured to compile only when the `axum` feature is enabled. This prevents compilation failures when running `cargo test` without the axum feature.

### Unit Test Issues

If you encounter unit test failures, this has been resolved by adding `required-features = ["axum"]` configuration for examples in `Cargo.toml`. This means:

- When running `cargo test`, examples won't be compiled, avoiding dependency issues
- When running `cargo test --features axum`, examples will compile and test normally
- Running examples still requires the `--features axum` parameter

### Port Already in Use Error

If you encounter an "Address already in use" error, make sure no other service is using port 3000, or modify the port number in the example code.
# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.1] - 2025-08-11

### Added

- Enhanced examples with comprehensive usage patterns
- Improved documentation with better explanations and code samples
- Added EXAMPLES.md with detailed instructions for running examples
- New tracing configuration examples showing different setup methods

### Improved

- Better README structure with clearer sections
- Enhanced code examples with more practical use cases
- Improved inline documentation and comments
- Better error handling examples and explanations

## [0.1.0] - 2025-08-10

### Added

- Initial release of trace_id library
- High-performance TraceId generation (W3C TraceContext compliant)
- Async context management using `tokio::task_local`
- Axum framework integration with middleware and extractor
- Smart ID validation with comprehensive error handling
- Comprehensive test suite with 100% pass rate
- Performance benchmarks and optimizations

### Features

- `TraceId::new()` - Generate new trace IDs
- `TraceId::from_string_validated()` - Validate and create from string
- `get_trace_id()` - Retrieve current trace ID from context
- `with_trace_id()` - Execute operations within trace context
- `TraceIdLayer` - Axum middleware for automatic trace ID handling
- HTTP header support (`x-trace-id`)
- Custom ID generator support

[0.1.1]: https://github.com/OpenMindOpenWorld/trace_id/releases/tag/v0.1.1
[0.1.0]: https://github.com/OpenMindOpenWorld/trace_id/releases/tag/v0.1.0
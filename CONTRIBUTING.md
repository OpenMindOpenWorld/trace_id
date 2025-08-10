# Contributing Guide

First off, thank you for considering contributing to trace_id! It's people like you that make the open-source community such a great place.

We welcome any type of contribution, not just code. You can help with:

- Reporting a bug
- Discussing the current state of the code
- Submitting a fix
- Proposing new features
- Becoming a maintainer

## We Develop with GitHub

We use GitHub to host code, to track issues and feature requests, as well as accept pull requests.

## All Code in Any Form is Licensed Under the MIT OR Apache-2.0

By contributing, you agree that your contributions will be licensed under its MIT OR Apache-2.0 License.

## How to Contribute

### Reporting Bugs

Bugs are tracked as GitHub issues. When you are creating a bug report, please include as many details as possible. Fill out the required template, the information it asks for helps us resolve issues faster.

### Suggesting Enhancements

Enhancement suggestions are tracked as GitHub issues. When you are creating an enhancement suggestion, please include as many details as possible.

### Pull Requests

Pull requests are the best way to propose changes to the codebase. We actively welcome your pull requests:

1. Fork the repo and create your branch from `main`
2. If you've added code that should be tested, add tests
3. If you've changed APIs, update the documentation
4. Ensure the test suite passes (`cargo test`)
5. Make sure your code lints (`cargo clippy`)
6. Format your code (`cargo fmt`)
7. Issue that pull request!

## Styleguides

### Git Commit Messages

- Use the present tense ("Add feature" not "Added feature")
- Use the imperative mood ("Move cursor to..." not "Moves cursor to...")
- Limit the first line to 72 characters or less
- Reference issues and pull requests liberally after the first line

### Rust Styleguide

We try to follow the official Rust style guidelines. You can ensure your code is formatted correctly by running `cargo fmt`.

## Development Environment Setup

1. Clone the repository:
   ```bash
   git clone https://github.com/your-username/trace_id.git
   cd trace_id
   ```

2. Run tests:
   ```bash
   cargo test
   cargo test --features axum
   ```

3. Run benchmarks:
   ```bash
   cargo bench
   ```

4. Check code quality:
   ```bash
   cargo clippy --all-features -- -D warnings
   cargo fmt --check
   ```

## Project Structure

- `src/` - Core library code
  - `trace_id.rs` - TraceId core implementation
  - `context.rs` - Context management
  - `integrations/` - Framework integrations
- `tests/` - Integration tests
- `benches/` - Performance benchmarks
- `docs/` - Project documentation

Thank you for your contribution!
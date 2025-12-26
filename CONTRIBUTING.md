# Contributing to domainstack

Thank you for your interest in contributing to domainstack! This guide will help you get started with development.

## Testing

```bash
cd domainstack

# Run all tests (149 unit + doc tests across all crates)
cargo test --all-features

# Test specific crate
cargo test -p domainstack --all-features
cargo test -p domainstack-derive
cargo test -p domainstack-envelope

# Run with coverage
cargo llvm-cov --all-features --workspace --html
```

## Running Examples

Examples are included in the repository (not published to crates.io). Clone the repo to try them:

```bash
# Clone the repository
git clone https://github.com/blackwell-systems/domainstack.git
cd domainstack/domainstack

# Manual validation examples
cargo run --example email_primitive --features regex
cargo run --example booking_aggregate --features regex
cargo run --example age_primitive

# Derive macro examples
cargo run --example v2_basic
cargo run --example v2_nested
cargo run --example v2_collections
cargo run --example v2_custom

# HTTP integration examples
cargo run --example v3_error_envelope_basic
cargo run --example v3_error_envelope_nested

# Builder customization examples
cargo run --example v4_builder_customization

# Cross-field validation examples
cargo run --example v5_cross_field_validation

# Async validation examples
cargo run --example async_validation --features async
cargo run --example async_sqlite --features async

# Phantom types examples
cargo run --example phantom_types --features regex

# OpenAPI schema generation examples
cd domainstack-schema
cargo run --example user_api
cargo run --example v08_features

# Framework examples
cd examples-axum && cargo run    # Axum booking service
cd examples-actix && cargo run   # Actix-web booking service
cd examples-rocket && cargo run  # Rocket booking service
```

## Development Workflow

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run tests to ensure everything passes
5. Submit a pull request

## Questions?

Feel free to open an issue if you have any questions or need help getting started.

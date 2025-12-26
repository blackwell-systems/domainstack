# domainstack-http

[![Crates.io](https://img.shields.io/crates/v/domainstack-http.svg)](https://crates.io/crates/domainstack-http)
[![Documentation](https://docs.rs/domainstack-http/badge.svg)](https://docs.rs/domainstack-http)
[![License: Apache 2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://github.com/blackwell-systems/domainstack/blob/main/LICENSE)

Framework-agnostic HTTP error handling for the [domainstack](https://crates.io/crates/domainstack) full-stack validation ecosystem.

## Overview

This crate provides shared HTTP error types and utilities used by framework-specific adapters like `domainstack-axum` and `domainstack-actix`.

**Most users should use framework adapters directly** instead of this crate:
- [domainstack-axum](https://crates.io/crates/domainstack-axum) - For Axum web framework
- [domainstack-actix](https://crates.io/crates/domainstack-actix) - For Actix-web framework

## Usage

Only use this crate directly if you're:
1. Building a new framework adapter for domainstack
2. Need framework-agnostic error handling

Add this to your `Cargo.toml`:

```toml
[dependencies]
domainstack = "1.0"
domainstack-http = "1.0"
```

## Types

**ErrorResponse** - Framework-agnostic HTTP error response:

```rust
use domainstack_http::ErrorResponse;
use domainstack::ValidationError;

let validation_error = ValidationError::single(
    Path::from("email"),
    "invalid_email",
    "Invalid email format"
);

let response = ErrorResponse::from(validation_error);
// Contains error-envelope formatted response with 400 status
```

## Framework Adapters

This crate is used internally by:

- **domainstack-axum** - Implements `IntoResponse` for Axum
- **domainstack-actix** - Implements `ResponseError` for Actix-web

Both adapters use `ErrorResponse` as their common error type and add framework-specific trait implementations.

## Building Custom Adapters

If you're integrating domainstack with another web framework:

1. Add `domainstack-http` and `domainstack-envelope` dependencies
2. Use `ErrorResponse` as your error type
3. Implement your framework's error trait
4. Convert `ErrorResponse::inner` (which is `error_envelope::Error`) to your framework's response type

See `domainstack-axum` or `domainstack-actix` source code for reference implementations.

## Documentation

For complete documentation and examples, see:

- [domainstack documentation](https://docs.rs/domainstack)
- [domainstack-axum documentation](https://docs.rs/domainstack-axum)
- [domainstack-actix documentation](https://docs.rs/domainstack-actix)
- [GitHub repository](https://github.com/blackwell-systems/domainstack)

## License

Apache 2.0

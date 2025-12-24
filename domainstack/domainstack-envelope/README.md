# domainstack-envelope

Convert [domainstack](https://crates.io/crates/domainstack) validation errors to [error-envelope](https://crates.io/crates/error-envelope) HTTP error format.

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
domainstack = "1.0"
domainstack-envelope = "1.0"
```

Convert validation errors to error-envelope format:

```rust
use domainstack::{Validate, ValidationError};
use domainstack_envelope::IntoEnvelopeError;
use error_envelope::Error;

fn create_user(user: User) -> Result<User, Error> {
    user.validate()
        .map_err(|e| e.into_envelope_error())?;

    Ok(user)
}
```

## Error Format

Converts structured validation errors to error-envelope's standard HTTP error format:

```json
{
  "error": {
    "code": "validation_failed",
    "message": "Validation failed",
    "details": {
      "email": ["Invalid email format"],
      "age": ["Must be at least 18"]
    }
  }
}
```

## Features

- **Structured field paths** - Nested fields like `user.address.city` map correctly
- **Multiple violations per field** - All errors preserved
- **HTTP status codes** - Returns 400 Bad Request for validation errors
- **Standard format** - Compatible with error-envelope ecosystem

## Framework Integration

Use with framework adapters for automatic error conversion:

```rust
// Axum
use domainstack_axum::{DomainJson, ErrorResponse};

async fn handler(DomainJson(user): DomainJson<User>) -> Result<Json<User>, ErrorResponse> {
    // Validation errors automatically converted to error-envelope format
    Ok(Json(user))
}

// Actix-web
use domainstack_actix::{DomainJson, ErrorResponse};

async fn handler(user: DomainJson<User>) -> Result<HttpResponse, ErrorResponse> {
    // Validation errors automatically converted to error-envelope format
    Ok(HttpResponse::Ok().json(user.into_inner()))
}
```

## Documentation

For complete documentation, examples, and usage guides, see:

- [domainstack documentation](https://docs.rs/domainstack)
- [error-envelope documentation](https://docs.rs/error-envelope)
- [GitHub repository](https://github.com/blackwell-systems/domainstack)

## License

Apache 2.0

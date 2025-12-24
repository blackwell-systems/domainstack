# domainstack

A Rust validation framework for domain-driven design with derive macro support and framework adapters.

## Overview

This workspace provides a complete validation solution for building domain models in Rust. It consists of nine crates:

**Core:**
- **[domainstack](./domainstack/)** - Core validation library with 31 built-in rules
- **[domainstack-derive](./domainstack-derive/)** - Derive macro for `#[derive(Validate)]`
- **[domainstack-envelope](./domainstack-envelope/)** - error-envelope integration for HTTP APIs

**Framework Adapters:**
- **[domainstack-http](./domainstack-http/)** - Framework-agnostic HTTP helpers
- **[domainstack-axum](./domainstack-axum/)** - Axum web framework integration
- **[domainstack-actix](./domainstack-actix/)** - Actix-web framework integration

**Examples:**
- **[examples](./examples/)** - Core validation examples
- **[examples-axum](./examples-axum/)** - Axum booking service example
- **[examples-actix](./examples-actix/)** - Actix-web booking service example

## Features

- **Valid-by-construction types** - Invalid states can't exist
- **31 built-in rules** - String (17), Numeric (8), Choice (3), Collection (3)
- **Derive macro support** - `#[derive(Validate)]` with powerful attributes
- **Framework adapters** - One-line integration with Axum and Actix-web
- **HTTP integration** - Structured error responses with error-envelope format
- **Composable rules** - Combine validation logic with `and`, `or`, `when`
- **Structured error paths** - Field-level error reporting (e.g., `guest.email`, `rooms[1].adults`)
- **Zero core dependencies** - Core crate uses only std (regex optional for email/URL validation)

## Quick Start

### With Derive Macro (Recommended)

```rust
use domainstack::prelude::*;

#[derive(Debug, Validate)]
struct User {
    #[validate(min_len = 3, max_len = 50)]
    name: String,

    #[validate(range = "18..=120")]
    age: u8,

    #[validate(email, max_len = 255)]
    email: String,
}

#[derive(Debug, Validate)]
struct Booking {
    #[validate(nested)]
    guest: User,

    #[validate(each(nested))]
    rooms: Vec<Room>,
}
```

### Manual Validation

```rust
use domainstack::prelude::*;

struct Username(String);

impl Username {
    pub fn new(raw: String) -> Result<Self, ValidationError> {
        let rule = rules::min_len(3).and(rules::max_len(20));
        validate("username", raw.as_str(), &rule)?;
        Ok(Self(raw))
    }
}
```

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
# Core library only
domainstack = "0.4"

# With derive macro support (recommended)
domainstack = { version = "0.4", features = ["derive"] }

# With regex validation (email, URL, pattern matching)
domainstack = { version = "0.4", features = ["derive", "regex"] }

# With error-envelope integration for HTTP APIs
domainstack-envelope = "0.4"

# Framework adapters
domainstack-axum = "0.4"    # For Axum
domainstack-actix = "0.4"   # For Actix-web
```

## Built-in Rules

### String Rules (17)

- `email()`† - Email format validation
- `url()`† - URL format validation
- `matches_regex(pattern)`† - Custom regex matching
- `non_empty()` - Non-empty string
- `non_blank()` - Not empty or whitespace-only
- `no_whitespace()` - No whitespace characters
- `ascii()` - ASCII-only characters
- `min_len(min)` - Minimum byte length
- `max_len(max)` - Maximum byte length
- `length(exact)` - Exact byte length
- `len_chars(min, max)` - Character count range
- `alphanumeric()` - Only letters and digits
- `alpha_only()` - Only letters
- `numeric_string()` - Only digits
- `contains(substring)` - Contains substring
- `starts_with(prefix)` - Starts with prefix
- `ends_with(suffix)` - Ends with suffix

### Numeric Rules (8)

- `range(min, max)` - Value range
- `min(min)` - Minimum value
- `max(max)` - Maximum value
- `positive()` - Greater than zero
- `negative()` - Less than zero
- `non_zero()` - Not equal to zero
- `finite()` - Finite floating-point (not NaN/Infinity)
- `multiple_of(n)` - Divisible by n

### Choice/Membership Rules (3)

- `equals(value)` - Equal to value
- `not_equals(value)` - Not equal to value
- `one_of(set)` - Member of set

### Collection Rules (3)

- `min_items(min)` - Minimum items
- `max_items(max)` - Maximum items
- `unique()` - All items unique

†Requires `regex` feature

### Rule Composition

- `rule1.and(rule2)` - Both rules must pass
- `rule1.or(rule2)` - Either rule must pass
- `rule.not(code, msg)` - Negate the rule
- `rule.when(predicate)` - Conditional validation
- `rule.map_path(prefix)` - Prefix error paths
- `rule.code(code)` - Override error code
- `rule.message(msg)` - Override error message
- `rule.meta(key, value)` - Add metadata

## Running Examples

```bash
# Basic examples (no features required)
cargo run --example age_primitive

# Examples requiring regex feature
cargo run --example email_primitive --features regex
cargo run --example booking_aggregate --features regex

# Package examples
cargo run -p domainstack-examples --example v2_basic
cargo run -p domainstack-examples --example v2_nested
cargo run -p domainstack-examples --example v2_collections
cargo run -p domainstack-examples --example v2_custom
cargo run -p domainstack-examples --example v3_error_envelope_basic
cargo run -p domainstack-examples --example v3_error_envelope_nested
cargo run -p domainstack-examples --example v4_builder_customization
cargo run -p domainstack-examples --example v5_cross_field_validation

# Framework examples
cargo run -p domainstack-examples-axum
cargo run -p domainstack-examples-actix
```

## Testing

```bash
# Run all tests (all crates)
cargo test --all

# Test with all features
cargo test --all --all-features

# Test specific crate
cargo test -p domainstack
cargo test -p domainstack --features regex
cargo test -p domainstack-derive
cargo test -p domainstack-envelope
```

## Error Handling

### Basic Error Handling

```rust
match user.validate() {
    Ok(_) => println!("Valid!"),
    Err(e) => {
        println!("Validation failed with {} errors:", e.violations.len());
        for v in &e.violations {
            println!("  [{}] {} - {}", v.path, v.code, v.message);
        }

        // For API responses
        let field_errors = e.field_errors_map();
        // { "name": ["Must be at least 3 characters"],
        //   "age": ["Must be between 18 and 120"] }
    }
}
```

### HTTP API Integration

Convert validation errors to error-envelope format:

```rust
use domainstack_envelope::IntoEnvelopeError;

async fn create_user(Json(user): Json<User>) -> Result<Json<User>, Error> {
    user.validate()
        .map_err(|e| e.into_envelope_error())?;

    // ... save user ...
    Ok(Json(user))
}
```

### Framework Adapters

Axum and Actix-web get one-line integration:

```rust
use domainstack_axum::{DomainJson, ErrorResponse};
use axum::post;

type UserJson = DomainJson<User, UserDto>;

#[post("/users")]
async fn create_user(
    UserJson { domain: user, .. }: UserJson
) -> Result<Json<User>, ErrorResponse> {
    // user is guaranteed valid!
    Ok(Json(save_user(user).await?))
}
```

The error response includes field-level details:

```json
{
  "code": "VALIDATION",
  "message": "Validation failed with 2 errors",
  "status": 400,
  "retryable": false,
  "details": {
    "fields": {
      "guest.email": [
        {
          "code": "invalid_email",
          "message": "Invalid email format"
        }
      ],
      "rooms[1].adults": [
        {
          "code": "out_of_range",
          "message": "Must be between 1 and 4"
        }
      ]
    }
  }
}
```

## Documentation

- [Core Library Documentation](./domainstack/README.md) - Detailed API documentation
- [Rules Reference](../docs/RULES.md) - All 31 validation rules
- [API Guide](../docs/api-guide.md) - Complete API usage guide
- [Architecture](../docs/architecture.md) - System design and data flow
- [Examples](./examples/) - Runnable code examples

## Workspace Structure

```
domainstack/
├── domainstack/                 # Core validation library
│   ├── src/
│   │   ├── error.rs             # ValidationError
│   │   ├── path.rs              # Structured error paths
│   │   ├── rule.rs              # Rule<T> and composition
│   │   ├── rules/               # Built-in validation rules
│   │   │   ├── string.rs        # 17 string rules
│   │   │   ├── numeric.rs       # 8 numeric rules
│   │   │   ├── choice.rs        # 3 choice/membership rules
│   │   │   └── collection.rs    # 3 collection rules
│   │   └── validate.rs          # Validate trait
│   └── examples/                # Basic examples
├── domainstack-derive/          # Derive macro implementation
│   ├── src/lib.rs               # #[derive(Validate)] proc macro
│   └── tests/                   # Macro integration tests
├── domainstack-envelope/        # error-envelope integration
│   ├── src/lib.rs               # IntoEnvelopeError trait
│   └── tests/                   # Conversion tests
├── domainstack-http/            # Framework-agnostic HTTP
│   └── src/lib.rs               # Common HTTP error handling
├── domainstack-axum/            # Axum framework adapter
│   └── src/lib.rs               # DomainJson extractor
├── domainstack-actix/           # Actix-web adapter
│   └── src/lib.rs               # DomainJson extractor
├── examples/                     # Core examples package
├── examples-axum/               # Axum booking service example
└── examples-actix/              # Actix-web booking service example
```

## License

Apache 2.0

## Author

Dayna Blackwell <blackwellsystems@protonmail.com>

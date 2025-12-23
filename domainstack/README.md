# domainstack

A Rust validation framework for domain-driven design with derive macro support.

## Overview

This workspace provides a complete validation solution for building domain models in Rust. It consists of four crates:

- **[domainstack](./domainstack/)** - Core validation library
- **[domainstack-derive](./domainstack-derive/)** - Derive macro for `#[derive(Validate)]`
- **[domainstack-error-envelope](./domainstack-error-envelope/)** - error-envelope integration (v0.3)
- **[examples](./examples/)** - Comprehensive examples

## Features

- **Valid-by-construction types** - Invalid states can't exist
- **Derive macro support** - `#[derive(Validate)]` with 5 attributes
- **HTTP integration** - One-line conversion to error-envelope format (v0.3)
- **Composable rules** - Combine validation logic with `and`, `or`, `when`
- **Structured error paths** - Field-level error reporting (e.g., `guest.email.value`, `rooms[1].adults`)
- **Zero dependencies** - Core crate uses only std (regex optional for email validation)

## Quick Start

### With Derive Macro (v0.2+)

```rust
use domainstack::prelude::*;
use domainstack_derive::Validate;

#[derive(Debug, Validate)]
struct User {
    #[validate(length(min = 1, max = 50))]
    name: String,
    
    #[validate(range(min = 18, max = 120))]
    age: u8,
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

struct Email(String);

impl Email {
    pub fn new(raw: String) -> Result<Self, ValidationError> {
        let rule = rules::email();
        validate("email", raw.as_str(), &rule)?;
        Ok(Self(raw))
    }
}
```

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
# Core library only
domainstack = "0.3"

# With derive macro support
domainstack = { version = "0.3", features = ["derive"] }
domainstack-derive = "0.3"

# With error-envelope integration for HTTP APIs (v0.3)
domainstack = { version = "0.3", features = ["derive"] }
domainstack-derive = "0.3"
domainstack-error-envelope = "0.3"

# Optional: enable regex-based email validation
domainstack = { version = "0.3", features = ["derive", "email"] }
```

## Derive Attributes

The `#[derive(Validate)]` macro supports 5 validation attributes:

### 1. Length Validation
```rust
#[validate(length(min = 1, max = 50))]
name: String,
```

### 2. Range Validation
```rust
#[validate(range(min = 18, max = 120))]
age: u8,
```

### 3. Nested Validation
```rust
#[validate(nested)]
guest: Guest,  // Guest must implement Validate
```

Errors are automatically prefixed: `guest.email.value`

### 4. Collection Validation
```rust
#[validate(each(nested))]
rooms: Vec<Room>,

#[validate(each(length(min = 3, max = 10)))]
tags: Vec<String>,
```

Errors include indices: `rooms[1].adults`, `tags[2]`

### 5. Custom Validation
```rust
#[validate(custom = "validate_even")]
value: u8,
```

Custom function signature:
```rust
fn validate_even(value: &u8) -> Result<(), ValidationError> {
    if *value % 2 == 0 {
        Ok(())
    } else {
        Err(ValidationError::single(Path::root(), "not_even", "Must be even"))
    }
}
```

## Built-in Rules

### String Rules
- `email()` - Email format validation
- `non_empty()` - Non-empty string
- `min_len(min)` - Minimum length
- `max_len(max)` - Maximum length
- `length(min, max)` - Length range

### Numeric Rules
- `range(min, max)` - Value range
- `min(min)` - Minimum value
- `max(max)` - Maximum value

### Rule Composition
- `rule1.and(rule2)` - Both rules must pass
- `rule1.or(rule2)` - Either rule must pass
- `rule.not(code, msg)` - Negate the rule
- `rule.when(predicate)` - Conditional validation
- `rule.map_path(prefix)` - Prefix error paths

## Running Examples

```bash
# v0.1 examples (manual validation)
cargo run --example email_primitive
cargo run --example age_primitive
cargo run --example booking_aggregate

# v0.2 examples (derive macro)
cargo run --example v2_basic
cargo run --example v2_nested
cargo run --example v2_collections
cargo run --example v2_custom

# v0.3 examples (error-envelope integration)
cargo run --example v3_error_envelope_basic
cargo run --example v3_error_envelope_nested
```

## Testing

```bash
# Run all tests (core + derive macro)
cargo test --all

# Test with all features
cargo test --all --all-features

# Test specific crate
cargo test -p domainstack
cargo test -p domainstack-derive
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
        // { "name": ["Must be at least 1 characters"],
        //   "age": ["Must be between 18 and 120"] }
    }
}
```

### HTTP API Integration (v0.3)

Convert validation errors to error-envelope format with one line:

```rust
use domainstack_error_envelope::IntoEnvelopeError;

async fn create_user(Json(user): Json<User>) -> Result<Json<User>, Error> {
    user.validate()
        .map_err(|e| e.into_envelope_error())?;
    
    // ... save user ...
    Ok(Json(user))
}
```

The error response includes field-level details with preserved paths:

```json
{
  "code": "VALIDATION",
  "message": "Validation failed with 2 errors",
  "status": 400,
  "retryable": false,
  "details": {
    "fields": {
      "guest.email.value": [
        {
          "code": "invalid_email",
          "message": "Invalid email format",
          "meta": {"max": 255}
        }
      ],
      "rooms[1].adults": [
        {
          "code": "out_of_range",
          "message": "Must be between 1 and 4",
          "meta": {"min": 1, "max": 4}
        }
      ]
    }
  }
}
```

## Documentation

- [Core Library Documentation](./domainstack/README.md) - Detailed API documentation
- [Examples](./examples/) - Runnable code examples
- [CONCEPT.md](../CONCEPT.md) - Design philosophy and architecture

## Workspace Structure

```
domainstack/
├── domainstack/                 # Core validation library
│   ├── src/
│   │   ├── error.rs             # ValidationError
│   │   ├── path.rs              # Structured error paths
│   │   ├── rule.rs              # Rule<T> and composition
│   │   ├── rules/               # Built-in validation rules
│   │   └── validate.rs          # Validate trait
│   └── examples/                # v0.1 manual validation examples
├── domainstack-derive/          # Derive macro implementation
│   ├── src/lib.rs               # #[derive(Validate)] proc macro
│   └── tests/                   # Macro integration tests
├── domainstack-error-envelope/  # error-envelope integration (v0.3)
│   ├── src/lib.rs               # IntoEnvelopeError trait
│   └── tests/                   # Conversion tests
└── examples/                     # Runnable examples (v0.1-v0.3)
```

## Version History

- **v0.3.0** - error-envelope integration, HTTP API support
- **v0.2.0** - Derive macro with 5 attributes, workspace structure
- **v0.1.0** - Core validation library with manual Validate trait

## License

Apache 2.0

## Author

Dayna Blackwell <blackwellsystems@protonmail.com>

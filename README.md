# domainstack

**Rust validation framework for domain-driven design**

[![Blackwell Systems™](https://raw.githubusercontent.com/blackwell-systems/blackwell-docs-theme/main/badge-trademark.svg)](https://github.com/blackwell-systems)
[![Crates.io](https://img.shields.io/crates/v/domainstack.svg)](https://crates.io/crates/domainstack)
[![Documentation](https://docs.rs/domainstack/badge.svg)](https://docs.rs/domainstack)
[![Rust Version](https://img.shields.io/badge/Rust-1.65%2B-CE422B?logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Version](https://img.shields.io/github/v/release/blackwell-systems/domainstack)](https://github.com/blackwell-systems/domainstack/releases)

[![License: Apache 2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](LICENSE)
[![Sponsor](https://img.shields.io/badge/Sponsor-Buy%20Me%20a%20Coffee-yellow?logo=buy-me-a-coffee&logoColor=white)](https://buymeacoffee.com/blackwellsystems)

> **Domain validation framework** • Derive macro support • HTTP integration • Made with Rust

## Features

- **Valid-by-construction types** - Invalid states can't exist
- **Derive macro support** - `#[derive(Validate)]` with 5 attributes
- **HTTP integration** - One-line conversion to error-envelope format
- **Composable rules** - Combine validation logic with `and`, `or`, `when`
- **Structured error paths** - Field-level error reporting (e.g., `guest.email.value`, `rooms[1].adults`)
- **Zero dependencies** - Core crate uses only std (regex optional for email validation)

## Quick Start

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

fn main() {
    let user = User {
        name: "Alice".to_string(),
        age: 30,
    };
    
    match user.validate() {
        Ok(_) => println!("Valid user!"),
        Err(e) => println!("Validation errors: {}", e),
    }
}
```

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
domainstack = { version = "0.3", features = ["derive"] }
domainstack-derive = "0.3"
```

For HTTP API integration:

```toml
[dependencies]
domainstack = { version = "0.3", features = ["derive"] }
domainstack-derive = "0.3"
domainstack-envelope = "0.3"
```

## Crates

This repository contains four crates:

- **[domainstack](./domainstack/)** - Core validation library with composable rules
- **[domainstack-derive](./domainstack/domainstack-derive/)** - Derive macro for `#[derive(Validate)]`
- **[domainstack-envelope](./domainstack/domainstack-envelope/)** - error-envelope integration for HTTP APIs
- **[examples](./domainstack/examples/)** - Comprehensive examples (v0.1-v0.3)

## Documentation

- **[Workspace README](./domainstack/README.md)** - Detailed documentation, all features
- **[Examples](./domainstack/examples/)** - 9 runnable examples
- **[CONCEPT.md](./CONCEPT.md)** - Design philosophy and architecture
- **[API Documentation](https://docs.rs/domainstack)** - Full API reference

## Examples

### Basic Validation

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

### Derive Macro

```rust
#[derive(Debug, Validate)]
struct Booking {
    #[validate(nested)]
    guest: Guest,
    
    #[validate(each(nested))]
    rooms: Vec<Room>,
    
    #[validate(range(min = 1, max = 30))]
    nights: u8,
}
```

### HTTP Integration

```rust
use domainstack_envelope::IntoEnvelopeError;

async fn create_user(Json(user): Json<User>) -> Result<Json<User>, Error> {
    user.validate()
        .map_err(|e| e.into_envelope_error())?;  // One line!
    
    // ... save user ...
    Ok(Json(user))
}
```

Error response with field-level details:

```json
{
  "code": "VALIDATION",
  "status": 400,
  "details": {
    "fields": {
      "guest.email.value": [
        {"code": "invalid_email", "message": "Invalid email format"}
      ],
      "rooms[1].adults": [
        {"code": "out_of_range", "message": "Must be between 1 and 4"}
      ]
    }
  }
}
```

## Running Examples

```bash
cd domainstack

# v0.1 examples (manual validation)
cargo run --example email_primitive
cargo run --example booking_aggregate

# v0.2 examples (derive macro)
cargo run --example v2_basic
cargo run --example v2_nested

# v0.3 examples (HTTP integration)
cargo run --example v3_error_envelope_basic
cargo run --example v3_error_envelope_nested
```

## Testing

```bash
cd domainstack

# Run all tests (89 tests across all crates)
cargo test --all

# Test specific crate
cargo test -p domainstack
cargo test -p domainstack-derive
cargo test -p domainstack-envelope
```

## Version History

- **v0.3.0** (current) - error-envelope integration, HTTP API support
- **v0.2.0** - Derive macro with 5 attributes, workspace structure
- **v0.1.0** - Core validation library with manual Validate trait

## License

Apache 2.0

## Author

Dayna Blackwell - [blackwellsystems@protonmail.com](mailto:blackwellsystems@protonmail.com)

## Contributing

This is an early-stage project. Issues and pull requests are welcome!

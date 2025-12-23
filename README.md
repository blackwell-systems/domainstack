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

// Email with custom validation
#[derive(Debug, Clone, Validate)]
struct Email {
    #[validate(length(min = 5, max = 255))]
    value: String,
}

// Nested validation with automatic path prefixing
#[derive(Debug, Validate)]
struct User {
    #[validate(length(min = 2, max = 50))]
    name: String,
    
    #[validate(range(min = 18, max = 120))]
    age: u8,
    
    #[validate(nested)]  // Validates email, errors appear as "email.value"
    email: Email,
}

// Collection validation with array indices
#[derive(Debug, Validate)]
struct Team {
    #[validate(length(min = 1, max = 50))]
    team_name: String,
    
    #[validate(each(nested))]  // Validates each member, errors like "members[0].name"
    members: Vec<User>,
}

fn main() {
    let team = Team {
        team_name: "Engineering".to_string(),
        members: vec![
            User {
                name: "Alice".to_string(),
                age: 30,
                email: Email { value: "alice@example.com".to_string() },
            },
            User {
                name: "".to_string(),  // Invalid!
                age: 200,               // Invalid!
                email: Email { value: "bob@example.com".to_string() },
            },
        ],
    };
    
    match team.validate() {
        Ok(_) => println!("✓ Team is valid"),
        Err(e) => {
            println!("✗ Validation failed with {} errors:", e.violations.len());
            for v in &e.violations {
                println!("  [{}] {} - {}", v.path, v.code, v.message);
            }
            // Output:
            //   [members[1].name] min_length - Must be at least 2 characters
            //   [members[1].age] out_of_range - Must be between 18 and 120
        }
    }
}
```

### HTTP Integration (One Line!)

```rust
use domainstack_envelope::IntoEnvelopeError;

// Before: Manual error handling (lots of boilerplate)
async fn create_team_manual(Json(team): Json<Team>) -> Result<Json<Team>, StatusCode> {
    match team.validate() {
        Ok(_) => Ok(Json(team)),
        Err(e) => {
            // 15+ lines of boilerplate to build proper JSON error response
            let mut field_errors = std::collections::HashMap::new();
            for violation in e.violations {
                field_errors.entry(violation.path.to_string())
                    .or_insert_with(Vec::new)
                    .push(serde_json::json!({
                        "code": violation.code,
                        "message": violation.message
                    }));
            }
            // ... more code to set status, format response, etc.
            Err(StatusCode::BAD_REQUEST)  // Lost all error details!
        }
    }
}

// After: One line with error-envelope
async fn create_team(Json(team): Json<Team>) -> Result<Json<Team>, Error> {
    team.validate().map_err(|e| e.into_envelope_error())?;  // ← One line!
    Ok(Json(team))
}

// Automatic error response with perfect structure:
// {
//   "code": "VALIDATION",
//   "status": 400,
//   "message": "Validation failed with 2 errors",
//   "details": {
//     "fields": {
//       "members[1].name": [
//         {"code": "min_length", "message": "Must be at least 2 characters"}
//       ],
//       "members[1].age": [
//         {"code": "out_of_range", "message": "Must be between 18 and 120"}
//       ]
//     }
//   }
// }
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

- **[API Guide](./docs/api-guide.md)** - Complete API documentation
- **[Rules Reference](./docs/rules.md)** - All validation rules
- **[Architecture](./docs/architecture.md)** - System design and data flow
- **[Workspace README](./domainstack/README.md)** - Detailed technical docs
- **[Examples](./domainstack/examples/)** - 9 runnable examples
- **[API Documentation](https://docs.rs/domainstack)** - Generated API reference

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

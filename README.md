# domainstack

[![Blackwell Systems™](https://raw.githubusercontent.com/blackwell-systems/blackwell-docs-theme/main/badge-trademark.svg)](https://github.com/blackwell-systems)
[![Crates.io](https://img.shields.io/crates/v/domainstack.svg)](https://crates.io/crates/domainstack)
[![Documentation](https://docs.rs/domainstack/badge.svg)](https://docs.rs/domainstack)
[![Rust Version](https://img.shields.io/badge/Rust-1.76%2B-CE422B?logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Version](https://img.shields.io/github/v/release/blackwell-systems/domainstack)](https://github.com/blackwell-systems/domainstack/releases)

[![License: Apache 2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](LICENSE)
[![CI](https://github.com/blackwell-systems/domainstack/workflows/CI/badge.svg)](https://github.com/blackwell-systems/domainstack/actions)
[![codecov](https://codecov.io/gh/blackwell-systems/domainstack/branch/main/graph/badge.svg)](https://codecov.io/gh/blackwell-systems/domainstack)
[![Sponsor](https://img.shields.io/badge/Sponsor-Buy%20Me%20a%20Coffee-yellow?logo=buy-me-a-coffee&logoColor=white)](https://buymeacoffee.com/blackwellsystems)

**Turn untrusted input into valid domain objects—with structured, field-level errors**

## What is domainstack?

domainstack helps you turn untrusted input into valid domain objects—then report failures back to clients with structured, field-level errors.

It's built around a service-oriented reality:

**Outside world (HTTP/JSON/etc.) → DTOs → Domain (valid-by-construction) → Business logic**

### The core idea

Most validation crates answer: **"Is this DTO valid?"**  
domainstack answers: **"How do I *safely construct domain models* from untrusted input, and return a stable error contract?"**

That means:
- **Domain-first modeling** - Invalid states are unrepresentable
- **Composable rules** - Rules are reusable values, not just attributes
- **Structured error paths** - `rooms[0].adults`, `guest.email.value`
- **Clean boundary mapping** - Optional error-envelope integration for APIs
- **Async checks** (planned) - Uniqueness/existence checks with context

## Quick Start

```rust
use domainstack::prelude::*;
use domainstack::Validate;

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

## Mental Model: DTOs → Domain → Business Logic

### 1) DTO at the boundary (untrusted)
```rust
#[derive(Deserialize)]
pub struct BookingDto {
    pub name: String,
    pub email: String,
    pub guests: u8,
}
```

### 2) Domain inside (trusted)
```rust
pub struct Email(String);

impl Email {
    pub fn new(raw: String) -> Result<Self, ValidationError> {
        validate("email", raw.as_str(), &rules::email().and(rules::max_len(255)))?;
        Ok(Self(raw))
    }
}

pub struct BookingRequest {
    name: String,      // Private!
    email: Email,
    guests: u8,
}

impl TryFrom<BookingDto> for BookingRequest {
    type Error = ValidationError;

    fn try_from(dto: BookingDto) -> Result<Self, Self::Error> {
        let email = Email::new(dto.email)
            .map_err(|e| e.prefixed("email"))?;
        
        validate("name", dto.name.as_str(), 
                 &rules::min_len(1).and(rules::max_len(50)))?;
        validate("guests", &dto.guests, &rules::range(1, 10))?;
        
        Ok(Self { name: dto.name, email, guests: dto.guests })
    }
}
```

### 3) API response mapping (optional)
```rust
use domainstack_envelope::IntoEnvelopeError;

async fn create_booking(Json(dto): Json<BookingDto>) -> Result<Json<Booking>, Error> {
    let domain: BookingRequest = dto.try_into()
        .map_err(|e: ValidationError| e.into_envelope_error())?;
    
    // domain is GUARANTEED valid here - use with confidence!
    Ok(Json(save_booking(domain).await?))
}
```

## How domainstack is Different

### What domainstack is NOT

- **Not "yet another derive macro for DTO validation"** - It's a domain modeling foundation
- **Not a web framework** - It's framework-agnostic validation primitives
- **Not a replacement for thiserror/anyhow** - It complements them for domain boundaries

### What domainstack IS

- **Valid-by-construction foundation** - Domain types created through smart constructors
- **Rules as values** - Build reusable, testable rule libraries
- **Error paths as first-class** - Designed for APIs and UIs from the ground up
- **Boundary adapters** - Optional crates map domain validation to HTTP errors

### Comparison Matrix

| Capability / Focus | domainstack | validator / garde / validify | nutype |
|-------------------|-------------|------------------------------|--------|
| Primary focus | Domain-first + boundary | DTO-first validation | Validated primitives |
| Valid-by-construction aggregates | Yes (core goal) | No (not primary) | No |
| Composable rule algebra (and/or/when) | Yes (core feature) | No / limited | Partial (predicate-based) |
| Structured error paths for APIs | Yes | Partial (varies) | No |
| Async validation w/ context | Planned | No | No |
| Error envelope integration | Yes (optional) | No | No |

### When to use domainstack

**Use domainstack if you want:**
- Clean DTO → Domain conversion
- Domain objects that can't exist in invalid states
- Reusable validation rules shared across services
- Consistent field-level errors that map to forms/clients
- Async validation with database/API context (coming soon)

**You might not need domainstack if:**
- You're validating only DTOs and your domain is basically DTO-shaped
- You don't care about structured field paths/codes across services
- You're happy with ad-hoc handler-level error mapping

## Valid-by-Construction Pattern

The recommended approach enforces validation at domain boundaries:

```rust
use domainstack::prelude::*;
use serde::Deserialize;

// DTO - Public, for deserialization
#[derive(Deserialize)]
pub struct UserDto {
    pub name: String,
    pub age: u8,
    pub email: String,
}

// Domain - Private fields, enforced validity
pub struct User {
    name: String,     // Private!
    age: u8,
    email: Email,
}

impl User {
    // Smart constructor - validation enforced here
    pub fn new(name: String, age: u8, email: String) -> Result<Self, ValidationError> {
        let mut err = ValidationError::new();
        
        let name_rule = rules::min_len(2).and(rules::max_len(50));
        if let Err(e) = validate("name", name.as_str(), &name_rule) {
            err.extend(e);
        }
        
        let age_rule = rules::range(18, 120);
        if let Err(e) = validate("age", &age, &age_rule) {
            err.extend(e);
        }
        
        let email = Email::new(email).map_err(|e| e.prefixed("email"))?;
        
        if !err.is_empty() {
            return Err(err);
        }
        
        Ok(Self { name, age, email })
    }
    
    // Getters only - no setters
    pub fn name(&self) -> &str { &self.name }
    pub fn age(&self) -> u8 { self.age }
    pub fn email(&self) -> &Email { &self.email }
}

// Conversion at boundary
impl TryFrom<UserDto> for User {
    type Error = ValidationError;
    
    fn try_from(dto: UserDto) -> Result<Self, Self::Error> {
        User::new(dto.name, dto.age, dto.email)
    }
}

// HTTP handler
async fn create_user(Json(dto): Json<UserDto>) -> Result<Json<User>, Error> {
    let user = User::try_from(dto)
        .map_err(|e| e.into_envelope_error())?;
    // user is GUARANTEED valid here - no need to check!
    Ok(Json(user))
}
```

**Key Points**:
- DTOs are public for deserialization
- Domain types have private fields
- Validation happens in constructors
- `TryFrom` enforces validation at boundary
- Invalid domain objects cannot exist

### HTTP Integration (Optional Adapter)

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

## Framework Adapters (v0.4+)

One-line DTO→Domain extraction for Axum and Actix-web.

### Axum

```rust
use domainstack_axum::{DomainJson, ErrorResponse};
use axum::{routing::post, Router, Json};

type UserJson = DomainJson<User, UserDto>;

async fn create_user(
    UserJson { domain: user, .. }: UserJson
) -> Result<Json<User>, ErrorResponse> {
    Ok(Json(save_user(user).await?))  // user is guaranteed valid!
}

let app = Router::new().route("/users", post(create_user));
```

### Actix-web

```rust
use domainstack_actix::{DomainJson, ErrorResponse};
use actix_web::{post, web};

type UserJson = DomainJson<User, UserDto>;

#[post("/users")]
async fn create_user(
    UserJson { domain: user, .. }: UserJson
) -> Result<web::Json<User>, ErrorResponse> {
    Ok(web::Json(save_user(user).await?))  // user is guaranteed valid!
}
```

**What the adapters provide:**
- `DomainJson<T, Dto>` extractor - Deserialize JSON → validate → convert to domain
- `ErrorResponse` - Automatic 400 responses with structured field-level errors
- `From` impls - `?` operator works with `ValidationError` and `error_envelope::Error`
- **Identical APIs** - Same pattern across both frameworks

See [domainstack-axum](./domainstack/domainstack-axum/) and [domainstack-actix](./domainstack/domainstack-actix/) for complete documentation.

## Installation

```toml
[dependencies]
# Core library only
domainstack = "0.3"

# With derive macro (recommended)
domainstack = { version = "0.3", features = ["derive"] }

# With email validation (adds regex dependency)
domainstack = { version = "0.3", features = ["derive", "email"] }

# Optional: HTTP error mapping
domainstack-envelope = "0.3"

# Optional: Framework adapters (v0.4+)
domainstack-axum = "0.4"    # For Axum web framework
domainstack-actix = "0.4"   # For Actix-web framework
```

## Crates

This repository contains seven crates:

**Core:**
- **[domainstack](./domainstack/)** - Core validation library with composable rules
- **[domainstack-derive](./domainstack/domainstack-derive/)** - Derive macro for `#[derive(Validate)]`
- **[domainstack-envelope](./domainstack/domainstack-envelope/)** - error-envelope integration for HTTP APIs

**Framework Adapters (v0.4+):**
- **[domainstack-http](./domainstack/domainstack-http/)** - Framework-agnostic HTTP helpers
- **[domainstack-axum](./domainstack/domainstack-axum/)** - Axum extractor and response implementations
- **[domainstack-actix](./domainstack/domainstack-actix/)** - Actix-web extractor and response implementations

**Examples:**
- **[examples](./domainstack/examples/)** - Core examples (v0.1-v0.3)
- **[examples-axum](./domainstack/examples-axum/)** - Axum booking service example
- **[examples-actix](./domainstack/examples-actix/)** - Actix-web booking service example

## Documentation

### Multi-README Structure

This project has **multiple README files** for different audiences:

1. **[README.md](./README.md)** (this file) - GitHub visitors
2. **[domainstack/README.md](./domainstack/README.md)** - Cargo/crates.io users
3. **Individual crate READMEs** - Library implementers

### Additional Documentation

- **[API Guide](./docs/api-guide.md)** - Complete API documentation
- **[Rules Reference](./docs/rules.md)** - All validation rules
- **[Architecture](./docs/architecture.md)** - System design and data flow
- **[Examples](./domainstack/examples/)** - 9 runnable examples
- **[API Documentation](https://docs.rs/domainstack)** - Generated API reference
- **[Publishing Guide](./PUBLISHING.md)** - How to publish to crates.io
- **[Coverage Guide](./COVERAGE.md)** - Running coverage locally

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

- **v0.4.0** (current) - Framework adapters for Axum and Actix-web
- **v0.3.0** - error-envelope integration, HTTP API support
- **v0.2.0** - Derive macro with 5 attributes, workspace structure
- **v0.1.0** - Core validation library with manual Validate trait

## License

Apache 2.0

## Author

Dayna Blackwell - [blackwellsystems@protonmail.com](mailto:blackwellsystems@protonmail.com)

## Contributing

This is an early-stage project. Issues and pull requests are welcome!

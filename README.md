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
- **Async validation** - Database uniqueness checks with context passing (v0.5)
- **Type-state tracking** - Compile-time guarantees with phantom types (v0.6)
- **OpenAPI schema generation** - Auto-generate API documentation from your types (v0.7-v0.8)

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
| Async validation w/ context | ✅ v0.5 | No | No |
| Type-state validation tracking | ✅ v0.6 | No | Partial |
| OpenAPI schema generation | ✅ v0.7-v0.8 | No | No |
| Error envelope integration | Yes (optional) | No | No |

### When to use domainstack

**Use domainstack if you want:**
- Clean DTO → Domain conversion
- Domain objects that can't exist in invalid states
- Reusable validation rules shared across services
- Consistent field-level errors that map to forms/clients
- Async validation with database/API context
- Compile-time guarantees that data was validated (phantom types)

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

## Framework Adapters

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
domainstack = "1.0"

# With derive macro (recommended)
domainstack = { version = "1.0", features = ["derive"] }

# With regex validation (adds regex dependency for email/URL/pattern matching)
domainstack = { version = "1.0", features = ["derive", "regex"] }

# With async validation (for database uniqueness checks, external APIs)
domainstack = { version = "1.0", features = ["async"] }

# All features
domainstack = { version = "1.0", features = ["derive", "regex", "async"] }

# Optional: HTTP error mapping
domainstack-envelope = "1.0"

# Optional: Framework adapters
domainstack-axum = "1.0"    # For Axum web framework
domainstack-actix = "1.0"   # For Actix-web framework
```

## Crates

This repository contains **10 workspace members** (7 publishable crates, 3 example crates):

**Core (Publishable):**
- **[domainstack](./domainstack/)** - Core validation library with composable rules
- **[domainstack-derive](./domainstack/domainstack-derive/)** - Derive macro for `#[derive(Validate)]`
- **[domainstack-envelope](./domainstack/domainstack-envelope/)** - error-envelope integration for HTTP APIs
- **[domainstack-schema](./domainstack/domainstack-schema/)** - OpenAPI 3.0 schema generation (v0.7-v0.8)

**Framework Adapters (Publishable):**
- **[domainstack-http](./domainstack/domainstack-http/)** - Framework-agnostic HTTP helpers
- **[domainstack-axum](./domainstack/domainstack-axum/)** - Axum extractor and response implementations
- **[domainstack-actix](./domainstack/domainstack-actix/)** - Actix-web extractor and response implementations

**Examples (Not Published):**
- **[domainstack-examples](./domainstack/domainstack-examples/)** - Core examples (v0.1-v0.6)
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
- **[Rules Reference](./docs/RULES.md)** - All validation rules
- **[Architecture](./docs/architecture.md)** - System design and data flow
- **[Examples](./domainstack/domainstack-examples/)** - 9 runnable examples
- **[API Documentation](https://docs.rs/domainstack)** - Generated API reference
- **[Publishing Guide](./PUBLISHING.md)** - How to publish to crates.io
- **[Coverage Guide](./COVERAGE.md)** - Running coverage locally

## Key Features

### Core Capabilities

- **31 Validation Rules** - String, numeric, collection validation out of the box
- **Composable Rules** - Combine with `.and()`, `.or()`, `.when()` for complex logic
- **Nested Validation** - Automatic path tracking for deeply nested structures
- **Collection Validation** - Array indices in error paths (`items[0].field`)
- **Builder Customization** - Customize error codes, messages, and metadata

### Advanced Features

#### Async Validation

Perform database queries, API calls, and I/O operations in validation:

```rust
use domainstack::{AsyncValidate, ValidationContext, ValidationError};
use async_trait::async_trait;

#[async_trait]
impl AsyncValidate for UserRegistration {
    async fn validate_async(&self, ctx: &ValidationContext) -> Result<(), ValidationError> {
        let db = ctx.get_resource::<Database>("db")?;

        // Check email uniqueness in database
        if db.email_exists(&self.email).await {
            return Err(ValidationError::single(
                Path::from("email"),
                "email_taken",
                "Email is already registered"
            ));
        }
        Ok(())
    }
}
```

**Use cases:** Database uniqueness checks, external API validation, rate limiting, cross-service validation.

#### Cross-Field Validation

Validate relationships between multiple fields:

```rust
#[derive(Validate)]
#[validate(
    check = "self.password == self.password_confirmation",
    code = "passwords_mismatch",
    message = "Passwords must match"
)]
struct RegisterForm {
    #[validate(length(min = 8))]
    password: String,
    password_confirmation: String,
}
```

**Use cases:** Password confirmation, date ranges, mutually exclusive fields, conditional business rules.

#### Type-State Validation

Compile-time guarantees with phantom types:

```rust
use domainstack::typestate::{Validated, Unvalidated};
use std::marker::PhantomData;

pub struct Email<State = Unvalidated> {
    value: String,
    _state: PhantomData<State>,
}

impl Email<Unvalidated> {
    pub fn validate(self) -> Result<Email<Validated>, ValidationError> {
        validate("email", self.value.as_str(), &rules::email())?;
        Ok(Email { value: self.value, _state: PhantomData })
    }
}

// Only accept validated emails!
fn send_email(email: Email<Validated>) {
    // Compiler GUARANTEES email is validated!
}
```

**Benefits:** Zero runtime cost, compile-time safety, self-documenting APIs.

#### OpenAPI Schema Generation (v0.7-v0.8)

Auto-generate OpenAPI 3.0 documentation from your domain types:

```rust
use domainstack_schema::{OpenApiBuilder, Schema, ToSchema};
use serde_json::json;

struct User {
    email: String,
    age: u8,
    name: String,
}

impl ToSchema for User {
    fn schema_name() -> &'static str { "User" }

    fn schema() -> Schema {
        Schema::object()
            .property("email", Schema::string()
                .format("email")
                .example(json!("user@example.com")))
            .property("age", Schema::integer()
                .minimum(18)
                .maximum(120))
            .property("name", Schema::string()
                .min_length(1)
                .max_length(100))
            .required(&["email", "age", "name"])
    }
}

// Generate OpenAPI spec
let spec = OpenApiBuilder::new("User API", "1.0.0")
    .description("User management API")
    .register::<User>()
    .build();

println!("{}", spec.to_json().unwrap());
// → Complete OpenAPI 3.0 JSON with schemas, constraints, examples
```

**What you get:**
- **Interactive docs** - Swagger UI, ReDoc, Postman collections
- **Client generation** - TypeScript, Python, Java clients (via openapi-generator)
- **Contract testing** - Frontend/backend validation agreement
- **API gateway integration** - Kong, AWS API Gateway, Traefik
- **Single source of truth** - Change Rust validation, docs update automatically

**Features (v0.8):**
- Schema composition (anyOf/allOf/oneOf)
- Rich metadata (default, example, examples)
- Request/response modifiers (readOnly, writeOnly, deprecated)
- Vendor extensions for non-mappable validations
- Type-safe fluent API

See [domainstack-schema/OPENAPI_CAPABILITIES.md](./domainstack/domainstack-schema/OPENAPI_CAPABILITIES.md) for complete documentation.

### 31 Built-in Validation Rules

**String Rules (17):**
- Length: `non_empty()`, `min_len()`, `max_len()`, `length()`, `len_chars()`
- Format: `email()`, `url()`, `matches_regex()`
- Content: `alpha_only()`, `alphanumeric()`, `numeric_string()`, `ascii()`
- Patterns: `contains()`, `starts_with()`, `ends_with()`, `non_blank()`, `no_whitespace()`

**Numeric Rules (8):**
- Comparison: `min()`, `max()`, `range()`
- Sign: `positive()`, `negative()`, `non_zero()`
- Special: `finite()` (for floats), `multiple_of()`

**Choice Rules (3):**
- `equals()`, `not_equals()`, `one_of()`

**Collection Rules (3):**
- `min_items()`, `max_items()`, `unique()`

**Combinators:**
- `.and()` - Both rules must pass
- `.or()` - Either rule must pass
- `.when()` - Conditional validation
- `.code()`, `.message()`, `.meta()` - Customize errors

See [Rules Reference](./docs/RULES.md) for complete documentation and examples.

## Examples

### Basic Validation

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
cargo run --example email_primitive --features regex
cargo run --example booking_aggregate --features regex
cargo run --example age_primitive

# v0.2 examples (derive macro)
cargo run --example v2_basic
cargo run --example v2_nested
cargo run --example v2_collections
cargo run --example v2_custom

# v0.3 examples (HTTP integration)
cargo run --example v3_error_envelope_basic
cargo run --example v3_error_envelope_nested

# v0.4 examples (builder customization)
cargo run --example v4_builder_customization

# v0.5 examples (cross-field validation)
cargo run --example v5_cross_field_validation

# v0.5 examples (async validation)
cargo run --example async_validation --features async
cargo run --example async_sqlite --features async

# v0.6 examples (phantom types)
cargo run --example phantom_types --features regex

# v0.7-v0.8 examples (OpenAPI schema generation)
cd domainstack-schema
cargo run --example user_api
cargo run --example v08_features

# Framework examples
cd examples-axum && cargo run    # Axum booking service
cd examples-actix && cargo run   # Actix-web booking service
```

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

## License

Apache 2.0

## Author

Dayna Blackwell - [blackwellsystems@protonmail.com](mailto:blackwellsystems@protonmail.com)

## Contributing

This is an early-stage project. Issues and pull requests are welcome!

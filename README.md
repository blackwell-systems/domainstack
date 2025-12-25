# domainstack

[![Blackwell Systems™](https://raw.githubusercontent.com/blackwell-systems/blackwell-docs-theme/main/badge-trademark.svg)](https://github.com/blackwell-systems)
[![Rust Version](https://img.shields.io/badge/Rust-1.76%2B-CE422B?logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Crates.io](https://img.shields.io/crates/v/domainstack.svg)](https://crates.io/crates/domainstack)
[![Documentation](https://docs.rs/domainstack/badge.svg)](https://docs.rs/domainstack)
[![Version](https://img.shields.io/github/v/release/blackwell-systems/domainstack)](https://github.com/blackwell-systems/domainstack/releases)
[![CI](https://github.com/blackwell-systems/domainstack/workflows/CI/badge.svg)](https://github.com/blackwell-systems/domainstack/actions)
[![codecov](https://codecov.io/gh/blackwell-systems/domainstack/branch/main/graph/badge.svg)](https://codecov.io/gh/blackwell-systems/domainstack)
[![License: Apache 2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](LICENSE)
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
- **Async validation** - Database uniqueness checks with context passing
- **Type-state tracking** - Compile-time guarantees with phantom types
- **Auto-derived OpenAPI schemas** - Write validation rules once, get OpenAPI 3.0 schemas automatically (zero duplication)

## Quick Start

```rust
use domainstack::prelude::*;
use domainstack::Validate;
use domainstack_derive::ToSchema;

// Email with custom validation
#[derive(Debug, Clone, Validate, ToSchema)]
struct Email {
    #[validate(length(min = 5, max = 255))]
    value: String,
}

// Nested validation with automatic path prefixing
#[derive(Debug, Validate, ToSchema)]
struct User {
    #[validate(length(min = 2, max = 50))]
    name: String,

    #[validate(range(min = 18, max = 120))]
    age: u8,

    #[validate(nested)]  // Validates email, errors appear as "email.value"
    email: Email,
}

// Collection validation with array indices
#[derive(Debug, Validate, ToSchema)]
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

    // Auto-generate OpenAPI schema from validation rules (zero duplication!)
    let user_schema = User::schema();
    // → name: { type: "string", minLength: 2, maxLength: 50 }
    // → age: { type: "integer", minimum: 18, maximum: 120 }
    // → email: { $ref: "#/components/schemas/Email" }
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
| **Core Philosophy** |
| Primary focus | Domain-first + boundary | DTO-first validation | Validated primitives |
| Valid-by-construction aggregates | Yes (core goal) | No (not primary) | No |
| Zero dependencies (core) | Yes | No (serde, regex, etc.) | Yes |
| **Validation Features** |
| Built-in validation rules | 37 (string, numeric, collection, date/time) | ~20-30 (varies) | Predicate-based |
| Composable rule algebra (and/or/when) | Yes (core feature) | No / limited | Partial (predicate-based) |
| Cross-field validation | Yes | Yes | No |
| Collection validation (unique, min/max items) | Yes | Partial | No |
| Date/time validation (past, future, age) | Yes (chrono feature) | Limited | No |
| **Advanced Capabilities** |
| Structured error paths for APIs | Yes | Partial (varies) | No |
| Async validation w/ context | Yes | No | No |
| Type-state validation tracking | Yes | No | Partial |
| **Integration** |
| OpenAPI schema generation | Yes (auto-derive) | No | No |
| Error envelope integration | Yes (optional) | No | No |
| Framework adapters (Axum, Actix, Rocket) | Yes | Varies by crate | No |

### When to use domainstack

**Use domainstack if you want:**
- Clean DTO → Domain conversion
- Domain objects that can't exist in invalid states
- Reusable validation rules shared across services
- Consistent field-level errors that map to forms/clients
- Async validation with database/API context
- Compile-time guarantees that data was validated (phantom types)

## Valid-by-Construction Pattern

The recommended approach enforces validation at domain boundaries:

```rust
use domainstack::prelude::*;
use domainstack::Validate;
use serde::Deserialize;

// DTO - Public, for deserialization
#[derive(Deserialize)]
pub struct UserDto {
    pub name: String,
    pub age: u8,
    pub email: String,
}

// Domain - Private fields, enforced validity
#[derive(Debug, Validate)]
pub struct User {
    #[validate(length(min = 2, max = 50))]
    name: String,     // Private!

    #[validate(range(min = 18, max = 120))]
    age: u8,

    #[validate(nested)]
    email: Email,
}

impl User {
    // Smart constructor - validation enforced here
    pub fn new(name: String, age: u8, email_raw: String) -> Result<Self, ValidationError> {
        let email = Email::new(email_raw).map_err(|e| e.prefixed("email"))?;

        let user = Self { name, age, email };
        user.validate()?;  // One line - validates all fields!
        Ok(user)
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
- `#[derive(Validate)]` eliminates manual error accumulation boilerplate
- Validation happens in constructors with a single `.validate()` call
- `TryFrom` enforces validation at boundary
- Invalid domain objects cannot exist

<details>
<summary>Manual validation (when you need fine-grained control)</summary>

If you need custom error messages or conditional logic, use the manual approach:

```rust
impl User {
    pub fn new(name: String, age: u8, email: String) -> Result<Self, ValidationError> {
        let mut err = ValidationError::new();

        let name_rule = rules::min_len(2)
            .and(rules::max_len(50))
            .code("invalid_name")
            .message("Name must be between 2 and 50 characters");
        if let Err(e) = validate("name", name.as_str(), &name_rule) {
            err.extend(e);
        }

        let age_rule = rules::range(18, 120)
            .code("invalid_age")
            .message("Age must be between 18 and 120");
        if let Err(e) = validate("age", &age, &age_rule) {
            err.extend(e);
        }

        let email = Email::new(email).map_err(|e| e.prefixed("email"))?;

        if !err.is_empty() {
            return Err(err);
        }

        Ok(Self { name, age, email })
    }
}
```

</details>

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

<details>
<summary>Click to expand installation options</summary>

### Core Library

```toml
[dependencies]
# Minimal - Core validation only (no dependencies except smallvec)
domainstack = "1.0"

# Recommended - Core + derive macro
domainstack = { version = "1.0", features = ["derive"] }

# All features enabled
domainstack = { version = "1.0", features = ["derive", "regex", "async", "chrono"] }
```

### Feature Flags

| Feature | Description | Added Dependencies |
|---------|-------------|-------------------|
| `std` | Standard library support (enabled by default) | None |
| `derive` | Enable `#[derive(Validate)]` macro | `domainstack-derive` |
| `regex` | Email, URL, and pattern matching rules | `regex`, `once_cell` |
| `async` | Async validation with context passing | `async-trait` |
| `chrono` | Date/time validation rules (past, future, age_range) | `chrono` |

**Examples:**

```toml
# Web API with validation and derive macros
domainstack = { version = "1.0", features = ["derive", "regex"] }

# Async validation for database checks
domainstack = { version = "1.0", features = ["derive", "async"] }

# Date/time validation
domainstack = { version = "1.0", features = ["derive", "chrono"] }
```

### Optional Companion Crates

```toml
# OpenAPI 3.0 schema generation
domainstack-schema = "1.0"

# HTTP error envelope integration (RFC 9457 Problem Details)
domainstack-envelope = "1.0"

# HTTP utilities (framework-agnostic)
domainstack-http = "1.0"

# Framework adapters - one-line DTO→Domain extraction
domainstack-axum = "1.0"      # For Axum 0.7+
domainstack-actix = "1.0"     # For Actix-web 4.x
domainstack-rocket = "1.0"    # For Rocket 0.5+
```

### Full Stack Example

```toml
[dependencies]
# Core validation with all features
domainstack = { version = "1.0", features = ["derive", "regex", "async", "chrono"] }

# Schema generation for OpenAPI docs
domainstack-schema = "1.0"

# Axum web framework integration
domainstack-axum = "1.0"
axum = "0.7"
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
```

</details>

## Key Features

### Core Capabilities

- **37 Validation Rules** - String, numeric, collection, and date/time validation out of the box
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

#### OpenAPI Schema Generation

Auto-generate OpenAPI 3.0 schemas directly from your validation rules—**zero duplication**:

```rust
use domainstack_derive::{Validate, ToSchema};
use domainstack_schema::OpenApiBuilder;

// Write validation rules ONCE, get BOTH runtime validation AND OpenAPI schemas!
#[derive(Validate, ToSchema)]
#[schema(description = "User in the system")]
struct User {
    #[validate(email)]
    #[validate(max_len = 255)]
    #[schema(description = "User's email", example = "user@example.com")]
    email: String,

    #[validate(range(min = 18, max = 120))]
    #[schema(description = "User's age")]
    age: u8,

    #[validate(min_len = 1)]
    #[validate(max_len = 100)]
    name: String,

    // Optional fields automatically excluded from required array
    #[validate(min_len = 1)]
    nickname: Option<String>,
}

// Generate OpenAPI spec with automatic constraint mapping
let spec = OpenApiBuilder::new("User API", "1.0.0")
    .register::<User>()
    .build();

// → email: { type: "string", format: "email", maxLength: 255, ... }
// → age: { type: "integer", minimum: 18, maximum: 120 }
// → name: { type: "string", minLength: 1, maxLength: 100 }
// → required: ["email", "age", "name"]  (nickname excluded)
```

**Automatic Rule → Schema Mapping:**
- `email()` → `format: "email"`
- `url()` → `format: "uri"`
- `min_len(n)` / `max_len(n)` → `minLength` / `maxLength`
- `range(min, max)` → `minimum` / `maximum`
- `min_items(n)` / `max_items(n)` → `minItems` / `maxItems`
- `alphanumeric()` → `pattern: "^[a-zA-Z0-9]*$"`
- `ascii()` → `pattern: "^[\x00-\x7F]*$"`
- `Option<T>` → excluded from `required` array
- `#[validate(nested)]` → `$ref: "#/components/schemas/TypeName"`
- `Vec<T>` with `each_nested` → `type: "array"` with `$ref` items

**Manual implementation** still supported for complex cases:

```rust
impl ToSchema for CustomType {
    fn schema() -> Schema {
        Schema::object()
            .property("field", Schema::string().pattern("custom"))
            .required(&["field"])
    }
}
```

**What you get:**
- **Interactive docs** - Swagger UI, ReDoc, Postman collections
- **Client generation** - TypeScript, Python, Java clients (via openapi-generator)
- **Contract testing** - Frontend/backend validation agreement
- **API gateway integration** - Kong, AWS API Gateway, Traefik
- **Single source of truth** - Change Rust validation, docs update automatically

**Features:**
- Schema composition (anyOf/allOf/oneOf)
- Rich metadata (default, example, examples)
- Request/response modifiers (readOnly, writeOnly, deprecated)
- Vendor extensions for non-mappable validations
- Type-safe fluent API

See [domainstack-schema/OPENAPI_CAPABILITIES.md](./domainstack/domainstack-schema/OPENAPI_CAPABILITIES.md) for complete documentation.

### 37 Built-in Validation Rules

**String Rules (17):**
- Length: `non_empty()`, `min_len()`, `max_len()`, `length()`, `len_chars()`
- Format: `email()`, `url()`, `matches_regex()`
- Content: `alpha_only()`, `alphanumeric()`, `numeric_string()`, `ascii()`
- Patterns: `contains()`, `starts_with()`, `ends_with()`, `non_blank()`, `no_whitespace()`

**All rules work with `each(rule)` for collection validation:**

```rust
#[derive(Validate)]
struct BlogPost {
    #[validate(each(email))]        // Validate each email in list
    author_emails: Vec<String>,

    #[validate(each(url))]           // Validate each URL
    related_links: Vec<String>,

    #[validate(each(length(min = 1, max = 50)))]  // Validate tag length
    tags: Vec<String>,
}
// Errors include array indices: tags[0], author_emails[1], etc.
```

**Numeric Rules (8):**
- Comparison: `min()`, `max()`, `range()`
- Sign: `positive()`, `negative()`, `non_zero()`
- Special: `finite()` (for floats), `multiple_of()`

**Choice Rules (3):**
- `equals()`, `not_equals()`, `one_of()`

**Collection Rules (4):**
- `min_items()`, `max_items()`, `unique()`, `non_empty_items()`

**Date/Time Rules (5):** (requires `chrono` feature)
- Temporal: `past()`, `future()`, `before()`, `after()`
- Age: `age_range()`

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

### Collection Item Validation

Validate each item in a collection with any validation rule:

```rust
#[derive(Debug, Validate)]
struct BlogPost {
    #[validate(min_len = 1)]
    #[validate(max_len = 200)]
    title: String,

    // Validate each email in the list
    #[validate(each(email))]
    #[validate(min_items = 1)]
    #[validate(max_items = 5)]
    author_emails: Vec<String>,

    // Validate each tag's length
    #[validate(each(length(min = 1, max = 50)))]
    tags: Vec<String>,

    // Validate each URL format
    #[validate(each(url))]
    related_links: Vec<String>,

    // Validate each keyword is alphanumeric
    #[validate(each(alphanumeric))]
    keywords: Vec<String>,
}
```

Error paths include array indices for precise error tracking:
- `author_emails[0]` - "Invalid email format"
- `tags[2]` - "Must be at most 50 characters"
- `related_links[1]` - "Invalid URL format"

## Running Examples

Examples are included in the repository (not published to crates.io). Clone the repo to try them:

<details>
<summary>Click to expand example commands</summary>

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

</details>

## Testing

<details>
<summary>Click to expand testing commands</summary>

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

</details>

## 📦 Crates

This repository contains **12 workspace members** (8 publishable crates, 4 example crates):

**Core (Publishable):**
- **[domainstack](./domainstack/)** - Core validation library with composable rules
- **[domainstack-derive](./domainstack/domainstack-derive/)** - Derive macro for `#[derive(Validate)]`
- **[domainstack-envelope](./domainstack/domainstack-envelope/)** - error-envelope integration for HTTP APIs
- **[domainstack-schema](./domainstack/domainstack-schema/)** - OpenAPI 3.0 schema generation

**Framework Adapters (Publishable):**
- **[domainstack-http](./domainstack/domainstack-http/)** - Framework-agnostic HTTP helpers
- **[domainstack-axum](./domainstack/domainstack-axum/)** - Axum extractor and response implementations
- **[domainstack-actix](./domainstack/domainstack-actix/)** - Actix-web extractor and response implementations
- **[domainstack-rocket](./domainstack/domainstack-rocket/)** - Rocket request guard and response implementations

**Examples (Available in Repository):**
- **[domainstack-examples](./domainstack/domainstack-examples/)** - Core validation examples
- **[examples-axum](./domainstack/examples-axum/)** - Axum booking service example
- **[examples-actix](./domainstack/examples-actix/)** - Actix-web booking service example
- **[examples-rocket](./domainstack/examples-rocket/)** - Rocket booking service example

**Note:** Example crates are not published to crates.io but are included in the [GitHub repository](https://github.com/blackwell-systems/domainstack). Clone the repo to run them locally.

## 📚 Documentation

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

## License

Apache 2.0

## Author

Dayna Blackwell - [blackwellsystems@protonmail.com](mailto:blackwellsystems@protonmail.com)

## Contributing

This is an early-stage project. Issues and pull requests are welcome!

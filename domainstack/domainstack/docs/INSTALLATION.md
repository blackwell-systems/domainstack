# Installation Guide

**Complete guide to installing and configuring domainstack and its companion crates.**

## Table of Contents

- [Quick Start](#quick-start)
- [Core Crate](#core-crate)
- [Feature Flags](#feature-flags)
- [Companion Crates](#companion-crates)
- [Common Configurations](#common-configurations)
- [Version Compatibility](#version-compatibility)

## Quick Start

Add domainstack to your `Cargo.toml`:

```toml
[dependencies]
domainstack = { version = "1.0", features = ["derive", "regex"] }
```

**Recommended features:**
- `derive` - Enables `#[derive(Validate)]` macro
- `regex` - Enables email, URL, and pattern matching rules

## Core Crate

### Basic Installation

```toml
[dependencies]
# Minimal - only core validation primitives
domainstack = "1.0"

# Recommended - includes derive macro
domainstack = { version = "1.0", features = ["derive"] }

# Full-featured - all optional features
domainstack = { version = "1.0", features = ["derive", "regex", "async", "chrono", "serde"] }
```

### Version Requirements

- **Rust:** 1.70 or later (for advanced const generics)
- **Edition:** 2021 or later

## Feature Flags

The `domainstack` crate provides several optional features:

| Feature | Adds | Use When | Dependencies |
|---------|------|----------|--------------|
| `derive` | `#[derive(Validate)]` macro | Declarative validation (recommended) | `domainstack-derive` |
| `regex` | Email, URL, pattern matching | Web APIs, user input validation | `regex` |
| `async` | `AsyncValidate` trait | Database checks, external API validation | `async-trait` |
| `chrono` | Date/time validation rules | Temporal constraints, age verification | `chrono` |
| `serde` | `ValidateOnDeserialize` derive | Validate during JSON/YAML parsing | `serde` |

### Feature Details

#### `derive` - Derive Macro

Enables `#[derive(Validate)]` for declarative validation:

```rust
#[derive(Validate)]
struct User {
    #[validate(email, max_len = 255)]
    email: String,

    #[validate(range(min = 18, max = 120))]
    age: u8,
}
```

**Adds:**
- `#[derive(Validate)]` procedural macro
- `#[validate(...)]` field attributes

**When to use:** Almost always. Manual `impl Validate` is verbose.

#### `regex` - Pattern Matching

Enables rules that require regular expressions:

```rust
use domainstack::rules::*;

let email_rule = email();                        // Requires regex
let url_rule = url();                             // Requires regex
let pattern_rule = matches_regex(r"^[A-Z0-9]+$"); // Requires regex
```

**Adds:**
- `email()` - RFC 5322 email validation
- `url()` - URL format validation
- `matches_regex()` - Custom regex patterns
- `alphanumeric()` - Alphanumeric-only validation
- `alpha_only()` - Alphabetic-only validation
- `numeric_string()` - Numeric string validation

**When to use:** Web APIs, user input, any validation requiring pattern matching.

#### `async` - Async Validation

Enables `AsyncValidate` trait for I/O-based validation:

```rust
use domainstack::{AsyncValidate, ValidationContext};
use async_trait::async_trait;

#[async_trait]
impl AsyncValidate for User {
    async fn validate_async(&self, ctx: &ValidationContext) -> Result<(), ValidationError> {
        // Database uniqueness check
        let db = ctx.get_resource::<PgPool>("db")?;
        let exists = query!("SELECT id FROM users WHERE email = $1", self.email)
            .fetch_optional(db)
            .await?;

        if exists.is_some() {
            return Err(ValidationError::single("email", "taken", "Email already exists"));
        }

        Ok(())
    }
}
```

**Adds:**
- `AsyncValidate` trait
- `ValidationContext` for passing resources

**When to use:**
- Database uniqueness checks
- External API validation
- Rate limiting
- Real-time availability checks

**Dependencies:** `async-trait = "0.1"`

#### `chrono` - Date/Time Validation

Enables date and time validation rules:

```rust
use domainstack::rules::*;
use chrono::NaiveDate;

let booking_date = NaiveDate::from_ymd_opt(2025, 6, 15).unwrap();

let future_rule = future();                              // Must be in future
let past_rule = past();                                  // Must be in past
let before_rule = before(NaiveDate::from_ymd_opt(2026, 1, 1).unwrap());
let after_rule = after(NaiveDate::from_ymd_opt(2025, 1, 1).unwrap());
let age_rule = min_age(18);                              // Age verification
```

**Adds:**
- `future()` - Date must be in the future
- `past()` - Date must be in the past
- `before()` - Date must be before a specific date
- `after()` - Date must be after a specific date
- `min_age()` - Age verification from date of birth

**When to use:**
- Event scheduling (booking dates, deadlines)
- Age verification (birth dates)
- Historical data validation
- Temporal constraints

**Dependencies:** `chrono = "0.4"`

#### `serde` - Validate on Deserialize

Enables `ValidateOnDeserialize` derive macro:

```rust
use serde::Deserialize;
use domainstack::ValidateOnDeserialize;

#[derive(Deserialize, ValidateOnDeserialize)]
struct User {
    #[validate(email)]
    email: String,

    #[validate(range(min = 18, max = 120))]
    age: u8,
}

// Validation happens automatically during deserialization
let user: User = serde_json::from_str(json)?;  // Validates!
```

**Adds:**
- `#[derive(ValidateOnDeserialize)]` macro
- Automatic validation during `serde` deserialization

**When to use:**
- API request handlers
- Configuration file parsing
- Any JSON/YAML/TOML deserialization

**Dependencies:** `serde = "1.0"`

## Companion Crates

### domainstack-schema - OpenAPI Generation

Generate OpenAPI 3.0 schemas from validation rules:

```toml
[dependencies]
domainstack = { version = "1.0", features = ["derive"] }
domainstack-schema = "1.0"
```

**Usage:**

```rust
use domainstack_schema::ToSchema;

#[derive(ToSchema)]
struct User {
    #[validate(email, max_len = 255)]
    email: String,

    #[validate(range(min = 18, max = 120))]
    age: u8,
}

let schema = User::schema();
// Generates OpenAPI schema with validation constraints
```

**Features:**
- Automatic schema generation from `#[validate(...)]` attributes
- Rule → OpenAPI constraint mapping
- Nested type support
- Collection and optional field handling

**See:** [OPENAPI_SCHEMA.md](OPENAPI_SCHEMA.md) for complete documentation

### domainstack-envelope - HTTP Error Envelopes

RFC 9457-compliant HTTP error responses:

```toml
[dependencies]
domainstack = { version = "1.0", features = ["derive"] }
domainstack-envelope = "1.0"
```

**Usage:**

```rust
use domainstack_envelope::IntoEnvelopeError;

async fn create_user(Json(dto): Json<UserDto>) -> Result<Json<User>, ErrorResponse> {
    let user = User::try_from(dto)
        .map_err(|e| e.into_envelope_error())?;  // Convert to RFC 9457 format
    Ok(Json(user))
}
```

**Produces:**

```json
{
  "code": "VALIDATION",
  "status": 400,
  "message": "Validation failed with 2 errors",
  "retryable": false,
  "details": {
    "fields": {
      "email": [{"code": "invalid_email", "message": "Invalid email format"}],
      "age": [{"code": "out_of_range", "message": "Must be between 18 and 120"}]
    }
  }
}
```

**See:** [HTTP_INTEGRATION.md](HTTP_INTEGRATION.md) for complete documentation

### domainstack-axum - Axum Integration

Axum 0.7+ extractors with automatic validation:

```toml
[dependencies]
domainstack = { version = "1.0", features = ["derive"] }
domainstack-axum = "1.0"
```

**Usage:**

```rust
use domainstack_axum::{DomainJson, ErrorResponse};

async fn create_user(
    DomainJson(dto, user): DomainJson<UserDto, User>
) -> Result<Json<User>, ErrorResponse> {
    // `user` is already validated via TryFrom<UserDto>
    Ok(Json(user))
}
```

**Features:**
- `DomainJson<Dto, Domain>` - Deserialize + validate + convert
- `ValidatedJson<T>` - Simple validation extractor
- Automatic error responses with field-level errors

**Requirements:** `axum = "0.7"`

**See:** [HTTP_INTEGRATION.md](HTTP_INTEGRATION.md) for complete documentation

### domainstack-actix - Actix-web Integration

Actix-web 4.x extractors with automatic validation:

```toml
[dependencies]
domainstack = { version = "1.0", features = ["derive"] }
domainstack-actix = "1.0"
```

**Usage:**

```rust
use domainstack_actix::{DomainJson, ErrorResponse};
use actix_web::{post, web::Json};

#[post("/users")]
async fn create_user(
    DomainJson(dto, user): DomainJson<UserDto, User>
) -> Result<Json<User>, ErrorResponse> {
    // `user` is already validated
    Ok(Json(user))
}
```

**Requirements:** `actix-web = "4"`

**See:** [HTTP_INTEGRATION.md](HTTP_INTEGRATION.md) for complete documentation

### domainstack-rocket - Rocket Integration

Rocket 0.5+ data guards with automatic validation:

```toml
[dependencies]
domainstack = { version = "1.0", features = ["derive"] }
domainstack-rocket = "1.0"
```

**Usage:**

```rust
use domainstack_rocket::{DomainJson, ErrorResponse};
use rocket::post;

#[post("/users", data = "<input>")]
async fn create_user(
    input: DomainJson<UserDto, User>
) -> Result<Json<User>, ErrorResponse> {
    // input.1 is the validated User
    Ok(Json(input.1))
}
```

**Requirements:** `rocket = "0.5"`

**See:** [HTTP_INTEGRATION.md](HTTP_INTEGRATION.md) for complete documentation

## Common Configurations

### Basic Web API

Most common setup for REST APIs:

```toml
[dependencies]
domainstack = { version = "1.0", features = ["derive", "regex"] }
domainstack-axum = "1.0"
```

**Use for:**
- REST APIs with JSON validation
- Email/URL validation
- Field-level error responses

### Full-Stack API with OpenAPI

Generate OpenAPI schemas + validation:

```toml
[dependencies]
domainstack = { version = "1.0", features = ["derive", "regex"] }
domainstack-schema = "1.0"
domainstack-axum = "1.0"
```

**Use for:**
- APIs with auto-generated OpenAPI docs
- Frontend integration with typed schemas
- Swagger UI / Redoc documentation

### API with Async Validation

Database uniqueness checks and external API validation:

```toml
[dependencies]
domainstack = { version = "1.0", features = ["derive", "regex", "async"] }
domainstack-axum = "1.0"
sqlx = { version = "0.7", features = ["postgres", "runtime-tokio"] }
```

**Use for:**
- User registration (email/username uniqueness)
- External API validation (VAT numbers, postal codes)
- Rate limiting checks

### Full-Featured Setup

All features enabled:

```toml
[dependencies]
domainstack = { version = "1.0", features = ["derive", "regex", "async", "chrono", "serde"] }
domainstack-schema = "1.0"
domainstack-envelope = "1.0"
domainstack-axum = "1.0"
```

**Use for:**
- Complex domain applications
- Multi-tenant systems
- Event scheduling with date validation
- Comprehensive API validation

### Minimal Setup

Core validation only (no derive macro):

```toml
[dependencies]
domainstack = "1.0"
```

**Use for:**
- Libraries (avoid macro dependencies)
- Manual `impl Validate` only
- Embedded systems (minimal dependencies)

## Version Compatibility

### domainstack

| Version | Rust | Features |
|---------|------|----------|
| 1.0.x   | 1.70+ | derive, regex, async, chrono, serde |

### Framework Adapters

| Crate | Framework Version | domainstack Version |
|-------|-------------------|---------------------|
| domainstack-axum | Axum 0.7+ | 1.0+ |
| domainstack-actix | Actix-web 4.x | 1.0+ |
| domainstack-rocket | Rocket 0.5+ | 1.0+ |

### Companion Crates

| Crate | domainstack Version |
|-------|---------------------|
| domainstack-schema | 1.0+ |
| domainstack-envelope | 1.0+ |

## See Also

- **[Quick Start](../README.md#quick-start)** - Get started in 5 minutes
- **[Core Concepts](CORE_CONCEPTS.md)** - Understanding domainstack fundamentals
- **[Derive Macro](DERIVE_MACRO.md)** - Using `#[derive(Validate)]`
- **[HTTP Integration](HTTP_INTEGRATION.md)** - Framework adapters guide
- **[OpenAPI Schema](OPENAPI_SCHEMA.md)** - Schema generation guide

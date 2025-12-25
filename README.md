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

**domainstack turns untrusted input into valid-by-construction domain objects, and returns stable, field-level errors your APIs and UIs can depend on.**

It's built for the boundary you actually live at:

**HTTP/JSON/etc. → DTOs → Domain (validated) → Business logic**

Most validation crates ask: **"Is this DTO valid?"**
domainstack asks: **"How do I safely construct domain models from untrusted input—and report failures with a consistent error contract?"**

### Two validation gates

**Gate 1: Serde** (decode + shape) — JSON → DTO. Fails on invalid JSON, type mismatches, missing required fields.

**Gate 2: Domain** (semantic validation) — DTO → Domain with business rules. Produces structured field-level errors your APIs depend on.

After domain validation succeeds, you can optionally run **async/context validation** (DB/API checks like uniqueness, rate limits, authorization) as a second phase.

## What that gives you

- **Domain-first modeling**: make invalid states difficult (or impossible) to represent
- **Composable rule algebra**: reusable rules with `.and()`, `.or()`, `.when()`
- **Structured error paths**: `rooms[0].adults`, `guest.email.value` (UI-friendly)
- **Async validation with context**: DB/API checks like uniqueness, rate limits
- **Cross-field validation**: invariants like password confirmation, date ranges
- **Type-state tracking**: phantom types to enforce "validated" at compile time
- **Schema + client parity**: generate OpenAPI (and TypeScript/Zod via CLI) from the same Rust rules
- **Framework adapters**: one-line boundary extraction (Axum / Actix / Rocket)
- **Lean core**: zero-deps base, opt-in features for regex / async / chrono / serde

### Why domainstack over validator/garde/etc.?

validator and garde focus on "Is this struct valid?"
domainstack focuses on DTO → Domain conversion with field-level paths designed for APIs, rules as values, optional async validation with context, and adapters that map validation errors into structured HTTP responses. If you want valid-by-construction domain types with errors that map cleanly to forms and clients, domainstack is purpose-built for that.

## Table of Contents

- [Quick Start](#quick-start)
- [Mental Model: DTOs → Domain → Business Logic](#mental-model-dtos--domain--business-logic)
- [How domainstack is Different](#how-domainstack-is-different)
- [Core Features](#core-features)
- [Installation](#installation)
- [Examples](#examples)
- [Documentation](#documentation)

## Quick Start

**Dependencies** (add to `Cargo.toml`):

```toml
[dependencies]
domainstack = { version = "1.0", features = ["derive", "regex", "chrono"] }
domainstack-derive = "1.0"
domainstack-axum = "1.0"  # Or domainstack-actix if using Actix-web
serde = { version = "1", features = ["derive"] }
chrono = "0.4"
axum = "0.7"
```

**Complete example** (with all imports):

```rust
use axum::Json;
use chrono::NaiveDate;
use domainstack::prelude::*;
use domainstack_axum::{DomainJson, ErrorResponse};
use domainstack_derive::{ToSchema, Validate};
use serde::Deserialize;

// DTO from HTTP/JSON (untrusted input)
#[derive(Deserialize)]
struct BookingDto {
    guest_email: String,
    check_in: String,
    check_out: String,
    rooms: Vec<RoomDto>,
}

#[derive(Deserialize)]
struct RoomDto {
    adults: u8,
    children: u8,
}

// Domain models with validation rules (invalid states impossible)
#[derive(Validate, ToSchema)]
#[validate(
    check = "self.check_in < self.check_out",
    message = "Check-out must be after check-in"
)]
struct Booking {
    #[validate(email, max_len = 255)]
    guest_email: String,

    check_in: NaiveDate,
    check_out: NaiveDate,

    #[validate(min_items = 1, max_items = 5)]
    #[validate(each(nested))]
    rooms: Vec<Room>,
}

#[derive(Validate, ToSchema)]
struct Room {
    #[validate(range(min = 1, max = 4))]
    adults: u8,

    #[validate(range(min = 0, max = 3))]
    children: u8,
}

// TryFrom: DTO → Domain conversion with validation
impl TryFrom<BookingDto> for Booking {
    type Error = ValidationError;

    fn try_from(dto: BookingDto) -> Result<Self, Self::Error> {
        let booking = Self {
            guest_email: dto.guest_email,
            check_in: NaiveDate::parse_from_str(&dto.check_in, "%Y-%m-%d")
                .map_err(|_| ValidationError::single("check_in", "invalid_date", "Invalid date format"))?,
            check_out: NaiveDate::parse_from_str(&dto.check_out, "%Y-%m-%d")
                .map_err(|_| ValidationError::single("check_out", "invalid_date", "Invalid date format"))?,
            rooms: dto.rooms.into_iter().map(|r| Room {
                adults: r.adults,
                children: r.children,
            }).collect(),
        };

        booking.validate()?;  // Validates all fields + cross-field rules!
        Ok(booking)
    }
}

// Axum handler: one-line extraction with automatic validation
type BookingJson = DomainJson<Booking, BookingDto>;

async fn create_booking(
    BookingJson { domain: booking, .. }: BookingJson
) -> Result<Json<Booking>, ErrorResponse> {
    // booking is GUARANTEED valid here - use with confidence!
    save_to_db(booking).await
}
```

**On validation failure, automatic structured errors:**

```json
{
  "status": 400,
  "message": "Validation failed with 3 errors",
  "details": {
    "fields": {
      "guest_email": [
        {"code": "invalid_email", "message": "Invalid email format"}
      ],
      "rooms[0].adults": [
        {"code": "out_of_range", "message": "Must be between 1 and 4"}
      ],
      "rooms[1].children": [
        {"code": "out_of_range", "message": "Must be between 0 and 3"}
      ]
    }
  }
}
```

**Auto-generated TypeScript/Zod from the same Rust code:**

```typescript
// Zero duplication - generated from Rust validation rules!
export const bookingSchema = z.object({
  guest_email: z.string().email().max(255),
  check_in: z.string(),
  check_out: z.string(),
  rooms: z.array(z.object({
    adults: z.number().min(1).max(4),
    children: z.number().min(0).max(3)
  })).min(1).max(5)
}).refine(
  (data) => data.check_in < data.check_out,
  { message: "Check-out must be after check-in" }
);
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
use domainstack::Validate;

pub struct Email(String);

impl Email {
    pub fn new(raw: String) -> Result<Self, ValidationError> {
        validate("email", raw.as_str(), &rules::email().and(rules::max_len(255)))?;
        Ok(Self(raw))
    }
}

// Derive Validate for automatic validation
#[derive(Validate)]
pub struct BookingRequest {
    #[validate(length(min = 1, max = 50))]
    name: String,      // Private!

    #[validate(nested)]
    email: Email,

    #[validate(range(min = 1, max = 10))]
    guests: u8,
}

impl TryFrom<BookingDto> for BookingRequest {
    type Error = ValidationError;

    fn try_from(dto: BookingDto) -> Result<Self, Self::Error> {
        let email = Email::new(dto.email).map_err(|e| e.prefixed("email"))?;

        let booking = Self {
            name: dto.name,
            email,
            guests: dto.guests,
        };

        booking.validate()?;  // One line validates all fields!
        Ok(booking)
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

```toml
[dependencies]
# Core validation + derive macro (recommended)
domainstack = { version = "1.0", features = ["derive"] }
```

### Feature Flags

| Feature | Adds | Use When |
|---------|------|----------|
| `derive` | `#[derive(Validate)]` macro | Declarative validation (recommended) |
| `regex` | Email, URL, pattern matching | Web APIs, user input |
| `async` | Database/API validation | Uniqueness checks, external validation |
| `chrono` | Date/time rules | Temporal constraints, age verification |
| `serde` | `ValidateOnDeserialize` | Validate during JSON/YAML parsing |

### Common Combinations

```toml
# Web API (most common)
domainstack = { version = "1.0", features = ["derive", "regex"] }

# API + async validation
domainstack = { version = "1.0", features = ["derive", "regex", "async"] }

# Full-stack with OpenAPI + framework adapter
domainstack = { version = "1.0", features = ["derive", "regex", "async"] }
domainstack-schema = "1.0"      # OpenAPI generation
domainstack-axum = "1.0"        # Axum integration
```

### Optional Companion Crates

```toml
domainstack-schema = "1.0"    # OpenAPI 3.0 schema generation
domainstack-envelope = "1.0"  # HTTP error envelopes (RFC 9457)
domainstack-axum = "1.0"      # Axum adapter (0.7+)
domainstack-actix = "1.0"     # Actix-web adapter (4.x)
domainstack-rocket = "1.0"    # Rocket adapter (0.5+)
```

## Key Features

### Core Capabilities

- **37 Validation Rules** - String, numeric, collection, and date/time validation out of the box
- **Composable Rules** - Combine with `.and()`, `.or()`, `.when()` for complex logic
- **Nested Validation** - Automatic path tracking for deeply nested structures
- **Collection Validation** - Array indices in error paths (`items[0].field`)
- **Builder Customization** - Customize error codes, messages, and metadata

### Advanced Features

#### Serde Integration - Validate on Deserialize ⚡

**NEW!** Automatically validate during JSON/YAML deserialization with a single derive:

```rust
use domainstack_derive::ValidateOnDeserialize;

#[derive(ValidateOnDeserialize, Debug)]
struct User {
    #[validate(email)]
    #[validate(max_len = 255)]
    email: String,

    #[validate(range(min = 18, max = 120))]
    age: u8,
}

// Single step: deserialize + validate automatically
let user: User = serde_json::from_str(json)?;
// ↑ If this succeeds, user is guaranteed valid!
```

**Benefits:**
- ✅ **Single step** - No separate `.validate()` call needed
- ✅ **Better errors** - "age must be between 18 and 120" vs "expected u8"
- ✅ **Type safety** - If you have `User`, it's guaranteed valid
- ✅ **Serde compatible** - Works with `#[serde(rename)]`, `#[serde(default)]`, etc.

**Use cases:** API request parsing, configuration file loading, message queue consumers, CLI argument validation.

**Example with serde attributes:**
```rust
#[derive(ValidateOnDeserialize)]
#[serde(rename_all = "camelCase")]
struct Config {
    #[validate(range(min = 1024, max = 65535))]
    server_port: u16,

    #[serde(default = "default_workers")]
    #[validate(range(min = 1, max = 128))]
    worker_threads: u8,
}
```

See [`examples/serde_validation.rs`](https://github.com/blackwell-systems/domainstack/blob/main/domainstack/domainstack/examples/serde_validation.rs) for complete examples.

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

### Validation Rules

**37 built-in rules** across string (17), numeric (8), collection (4), date/time (5), and choice (3) validation—with composable `.and()`, `.or()`, `.when()` combinators. All rules work with `each(rule)` for collection validation with array indices in errors (`tags[0]`, `emails[1]`).

📖 **[Complete Rules Reference](./domainstack/domainstack/docs/RULES.md)** - Detailed documentation with examples for all 37 rules.

## Examples

### Derive Macro (Recommended)

Most validation is declarative with `#[derive(Validate)]`:

```rust
#[derive(Debug, Validate)]
struct User {
    #[validate(length(min = 3, max = 20))]
    #[validate(alphanumeric)]
    username: String,

    #[validate(email)]
    #[validate(max_len = 255)]
    email: String,

    #[validate(range(min = 18, max = 120))]
    age: u8,
}

// Validate all fields at once
let user = User { username, email, age };
user.validate()?;  // ✓ Validates all constraints
```

### Nested Validation

Compose complex domain models with automatic path tracking:

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

// Errors include full paths: "rooms[0].adults", "guest.email.value"
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

### Manual Validation (For Primitives & Fine-Grained Control)

For newtype wrappers or custom logic, use manual validation:

```rust
use domainstack::prelude::*;

struct Username(String);

impl Username {
    pub fn new(raw: String) -> Result<Self, ValidationError> {
        let rule = rules::min_len(3)
            .and(rules::max_len(20))
            .and(rules::alphanumeric());
        validate("username", raw.as_str(), &rule)?;
        Ok(Self(raw))
    }
}
```

**When to use manual:**
- Newtype wrappers (tuple structs like `Email(String)`)
- Custom validation logic beyond declarative rules
- Fine-grained error message control

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

**8 publishable crates** for modular adoption:

| Category | Crates |
|----------|--------|
| **Core** | `domainstack`, `domainstack-derive`, `domainstack-schema`, `domainstack-envelope` |
| **Web Frameworks** | `domainstack-axum`, `domainstack-actix`, `domainstack-rocket`, `domainstack-http` |

**4 example crates** (repository only): `domainstack-examples`, `examples-axum`, `examples-actix`, `examples-rocket`

📦 **[Complete Crate List](./domainstack/README.md#crates)** - Detailed descriptions and links

## 📚 Documentation

### Multi-README Structure

This project has **multiple README files** for different audiences:

1. **[README.md](./README.md)** (this file) - GitHub visitors
2. **[domainstack/README.md](./domainstack/README.md)** - Cargo/crates.io users
3. **Individual crate READMEs** - Library implementers

### Additional Documentation

- **[API Guide](./domainstack/domainstack/docs/api-guide.md)** - Complete API documentation
- **[Rules Reference](./domainstack/domainstack/docs/RULES.md)** - All validation rules
- **[Architecture](./domainstack/domainstack/docs/architecture.md)** - System design and data flow
- **[OpenAPI Schema Derivation](./domainstack/domainstack/docs/SCHEMA_DERIVATION.md)** - OpenAPI 3.0 schema generation guide
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

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

**Gate 2: Domain** (construct + validate) — DTO → Domain. Produces structured field-level errors your APIs depend on.

After domain validation succeeds, you can optionally run **async/context validation** (DB/API checks like uniqueness, rate limits, authorization) as a post-validation phase.

## Why domainstack over validator/garde/etc.?

**validator and garde** focus on: *"Is this struct valid?"*

**domainstack** focuses on:
- DTO → Domain conversion with field-level error paths
- Rules as composable values (`.and()`, `.or()`, `.when()`)
- Async validation with context (DB checks, API calls)
- Framework adapters that map errors to structured HTTP responses

**If you want valid-by-construction domain types with errors that map cleanly to forms and clients, domainstack is purpose-built for that.**

## What that gives you

- **Domain-first modeling**: make invalid states difficult (or impossible) to represent
- **Composable rule algebra**: reusable rules with `.and()`, `.or()`, `.when()`
- **Structured error paths**: `rooms[0].adults`, `guest.email.value` (UI-friendly)
- **Cross-field validation**: invariants like password confirmation, date ranges
- **Type-state tracking**: phantom types to enforce "validated" at compile time
- **Schema + client parity**: generate OpenAPI (and TypeScript/Zod via CLI) from the same Rust rules
- **Framework adapters**: one-line boundary extraction (Axum / Actix / Rocket)
- **Lean core**: zero-deps base, opt-in features for regex / async / chrono / serde

## Table of Contents

- [Quick Start](#quick-start)
- [Installation](#installation)
- [Key Features](#key-features)
- [Examples](#examples)
- [Documentation](#documentation)
- [Crates](#-crates)

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

## Installation

```toml
[dependencies]
domainstack = { version = "1.0", features = ["derive", "regex"] }
domainstack-axum = "1.0"  # or domainstack-actix, domainstack-rocket

# Optional
domainstack-schema = "1.0"  # OpenAPI generation
```

**For complete installation guide, feature flags, and companion crates, see [INSTALLATION.md](./domainstack/domainstack/docs/INSTALLATION.md)**

## Key Features

- **37 Validation Rules** - String, numeric, collection, and date/time validation → [RULES.md](./domainstack/domainstack/docs/RULES.md)
- **Derive Macros** - `#[derive(Validate)]` for declarative validation → [DERIVE_MACRO.md](./domainstack/domainstack/docs/DERIVE_MACRO.md)
- **Composable Rules** - `.and()`, `.or()`, `.when()` combinators for complex logic
- **Nested Validation** - Automatic path tracking for deeply nested structures
- **Collection Validation** - Array indices in error paths (`items[0].field`)
- **Serde Integration** - Validate during deserialization → [SERDE_INTEGRATION.md](./domainstack/domainstack/docs/SERDE_INTEGRATION.md)
- **Async Validation** - Database/API checks with `AsyncValidate` → [ADVANCED_PATTERNS.md](./domainstack/domainstack/docs/ADVANCED_PATTERNS.md#async-validation)
- **Cross-Field Validation** - Password confirmation, date ranges → [DERIVE_MACRO.md](./domainstack/domainstack/docs/DERIVE_MACRO.md#cross-field-validation)
- **Type-State Validation** - Compile-time guarantees with phantom types → [ADVANCED_PATTERNS.md](./domainstack/domainstack/docs/ADVANCED_PATTERNS.md#type-state-validation)
- **OpenAPI Schema Generation** - Auto-generate schemas from rules → [OPENAPI_SCHEMA.md](./domainstack/domainstack/docs/OPENAPI_SCHEMA.md)
- **Framework Adapters** - Axum, Actix-web, Rocket extractors → [HTTP_INTEGRATION.md](./domainstack/domainstack/docs/HTTP_INTEGRATION.md)

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

See `examples/` folder for more:
- Nested validation with path tracking
- Collection item validation
- Manual validation for newtypes
- Cross-field validation
- Async validation
- Type-state patterns
- OpenAPI schema generation
- Framework integration examples

## Framework Adapters

Automatic DTO→Domain conversion with one-line extractors for **Axum**, **Actix-web**, and **Rocket**. Validation errors are automatically converted to structured JSON responses.

**📖 [HTTP Integration Guide](./domainstack/domainstack/docs/HTTP_INTEGRATION.md)**

## Running Examples

See **[examples/README.md](./examples/README.md)** for instructions on running all examples.

## 📦 Crates

**8 publishable crates** for modular adoption:

| Category | Crates |
|----------|--------|
| **Core** | `domainstack`, `domainstack-derive`, `domainstack-schema`, `domainstack-envelope` |
| **Web Framework Integrations** | `domainstack-axum`, `domainstack-actix`, `domainstack-rocket`, `domainstack-http` |

**4 example crates** (repository only): `domainstack-examples`, `examples-axum`, `examples-actix`, `examples-rocket`

📦 **[Complete Crate List](./domainstack/README.md#crates)** - Detailed descriptions and links

## 📚 Documentation

### Guides

**Foundation:**
- **[Core Concepts](./domainstack/domainstack/docs/CORE_CONCEPTS.md)** - Valid-by-construction types, structured error paths, composable rules
- **[Domain Modeling Patterns](./domainstack/domainstack/docs/PATTERNS.md)** - DTO→Domain conversion, smart constructors, private fields
- **[Manual Validation](./domainstack/domainstack/docs/MANUAL_VALIDATION.md)** - When and how to implement `Validate` trait manually
- **[Error Handling](./domainstack/domainstack/docs/ERROR_HANDLING.md)** - Working with `ValidationError`, violations, i18n

**Features:**
- **[Derive Macro](./domainstack/domainstack/docs/DERIVE_MACRO.md)** - Complete `#[derive(Validate)]` guide
- **[Validation Rules](./domainstack/domainstack/docs/RULES.md)** - All 37 built-in validation rules
- **[Advanced Patterns](./domainstack/domainstack/docs/ADVANCED_PATTERNS.md)** - Async validation, type-state, context-dependent validation

**Integration:**
- **[Installation Guide](./domainstack/domainstack/docs/INSTALLATION.md)** - Feature flags, companion crates, version compatibility
- **[Serde Integration](./domainstack/domainstack/docs/SERDE_INTEGRATION.md)** - Validate on deserialize
- **[HTTP Integration](./domainstack/domainstack/docs/HTTP_INTEGRATION.md)** - Axum, Actix-web, Rocket adapters
- **[OpenAPI Schema Generation](./domainstack/domainstack/docs/OPENAPI_SCHEMA.md)** - Auto-generate schemas from rules
- **[CLI Guide](./domainstack/domainstack/docs/CLI_GUIDE.md)** - Generate TypeScript/Zod schemas

### Multi-README Structure

This project has **multiple README files** for different audiences:

1. **[README.md](./README.md)** (this file) - GitHub visitors
2. **[domainstack/README.md](./domainstack/README.md)** - Cargo/crates.io users
3. **Individual crate READMEs** - Library implementers

### Additional Documentation

- **[API Guide](./domainstack/domainstack/docs/api-guide.md)** - Complete API documentation
- **[Architecture](./domainstack/domainstack/docs/architecture.md)** - System design and data flow
- **[API Documentation](https://docs.rs/domainstack)** - Generated API reference
- **[Examples](./domainstack/domainstack-examples/)** - 9 runnable examples
- **[Publishing Guide](./PUBLISHING.md)** - How to publish to crates.io
- **[Coverage Guide](./COVERAGE.md)** - Running coverage locally

## License

Apache 2.0

## Author

Dayna Blackwell - [blackwellsystems@protonmail.com](mailto:blackwellsystems@protonmail.com)

## Contributing

This is an early-stage project. Issues and pull requests are welcome!

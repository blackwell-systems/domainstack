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
- **Serde integration** - Validate automatically during JSON/YAML deserialization with `#[derive(ValidateOnDeserialize)]`

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

// Or use ValidateOnDeserialize to validate automatically during JSON parsing:
// #[derive(ValidateOnDeserialize)] - see "Serde Integration" section below

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

```
HTTP Request → DTO (untrusted) → Domain (validated) → Business Logic
```

domainstack enforces validation at boundaries through smart constructors and `TryFrom` implementations:

```rust
// DTO - public fields for deserialization
#[derive(Deserialize)]
pub struct UserDto { pub email: String, pub age: u8 }

// Domain - private fields, guaranteed valid
#[derive(Validate)]
pub struct User {
    #[validate(email)] email: String,
    #[validate(range(min = 18, max = 120))] age: u8,
}

impl TryFrom<UserDto> for User { /* validation enforced here */ }
```

📘 **[Complete Patterns Guide](./domainstack/domainstack/docs/PATTERNS.md)** - Valid-by-construction, smart constructors, boundary enforcement

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

## HTTP Integration

domainstack provides framework adapters for clean DTO→Domain extraction:

```rust
use domainstack_axum::{DomainJson, ErrorResponse};

async fn create_user(
    DomainJson(request, user): DomainJson<UserDto, User>
) -> Result<Json<User>, ErrorResponse> {
    // `user` is guaranteed valid - automatic validation!
    Ok(Json(save_user(user).await?))
}
```

**Supported frameworks:** Axum, Actix-web, Rocket
**Auto-generated errors:** Structured field-level 400 responses
**Envelope integration:** RFC 9457 Problem Details support

📦 **[Framework Adapters](./domainstack/domainstack-http/)** - Axum, Actix, Rocket integration guides

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

### 37 Built-in Validation Rules

| Category | Rules |
|----------|-------|
| **String (17)** | `email`, `url`, `min_len`, `max_len`, `length`, `non_empty`, `non_blank`, `alphanumeric`, `alpha_only`, `numeric_string`, `ascii`, `no_whitespace`, `contains`, `starts_with`, `ends_with`, `matches_regex`, `len_chars` |
| **Numeric (8)** | `range`, `min`, `max`, `positive`, `negative`, `non_zero`, `multiple_of`, `finite` |
| **Choice (3)** | `equals`, `not_equals`, `one_of` |
| **Collection (4)** | `min_items`, `max_items`, `unique`, `non_empty_items` |
| **Date/Time (5)** | `past`, `future`, `before`, `after`, `age_range` (requires `chrono` feature) |

**Combinators:** `.and()`, `.or()`, `.when()`, `.code()`, `.message()`, `.meta()`

**All rules work with `each(rule)` for collection validation.** Errors include array indices (`tags[0]`, `emails[1]`).

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

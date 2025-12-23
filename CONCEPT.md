# domain-model: Rust Domain Validation Framework

**Status:** Concept / Pre-Development  
**Type:** Rust Workspace (4 crates)  
**MSRV:** 1.65+ (GATs required)  
**License:** MIT (tentative)  
**Target Audience:** Service-oriented Rust applications (Axum, Actix, etc.)

---

## Problem Statement

Rust services need a unified approach to domain modeling and validation that:

1. **Enforces valid-by-construction types** (invalid states are unrepresentable)
2. **Provides structured, field-level error reporting** for APIs
3. **Supports both sync and async validation** (format checks + DB uniqueness checks)
4. **Integrates with the Rust HTTP ecosystem** (serde, axum, error_envelope)
5. **Uses composable validation rules** (not just attributes)

### Current Ecosystem Gaps

| Crate | Strength | Weakness |
|-------|----------|----------|
| `validator` | Attribute-based validation | DTO-first, no domain modeling |
| `garde` | Good nested validation | Still DTO-focused, no async |
| `nutype` | Valid-by-construction newtypes | Only primitives, no aggregates |
| `serde_valid` | Validate-on-deserialize | Couples validation to serialization |

**None provide:**
- Domain-first design (DTO → Domain boundary)
- Composable rule algebra (`and`, `or`, `when`)
- Async validation with context
- First-class integration with API error envelopes

---

## Quick Start

**Want to see what this looks like in practice?**

```rust
use domain_model::prelude::*;

// 1. Define domain primitive
struct Email(String);
impl Email {
    pub fn new(s: String) -> Result<Self, ValidationError> {
        validate("email", &s.as_str(), rules::email())?;
        Ok(Self(s))
    }
}

// 2. Define aggregate with validation
#[derive(Validate)]
struct User {
    #[validate(nested)]
    email: Email,
    
    #[validate(range(min = 1, max = 120))]
    age: u8,
}

// 3. Use in HTTP handler
async fn create_user(
    ValidatedJson(user): ValidatedJson<User>
) -> Result<Json<UserId>, Error> {
    // user is guaranteed valid
    let id = db::insert_user(user).await?;
    Ok(Json(id))
}
```

**Benefits:**
- Invalid states can't exist (Email guarantees valid email)
- Structured field errors for APIs
- DTO → Domain boundary is explicit
- Works with your existing error handling

**Jump to:** [Full Example](#real-world-example-hotel-booking) | [Roadmap](#development-roadmap) | [vs Alternatives](#comparison-with-existing-solutions)

---

## Design Philosophy

### 1. Valid-by-Construction Domain Types

```rust
// ❌ Current: Any string accepted
#[derive(Deserialize)]
struct Guest {
    email: String,  // Could be "not-an-email"
}

// ✓ Domain model: Invalid states prevented
struct Guest {
    email: Email,  // Guaranteed valid email
}

impl Guest {
    pub fn new(email: Email) -> Result<Self, ValidationError> {
        Ok(Self { email })
    }
}
```

### 2. DTO → Domain Boundary

```rust
// HTTP boundary: DTOs for deserialization
#[derive(Deserialize)]
struct GuestDto {
    email: String,
}

// Domain boundary: Smart constructors
impl TryFrom<GuestDto> for Guest {
    type Error = ValidationError;
    
    fn try_from(dto: GuestDto) -> Result<Self, Self::Error> {
        Guest::new(Email::new(dto.email)?)
    }
}
```

**Why:** Keeps serde concerns separate from domain logic.

### 3. Structured Error Paths

```rust
// Single error type, multiple outputs
pub struct ValidationError {
    pub violations: Vec<Violation>,
}

pub struct Violation {
    pub path: Path,              // "guest.email", "rooms[0].adults"
    pub code: &'static str,       // Stable identifier: "invalid_email"
    pub message: String,          // Human message
    pub meta: Meta,               // Optional: {min: 3, max: 50}
}
```

**Result:**
```json
{
  "code": "VALIDATION_FAILED",
  "details": {
    "fields": {
      "guest.email": ["Invalid email format"],
      "rooms[0].adults": ["Must be between 1 and 6"]
    }
  }
}
```

### 4. Composable Validation Rules

```rust
// Rules are first-class values
let email_rule = rules::email()
    .and(rules::max_len(255))
    .map_path("contact");

// Reuse across types
validate("email", &value, email_rule)?;
```

**Why:** More powerful than attributes alone—rules can be stored, composed, tested independently.

---

## Public API Design

### Core Traits

```rust
/// Synchronous validation (format, range, length)
pub trait Validate {
    fn validate(&self) -> Result<(), ValidationError>;
}

/// Asynchronous validation (DB checks, external APIs)
pub trait AsyncValidate<Ctx> {
    type Fut<'a>: Future<Output = Result<(), ValidationError>> + Send + 'a
    where Self: 'a, Ctx: 'a;
    
    fn validate_async<'a>(&'a self, ctx: &'a Ctx) -> Self::Fut<'a>;
}
```

### Core Types

```rust
/// Structured path to a field
pub struct Path(Vec<PathSegment>);

pub enum PathSegment {
    Field(&'static str),  // .email
    Index(usize),         // [0]
}

impl Path {
    /// Create an empty root path
    pub fn root() -> Self;
    
    /// Append a field segment
    pub fn field(self, name: &'static str) -> Self;
    
    /// Append an index segment
    pub fn index(self, idx: usize) -> Self;
    
    /// Parse a path string like "rooms[0].adults"
    /// Returns Path::root() if parsing fails
    pub fn parse(s: &str) -> Self;
}

impl core::fmt::Display for Path {
    // Formats as: "field.nested[0].item"
}

/// Single validation violation
pub struct Violation {
    pub path: Path,
    pub code: &'static str,
    pub message: String,
    pub meta: Meta,
}

/// Collection of violations
pub struct ValidationError {
    pub violations: Vec<Violation>,
}

impl ValidationError {
    pub fn is_empty(&self) -> bool;
    
    /// Create a ValidationError with a single violation
    pub fn single(
        path: impl Into<Path>,
        code: &'static str,
        message: impl Into<String>,
    ) -> Self;
    
    pub fn push(
        &mut self,
        path: impl Into<Path>,
        code: &'static str,
        message: impl Into<String>,
    );
    
    pub fn merge_prefixed(&mut self, prefix: impl Into<Path>, other: ValidationError);
    
    pub fn field_errors_map(&self) -> BTreeMap<String, Vec<Violation>>;
}
```

### Rule Algebra

```rust
pub struct Rule<T>(Arc<dyn Fn(&T) -> ValidationError + Send + Sync>);

impl<T> Rule<T> {
    pub fn and(self, other: Rule<T>) -> Rule<T>;
    pub fn or(self, other: Rule<T>) -> Rule<T>;
    pub fn not(self, code: &'static str, message: &'static str) -> Rule<T>;
    pub fn map_path(self, prefix: impl Into<Path>) -> Rule<T>;
    pub fn when(self, predicate: impl Fn() -> bool + Send + Sync + 'static) -> Rule<T>;
}
```

### Built-in Rules

```rust
pub mod rules {
    pub fn email() -> Rule<&str>;
    pub fn non_empty() -> Rule<&str>;
    pub fn min_len(n: usize) -> Rule<&str>;
    pub fn max_len(n: usize) -> Rule<&str>;
    pub fn range<T: PartialOrd + Copy>(min: T, max: T) -> Rule<T>;
}
```

### Validation Helper

```rust
pub fn validate<T>(
    path: impl Into<Path>,
    value: &T,
    rule: Rule<T>
) -> Result<(), ValidationError>;
```

---

## Derive Macro: `#[derive(Validate)]`

### Supported Attributes

```rust
#[derive(Validate)]
struct BookingRequest {
    // Length constraints
    #[validate(length(min = 3, max = 50, code = "invalid_length"))]
    name: String,
    
    // Range constraints
    #[validate(range(min = 1, max = 10))]
    guests: u8,
    
    // Nested validation
    #[validate(nested)]
    primary_guest: Guest,
    
    // Collection validation (each element)
    #[validate(length(min = 1, message = "At least one room required"))]
    #[validate(each(nested))]
    rooms: Vec<Room>,
    
    // Custom validator function
    #[validate(custom = "validate_dates")]
    dates: DateRange,
    
    // Conditional validation
    #[validate(when = "requires_payment")]
    #[validate(nested)]
    payment: Option<Payment>,
}

fn requires_payment(req: &BookingRequest) -> bool {
    req.guests > 5
}
```

### Generated Code Pattern

```rust
impl Validate for BookingRequest {
    fn validate(&self) -> Result<(), ValidationError> {
        let mut err = ValidationError::default();
        
        // Length check on name
        if self.name.len() < 3 || self.name.len() > 50 {
            err.push("name", "invalid_length", "Must be 3-50 characters");
        }
        
        // Range check on guests
        if !(1..=10).contains(&self.guests) {
            err.push("guests", "out_of_range", "Must be between 1 and 10");
        }
        
        // Nested validation with path prefixing
        if let Err(e) = self.primary_guest.validate() {
            err.merge_prefixed("primary_guest", e);
        }
        
        // Collection validation with indexed paths
        for (i, room) in self.rooms.iter().enumerate() {
            if let Err(e) = room.validate() {
                err.merge_prefixed(format!("rooms[{}]", i), e);
            }
        }
        
        if err.is_empty() { Ok(()) } else { Err(err) }
    }
}
```

---

## Real-World Example: Hotel Booking

### Domain Primitives

```rust
use domain_model::prelude::*;

/// Email with validation
#[derive(Debug, Clone)]
pub struct Email(String);

impl Email {
    pub fn new(raw: impl Into<String>) -> Result<Self, ValidationError> {
        let raw = raw.into();
        validate("email", &raw, rules::email().and(rules::max_len(255)))?;
        Ok(Self(raw))
    }
    
    pub fn as_str(&self) -> &str { &self.0 }
}

/// Check-in date (cannot be in the past)
#[derive(Debug, Clone)]
pub struct CheckInDate(DateTime<Utc>);

impl CheckInDate {
    pub fn new(date: DateTime<Utc>) -> Result<Self, ValidationError> {
        let today = Utc::now().date_naive();
        if date.date_naive() < today {
            return Err(ValidationError::single(
                "check_in",
                "past_date",
                "Check-in cannot be in the past"
            ));
        }
        Ok(Self(date))
    }
}
```

### Domain Aggregate

```rust
#[derive(Debug, Validate)]
pub struct BookingRequest {
    #[validate(length(min = 1, message = "Name required"))]
    pub name: String,
    
    #[validate(nested)]
    pub email: Email,
    
    #[validate(range(min = 1, max = 10))]
    pub guests: u8,
    
    #[validate(nested)]
    pub check_in: CheckInDate,
    
    #[validate(nested)]
    pub check_out: CheckOutDate,
}

impl BookingRequest {
    pub fn new(
        name: String,
        email: Email,
        guests: u8,
        check_in: CheckInDate,
        check_out: CheckOutDate,
    ) -> Result<Self, ValidationError> {
        let req = Self { name, email, guests, check_in, check_out };
        
        // Derive-generated field validation
        req.validate()?;
        
        // Cross-field validation
        if check_out.0 <= check_in.0 {
            return Err(ValidationError::single(
                "check_out",
                "invalid_date_range",
                "Check-out must be after check-in"
            ));
        }
        
        Ok(req)
    }
}
```

### Async Validation (Uniqueness Checks)

```rust
pub trait BookingChecks: Send + Sync {
    fn email_exists(&self, email: &str) -> impl Future<Output = Result<bool, anyhow::Error>> + Send;
}

impl<C: BookingChecks> AsyncValidate<C> for BookingRequest {
    type Fut<'a> = impl Future<Output = Result<(), ValidationError>> + Send + 'a
    where Self: 'a, C: 'a;
    
    fn validate_async<'a>(&'a self, ctx: &'a C) -> Self::Fut<'a> {
        async move {
            // Sync validation first
            self.validate()?;
            
            // Async check: email uniqueness
            if ctx.email_exists(self.email.as_str()).await? {
                return Err(ValidationError::single(
                    "email",
                    "email_exists",
                    "Email already registered"
                ));
            }
            
            Ok(())
        }
    }
}
```

### HTTP Handler Integration

```rust
use axum::{Json, extract::State};
use error_envelope::Error;

#[derive(Deserialize)]
struct BookingDto {
    name: String,
    email: String,
    guests: u8,
    check_in: String,
    check_out: String,
}

impl TryFrom<BookingDto> for BookingRequest {
    type Error = ValidationError;
    
    fn try_from(dto: BookingDto) -> Result<Self, Self::Error> {
        BookingRequest::new(
            dto.name,
            Email::new(dto.email)?,
            dto.guests,
            CheckInDate::parse(&dto.check_in)?,
            CheckOutDate::parse(&dto.check_out)?,
        )
    }
}

async fn create_booking(
    State(checks): State<Arc<dyn BookingChecks>>,
    ValidatedJson(request): ValidatedJson<BookingRequest>,
) -> Result<Json<BookingConfirmation>, Error> {
    // Sync validation already done by ValidatedJson
    
    // Async validation
    request.validate_async(&*checks)
        .await
        .map_err(|e| Error::validation(e.field_errors_map()))?;
    
    // Domain logic
    let booking = create_booking_internal(request).await?;
    Ok(Json(booking))
}
```

---

## Crate Structure

```
domain-model/
├── domain-model/           # Core crate (no macros)
│   ├── lib.rs
│   ├── path.rs            # Path, PathSegment
│   ├── violation.rs       # Violation, Meta
│   ├── error.rs           # ValidationError
│   ├── rule.rs            # Rule<T> algebra
│   ├── rules/             # Built-in rules
│   │   ├── mod.rs
│   │   ├── string.rs      # email, min_len, max_len
│   │   ├── numeric.rs     # range
│   │   └── optional.rs    # regex, url (feature-gated)
│   └── prelude.rs
│
├── domain-model-derive/    # Proc macros
│   ├── lib.rs
│   └── validate.rs        # #[derive(Validate)]
│
├── domain-model-serde/     # Optional serde integration
│   ├── lib.rs
│   └── validated.rs       # ValidatedJson<T>
│
├── domain-model-error-envelope/  # Optional error_envelope integration
│   └── lib.rs             # impl From<ValidationError> for Error
│
└── examples/
    ├── hotel_booking.rs
    └── user_registration.rs
```

### Feature Flags

```toml
[features]
default = ["std"]
std = []
alloc = []

# Optional rules
regex = ["dep:regex"]
email = ["dep:regex"]
url = ["dep:url"]

# Integration
serde = ["dep:serde", "dep:serde_json"]
axum = ["dep:axum", "serde"]
error_envelope = ["dep:error-envelope"]
```

---

## Development Roadmap

### v0.1 (MVP - 2-3 weeks)
- [ ] Core types: `ValidationError`, `Violation`, `Path` with full API
- [ ] `Rule<T>` with `and`, `or`, `when` combinators
- [ ] Built-in rules: `email`, `min_len`, `max_len`, `range`, `non_empty`
- [ ] `validate()` helper function
- [ ] Manual domain type pattern (no macros yet)
- [ ] Basic tests and examples

**Example (v0.1 manual validation):**
```rust
use domain_model::prelude::*;

// Domain primitive with manual validation
pub struct Email(String);

impl Email {
    pub fn new(raw: String) -> Result<Self, ValidationError> {
        validate("email", &raw.as_str(), rules::email())?;
        Ok(Self(raw))
    }
    
    pub fn as_str(&self) -> &str { &self.0 }
}

// Manual Validate implementation (no derive yet)
impl Validate for Email {
    fn validate(&self) -> Result<(), ValidationError> {
        validate("email", self.0.as_str(), rules::email())
    }
}

// Usage
let email = Email::new("test@example.com".to_string())?;
email.validate()?; // Can re-validate later if needed
```

**Deliverable:** Can manually build validated domain types with composable rules.

### v0.2 (Derive Macro)
- [ ] `#[derive(Validate)]` for structs
- [ ] Attributes: `length`, `range`, `nested`, `each(nested)`, `custom`
- [ ] Path prefixing for nested structs and collections
- [ ] Good compiler error messages
- [ ] Comprehensive macro tests

**Deliverable:** Ergonomic attribute-based validation with zero boilerplate.

### v0.3 (Serde Integration)
- [ ] `domain-model-serde` crate
- [ ] `ValidatedJson<T>` wrapper for Axum
- [ ] DTO → Domain conversion pattern
- [ ] Example: hotel booking API

**Deliverable:** Drop-in HTTP integration for web services.

### v0.4 (Error Envelope Integration)
- [ ] `domain-model-error-envelope` crate
- [ ] `impl From<ValidationError> for error_envelope::Error`
- [ ] Structured field errors in HTTP responses
- [ ] Example: error envelope with field paths

**Deliverable:** Production-ready API error responses.

### v0.5 (Async Validation)
- [ ] `AsyncValidate<Ctx>` trait (GAT-based)
- [ ] Context pattern for DB/API checks
- [ ] Async rule helpers: `validate_each_concurrent`
- [ ] Path prefixing for async violations
- [ ] Example: uniqueness checks

**Deliverable:** Full async validation support for service backends.

### v0.6+ (Future)
- [ ] Conditional validation (`when` attribute)
- [ ] Schema generation (OpenAPI / JSON Schema)
- [ ] More built-in rules (url, uuid, phone, etc.)
- [ ] Performance benchmarks
- [ ] `no_std` support

---

## Success Metrics

### Adoption Indicators
- Used in 5+ production services
- 100+ GitHub stars
- Mentioned in Rust web framework guides
- Integration examples in Axum/Actix docs

### Technical Goals
- Zero-cost abstractions (no runtime overhead vs manual validation)
- Compile times < 5s for derive macro heavy projects
- Error messages as clear as standard library
- 100% test coverage for core + derive

### Community Validation
- Positive feedback from domain-driven design practitioners
- Comparison blog posts vs `validator`/`garde`
- Requests for specific framework integrations

---

## Comparison with Existing Solutions

### vs validator

| Aspect | validator | domain-model |
|--------|-----------|--------------|
| **Philosophy** | DTO validation | Domain modeling |
| **Error paths** | Basic | Fully structured |
| **Async validation** | No | Yes (context-based) |
| **Rule composition** | Attributes only | First-class values |
| **HTTP integration** | Manual | Built-in (optional) |

### vs garde

| Aspect | garde | domain-model |
|--------|-------|--------------|
| **Philosophy** | DTO validation | Domain modeling |
| **Nested errors** | Yes | Yes (better paths) |
| **Async validation** | No | Yes |
| **Composable rules** | Limited | Full algebra |
| **Domain types** | No focus | Primary focus |

### vs nutype

| Aspect | nutype | domain-model |
|--------|--------|--------------|
| **Philosophy** | Newtype primitives | Full aggregates |
| **Scope** | Single values | Structs + collections |
| **Error reporting** | Per-field | Structured paths |
| **Async validation** | No | Yes |
| **Composition** | Limited | Full support |

### vs serde_valid

| Aspect | serde_valid | domain-model |
|--------|-------------|--------------|
| **Philosophy** | Validate-on-deserialize | DTO → Domain |
| **JSON Schema** | First-class | Optional (future) |
| **Domain types** | No (couples to serde) | Yes (primary) |
| **Async validation** | No | Yes |
| **HTTP integration** | Manual | Built-in (optional) |
| **Composability** | Limited | Full rule algebra |

### Unique Selling Points

1. **Domain-first design** - DTO → Domain is the primary path
2. **Full error paths** - `rooms[0].adults`, not just field names
3. **Composable rules** - Rules as first-class values
4. **Async + sync** - One coherent framework for both
5. **HTTP ecosystem** - error_envelope, axum, actix integration

---

## Migration Guide

### From `validator`

**Before (validator):**
```rust
use validator::Validate;

#[derive(Validate, Deserialize)]
struct User {
    #[validate(email)]
    email: String,
    
    #[validate(range(min = 18, max = 120))]
    age: u8,
}

// Handler
async fn create_user(Json(user): Json<User>) -> Result<Json<UserId>, Error> {
    user.validate().map_err(|e| Error::BadRequest(e.to_string()))?;
    let id = db::insert(user).await?;
    Ok(Json(id))
}
```

**After (domain-model):**
```rust
// DTO (unchanged, for deserialization)
#[derive(Deserialize)]
struct UserDto {
    email: String,
    age: u8,
}

// Domain type with guarantees
struct User {
    email: Email,    // Validated email
    age: Age,        // Validated age (18-120)
}

impl TryFrom<UserDto> for User {
    type Error = ValidationError;
    
    fn try_from(dto: UserDto) -> Result<Self, Self::Error> {
        Ok(Self {
            email: Email::new(dto.email)?,
            age: Age::new(dto.age)?,
        })
    }
}

// Handler
async fn create_user(
    ValidatedJson(user): ValidatedJson<User>
) -> Result<Json<UserId>, Error> {
    // user is guaranteed valid
    let id = db::insert(user).await?;
    Ok(Json(id))
}
```

**Migration Steps:**
1. Keep existing DTOs for deserialization
2. Create domain types with smart constructors
3. Add `TryFrom<Dto>` implementations
4. Update handlers to use `ValidatedJson<Domain>` instead of `Json<Dto>`
5. Remove validation calls from handler logic

**Benefits:**
- Explicit DTO → Domain boundary
- Type-safe domain types (invalid states can't exist)
- Better error paths for clients
- Domain logic works with validated types only

### From `garde`

Similar pattern—garde can remain on DTOs for quick checks, domain-model handles domain types. They can coexist during migration.

### From Manual Validation

**Before (manual validation):**
```rust
async fn create_user(Json(dto): Json<UserDto>) -> Result<Json<UserId>, Error> {
    // 20+ lines of manual validation
    if dto.email.is_empty() {
        return Err(Error::BadRequest("Email required"));
    }
    if !dto.email.contains('@') {
        return Err(Error::BadRequest("Invalid email"));
    }
    if dto.age < 18 || dto.age > 120 {
        return Err(Error::BadRequest("Age must be 18-120"));
    }
    // ... more validation
    
    let id = db::insert(dto).await?;
    Ok(Json(id))
}
```

**After (domain-model):**
```rust
async fn create_user(
    ValidatedJson(user): ValidatedJson<User>
) -> Result<Json<UserId>, Error> {
    // user is guaranteed valid
    let id = db::insert(user).await?;
    Ok(Json(id))
}
```

**Migration time:** ~1 hour for typical service with 10-20 endpoints.

---

## Performance Characteristics

### Compile Time

**Zero-cost validation:**
- `Rule<T>` uses `Arc<dyn Fn>` - one indirection, amortized across validations
- Derive macro generates direct code (no reflection, no runtime overhead)
- Generic rule combinators are monomorphized by the compiler

**Expected compile times:**
- v0.1 (manual): <1s overhead vs no validation
- v0.2 (with derive): 2-5s for projects with 50+ validated types
- Incremental builds: Fast (only changed types revalidate)

**Mitigation strategies:**
- Avoid deeply nested generic rule chains (prefer custom validators)
- Use `cargo build --timings` to identify bottlenecks
- Consider splitting large validation graphs into modules

### Runtime Performance

**Validation costs (estimated):**
- Field checks: ~10ns per check (comparable to manual `if` statements)
- Nested validation: ~50ns overhead for path prefixing
- Rule composition: ~20ns per combinator (one Arc clone + function call)
- Total: <100ns for typical domain type with 5-10 fields

**Memory footprint:**
- `ValidationError`: ~48 bytes + violations vector
- `Violation`: ~80 bytes (includes String message, Path segments)
- `Rule<T>`: 8 bytes (Arc pointer)
- Path segments: 16 bytes per segment

**Async validation:**
- No additional allocations vs hand-written async code
- Context pattern allows connection pooling (reuse DB connections)
- Concurrent validation supported via helpers (opt-in, v0.5)

**Benchmark targets (v0.6):**
- 5-10% overhead vs hand-written validation
- <100ns for typical domain type validation
- <10µs for complex aggregates with 20+ fields
- Zero allocations for validation logic (errors allocate on failure only)

---

## Risks & Mitigation

### Technical Risks

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| **Macro complexity** | High compile times, unclear error messages | High | Start with simple cases, add features incrementally. Invest in diagnostic messages early. |
| **GAT stability** | MSRV bump, ecosystem friction | Medium | Provide `Pin<Box<dyn Future>>` alternative behind feature flag for lower MSRV. |
| **Rule performance** | Runtime overhead vs manual validation | Low | Benchmark early (v0.1), inline critical paths, profile before release. |
| **Path parsing** | Edge cases, ambiguity in syntax | Medium | Use structured Path builder primarily, parsing as convenience only. |
| **Async context design** | Complex lifetimes, poor DX | Medium | Provide helper macros, extensive examples, dedicated docs. |

### Adoption Risks

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| **Learning curve** | Slow adoption | Medium | Excellent docs, migration guides, video tutorials, blog posts. |
| **Ecosystem fragmentation** | Competes with validator/garde | High | Focus on differentiation (domain-first), not replacement. Coexistence is fine. |
| **Framework coupling** | Perceived Axum/Actix lock-in | Low | Keep core framework-agnostic, integrations are optional features. |
| **Breaking changes** | API churn in early versions | High | Use 0.x versioning with clear deprecation policy. Lock core in v0.2. |

### Mitigation Strategies

**For Technical Risks:**
1. **Incremental releases** - Ship v0.1 without macros to validate core design with real users
2. **Comprehensive tests** - 100% coverage for core, property-based testing for parsers
3. **Performance gate** - Automated benchmarks in CI, reject PRs that regress >10%
4. **Early user feedback** - Private alpha with 3-5 real-world projects before public release

**For Adoption Risks:**
1. **Clear positioning** - Market as "Domain modeling framework" not "validation replacement"
2. **Migration tools** - Provide automated scaffolding (stretch goal: cargo-domain CLI)
3. **Framework guides** - Dedicated documentation for Axum, Actix, Rocket
4. **Stability promise** - Lock core API by v0.2, only add features (no breaking changes) after

---

## Non-Goals

This library explicitly does **not** aim to:

1. **Replace serde** - We integrate with serde, not replace it. Serde handles serialization, we handle validation.

2. **Schema-first design** - We're domain-first (types define validation), not schema-first (schemas generate types).

3. **Runtime reflection** - All validation is compile-time generated. No dynamic validation rules.

4. **Universal validator** - We don't try to handle every validation pattern (e.g., no built-in credit card validation, complex regex patterns are opt-in).

5. **ORM/database layer** - Validation only, no persistence logic. Integrate with sqlx/diesel/etc.

6. **Form handling** - That's the web framework's job. We provide validation, not UI bindings.

7. **Replace thiserror/anyhow** - These are complementary. Use them for error handling, use domain-model for validation.

---

## Frequently Asked Questions

### Why not just use `validator` or `garde`?

They're excellent for DTO validation, but don't support:
- Domain-first design (types that can't be constructed invalid)
- Async validation with context (DB checks, API calls)
- Composable rule algebra (rules as first-class values)
- First-class error_envelope integration

**Use them for DTO validation, use domain-model for domain types.** They can coexist.

### Can I use this with `validator`?

Yes! Common pattern:
```rust
// Use validator on DTOs for quick structural checks
#[derive(Validate, Deserialize)]
struct UserDto {
    #[validate(email)]
    email: String,
}

// Use domain-model for domain types with guarantees
impl TryFrom<UserDto> for User {
    type Error = ValidationError;
    fn try_from(dto: UserDto) -> Result<Self, Self::Error> {
        Ok(Self { email: Email::new(dto.email)? })
    }
}
```

### What's the MSRV (Minimum Supported Rust Version)?

- **v0.1-0.5:** Rust 1.65+ (GATs required for `AsyncValidate`)
- **v0.6+:** Potentially lower with feature flags (`boxed-async` feature for older Rust)

### Does this work with `no_std`?

Planned for v0.6+. Core types will work (with `alloc` feature), async validation won't (requires `std`).

### How does this compare to "Parse, Don't Validate"?

This library embodies the "parse, don't validate" principle:
- **Smart constructors** = parsing (DTO → Domain conversion)
- **Domain types** = validated representation (can't be invalid)
- **DTO → Domain boundary** = parse boundary (explicit in code)

The difference: we provide tooling to make this pattern ergonomic.

### Can I use this in libraries?

Yes, but with caveats:
- **Expose `ValidationError`** to users (don't hide behind anyhow)
- **Avoid async validation** (libraries shouldn't require context/DB access)
- **Consider re-exporting rules** so users can compose their own validation
- **Document validation behavior** in public types

### Is this production-ready?

**Not yet.** This is a concept document. v0.1 will be experimental, v0.2+ will stabilize the API.

**For production use:**
- Wait for v0.3+ (serde integration)
- Monitor for breaking changes in 0.x versions
- Consider contributing feedback during alpha

---

## Open Questions

### Design Decisions Needed

1. **Error collection strategy:**
   - Collect all errors (current design)
   - Or fail-fast on first error?
   - Make it configurable?

2. **Macro expansion:**
   - Generate `impl Validate` directly
   - Or generate helper functions and call from trait impl?

3. **Async trait design:**
   - Use GAT (current, requires MSRV 1.65+)
   - Or require `Pin<Box<dyn Future>>` for lower MSRV?

4. **Meta field:**
   - Feature-gate `serde_json` for zero-cost opt-out
   - Or always include but keep lightweight?

5. **Path syntax:**
   - Support string parsing: `"rooms[0].adults"`
   - Or require programmatic construction?

### Community Input Needed

- What validation rules are most commonly needed?
- What async validation patterns are common in services?
- What HTTP frameworks should be prioritized?
- Should we support GraphQL / gRPC error mapping?

---

## References

### Inspiration

- **ZIO Schema** (Scala) - Type-safe schema definitions
- **dry-validation** (Ruby) - Declarative validation rules
- **validator** (Rust) - Attribute-based validation
- **garde** (Rust) - Nested validation
- **nutype** (Rust) - Validated newtypes

### Related Crates

- `thiserror` - Custom error types
- `anyhow` - Flexible error handling
- `error-envelope` - HTTP error responses
- `serde` - Serialization/deserialization
- `axum` - Web framework
- `validator` - DTO validation
- `garde` - Nested validation
- `nutype` - Validated primitives

---

## License

MIT (tentative)

---

## Contact

**Maintainer:** TBD  
**Repository:** (not yet created)  
**Discussions:** (not yet created)

---

**Last Updated:** 2025-12-23

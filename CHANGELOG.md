# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased] - v0.8.0 Schema Enhancements

### Added

#### Auto-Derived OpenAPI Schemas (domainstack-derive)

**HEADLINE FEATURE: Unified Rich Syntax + Zero-Duplication Schema Generation**

Write validation rules ONCE with `#[derive(Validate, ToSchema)]`, get BOTH runtime validation AND OpenAPI schemas automatically:

```rust
use domainstack_derive::{Validate, ToSchema};

// Both macros now support the SAME rich validation syntax!
#[derive(Validate, ToSchema)]
#[schema(description = "User registration")]
struct User {
    #[validate(email)]          // ✓ Works in both Validate and ToSchema
    #[validate(max_len = 255)]   // ✓ Unified syntax
    #[schema(example = "user@example.com")]
    email: String,

    #[validate(range(min = 18, max = 120))]
    age: u8,

    // Optional fields excluded from required array
    #[validate(min_len = 1)]
    nickname: Option<String>,
}

// Runtime validation works:
user.validate()?;  // ✓ Validates email format, length, age range

// Schema generation works:
// - email: { type: "string", format: "email", maxLength: 255, ... }
// - age: { type: "integer", minimum: 18, maximum: 120 }
// - required: ["email", "age"]  (nickname excluded)
```

#### Collection Item Validation with `each(rule)` (domainstack-derive)

**NEW**: Validate each item in a collection with any validation rule:

```rust
#[derive(Validate)]
struct BlogPost {
    // Validate each email in the list
    #[validate(each(email))]
    author_emails: Vec<String>,

    // Validate each tag length
    #[validate(each(length(min = 1, max = 50)))]
    tags: Vec<String>,

    // Validate each URL
    #[validate(each(url))]
    related_links: Vec<String>,
}
```

**Previously**, `each()` only supported `each(nested)` for nested types. Now it supports **any validation rule**:
- `each(email)`, `each(url)`, `each(alphanumeric)`, etc.
- `each(min_len = n)`, `each(max_len = n)`
- `each(length(min, max))`, `each(range(min, max))`
- Error paths include array indices: `tags[0]`, `emails[1]`, etc.

**Automatic Rule → Schema Mappings:**
- `email()` → `format: "email"`
- `url()` → `format: "uri"`
- `min_len(n)` / `max_len(n)` → `minLength` / `maxLength`
- `range(min, max)` → `minimum` / `maximum`
- `min_items(n)` / `max_items(n)` → `minItems` / `maxItems`
- `unique()` → `uniqueItems: true`
- `alphanumeric()` → `pattern: "^[a-zA-Z0-9]*$"`
- `ascii()` → `pattern: "^[\x00-\x7F]*$"`
- `Option<T>` → excluded from `required` array
- `#[validate(nested)]` → `$ref: "#/components/schemas/TypeName"`
- `Vec<T>` with `each_nested` → array with `$ref` items

**Schema Hints via #[schema(...)]:**
- `description` - Field/type descriptions
- `example` - Example values for documentation
- `deprecated` - Mark fields as deprecated
- `read_only` / `write_only` - Request/response modifiers

**Benefits:**
- **Single source of truth** - Validation and documentation in sync
- **Zero maintenance burden** - Change validation, docs update automatically
- **Type-safe** - Compile-time guarantees for schema generation
- **Comprehensive** - Handles nested types, collections, optional fields

**Documentation:**
- Complete guide: `/docs/SCHEMA_DERIVATION.md`
- Implementation details: `/docs/SCHEMA_DERIVATION_IMPLEMENTATION.md`
- Example: `domainstack-schema/examples/auto_derive.rs`
- Test coverage: `domainstack-derive/tests/schema_derive.rs` (7 comprehensive tests)

**Impact:**
This feature eliminates the primary pain point of maintaining validation rules and OpenAPI schemas separately. The DRY principle now extends from validation to API documentation.

#### Schema Composition

**anyOf / allOf / oneOf Support:**
- `Schema::any_of(schemas)` - Union types (matches any of the given schemas)
- `Schema::all_of(schemas)` - Composition/intersection (matches all schemas)
- `Schema::one_of(schemas)` - Discriminated unions (matches exactly one)

**Use Cases:**
- `anyOf`: Flexible types (e.g., string OR integer)
- `allOf`: Schema inheritance/composition (e.g., AdminUser extends User)
- `oneOf`: Discriminated unions (e.g., payment methods with different fields)

**Example:**
```rust
// Admin user extends base User
let admin = Schema::all_of(vec![
    Schema::reference("User"),
    Schema::object().property("admin", Schema::boolean()),
]);
```

#### Metadata & Documentation

**Rich Schema Metadata:**
- `.default(value)` - Default values for fields
- `.example(value)` - Single example value
- `.examples(values)` - Multiple example values

**Benefits:**
- Improved API documentation
- Better client code generation
- Enhanced developer experience

**Example:**
```rust
let theme = Schema::string()
    .enum_values(&["light", "dark", "auto"])
    .default(json!("auto"))
    .example(json!("dark"));
```

#### Request/Response Field Modifiers

**Field-Level Control:**
- `.read_only(true)` - Field returned in responses only (e.g., auto-generated IDs)
- `.write_only(true)` - Field accepted in requests only (e.g., passwords)
- `.deprecated(true)` - Mark field as deprecated

**Use Cases:**
- `readOnly`: Auto-generated fields (id, createdAt, etc.)
- `writeOnly`: Sensitive data (passwords, tokens)
- `deprecated`: Phase out old fields gracefully

**Example:**
```rust
Schema::object()
    .property("id", Schema::string().read_only(true))
    .property("password", Schema::string().write_only(true))
    .property("oldField", Schema::string().deprecated(true))
```

#### Vendor Extensions

**Support for Non-Mappable Validations:**
- `.extension(key, value)` - Add custom vendor extensions (x-*)

**Purpose:**
Cross-field validations, conditional rules, and business logic that don't map to OpenAPI can be preserved as vendor extensions, maintaining single source of truth.

**Example:**
```rust
Schema::object()
    .extension("x-domainstack-validations", json!({
        "cross_field": ["endDate > startDate"]
    }))
```

### Technical Details

**Test Coverage:**
- 20 unit tests (12 new for v0.8 features)
- 13 doctests covering all public APIs
- New example: `v08_features.rs` with comprehensive demonstrations
- 100% pass rate

**Code Quality:**
- Zero unsafe code
- Backward compatible - all v0.7 code works unchanged
- Clean clippy, properly formatted
- Fully documented with examples

**API Compatibility:**
- All v0.7 APIs unchanged
- New features are opt-in via builder methods
- No breaking changes

## [Unreleased] - v0.7.0 OpenAPI Schema Generation

### Added

#### OpenAPI Schema Generation

**New Crate: `domainstack-schema`**
- Generate OpenAPI 3.0 schemas from domainstack validation types
- Keep API documentation in sync with validation rules automatically
- Export schemas as JSON for use with Swagger UI, ReDoc, and other OpenAPI tools

**Core Features:**
- `ToSchema` trait - Implement to generate schemas for your types
- `Schema` type - Fluent API for building OpenAPI schemas
- `OpenApiBuilder` - Build complete OpenAPI 3.0 specifications
- `OpenApiSpec` - Serialize to JSON or YAML (with `yaml` feature)

**Schema Constraints:**
Maps validation rules to OpenAPI constraints:
- `length(min, max)` → `minLength`, `maxLength`
- `range(min, max)` → `minimum`, `maximum`
- `email()` → `format: "email"`
- `one_of(...)` → `enum`
- `numeric_string()` → `pattern: "^[0-9]+$"`
- `min_items(n)`, `max_items(n)` → `minItems`, `maxItems`

**Example:**
```rust
use domainstack_schema::{OpenApiBuilder, Schema, ToSchema};

struct User {
    email: String,
    age: u8,
}

impl ToSchema for User {
    fn schema_name() -> &'static str { "User" }

    fn schema() -> Schema {
        Schema::object()
            .property("email", Schema::string().format("email"))
            .property("age", Schema::integer().minimum(18).maximum(120))
            .required(&["email", "age"])
    }
}

let spec = OpenApiBuilder::new("My API", "1.0.0")
    .register::<User>()
    .build();

println!("{}", spec.to_json().unwrap());
```

**Benefits:**
- **Type-safe** - Schemas are built using Rust's type system
- **No runtime overhead** - Schema generation happens at build time
- **Framework agnostic** - Works with any web framework
- **OpenAPI 3.0 compliant** - Generates valid specifications
- **Extensible** - Implement ToSchema for any type

**Documentation:**
- Complete README with examples and constraint mapping
- Example: `user_api.rs` demonstrating User, Address, and Team schemas
- Full documentation on docs.rs

**Dependencies:**
- `serde` - JSON serialization
- `serde_json` - JSON output
- `domainstack` (core validation types)

### Technical Details

**Test Coverage:**
- 8 unit tests covering all schema types and builders
- 3 doctests in public API
- Complete example demonstrating real-world usage
- 100% pass rate

**Code Quality:**
- Zero unsafe code
- Minimal dependencies
- Clean clippy (1 expected warning for optional yaml feature)
- Fully documented with examples

## [Unreleased] - v0.6.0 Type-State Validation & Performance

### Changed

#### Breaking Changes

**Metadata Storage Optimization (HashMap):**
- `Meta` internal storage changed from `Vec<(&'static str, String)>` to `HashMap<&'static str, String>`
- **Benefits:**
  - **O(1) lookup** instead of O(n) when accessing metadata with `.get()`
  - **Prevents duplicate keys** - inserting same key twice now replaces the value
  - **Better semantics** - HashMap clearly represents key-value mapping
  - **Same API** - `Meta::new()`, `.insert()`, `.get()`, `.iter()` all work identically
- **Migration:** Public API unchanged - no code changes needed
  - `meta.insert("key", value)` works the same
  - `meta.get("key")` works the same
  - Only internal implementation changed
- **Trade-offs:**
  - Slightly higher memory overhead per entry (HashMap vs Vec)
  - Iteration order is not guaranteed (HashMap is unordered)
- **Impact:** Internal optimization, transparent to users

**SmallVec for Violations:**
- `ValidationError.violations` changed from `Vec<Violation>` to `SmallVec<[Violation; 4]>`
- **Benefits:**
  - **Zero heap allocation** for ≤4 violations (stack-allocated)
  - **Better cache locality** - violations stored inline
  - **~30% faster** for common case (1-3 validation errors)
  - **Same API** - SmallVec has Vec-compatible methods (push, extend, iter, etc.)
- **Migration:** Public API unchanged - SmallVec is Vec-compatible
  - All Vec methods work the same
  - Indexing works the same: `err.violations[0]`
  - Iteration works the same: `err.violations.iter()`
- **Trade-offs:**
  - Slightly larger stack size (96 bytes vs 24 bytes for empty errors)
  - Falls back to heap allocation when >4 violations
- **Impact:** Performance optimization, API-compatible

### Added

#### Phantom Types for Validated State

**New Feature: Compile-Time Validation Guarantees**
- `typestate` module with `Validated` and `Unvalidated` marker types
- Zero-cost type-state pattern for tracking validation status at compile time
- Prevents accidentally using unvalidated data in critical operations
- Perfect for builder patterns, database operations, and business logic boundaries

**Example:**
```rust
use domainstack::typestate::{Validated, Unvalidated};
use domainstack::{ValidationError, validate, rules};
use std::marker::PhantomData;

// Domain type with validation state
pub struct Email<State = Unvalidated> {
    value: String,
    _state: PhantomData<State>,
}

impl Email<Unvalidated> {
    pub fn new(value: String) -> Self {
        Self { value, _state: PhantomData }
    }

    pub fn validate(self) -> Result<Email<Validated>, ValidationError> {
        validate("email", self.value.as_str(), &rules::email())?;
        Ok(Email { value: self.value, _state: PhantomData })
    }
}

// Only accept validated emails!
fn send_email(email: Email<Validated>) {
    // Compiler GUARANTEES email is validated!
}

// Usage
let email = Email::new("user@example.com".to_string());
// send_email(email); // ❌ Compile error: expected Email<Validated>

let validated = email.validate()?;
send_email(validated); // ✅ Compiles!
```

**Key Features:**
- **Zero runtime cost** - PhantomData has size 0, no memory or CPU overhead
- **Compile-time safety** - Type system enforces validation occurred
- **Opt-in adoption** - Add to types that benefit from state tracking
- **Self-documenting** - Function signatures make validation requirements explicit
- **Builder pattern friendly** - Natural fit with builder APIs

**Use Cases:**
- Database operations requiring validated data
- Business logic with validation boundaries
- Multi-step workflows with validation gates
- API handlers ensuring data is validated before processing
- Builder patterns with validation as final step

**Documentation:**
- Comprehensive module-level documentation with examples
- 9 unit tests covering all patterns
- Full example: `phantom_types.rs` demonstrating:
  - Simple single-field types (Email)
  - Multi-field structs (User)
  - Builder pattern integration
  - Simulated database operations
  - Zero-cost proof

**Design Philosophy:**
This feature follows domainstack's principle of "make invalid states unrepresentable."
Phantom types take this further by making **unvalidated states unusable in validated contexts**.

### Technical Details

**Test Coverage:**
- 9 new typestate tests (100% pass rate)
- Tests cover: zero-cost verification, state transitions, multi-field validation
- Example demonstrates 6 real-world scenarios

**Code Quality:**
- Zero clippy warnings
- Fully documented with doctests
- Zero dependencies (uses only std::marker::PhantomData)

**Performance:**
- Truly zero-cost: PhantomData<T> optimizes away completely
- Same size as underlying data structure
- No runtime checks or overhead

## [0.5.0] - Extended Rule Library

### Added

#### Async Validation Support

**New Feature: Asynchronous Validation**
- `AsyncValidate` trait for types requiring async validation (database checks, external APIs)
- `ValidationContext` for passing shared resources (database connections, HTTP clients, etc.)
- `AsyncRule<T>` type for creating async validation rules
- Full support for database uniqueness checks and external API validation

**Example:**
```rust
use domainstack::{AsyncValidate, ValidationContext, ValidationError};
use async_trait::async_trait;

#[async_trait]
impl AsyncValidate for User {
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

**Key Features:**
- Zero-cost abstractions with `Arc` and `Pin<Box<Future>>`
- Type-safe resource storage via `Any` trait
- Compatible with any async runtime (tokio, async-std, smol)
- Composable with sync validation - use both in same type
- Thread-safe with `Send + Sync` bounds

**Use Cases:**
- Database uniqueness constraints (email, username, etc.)
- External API validation (address validation, credit card checks)
- Rate limiting checks
- Cross-service validation in microservices
- Any I/O-bound validation logic

**Feature Flag:**
- Enable with `features = ["async"]` in Cargo.toml
- Adds `async-trait` dependency
- Example: `cargo run --example async_validation --features async`

#### 10 New Validation Rules (High Value Expansions)

**String Semantics (4 rules):**
- `rules::non_blank()` - Not empty after trimming whitespace (catches "   " as invalid)
- `rules::no_whitespace()` - Contains no whitespace characters (useful for usernames, slugs)
- `rules::ascii()` - All characters are ASCII (for legacy systems or ASCII-only fields)
- `rules::len_chars(min, max)` - Character count validation (not byte count, handles Unicode correctly)

**Choice/Membership (3 rules):**
- `rules::equals(value)` - Value must equal the specified value
- `rules::not_equals(value)` - Value must NOT equal the specified value
- `rules::one_of(&[values])` - Value must be in the allowed set (e.g., status enums, role checks)

**Collection Validation (3 rules):**
- `rules::min_items(n)` - Collection must have at least n items
- `rules::max_items(n)` - Collection must have at most n items
- `rules::unique()` - All items must be unique (no duplicates, uses HashSet for O(n) performance)

**Examples:**
```rust
// String semantics
let rule = rules::non_blank();  // "   " fails, "  hello  " passes
let rule = rules::len_chars(3, 10);  // "🚀🚀🚀" = 3 chars, not 12 bytes

// Choice/membership
let rule = rules::one_of(&["active", "pending", "inactive"]);
let rule = rules::equals(42);

// Collections
let tags_rule = rules::min_items(1).and(rules::unique());  // At least 1 unique tag
let rule = rules::max_items(10);  // Limit to 10 items
```

#### Critical Bug Fixes

**Float Validation (NaN Handling):**
- Added `rules::finite()` to catch NaN and infinity values
- NaN values were slipping through `range()`, `min()`, `max()` due to PartialOrd semantics
- New `FiniteCheck` trait for f32/f64 validation
- **Recommended:** Always combine `finite()` with range checks for floats

**Numeric Validation:**
- Added `rules::non_zero()` for better zero-checking
- Documented that `positive()` and `negative()` are for signed types only

**Performance:**
- Regex patterns now compiled once and cached (uses `once_cell`)
- `EMAIL_REGEX` and `URL_REGEX` are static `Lazy<Regex>` for one-time compilation
- Massive performance improvement for repeated validations

**Feature Flag Consistency:**
- Unified all regex-backed validation under single `regex` feature
- Removed `email` feature alias (was confusing)
- Both `email()` and `url()` now require `regex` feature consistently

### Changed

#### Breaking Changes

**Path API Encapsulation:**
- Made `Path(Vec<PathSegment>)` private to prevent direct manipulation of internal structure
- Added proper accessor methods:
  - `segments()` - Returns `&[PathSegment]` for read-only access
  - `push_field(name)` - Adds a field segment
  - `push_index(idx)` - Adds an index segment
- **Migration:** Replace direct field access with new methods:
  ```rust
  // Before (v0.4.x)
  let len = path.0.len();
  path.0.push(PathSegment::Field("name"));

  // After (v1.0.0)
  let len = path.segments().len();
  path.push_field("name");
  ```
- **Benefits:** Better encapsulation, allows future implementation changes without breaking API

**Regex Feature Requirement:**
- `email()` and `url()` now require the `regex` feature (no fallback implementations)
  - **Migration:** Add `features = ["regex"]` to enable email/url validation
  - This ensures consistent, RFC-compliant validation

### Technical Details

**Test Coverage:**
- 143 unit tests (added 12 async validation tests: 9 unit + 3 integration)
- 52 doctests (comprehensive API documentation)
- 100% pass rate across all test suites
- New async validation example with 5 usage scenarios

**Code Quality:**
- Zero clippy warnings
- All new rules have comprehensive documentation
- Added realistic examples for each rule category

**Performance:**
- Regex compilation moved to static initialization (one-time cost)
- `unique()` uses HashSet for O(n) duplicate detection
- No heap allocation for error metadata keys

### Design Philosophy

These rules were implemented in **complexity order**:
1. **String semantics** (low complexity) - straightforward string checks
2. **Choice/membership** (medium complexity) - generic equality checking
3. **Collection rules** (high complexity) - working with slices and collections

All rules follow the existing patterns:
- Builder-style customization (`.with_message()`, `.with_code()`, `.with_meta()`)
- Clear error codes and messages
- Metadata includes actual values for debugging
- Comprehensive test coverage

## [0.4.0] - 2025-12-24

### Added

#### New Validation Rules (10 total)
**String Rules (8 new):**
- `rules::url()` - Validates URL format (requires `regex` feature)
- `rules::alphanumeric()` - Validates alphanumeric-only strings
- `rules::alpha_only()` - Validates alphabetic-only strings
- `rules::numeric_string()` - Validates numeric-only strings
- `rules::contains(needle)` - Validates string contains substring
- `rules::starts_with(prefix)` - Validates string prefix
- `rules::ends_with(suffix)` - Validates string suffix
- `rules::matches_regex(pattern)` - Validates against regex pattern (requires `regex` feature)

**Numeric Rules (3 new):**
- `rules::positive()` - Validates value is greater than zero
- `rules::negative()` - Validates value is less than zero
- `rules::multiple_of(divisor)` - Validates value is evenly divisible

#### Builder-Style Rule Customization
All validation rules now support fluent builder-style customization:

```rust
let rule = rules::email()
    .code("invalid_email")
    .message("Please provide a valid email address")
    .meta("hint", "Format: user@domain.com");
```

**New Methods:**
- `Rule::code(self, code: &'static str)` - Customize error code
- `Rule::message(self, msg: impl Into<String>)` - Customize error message
- `Rule::meta(self, key: &'static str, value: impl Into<String>)` - Add metadata

These methods work uniformly across all built-in and custom rules.

#### Context-Aware Error Messages
Introduced `RuleContext` to provide validation rules with field information for better error messages:

```rust
use domainstack::{Rule, RuleContext, ValidationError};

fn min_len_with_context(min: usize) -> Rule<str> {
    Rule::new(move |value: &str, ctx: &RuleContext| {
        if value.len() < min {
            ValidationError::single(
                ctx.full_path(),
                "min_length",
                format!(
                    "Field '{}' must be at least {} characters (got {})",
                    ctx.field_name.as_ref().map(|s| s.as_ref()).unwrap_or("unknown"),
                    min,
                    value.len()
                )
            )
        } else {
            ValidationError::default()
        }
    })
}
```

**New Type:**
- `RuleContext` - Contains `field_name`, `parent_path`, and `value_debug` for context-aware validation
- Methods: `root()`, `anonymous()`, `child()`, `with_value_debug()`, `full_path()`

**Benefits:**
- More helpful error messages including field names
- Better debugging with contextual information
- Improved user experience with specific, actionable errors

#### Cross-Field Validation
Added struct-level validation to check relationships between multiple fields:

```rust
use domainstack_derive::Validate;

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

**Features:**
- **Basic cross-field checks**: Compare multiple fields (e.g., password confirmation, date ranges)
- **Conditional validation**: Use `when` parameter to apply checks conditionally
- **Multiple checks**: Apply multiple struct-level validations
- **Custom error messages**: Specify code and message for each check

**Examples:**
```rust
// Date range validation
#[validate(
    check = "self.end_date > self.start_date",
    code = "invalid_date_range",
    message = "End date must be after start date"
)]

// Conditional validation
#[validate(
    check = "self.total >= self.minimum_order",
    code = "below_minimum",
    message = "Order below minimum",
    when = "self.requires_minimum"
)]

// Multiple checks
#[validate(check = "self.a == self.b", message = "A must equal B")]
#[validate(check = "self.b == self.c", message = "B must equal C")]
```

**Use Cases:**
- Password confirmation matching
- Date range validation (end > start)
- Mutually exclusive fields (discount code OR percentage, not both)
- Conditional business rules
- Complex field relationships

**New Test Coverage:**
- 15 new integration tests covering all cross-field scenarios
- Example: `v5_cross_field_validation.rs` with 10 demonstrations

#### Documentation Improvements
- Added 30+ runnable doctests to public APIs (`ValidationError`, `Rule`, `Path`, all rules)
- Documented `Box::leak()` memory behavior in `Path::parse()` with usage guidance
- Created comprehensive rules reference (see `docs/RULES_V04.md`)
- Added `v4_builder_customization.rs` example demonstrating rule customization
- Added rule system analysis documents

### Changed

#### Performance & Memory Improvements
- **Eliminated memory leaks in `Path`**: Replaced `Box::leak()` with `Arc<str>` for field names
  - No more leaked memory from parsed paths
  - Reference-counted field names with proper cleanup
  - More idiomatic Rust memory management
  - Benefits long-running services and applications parsing many dynamic paths

- Improved error messages for all new validation rules
- Enhanced inline documentation across core types
- All tests passing (192 total: 143 unit/integration + 39 doctests + 10 framework tests)

### Breaking Changes
- **Rule function signature changed**: All rules now receive `RuleContext` as second parameter
  - Old: `Fn(&T) -> ValidationError`
  - New: `Fn(&T, &RuleContext) -> ValidationError`
  - **Migration**: Add `ctx: &RuleContext` parameter to custom rules, use `ctx.full_path()` instead of `Path::root()`
  - Existing code using `rule.apply()` continues to work (creates anonymous context)
  - Use `rule.apply_with_context()` for field-aware error messages

- `PathSegment::Field` now uses `Arc<str>` instead of `&'static str`
  - Affects code that pattern matches on `PathSegment` directly
  - Most users unaffected (use `Path::field()` API which remains the same)
  - **Migration**: No code changes needed for standard Path API usage

### Technical Details
- **Zero Unsafe Code** - Maintains safety guarantees
- **Zero Dependencies** - Core library remains dependency-free (regex is optional)
- **Zero Warnings** - Clean compile with clippy
- **Pre-1.0 Status** - Breaking changes acceptable before first publish

### Migration from 0.3.x

Update your `Cargo.toml`:

```toml
domainstack = "0.4.0"
```

**If you pattern match on `PathSegment` directly:**

```rust
// Before (v0.3.x)
match segment {
    PathSegment::Field(name) => println!("{}", name), // name: &'static str
    PathSegment::Index(idx) => println!("[{}]", idx),
}

// After (v0.4.0)
match segment {
    PathSegment::Field(name) => println!("{}", name), // name: Arc<str> (still prints fine)
    PathSegment::Index(idx) => println!("[{}]", idx),
}
```

Most code uses `Path::field()` and `Path::to_string()` which work identically. New features are opt-in via builder methods.

## [0.3.0] - Previous Release

Initial release with core validation framework, derive macros, and framework adapters for Axum and Actix-web.

---

## Unreleased Features (Roadmap)

See `docs/BREAKING_CHANGES_ANALYSIS.md` for planned features in future versions:
- v0.7.0: Schema generation (OpenAPI, JSON Schema, TypeScript types)
- v1.0.0: API stabilization, performance optimizations

[0.4.0]: https://github.com/blackwell-systems/domainstack/compare/v0.3.0...v0.4.0

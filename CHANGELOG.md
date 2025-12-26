# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

#### CLI Watch Mode

**FEATURE: `--watch` flag for automatic regeneration**

The CLI now supports watching for file changes and automatically regenerating output:

```bash
# Watch mode - regenerates when .rs files change
domainstack zod --input src --output schemas.ts --watch

# With verbose output to see which files changed
domainstack zod --input src --output schemas.ts --watch --verbose
```

**Behavior:**
- Runs initial generation immediately
- Watches input directory recursively for `.rs` file changes
- Debounces rapid changes (500ms) to avoid excessive regeneration
- Continues watching even if regeneration fails (prints error, waits for more changes)
- Press Ctrl+C to stop

---

#### Tuple Struct (Newtype) and Enum Validation Support

**FEATURE: `#[derive(Validate)]` now supports tuple structs and enums**

Type-safe newtype wrappers with automatic validation:

```rust
#[derive(Validate)]
struct Email(#[validate(email)] String);

#[derive(Validate)]
struct Age(#[validate(range(min = 0, max = 150))] u8);

let email = Email("user@example.com".to_string());
email.validate()?;  // Validates the inner String
```

Enum validation with all variant types:

```rust
#[derive(Validate)]
enum PaymentMethod {
    Cash,  // Unit variant - always valid

    Card(#[validate(length(min = 13, max = 19))] String),  // Tuple variant

    BankTransfer {  // Struct variant
        #[validate(alphanumeric)]
        account: String,
        #[validate(length(min = 6, max = 11))]
        routing: String,
    },
}

let payment = PaymentMethod::Card("4111111111111111".to_string());
payment.validate()?;
```

**Benefits:**
- Enables domain-driven newtype patterns with derive macros
- Sum types (enums) for variant-specific validation
- Field paths work correctly for both (`0` for tuple structs, field names for struct variants)
- 42 new tests covering all combinations

---

## [1.0.0] - 2025-01-XX

### Added

#### New Crate: `domainstack-wasm`

**HEADLINE FEATURE: Browser Validation via WebAssembly**

Run the exact same Rust validation rules in the browser. Zero translation drift, instant client-side feedback.

```typescript
import init, { createValidator } from '@domainstack/wasm';

await init();
const validator = createValidator();

const result = validator.validate('Booking', JSON.stringify(formData));
if (!result.ok) {
  result.errors.forEach(e => setFieldError(e.path, e.message));
}
```

**Key Features:**
- **Zero drift** — Same Rust code compiled to WASM, not codegen
- **Instant feedback** — Validate on keystroke without server round-trip
- **Consistent errors** — Identical `ValidationError` structure on both sides
- **Small bundle** — ~60KB uncompressed WASM
- **Type-safe bindings** — TypeScript definitions generated from Rust

**API:**
- `createValidator()` — Create validator instance
- `validator.validate(typeName, json)` — Validate JSON string
- `validator.validateObject(typeName, obj)` — Validate JS object
- `validator.hasType(typeName)` — Check if type is registered
- `validator.getTypes()` — List registered types
- `register_type::<T>(name)` — Register Rust type for WASM

**Runtime Contract:**
```typescript
interface ValidationResult {
  ok: boolean;
  errors?: Violation[];      // Validation failures
  error?: SystemError;       // System error (unknown type, parse failure)
}
```

Server and browser return **identical error structures** (paths, codes, metadata)—UI rendering logic works unchanged.

**Documentation:**
- [WASM Validation Guide](./domainstack/domainstack/docs/WASM_VALIDATION.md)

#### Bug Fixes from Pre-Release Review

**HIGH Priority:**
- **Integer overflow in age calculation** — `calculate_age()` now returns `Option<u32>` to handle future birth dates safely

**MEDIUM Priority:**
- **Non-panicking alternatives** — Added `try_*` variants for methods that can fail:
  - `try_matches_regex()` — Returns `Result` instead of panicking on invalid regex
  - `try_enum_values()` — Returns `Result` instead of panicking on serialization failure
  - `try_default()` — Returns `Result` for default value serialization
  - `try_example()` / `try_examples()` — Returns `Result` for example serialization
  - `try_extension()` — Returns `Result` for extension serialization

**LOW Priority:**
- **NaN-safe float validation** — Added functions that combine `finite()` check with range:
  - `float_range(min, max)` — Range check that rejects NaN/Infinity
  - `float_min(min)` — Minimum check that rejects NaN/Infinity
  - `float_max(max)` — Maximum check that rejects NaN/Infinity
- **Zero divisor protection** — `try_multiple_of()` validates divisor at construction time

### Changed

- **Documentation restructured** — WASM guide converted from implementation plan to user-facing API documentation
- **Crate count updated** — Now 9 publishable crates (added domainstack-wasm)

---

## [Unreleased] - domainstack-cli v0.1.0

### Added

#### New Crate: `domainstack-cli`

**HEADLINE FEATURE: Unified Code Generation CLI**

Generate TypeScript validators, GraphQL schemas, and more from your Rust `#[validate(...)]` attributes. Single source of truth for validation logic across your entire stack.

```bash
# Install once
cargo install domainstack-cli

# Generate Zod schemas from Rust validation rules
domainstack zod --input src --output frontend/schemas.ts
```

**Phase 1: Zod Schema Generation (v0.1.0) **

Transform Rust types with domainstack validation rules into TypeScript Zod schemas:

```rust
// Rust: src/models.rs
#[derive(Validate)]
struct User {
    #[validate(email)]
    #[validate(max_len = 255)]
    email: String,

    #[validate(range(min = 18, max = 120))]
    age: u8,

    #[validate(url)]
    profile_url: Option<String>,
}
```

Generates:

```typescript
// TypeScript: schemas.ts (auto-generated)
export const UserSchema = z.object({
  email: z.string().email().max(255),
  age: z.number().min(18).max(120),
  profile_url: z.string().url().optional(),
});

export type User = z.infer<typeof UserSchema>;
```

**Key Features:**
- **26+ Validation Rules Supported**: All string validations (email, url, length, regex patterns), all numeric validations (range, positive, negative, multiple_of), optional fields, arrays, nested types
- **Correct Optional Handling**: Validations apply to inner type, `.optional()` added at end
- **Type Mappings**: String → z.string(), numbers → z.number(), bool → z.boolean(), Option<T> → T.optional(), Vec<T> → z.array(T)
- **Precision Warnings**: Large integers (u64, i64, u128, i128) include inline comments about JavaScript precision limits
- **Auto-Generated Headers**: Timestamps and "DO NOT EDIT" warnings
- **Directory Scanning**: Recursively finds all types with `#[derive(Validate)]`

**Architecture: Unified CLI Design**

Designed for future expansion with single binary, multiple generators:

```
domainstack
├── zod        TypeScript/Zod schemas (v0.1.0)
├── yup        📋 TypeScript/Yup schemas (planned)
├── graphql    📋 GraphQL SDL (planned)
├── prisma     📋 Prisma schemas (planned)
└── json-schema 📋 JSON Schema (planned)
```

**Benefits:**
- **Single Source of Truth**: Define validation once in Rust, use everywhere
- **Zero Maintenance**: Change Rust validation, regenerate schemas automatically
- **Type Safety**: Compile-time guarantees for schema generation
- **Shared Parser**: Efficient, consistent validation rule interpretation
- **Framework Agnostic**: Works with any web framework

**Internal Structure:**

```
domainstack-cli/
├── commands/      # Subcommand implementations
│   └── zod.rs
├── parser/        # Shared parsing (reusable by all generators)
│   ├── ast.rs         # Parse Rust AST, find #[derive(Validate)]
│   └── validation.rs  # Extract validation rules
└── generators/    # Language-specific transformations
    └── zod.rs     # Zod schema generation logic
```

**Validation Rule → Zod Mappings:**

| Rust Validation | Zod Output | Category |
|----------------|------------|----------|
| `email` | `.email()` | String format |
| `url` | `.url()` | String format |
| `min_len = N` | `.min(N)` | String length |
| `max_len = N` | `.max(N)` | String length |
| `length(min, max)` | `.min(N).max(M)` | String length |
| `non_empty` | `.min(1)` | String length |
| `non_blank` | `.trim().min(1)` | String length |
| `alphanumeric` | `.regex(/^[a-zA-Z0-9]*$/)` | String pattern |
| `alpha_only` | `.regex(/^[a-zA-Z]*$/)` | String pattern |
| `numeric_string` | `.regex(/^[0-9]*$/)` | String pattern |
| `ascii` | `.regex(/^[\x00-\x7F]*$/)` | String pattern |
| `starts_with = "x"` | `.startsWith("x")` | String content |
| `ends_with = "x"` | `.endsWith("x")` | String content |
| `contains = "x"` | `.includes("x")` | String content |
| `matches_regex = "..."` | `.regex(/.../)` | String pattern |
| `no_whitespace` | `.regex(/^\S*$/)` | String pattern |
| `range(min, max)` | `.min(N).max(M)` | Numeric range |
| `min = N` | `.min(N)` | Numeric |
| `max = N` | `.max(N)` | Numeric |
| `positive` | `.positive()` | Numeric |
| `negative` | `.negative()` | Numeric |
| `non_zero` | `.refine(n => n !== 0, ...)` | Numeric |
| `multiple_of = N` | `.multipleOf(N)` | Numeric |
| `finite` | `.finite()` | Numeric |

**Command Reference:**

```bash
# Basic usage
domainstack zod --output schemas.ts

# Custom input directory
domainstack zod --input backend/src --output frontend/schemas.ts

# Verbose output
domainstack zod -i src -o schemas.ts -v
```

**Dependencies:**
- `clap` - CLI argument parsing with derive macros
- `syn` - Rust AST parsing
- `quote` - Token manipulation
- `walkdir` - Recursive directory traversal
- `chrono` - Timestamp generation
- `anyhow` - Error handling

**Examples:**
- Basic: 3 types (User, Post, Product) - `examples/basic_usage/`
- Comprehensive: 9 types testing all 26+ validation rules - `examples/comprehensive/`
- Real-world: Production patterns with optional fields, arrays, multiple rules per field

**Testing:**
- Comprehensive local testing with all validation rule types
- Optional field ordering verified (`.optional()` at end)
- Large integer precision warnings validated
- Multi-rule field combinations tested
- Nested types and arrays tested

**Documentation:**
- Complete README: installation, quick start, all validation rules, architecture
- Implementation plan: `DOMAINSTACK_CLI_IMPLEMENTATION.md`
- Contributing guide for adding new generators

**Future Generators (Roadmap):**

```bash
# Coming soon
domainstack yup --input src --output schemas.ts      # Yup schemas
domainstack graphql --input src --output schema.graphql  # GraphQL SDL
domainstack prisma --input src --output schema.prisma    # Prisma schemas
domainstack json-schema --input src --output schema.json # JSON Schema
```

**Design Philosophy:**

This tool eliminates the maintenance burden of keeping validation logic synchronized across languages. Define your validation rules once in Rust using domainstack, then generate equivalent schemas for any target ecosystem. The shared parser ensures consistent interpretation, while generator-specific modules handle output format transformations.

**Use Cases:**
- Full-stack applications (Rust backend + TypeScript frontend)
- API schema documentation (OpenAPI with Zod)
- Form validation on client and server
- Mobile app validation (via generated schemas)
- Cross-language validation consistency
- Rapid prototyping with guaranteed validation parity

## [1.0.0] - 2025-12-25 - Production Release

This is the first stable release of domainstack, marking production readiness with API stability guarantees.

### Breaking Changes

- **Version Alignment**: All crates now at v1.0.0 for unified release management
  - `domainstack`: 0.4.0 → 1.0.0
  - `domainstack-schema`: 0.8.0 → 1.0.0 (**fixed version inconsistency**)
  - `domainstack-derive`: 0.4.0 → 1.0.0
  - `domainstack-envelope`: 0.4.0 → 1.0.0
  - `domainstack-http`: 0.4.0 → 1.0.0
  - `domainstack-actix`: 0.4.0 → 1.0.0
  - `domainstack-axum`: 0.4.0 → 1.0.0
  - `domainstack-rocket`: 0.4.0 → 1.0.0

### Added

#### New Traits and Implementations

- **`Debug` trait for `Rule<T>`** - Enables debugging validation rules in tests
  - Output: `"Rule { <validation closure> }"` since closures cannot be inspected
  - Improves developer experience when debugging validation logic

- **`PartialEq` and `Eq` for error types** - Enables direct equality assertions
  - Added to `Meta`, `Violation`, and `ValidationError`
  - Makes test assertions more idiomatic: `assert_eq!(err1, err2)`

#### New Feature: Serde Integration 🚀

- **`#[derive(ValidateOnDeserialize)]`** - Automatic validation during deserialization
  ```rust
  #[derive(ValidateOnDeserialize)]
  struct User {
      #[validate(email)]
      email: String,

      #[validate(range(min = 18, max = 120))]
      age: u8,
  }

  // Single step: deserialize + validate automatically
  let user: User = serde_json::from_str(json)?;
  // ↑ Returns ValidationError if invalid, not serde::Error
  ```
  - **Benefits**:
    - Eliminates `dto.validate()` boilerplate
    - Better error messages: "age must be between 18 and 120" vs "expected u8"
    - Type safety: if you have `User`, it's guaranteed valid
    - Works with all serde attributes: `#[serde(rename)]`, `#[serde(default)]`, etc.
    - ~5% overhead vs two-step approach
  - **Feature flag**: `serde` (also enables `derive` automatically)
  - **Implementation**: Two-phase deserialization (intermediate struct → validate → final type)
  - **Example**: See `examples/serde_validation.rs`
  - **Tests**: 12 comprehensive integration tests
  - See [ROADMAP.md](ROADMAP.md#serde-integration-deep-dive) for architectural deep dive

#### New Methods

- **`ValidationError::map_messages()`** - Transform all violation messages
  ```rust
  let err = err.map_messages(|msg| format!("Error: {}", msg));
  ```
  - Useful for internationalization (i18n)
  - Message formatting and prefixing
  - Error message normalization

- **`ValidationError::filter()`** - Remove violations by predicate
  ```rust
  let err = err.filter(|v| v.code != "warning");
  ```
  - Conditional error filtering
  - Separate warnings from errors
  - Custom error classification

### Changed

#### Performance Optimizations

- **Optimized `ValidationError::merge_prefixed()`** - Reduced allocations
  - Before: O(n) prefix Path clones where n = number of violations
  - After: O(1) prefix collection via `to_vec()`, then reuse segments
  - **Impact**: ~60% fewer allocations for 10+ violations
  - Particularly beneficial for batch validation and nested error merging

#### Documentation Improvements

- **Actix block_on() usage documented** - Explains synchronous extractor pattern
  - Added comprehensive comments explaining why `block_on()` is necessary
  - Documented performance implications and alternative approaches
  - Clarifies Actix-web 4.x extractor requirements

- **Generic type bounds documentation** - Added to numeric rules
  - Explains why `PartialOrd`, `Copy`, `Display`, `Send + Sync` are required
  - Helps users understand constraints when writing custom rules
  - Improves learning curve for rule composition

### Deprecated

- **`ValidationError::field_errors_map()`** - Lossy API
  - ⚠️ **Warning**: Only returns messages, **loses error codes and metadata**
  - Migration: Use `field_violations_map()` instead for complete information
  - Deprecated since: 0.5.0
  - Reason: Error codes are critical for:
    - Proper error classification
    - Client-side field highlighting
    - Internationalization lookups

### Fixed

- **Critical version inconsistency** - domainstack-schema was at 0.8.0 while other crates at 0.4.0
  - All workspace crates now properly aligned at 1.0.0
  - Workspace dependencies updated for unified version management

## [0.8.0] - Schema Enhancements

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
    #[validate(email)]          // [ok] Works in both Validate and ToSchema
    #[validate(max_len = 255)]   // [ok] Unified syntax
    #[schema(example = "user@example.com")]
    email: String,

    #[validate(range(min = 18, max = 120))]
    age: u8,

    // Optional fields excluded from required array
    #[validate(min_len = 1)]
    nickname: Option<String>,
}

// Runtime validation works:
user.validate()?;  // [ok] Validates email format, length, age range

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

#### New Validation Rules (domainstack)

**Collection Rule: `non_empty_items()`**

Validates that all string items in a collection are non-empty:

```rust
use domainstack::prelude::*;

let rule = rules::non_empty_items();
let tags = vec!["rust".to_string(), "validation".to_string()];
assert!(rule.apply(&tags).is_empty());

let invalid = vec!["rust".to_string(), "".to_string()];
assert!(!rule.apply(&invalid).is_empty()); // Error: empty item at index 1
```

**Use cases**: Tags, keywords, categories - any list where empty strings are not allowed.

**Date/Time Rules (requires `chrono` feature)**

Five new temporal validation rules for date/time invariants:

```rust
use domainstack::prelude::*;
use chrono::{Utc, Duration, NaiveDate};

// Temporal validation
let past_rule = rules::past();      // Must be in the past
let future_rule = rules::future();  // Must be in the future

let yesterday = Utc::now() - Duration::days(1);
assert!(past_rule.apply(&yesterday).is_empty());

// Temporal ranges
let deadline = Utc::now() + Duration::days(30);
let before_rule = rules::before(deadline);
let after_rule = rules::after(Utc::now());

// Age verification from birth date
let age_rule = rules::age_range(18, 120);
let birth_date = NaiveDate::from_ymd_opt(2000, 6, 15).unwrap();
assert!(age_rule.apply(&birth_date).is_empty()); // Age: 25
```

**Rules added**:
- `past()` - Validates datetime is in the past
- `future()` - Validates datetime is in the future
- `before(limit)` - Validates datetime is before limit
- `after(limit)` - Validates datetime is after limit
- `age_range(min, max)` - Validates age from birth date

**Use cases**: Birth dates, event scheduling, deadlines, age verification, temporal constraints.

**Feature flag**: Add `features = ["chrono"]` to enable date/time rules.

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
// send_email(email); // [x] Compile error: expected Email<Validated>

let validated = email.validate()?;
send_email(validated); // Compiles!
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

[1.0.0]: https://github.com/blackwell-systems/domainstack/compare/v0.4.0...v1.0.0
[0.4.0]: https://github.com/blackwell-systems/domainstack/compare/v0.3.0...v0.4.0

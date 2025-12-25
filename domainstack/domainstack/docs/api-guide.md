# API Guide

Complete guide to using domainstack for domain validation.

## Table of Contents

- [Core Concepts](#core-concepts)
- [Manual Validation](#manual-validation)
- [Derive Macro](#derive-macro)
- [Error Handling](#error-handling)
- [Validation Rules](#validation-rules)
- [Date/Time Validation](#datetime-validation)
- [Code Generation (CLI)](#code-generation-cli)
- [Advanced Patterns](#advanced-patterns)
  - [Cross-Field Validation](#cross-field-validation)
  - [Conditional Validation](#conditional-validation)
  - [Validation with Context](#validation-with-context)
  - [Async Validation](#async-validation)
  - [Type-State Validation](#type-state-validation)
  - [Builder-Style Rule Customization](#builder-style-rule-customization)

## Integration Guides

- **[Derive Macro](DERIVE_MACRO.md)** - Complete guide to `#[derive(Validate)]` and `#[validate(...)]` attributes
- **[Serde Integration](SERDE_INTEGRATION.md)** - Validate on deserialize
- **[OpenAPI Schema Generation](OPENAPI_SCHEMA.md)** - Auto-generate schemas from validation rules
- **[HTTP Integration](HTTP_INTEGRATION.md)** - Axum, Actix-web, and Rocket adapters

## Core Concepts

### Valid-by-Construction Types

Domain types that enforce validity at construction time:

```rust
use domainstack::prelude::*;

pub struct Email(String);

impl Email {
    pub fn new(raw: String) -> Result<Self, ValidationError> {
        let rule = rules::email().and(rules::max_len(255));
        validate("email", raw.as_str(), &rule)?;
        Ok(Self(raw))
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

// Usage
let email = Email::new("user@example.com".to_string())?;
// Invalid email cannot exist!
```

### Structured Error Paths

Errors include precise field paths:

```rust
// Simple path
Path::from("email")              // "email"

// Nested path
Path::root()
    .field("guest")
    .field("email")              // "guest.email"

// Collection path
Path::root()
    .field("rooms")
    .index(0)
    .field("adults")             // "rooms[0].adults"
```

### Validation Rules

Rules are composable and type-safe:

```rust
use domainstack::rules::*;

// Basic rules
let r1 = email();
let r2 = min_len(5);
let r3 = max_len(255);
let r4 = range(18, 120);

// Composition
let email_rule = email().and(max_len(255));
let name_rule = min_len(1).and(max_len(50));
let age_rule = range(18, 120);

// Conditional
let optional_rule = some_rule.when(|value| should_validate(value));
```

## Manual Validation

### Implementing Validate Trait

```rust
use domainstack::prelude::*;

pub struct User {
    pub name: String,
    pub email: Email,
    pub age: u8,
}

impl Validate for User {
    fn validate(&self) -> Result<(), ValidationError> {
        let mut err = ValidationError::new();
        
        // Validate name
        let name_rule = rules::min_len(1).and(rules::max_len(50));
        if let Err(e) = validate("name", self.name.as_str(), &name_rule) {
            err.extend(e);
        }
        
        // Validate nested email
        if let Err(e) = self.email.validate() {
            err.merge_prefixed("email", e);
        }
        
        // Validate age
        let age_rule = rules::range(18, 120);
        if let Err(e) = validate("age", &self.age, &age_rule) {
            err.extend(e);
        }
        
        if err.is_empty() {
            Ok(())
        } else {
            Err(err)
        }
    }
}
```

### Validating Collections

```rust
impl Validate for Team {
    fn validate(&self) -> Result<(), ValidationError> {
        let mut err = ValidationError::new();
        
        // Validate each member
        for (i, member) in self.members.iter().enumerate() {
            if let Err(e) = member.validate() {
                let path = Path::root().field("members").index(i);
                err.merge_prefixed(path, e);
            }
        }
        
        if err.is_empty() { Ok(()) } else { Err(err) }
    }
}
```

## Derive Macro

The `#[derive(Validate)]` macro automatically implements validation for your structs. Use `#[validate(...)]` attributes to declare rules.

**Quick example:**

```rust
#[derive(Validate)]
struct User {
    #[validate(length(min = 1, max = 50))]
    name: String,

    #[validate(range(min = 18, max = 120))]
    age: u8,
}
```

**For complete documentation, see [DERIVE_MACRO.md](DERIVE_MACRO.md)** covering:
- Basic attributes (length, range, email, url, etc.)
- Nested validation with `#[validate(nested)]`
- Collection validation with `each(nested)` and `each(rule)`
- Cross-field validation with struct-level `#[validate(check = "...")]`
- Conditional validation with `when` parameter
- Custom validation with `#[validate(custom = "...")]`

## Error Handling

### ValidationError API

```rust
// Create error
let mut err = ValidationError::new();

// Add violation
err.push("email", "invalid_email", "Invalid email format");

// Extend with another error
err.extend(other_error);

// Merge with path prefix
err.merge_prefixed("guest", nested_error);

// Transform paths
let prefixed = err.prefixed("booking");

// Check if empty
if err.is_empty() { /* ... */ }

// Access violations
for v in &err.violations {
    println!("{}: {}", v.path, v.message);
}

// Get field map
let map = err.field_errors_map();  // BTreeMap<String, Vec<String>>
let detailed = err.field_violations_map();  // BTreeMap<String, Vec<&Violation>>
```

### Violation Structure

```rust
pub struct Violation {
    pub path: Path,           // Field path (e.g., "guest.email")
    pub code: &'static str,   // Machine-readable code (e.g., "invalid_email")
    pub message: String,      // Human-readable message
    pub meta: Meta,           // Additional context (min, max, etc.)
}
```

### Meta Fields

```rust
let mut meta = Meta::new();
meta.insert("min", 18);
meta.insert("max", 120);

// Check if empty
if meta.is_empty() { /* ... */ }

// Iterate
for (key, value) in meta.iter() {
    println!("{}: {}", key, value);
}

// Get specific value
if let Some(min) = meta.get("min") {
    println!("Minimum: {}", min);
}
```

## Validation Rules

### Complete Rules Reference

domainstack provides 37+ built-in validation rules across 5 categories.

**For complete documentation of all rules, see [`RULES.md`](./RULES.md)**

### String Rules (17 rules)

```rust
use domainstack::rules::*;

// Format validation
email()                              // Valid email address
url()                                // Valid URL

// Length validation
min_len(n)                          // Minimum string length
max_len(n)                          // Maximum string length
length(min, max)                    // String length range
non_empty()                         // Not empty (length >= 1)
non_blank()                         // Not blank after trimming
len_chars(min, max)                 // Unicode character count

// Pattern validation
alphanumeric()                      // Letters and numbers only
alpha_only()                        // Letters only
numeric_string()                    // Digits only
ascii()                             // ASCII characters only
no_whitespace()                     // No whitespace allowed

// Content validation
starts_with(prefix)                 // Must start with prefix
ends_with(suffix)                   // Must end with suffix
contains(substring)                 // Must contain substring
matches_regex(pattern)              // Custom regex pattern
```

### Numeric Rules (8 rules)

```rust
// Range validation
range(min, max)                     // Numeric range
min(n)                              // Minimum value
max(n)                              // Maximum value

// Sign validation
positive()                          // Greater than zero
negative()                          // Less than zero
non_zero()                          // Cannot be zero

// Other
multiple_of(n)                      // Must be multiple of n
finite()                            // Not NaN or Infinity (floats)
```

**Important:** Always use `finite()` with float validation to catch NaN/Infinity values.

```rust
#[derive(Validate)]
struct Measurement {
    #[validate(range(min = 0.0, max = 100.0))]
    #[validate(finite)]  // ✅ Catches NaN and Infinity
    value: f64,
}
```

### Choice Rules (3 rules)

```rust
// Equality checking
equals(value)                       // Must equal specific value
not_equals(value)                   // Must not equal value
one_of(&[values])                   // Must be in allowed set

// Example: Status validation
#[derive(Validate)]
struct Order {
    #[validate(one_of(&["pending", "shipped", "delivered", "cancelled"]))]
    status: String,
}
```

### Collection Rules (4 rules)

```rust
// Size validation
min_items(n)                        // Minimum collection size
max_items(n)                        // Maximum collection size

// Content validation
unique()                            // All items must be unique
non_empty_items()                   // No empty string items (for Vec<String>)

// Example
#[derive(Validate)]
struct BlogPost {
    #[validate(min_items(1))]
    #[validate(max_items(10))]
    #[validate(unique)]
    tags: Vec<String>,
}
```

### Date/Time Rules (5 rules)

**Feature flag:** `chrono`

```rust
use chrono::{NaiveDate, DateTime, Utc};

// Temporal validation
past()                              // Must be in the past
future()                            // Must be in the future
before(limit)                       // Must be before limit
after(limit)                        // Must be after limit

// Age calculation
age_range(min_years, max_years)     // Age from birth date

// Example
#[derive(Validate)]
struct Event {
    #[validate(after(Utc::now()))]
    start_date: DateTime<Utc>,

    #[validate(before(Utc::now() + Duration::days(365)))]
    end_date: DateTime<Utc>,
}
```

### Builder-Style Rule Customization

Customize error codes and messages for any rule:

```rust
use domainstack::rules::*;

let rule = email()
    .code("custom_email_error")
    .message("Please provide a valid company email address")
    .meta("hint", "Use your @company.com address");

// Works with all rules
let age_rule = range(18, 65)
    .code("invalid_age")
    .message("Age must be between 18 and 65 for this program")
    .meta("min", "18")
    .meta("max", "65");

// Useful for internationalization
let localized_rule = min_len(3)
    .message(get_i18n_message("validation.min_length", locale));
```

**Available customization methods:**
- `.code(code)` - Custom error code
- `.message(msg)` - Custom error message
- `.meta(key, value)` - Add metadata field

### Rule Composition

Combine rules with logical operators:

```rust
// AND - both rules must pass
let rule = email().and(max_len(255));

// WHEN - conditional validation
let rule = some_rule.when(|value| should_validate(value));

// Multiple conditions
#[derive(Validate)]
struct User {
    #[validate(length(min = 3, max = 50))]
    #[validate(alphanumeric)]
    #[validate(no_whitespace)]
    username: String,
}
```

_For HTTP framework integration (Axum, Actix-web, Rocket), see **[HTTP_INTEGRATION.md](HTTP_INTEGRATION.md)**_

## Date/Time Validation

**Requires `chrono` feature flag**

Date and time validation is essential for domain modeling - user registration, event scheduling, deadlines, age verification, and temporal constraints.

### Basic Temporal Validation

```rust
use domainstack::prelude::*;
use chrono::{Utc, Duration, NaiveDate};

// Birth dates must be in the past
let birth_date_rule = rules::past();
let birth_date = Utc::now() - Duration::days(365 * 25);  // 25 years ago
validate("birth_date", &birth_date, &birth_date_rule)?;

// Event dates must be in the future
let event_rule = rules::future();
let event_date = Utc::now() + Duration::days(30);  // 30 days from now
validate("event_date", &event_date, &event_rule)?;
```

### Temporal Range Constraints

```rust
use chrono::NaiveDate;

// Registration deadline - must be before cutoff
let deadline = NaiveDate::from_ymd_opt(2025, 12, 31)
    .unwrap()
    .and_hms_opt(23, 59, 59)
    .unwrap()
    .and_utc();

let before_rule = rules::before(deadline);
validate("registration_date", &registration_date, &before_rule)?;

// Event must be after start date
let start_date = NaiveDate::from_ymd_opt(2025, 6, 1)
    .unwrap()
    .and_hms_opt(0, 0, 0)
    .unwrap()
    .and_utc();

let after_rule = rules::after(start_date);
validate("event_date", &event_date, &after_rule)?;
```

### Age Verification

```rust
use chrono::NaiveDate;

// User must be 18-120 years old
let age_rule = rules::age_range(18, 120);

// Birth date for someone 25 years old
let birth_date = NaiveDate::from_ymd_opt(Utc::now().year() - 25, 6, 15).unwrap();
validate("birth_date", &birth_date, &age_rule)?;  // ✓ Valid

// Too young
let minor_birth_date = NaiveDate::from_ymd_opt(Utc::now().year() - 10, 6, 15).unwrap();
let result = validate("birth_date", &minor_birth_date, &age_rule);
// ✗ Error: age_out_of_range
```

### Temporal Window Validation

```rust
// Event must be within a specific window (30-60 days from now)
let start = Utc::now() + Duration::days(30);
let end = Utc::now() + Duration::days(60);

let window_rule = rules::after(start).and(rules::before(end));

let valid_event = Utc::now() + Duration::days(45);
validate("event_date", &valid_event, &window_rule)?;  // ✓ Valid
```

### Domain Model with Date/Time

```rust
use domainstack::prelude::*;
use chrono::{DateTime, Utc, NaiveDate};

pub struct UserRegistration {
    birth_date: NaiveDate,
    registration_date: DateTime<Utc>,
}

impl UserRegistration {
    pub fn new(birth_date: NaiveDate, registration_date: DateTime<Utc>)
        -> Result<Self, ValidationError>
    {
        let mut err = ValidationError::new();

        // Birth date must be in the past and age 18-120
        let age_rule = rules::age_range(18, 120);
        if let Err(e) = validate("birth_date", &birth_date, &age_rule) {
            err.extend(e);
        }

        // Registration date must be in the past (already happened)
        let past_rule = rules::past();
        if let Err(e) = validate("registration_date", &registration_date, &past_rule) {
            err.extend(e);
        }

        if !err.is_empty() {
            return Err(err);
        }

        Ok(Self { birth_date, registration_date })
    }
}

pub struct Event {
    name: String,
    start_date: DateTime<Utc>,
    end_date: DateTime<Utc>,
}

impl Event {
    pub fn new(name: String, start_date: DateTime<Utc>, end_date: DateTime<Utc>)
        -> Result<Self, ValidationError>
    {
        let mut err = ValidationError::new();

        // Start date must be in the future
        let future_rule = rules::future();
        if let Err(e) = validate("start_date", &start_date, &future_rule) {
            err.extend(e);
        }

        // End date must be after start date
        let after_start = rules::after(start_date);
        if let Err(e) = validate("end_date", &end_date, &after_start) {
            err.extend(e);
        }

        if !err.is_empty() {
            return Err(err);
        }

        Ok(Self { name, start_date, end_date })
    }
}
```

### Error Handling for Date/Time

```rust
use chrono::{NaiveDate, Utc};

let age_rule = rules::age_range(18, 120);
let birth_date = NaiveDate::from_ymd_opt(Utc::now().year() - 10, 6, 15).unwrap();

match validate("birth_date", &birth_date, &age_rule) {
    Ok(_) => println!("Valid age"),
    Err(e) => {
        for v in &e.violations {
            println!("Error at {}: {}", v.path, v.message);
            // Access metadata
            if let Some(age) = v.meta.get("age") {
                println!("Actual age: {}", age);
            }
            if let Some(min) = v.meta.get("min") {
                println!("Minimum age: {}", min);
            }
        }
        // Output:
        // Error at birth_date: Age must be between 18 and 120 years
        // Actual age: 10
        // Minimum age: 18
    }
}
```

## Code Generation (CLI)

### TypeScript/Zod Schema Generation

**Tool:** `domainstack-cli`

Generate TypeScript Zod validation schemas from Rust validation rules - single source of truth for frontend and backend validation.

```bash
# Install the CLI
cargo install domainstack-cli

# Generate Zod schemas from Rust types
domainstack zod --input src --output frontend/src/schemas.ts
```

**From Rust:**

```rust
#[derive(Validate)]
struct User {
    #[validate(email)]
    #[validate(max_len = 255)]
    email: String,

    #[validate(range(min = 18, max = 120))]
    age: u8,

    #[validate(url)]
    website: Option<String>,
}
```

**Generates TypeScript/Zod:**

```typescript
// frontend/src/schemas.ts (AUTO-GENERATED)
import { z } from "zod";

export const UserSchema = z.object({
  email: z.string().email().max(255),
  age: z.number().min(18).max(120),
  website: z.string().url().optional(),
});

export type User = z.infer<typeof UserSchema>;
```

**Supported Validation Rules:**

26+ validation rules are supported:
- ✅ All string validations (email, url, length, patterns)
- ✅ All numeric validations (range, min/max, positive/negative)
- ✅ Optional fields with correct `.optional()` ordering
- ✅ Arrays (`Vec<T>` → `z.array(T)`)
- ✅ Nested types with references
- ✅ Custom type references

**Benefits:**

- **Single source of truth** - Change validation once, regenerate schemas
- **Frontend/backend in sync** - Guaranteed consistency
- **Zero maintenance** - No manual schema writing
- **Type-safe** - Zod's type inference works automatically

**CLI Usage:**

```bash
# Basic usage
domainstack zod --output schemas.ts

# Custom input directory
domainstack zod --input backend/src --output frontend/schemas.ts

# Verbose output
domainstack zod -i src -o schemas.ts -v
```

**Integration with CI/CD:**

```yaml
# .github/workflows/ci.yml
- name: Generate Zod schemas
  run: domainstack zod --input src --output frontend/src/schemas.ts

- name: Check for uncommitted changes
  run: |
    git diff --exit-code frontend/src/schemas.ts || \
      (echo "❌ Schemas out of date! Run: npm run codegen" && exit 1)
```

**Example: Full-Stack Validation**

```rust
// Backend: src/models.rs
#[derive(Validate)]
struct CreateUserRequest {
    #[validate(email)]
    email: String,

    #[validate(length(min = 3, max = 50))]
    #[validate(alphanumeric)]
    username: String,

    #[validate(range(min = 18, max = 120))]
    age: u8,
}
```

```typescript
// Frontend: Generated automatically
const result = CreateUserRequestSchema.safeParse(formData);
if (result.success) {
  // Type-safe validated data
  const request: CreateUserRequest = result.data;
  await api.createUser(request);
} else {
  // Display field-level errors
  displayErrors(result.error);
}
```

**Future Generators (Planned):**

- `domainstack yup` - Yup schemas for React ecosystem
- `domainstack graphql` - GraphQL SDL generation
- `domainstack prisma` - Prisma schemas with validation
- `domainstack json-schema` - JSON Schema generation

**See also:**
- Complete guide: `domainstack-cli/README.md`
- 32 unit tests with 100% pass rate
- CHANGELOG for v0.1.0 feature details

## Advanced Patterns

### Cross-Field Validation

Validate relationships between fields using struct-level `#[validate(check = "...")]` attributes.

**For complete documentation, see [DERIVE_MACRO.md](DERIVE_MACRO.md#cross-field-validation)** covering:
- Basic cross-field rules with `check` parameter
- Multiple cross-field validations
- Conditional cross-field validation with `when` parameter
- Password confirmation example
- Manual implementation for complex logic

### Conditional Validation

```rust
impl Validate for Order {
    fn validate(&self) -> Result<(), ValidationError> {
        let mut err = ValidationError::new();
        
        // Only validate shipping address if shipped
        if self.requires_shipping {
            if let Err(e) = self.shipping_address.validate() {
                err.merge_prefixed("shipping_address", e);
            }
        }
        
        if err.is_empty() { Ok(()) } else { Err(err) }
    }
}
```

### Validation with Context

For complex validations requiring external state:

```rust
pub struct ValidationContext {
    pub existing_emails: HashSet<String>,
}

impl User {
    pub fn validate_with_context(
        &self,
        ctx: &ValidationContext
    ) -> Result<(), ValidationError> {
        let mut err = ValidationError::new();
        
        // Basic validation
        if let Err(e) = self.validate() {
            err.extend(e);
        }
        
        // Context-dependent validation
        if ctx.existing_emails.contains(&self.email.value) {
            err.push(
                "email",
                "email_taken",
                "Email already exists"
            );
        }
        
        if err.is_empty() { Ok(()) } else { Err(err) }
    }
}
```

### Async Validation

**Feature:** Async validation with `AsyncValidate` trait for database queries, API calls, and external service checks.

Use async validation when you need to check constraints that require I/O operations - database uniqueness, external API validation, rate limiting, etc.

#### AsyncValidate Trait

```rust
use domainstack::{AsyncValidate, ValidationError, ValidationContext, Path};
use async_trait::async_trait;

#[async_trait]
pub trait AsyncValidate {
    async fn validate_async(&self, ctx: &ValidationContext) -> Result<(), ValidationError>;
}
```

#### Database Uniqueness Check

```rust
use domainstack::{AsyncValidate, ValidationError, ValidationContext, Path};
use async_trait::async_trait;
use sqlx::{PgPool, query};

pub struct User {
    pub email: String,
    pub username: String,
    pub age: u8,
}

#[async_trait]
impl AsyncValidate for User {
    async fn validate_async(&self, ctx: &ValidationContext) -> Result<(), ValidationError> {
        let mut err = ValidationError::new();

        // Sync validation first
        if let Err(e) = self.validate() {
            err.extend(e);
        }

        // Get database connection from context
        let db = ctx.get_resource::<PgPool>("db")?;

        // Check email uniqueness
        let email_exists = query!("SELECT id FROM users WHERE email = $1", self.email)
            .fetch_optional(db)
            .await?;

        if email_exists.is_some() {
            err.push(
                Path::from("email"),
                "email_taken",
                "Email is already registered"
            );
        }

        // Check username uniqueness
        let username_exists = query!("SELECT id FROM users WHERE username = $1", self.username)
            .fetch_optional(db)
            .await?;

        if username_exists.is_some() {
            err.push(
                Path::from("username"),
                "username_taken",
                "Username is already taken"
            );
        }

        if err.is_empty() { Ok(()) } else { Err(err) }
    }
}
```

#### Using ValidationContext

Pass external resources (database pools, HTTP clients, caches) via `ValidationContext`:

```rust
use domainstack::ValidationContext;
use sqlx::PgPool;

// Create context with resources
let mut ctx = ValidationContext::new();
ctx.insert_resource("db", db_pool.clone());
ctx.insert_resource("cache", redis_client.clone());

// Run async validation
let user = User {
    email: "user@example.com".to_string(),
    username: "johndoe".to_string(),
    age: 25,
};

user.validate_async(&ctx).await?;  // ✓ or ✗ with field-level errors
```

#### Axum Integration with Async Validation

```rust
use axum::{extract::State, Json};
use domainstack::{AsyncValidate, ValidationContext};
use domainstack_axum::ErrorResponse;
use sqlx::PgPool;

async fn create_user(
    State(db): State<PgPool>,
    Json(user): Json<User>
) -> Result<Json<User>, ErrorResponse> {
    // Create validation context
    let mut ctx = ValidationContext::new();
    ctx.insert_resource("db", db.clone());

    // Async validation (checks uniqueness)
    user.validate_async(&ctx)
        .await
        .map_err(|e| ErrorResponse::from(e))?;

    // User is valid and unique - proceed with insertion
    let created = insert_user(&db, user).await?;
    Ok(Json(created))
}
```

#### External API Validation

Validate data against external services:

```rust
use domainstack::{AsyncValidate, ValidationError, ValidationContext, Path};
use async_trait::async_trait;
use reqwest::Client;

pub struct VATRegistration {
    pub vat_number: String,
    pub country_code: String,
}

#[async_trait]
impl AsyncValidate for VATRegistration {
    async fn validate_async(&self, ctx: &ValidationContext) -> Result<(), ValidationError> {
        let http_client = ctx.get_resource::<Client>("http_client")?;

        // Call EU VAT validation API
        let response = http_client
            .get(&format!(
                "https://vat-api.eu/check/{}/{}",
                self.country_code, self.vat_number
            ))
            .send()
            .await?;

        let result: VATResponse = response.json().await?;

        if !result.is_valid {
            return Err(ValidationError::single(
                Path::from("vat_number"),
                "invalid_vat",
                "VAT number is not valid"
            ));
        }

        Ok(())
    }
}
```

#### Rate Limiting with Async Validation

```rust
use domainstack::{AsyncValidate, ValidationError, ValidationContext, Path};
use async_trait::async_trait;
use redis::AsyncCommands;

pub struct LoginAttempt {
    pub email: String,
    pub password: String,
    pub ip_address: String,
}

#[async_trait]
impl AsyncValidate for LoginAttempt {
    async fn validate_async(&self, ctx: &ValidationContext) -> Result<(), ValidationError> {
        let mut redis = ctx.get_resource::<redis::Client>("redis")?.get_async_connection().await?;

        // Check rate limit (max 5 attempts per 15 minutes)
        let key = format!("login_attempts:{}", self.ip_address);
        let attempts: i32 = redis.get(&key).await.unwrap_or(0);

        if attempts >= 5 {
            return Err(ValidationError::single(
                Path::root(),
                "rate_limited",
                "Too many login attempts. Please try again later."
            ));
        }

        // Increment attempt counter
        redis.incr(&key, 1).await?;
        redis.expire(&key, 900).await?;  // 15 minutes

        Ok(())
    }
}
```

#### Combining Sync and Async Validation

Best practice: Run synchronous validation first (fast), then async validation (slow):

```rust
#[async_trait]
impl AsyncValidate for User {
    async fn validate_async(&self, ctx: &ValidationContext) -> Result<(), ValidationError> {
        let mut err = ValidationError::new();

        // 1. Synchronous validation (instant)
        //    Check format, length, ranges, etc.
        if let Err(e) = self.validate() {
            err.extend(e);
            // Early return if basic validation fails
            // No need to hit database if email format is invalid!
            return Err(err);
        }

        // 2. Async validation (I/O required)
        //    Check uniqueness, external APIs, etc.
        let db = ctx.get_resource::<PgPool>("db")?;

        let email_exists = query!("SELECT id FROM users WHERE email = $1", self.email)
            .fetch_optional(db)
            .await?;

        if email_exists.is_some() {
            err.push(
                Path::from("email"),
                "email_taken",
                "Email is already registered"
            );
        }

        if err.is_empty() { Ok(()) } else { Err(err) }
    }
}
```

#### Error Handling

```rust
match user.validate_async(&ctx).await {
    Ok(_) => {
        println!("✓ User is valid and unique!");
    }
    Err(e) => {
        for violation in &e.violations {
            println!("Error at {}: {}", violation.path, violation.message);
        }
        // Example output:
        // Error at email: Email is already registered
        // Error at username: Username is already taken
    }
}
```

**Benefits of Async Validation:**

- **Database integrity** - Prevent duplicate records before insertion
- **External validation** - Verify data with third-party APIs
- **Rate limiting** - Protect against abuse
- **Real-time checks** - Validate against live data sources
- **Clean error messages** - Field-level errors just like sync validation

**Performance Tip:** Always run sync validation first to fail fast on basic errors before expensive I/O operations.

### Type-State Validation

**Feature:** Compile-time validation guarantees using phantom types

Use the type system to enforce that data has been validated, preventing use of unvalidated data in critical operations.

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
        Self {
            value,
            _state: PhantomData,
        }
    }

    pub fn validate(self) -> Result<Email<Validated>, ValidationError> {
        validate("email", self.value.as_str(), &rules::email())?;
        Ok(Email {
            value: self.value,
            _state: PhantomData,
        })
    }
}

impl Email<Validated> {
    pub fn as_str(&self) -> &str {
        &self.value
    }
}

// Only accept validated emails!
fn send_email(email: Email<Validated>) {
    // Compiler GUARANTEES email is validated!
    println!("Sending to: {}", email.as_str());
}

// Usage
let email = Email::new("user@example.com".to_string());
// send_email(email); // ❌ Compile error: expected Email<Validated>

let validated = email.validate()?;
send_email(validated); // ✅ Compiles!
```

**Benefits:**

- **Zero runtime cost** - PhantomData has size 0, no memory or CPU overhead
- **Compile-time safety** - Type system enforces validation occurred
- **Self-documenting** - Function signatures make validation requirements explicit
- **Builder pattern friendly** - Natural fit with builder APIs

**Multi-Field Example:**

```rust
use domainstack::typestate::{Validated, Unvalidated};
use std::marker::PhantomData;

pub struct User<State = Unvalidated> {
    pub email: String,
    pub username: String,
    pub age: u8,
    _state: PhantomData<State>,
}

impl User<Unvalidated> {
    pub fn new(email: String, username: String, age: u8) -> Self {
        Self {
            email,
            username,
            age,
            _state: PhantomData,
        }
    }

    pub fn validate(self) -> Result<User<Validated>, ValidationError> {
        let mut err = ValidationError::new();

        if let Err(e) = validate("email", &self.email, &rules::email()) {
            err.extend(e);
        }
        if let Err(e) = validate("username", &self.username,
                                 &rules::length(3, 50).and(rules::alphanumeric())) {
            err.extend(e);
        }
        if let Err(e) = validate("age", &self.age, &rules::range(18, 120)) {
            err.extend(e);
        }

        if err.is_empty() {
            Ok(User {
                email: self.email,
                username: self.username,
                age: self.age,
                _state: PhantomData,
            })
        } else {
            Err(err)
        }
    }
}

// Database operations require validated users
async fn insert_user(db: &Database, user: User<Validated>) -> Result<i64> {
    // Type system guarantees user is validated!
    db.insert(user.email, user.username, user.age).await
}
```

**Builder Pattern Integration:**

```rust
pub struct UserBuilder<State = Unvalidated> {
    email: Option<String>,
    username: Option<String>,
    age: Option<u8>,
    _state: PhantomData<State>,
}

impl UserBuilder<Unvalidated> {
    pub fn new() -> Self {
        Self {
            email: None,
            username: None,
            age: None,
            _state: PhantomData,
        }
    }

    pub fn email(mut self, email: String) -> Self {
        self.email = Some(email);
        self
    }

    pub fn username(mut self, username: String) -> Self {
        self.username = Some(username);
        self
    }

    pub fn age(mut self, age: u8) -> Self {
        self.age = Some(age);
        self
    }

    pub fn build(self) -> Result<UserBuilder<Validated>, ValidationError> {
        // Validate all fields
        let user = User::new(
            self.email.ok_or_else(|| ValidationError::single(
                Path::from("email"), "required", "Email is required"
            ))?,
            self.username.ok_or_else(|| ValidationError::single(
                Path::from("username"), "required", "Username is required"
            ))?,
            self.age.ok_or_else(|| ValidationError::single(
                Path::from("age"), "required", "Age is required"
            ))?,
        );

        user.validate()?;

        Ok(UserBuilder {
            email: Some(user.email),
            username: Some(user.username),
            age: Some(user.age),
            _state: PhantomData,
        })
    }
}

impl UserBuilder<Validated> {
    pub fn into_user(self) -> User<Validated> {
        User {
            email: self.email.unwrap(),
            username: self.username.unwrap(),
            age: self.age.unwrap(),
            _state: PhantomData,
        }
    }
}
```

**Use Cases:**

- Database operations requiring validated data
- Business logic with validation boundaries
- Multi-step workflows with validation gates
- API handlers ensuring data is validated before processing
- Builder patterns with validation as final step

**See also:**
- Complete module documentation: `domainstack/src/typestate.rs`
- Example: `domainstack/examples/phantom_types.rs`
- 9 unit tests demonstrating all patterns

## Best Practices

1. **Use derive macro for simple cases** - Less boilerplate
2. **Manual implementation for complex logic** - Cross-field, conditional
3. **Compose rules** - Build reusable validation components
4. **Structured error paths** - Use Path API, not string formatting
5. **Framework-agnostic core** - Keep domain logic separate from HTTP
6. **One validation point** - Validate at domain boundaries, not everywhere
7. **Use error-envelope for HTTP** - Automatic structured responses
8. **Custom functions for domain rules** - Encapsulate business logic

## See Also

- [Rules Reference](./RULES.md) - Complete list of built-in rules
- [Examples](../domainstack/examples/) - Runnable code examples
- [API Documentation](https://docs.rs/domainstack) - Full API reference

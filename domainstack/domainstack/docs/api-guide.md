# API Guide

Complete guide to using domainstack for domain validation.

## Table of Contents

- [Quick Reference](#quick-reference)
- [Integration Guides](#integration-guides)
- [Core Concepts](#core-concepts)
- [Manual Validation](#manual-validation)
- [Error Handling](#error-handling)
- [Validation Rules](#validation-rules)
- [Advanced Patterns](#advanced-patterns)
- [Best Practices](#best-practices)

## Quick Reference

```rust
use domainstack::prelude::*;

// 1. Validate a single value
validate("email", email, &rules::email())?;

// 2. Derive validation for structs
#[derive(Validate)]
struct User {
    #[validate(email, max_len = 255)]
    email: String,

    #[validate(range(min = 18, max = 120))]
    age: u8,
}

// 3. Manual implementation for complex logic
impl Validate for CustomType {
    fn validate(&self) -> Result<(), ValidationError> {
        // Custom validation logic
    }
}

// 4. Use in HTTP handlers
async fn create_user(
    DomainJson(dto, user): DomainJson<UserDto, User>
) -> Result<Json<User>> {
    // user is already validated!
    Ok(Json(user))
}
```

## Integration Guides

Detailed guides for specific features and integrations:

- **[Core Concepts](CORE_CONCEPTS.md)** - Foundation principles: valid-by-construction types, structured error paths, composable rules, domain vs DTO separation, smart constructors
- **[Manual Validation](MANUAL_VALIDATION.md)** - When and how to implement the `Validate` trait manually for custom validation logic
- **[Error Handling](ERROR_HANDLING.md)** - Working with `ValidationError`, `Violation`, message transformation, and HTTP integration
- **[Derive Macro](DERIVE_MACRO.md)** - Complete guide to `#[derive(Validate)]` and `#[validate(...)]` attributes
- **[Validation Rules](RULES.md)** - Complete reference for all 37 built-in validation rules
- **[Advanced Patterns](ADVANCED_PATTERNS.md)** - Async validation, type-state validation, context-dependent validation, and more
- **[Serde Integration](SERDE_INTEGRATION.md)** - Validate on deserialize with `ValidateOnDeserialize`
- **[OpenAPI Schema Generation](OPENAPI_SCHEMA.md)** - Auto-generate schemas from validation rules
- **[HTTP Integration](HTTP_INTEGRATION.md)** - Axum, Actix-web, and Rocket adapters
- **[CLI Guide](CLI_GUIDE.md)** - Generate TypeScript/Zod schemas with domainstack-cli

## Core Concepts

domainstack is built on three foundation principles: **valid-by-construction types**, **structured error paths**, and **composable rules**.

### Valid-by-Construction Types

Domain types that enforce validity at construction time—invalid states become impossible to represent:

```rust
pub struct Email(String);

impl Email {
    pub fn new(raw: String) -> Result<Self, ValidationError> {
        let rule = rules::email().and(rules::max_len(255));
        validate("email", raw.as_str(), &rule)?;
        Ok(Self(raw))  // ✓ If this succeeds, email is GUARANTEED valid
    }
}
```

### Structured Error Paths

Type-safe error paths that map directly to form fields:

```rust
// Simple: "email"
// Nested: "guest.email"
// Collection: "rooms[0].adults"

let path = Path::root()
    .field("rooms")
    .index(0)
    .field("adults");
```

### Composable Rules

Rules are values that compose with `.and()`, `.or()`, `.when()`:

```rust
let email_rule = rules::email().and(rules::max_len(255));
let age_rule = rules::range(18, 120);
let optional_url = rules::url().when(|s: &String| !s.is_empty());
```

**For complete documentation, see [CORE_CONCEPTS.md](CORE_CONCEPTS.md)** covering:
- Newtype pattern and smart constructors
- Structs with private fields
- Path API and transformations
- Rules as values and composition operators
- Domain vs DTO separation with `TryFrom`

## Manual Validation

Implement the `Validate` trait manually when you need custom validation logic beyond declarative rules.

### When to Use Manual Validation

- **Newtype wrappers** - Tuple structs like `Email(String)`
- **Complex business logic** - Multi-field calculations, conditional validation
- **Custom error messages** - Fine-grained control over codes and messages
- **Dynamic validation rules** - Runtime-determined validation

### Basic Pattern

```rust
impl Validate for User {
    fn validate(&self) -> Result<(), ValidationError> {
        let mut err = ValidationError::new();

        // Validate each field
        if let Err(e) = validate("name", &self.name, &name_rule) {
            err.extend(e);
        }

        // Validate nested types
        if let Err(e) = self.email.validate() {
            err.merge_prefixed("email", e);
        }

        if err.is_empty() { Ok(()) } else { Err(err) }
    }
}
```

**For complete documentation, see [MANUAL_VALIDATION.md](MANUAL_VALIDATION.md)** covering:
- Step-by-step implementation pattern
- Validating collections with array indices
- Merging nested errors: `extend()` vs `merge_prefixed()`
- Manual vs Derive comparison
- Custom validation functions and builder pattern integration

## Error Handling

`ValidationError` accumulates violations with structured paths, error codes, messages, and metadata.

### Basic API

```rust
// Create and add violations
let mut err = ValidationError::new();
err.push("email", "invalid_email", "Invalid email format");

// Merge nested errors
err.merge_prefixed("guest", nested_error);

// Extract violations
for v in &err.violations {
    println!("[{}] {}: {}", v.code, v.path, v.message);
}

// Get field map
let map = err.field_violations_map();  // Preserves codes + meta
```

### Violation Structure

```rust
pub struct Violation {
    pub path: Path,           // "guest.email", "rooms[0].adults"
    pub code: &'static str,   // "invalid_email", "out_of_range"
    pub message: String,      // Human-readable message
    pub meta: Meta,           // Additional context (min, max, etc.)
}
```

**For complete documentation, see [ERROR_HANDLING.md](ERROR_HANDLING.md)** covering:
- ValidationError construction and manipulation
- Adding and merging violations
- Extracting error information (maps, iteration)
- Message transformation for i18n
- HTTP integration with structured error format
- Best practices for error codes and paths

## Validation Rules

domainstack provides **37 built-in validation rules** across 5 categories:

- **String (17)** - email, url, length, pattern matching, unicode
- **Numeric (8)** - range, min/max, sign validation, divisibility
- **Choice (3)** - equals, one_of, membership checking
- **Collection (4)** - size, uniqueness, non-empty items
- **Date/Time (5)** - past, future, before/after, age verification

### Quick Examples

```rust
use domainstack::rules::*;

// String validation
let email_rule = email().and(max_len(255));

// Numeric validation
let age_rule = range(18, 120);

// Collection validation
let tags_rule = min_items(1).and(unique());

// Customize any rule
let custom_rule = email()
    .code("invalid_company_email")
    .message("Must use company email")
    .meta("domain", "company.com");
```

**For complete documentation, see [RULES.md](RULES.md)** covering:
- All 37 rules with examples
- Rule composition (`.and()`, `.or()`, `.not()`, `.when()`)
- Builder customization (`.code()`, `.message()`, `.meta()`)
- Creating custom rules

## Advanced Patterns

Advanced validation techniques for complex use cases.

### Conditional Validation

Validate fields based on runtime conditions:

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

### Async Validation

Database uniqueness checks, API validation, rate limiting:

```rust
use domainstack::{AsyncValidate, ValidationContext};

#[async_trait]
impl AsyncValidate for User {
    async fn validate_async(&self, ctx: &ValidationContext) -> Result<(), ValidationError> {
        // Sync validation first (fast)
        self.validate()?;

        // Async validation (I/O required)
        let db = ctx.get_resource::<PgPool>("db")?;

        let email_exists = query!("SELECT id FROM users WHERE email = $1", self.email)
            .fetch_optional(db)
            .await?;

        if email_exists.is_some() {
            return Err(ValidationError::single(
                "email", "email_taken", "Email already registered"
            ));
        }

        Ok(())
    }
}
```

### Type-State Validation

Compile-time validation guarantees with phantom types:

```rust
use domainstack::typestate::{Validated, Unvalidated};

pub struct Email<State = Unvalidated> {
    value: String,
    _state: PhantomData<State>,
}

impl Email<Unvalidated> {
    pub fn validate(self) -> Result<Email<Validated>, ValidationError> {
        // Validation logic...
        Ok(Email { value: self.value, _state: PhantomData })
    }
}

// Only accept validated emails!
fn send_email(email: Email<Validated>) {
    // Compiler GUARANTEES email is validated!
}
```

**For complete documentation, see [ADVANCED_PATTERNS.md](ADVANCED_PATTERNS.md)** covering:
- Conditional validation patterns
- Validation with context (external state)
- Async validation with `AsyncValidate` trait
- Database uniqueness checks
- External API validation
- Rate limiting patterns
- Type-state validation with phantom types
- Builder pattern integration

## Best Practices

1. **Use derive macro for simple cases** - Less boilerplate for straightforward field validation
2. **Manual implementation for complex logic** - Cross-field rules, conditional validation, custom business logic
3. **Compose rules** - Build reusable validation components with `.and()`, `.or()`, `.when()`
4. **Structured error paths** - Use Path API for type-safe error paths, not string formatting
5. **Framework-agnostic core** - Keep domain logic separate from HTTP layer
6. **One validation point** - Validate at domain boundaries (DTO → Domain), not everywhere
7. **Use error-envelope for HTTP** - Automatic structured error responses
8. **Custom functions for domain rules** - Encapsulate business logic in reusable validation functions

## See Also

- **[Examples](../domainstack/examples/)** - Runnable code examples
- **[API Documentation](https://docs.rs/domainstack)** - Full API reference
- **[GitHub Repository](https://github.com/blackwell-ai/domainstack)** - Source code and issues

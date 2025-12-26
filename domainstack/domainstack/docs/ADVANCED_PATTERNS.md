# Advanced Patterns

**Overview of advanced validation techniques with links to dedicated guides.**

## Table of Contents

- [Overview](#overview)
- [Conditional Validation](#conditional-validation)
- [Async Validation](#async-validation)
- [Type-State Validation](#type-state-validation)
- [Cross-Field Validation](#cross-field-validation)
- [Collection Validation](#collection-validation)
- [Best Practices](#best-practices)

## Overview

domainstack provides several advanced patterns for complex validation scenarios:

| Pattern | Use Case | Dedicated Guide |
|---------|----------|-----------------|
| **Conditional** | Runtime-determined rules, enum variants | [CONDITIONAL_VALIDATION.md](CONDITIONAL_VALIDATION.md) |
| **Async** | Database checks, external APIs, rate limiting | [ASYNC_VALIDATION.md](ASYNC_VALIDATION.md) |
| **Type-State** | Compile-time guarantees, phantom types | [TYPE_STATE.md](TYPE_STATE.md) |
| **Cross-Field** | Date ranges, password confirmation | [CROSS_FIELD_VALIDATION.md](CROSS_FIELD_VALIDATION.md) |
| **Collection** | Arrays, vectors, `each()` patterns | [COLLECTION_VALIDATION.md](COLLECTION_VALIDATION.md) |

## Conditional Validation

Apply different rules based on runtime conditions: field values, configuration, or external context.

```rust
// Using .when() combinator
let optional_url_rule = rules::url()
    .when(|s: &String| !s.is_empty());

// Manual conditional validation
if self.requires_shipping {
    if let Err(e) = self.shipping_address.validate() {
        err.merge_prefixed("shipping_address", e);
    }
}
```

**Key patterns:**
- `.when()` combinator for conditional rules
- Context-based validation with external state
- Configuration-driven rules
- Multi-branch validation for enums

**Full guide:** [CONDITIONAL_VALIDATION.md](CONDITIONAL_VALIDATION.md)

## Async Validation

Validate against external resources: databases, APIs, caches.

```rust
use domainstack::{AsyncValidate, ValidationContext};
use async_trait::async_trait;

#[async_trait]
impl AsyncValidate for User {
    async fn validate_async(&self, ctx: &ValidationContext) -> Result<(), ValidationError> {
        // Sync validation first
        self.validate()?;

        // Async validation (database check)
        let db = ctx.get_resource::<PgPool>("db")?;
        if email_exists(db, &self.email).await? {
            return Err(ValidationError::single("email", "taken", "Email already registered"));
        }

        Ok(())
    }
}
```

**Key patterns:**
- `AsyncValidate` trait for I/O-bound checks
- `ValidationContext` for passing resources
- Database uniqueness checks
- External API validation
- Rate limiting with Redis

**Full guide:** [ASYNC_VALIDATION.md](ASYNC_VALIDATION.md)

## Type-State Validation

Compile-time guarantees using phantom types - make invalid states unrepresentable.

```rust
use domainstack::typestate::{Validated, Unvalidated};

pub struct Email<State = Unvalidated> {
    value: String,
    _state: PhantomData<State>,
}

// Only accepts validated email - compiler enforced!
fn send_newsletter(email: Email<Validated>) {
    println!("Sending to: {}", email.as_str());
}

let email = Email::new("user@example.com").validate()?;
send_newsletter(email);  // Compiles!
```

**Key patterns:**
- Phantom type markers (`Validated`, `Unvalidated`)
- Validation as state transition
- Builder pattern integration
- Zero runtime cost

**Full guide:** [TYPE_STATE.md](TYPE_STATE.md)

## Cross-Field Validation

Enforce relationships between multiple fields.

```rust
#[derive(Validate)]
#[validate(
    check = "self.check_out > self.check_in",
    code = "invalid_date_range",
    message = "Check-out must be after check-in"
)]
struct Booking {
    check_in: NaiveDate,
    check_out: NaiveDate,
}
```

**Key patterns:**
- Derive macro with `check` attribute
- Date range validation
- Password confirmation
- Conditional cross-field rules with `when`

**Full guide:** [CROSS_FIELD_VALIDATION.md](CROSS_FIELD_VALIDATION.md)

## Collection Validation

Validate arrays and vectors: size constraints and item validation.

```rust
#[derive(Validate)]
struct Article {
    #[validate(min_items = 1, max_items = 10)]
    #[validate(unique)]
    #[validate(each(length(min = 1, max = 50)))]
    #[validate(each(alphanumeric))]
    tags: Vec<String>,

    #[validate(each(nested))]
    comments: Vec<Comment>,
}
```

**Key patterns:**
- Collection rules: `min_items`, `max_items`, `unique`
- Item rules: `each(rule)`
- Nested types: `each(nested)`
- Error paths with array indices

**Full guide:** [COLLECTION_VALIDATION.md](COLLECTION_VALIDATION.md)

## Best Practices

### 1. Choose the Right Pattern

| Scenario | Pattern |
|----------|---------|
| Simple field validation | `#[derive(Validate)]` |
| Field relationships | Cross-field validation |
| Runtime conditions | Conditional validation |
| Database/API checks | Async validation |
| Compile-time safety | Type-state |

### 2. Layer Validation

```rust
// 1. Sync validation (fast, format checks)
self.validate()?;

// 2. Async validation (slow, I/O required)
self.validate_async(&ctx).await?;
```

### 3. Keep Domain Logic Pure

```rust
// Domain validation - no framework dependencies
impl Validate for User { ... }

// Framework integration in HTTP layer
async fn create(DomainJson { domain, .. }: DomainJson<User, Dto>) { ... }
```

### 4. Use Type-State for Critical Paths

```rust
// Database operations require validated data
async fn insert_user(user: User<Validated>) { ... }
```

### 5. Compose Reusable Rules

```rust
pub fn company_email() -> impl Rule<str> {
    rules::email()
        .and(rules::ends_with("@company.com"))
        .code("invalid_company_email")
}
```

## See Also

**Dedicated Guides:**
- [Conditional Validation](CONDITIONAL_VALIDATION.md) - Runtime-determined rules
- [Async Validation](ASYNC_VALIDATION.md) - Database and API checks
- [Type-State Validation](TYPE_STATE.md) - Compile-time guarantees
- [Cross-Field Validation](CROSS_FIELD_VALIDATION.md) - Field relationships
- [Collection Validation](COLLECTION_VALIDATION.md) - Arrays and vectors

**Foundation:**
- [Core Concepts](CORE_CONCEPTS.md) - Valid-by-construction philosophy
- [Derive Macro](DERIVE_MACRO.md) - Declarative validation
- [Manual Validation](MANUAL_VALIDATION.md) - Implementing `Validate` trait
- [Error Handling](ERROR_HANDLING.md) - Working with `ValidationError`

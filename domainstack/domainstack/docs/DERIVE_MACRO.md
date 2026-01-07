# Derive Macro Guide

Complete reference for the `#[derive(Validate)]` macro and `#[validate(...)]` attributes.

## Table of Contents

- [Overview](#overview)
- [Basic Attributes](#basic-attributes)
- [Nested Validation](#nested-validation)
- [Collection Validation](#collection-validation)
- [Cross-Field Validation](#cross-field-validation)
- [Custom Validation](#custom-validation)
- [Multiple Attributes](#multiple-attributes)

## Overview

The `#[derive(Validate)]` macro automatically implements the `Validate` trait for your structs. Use `#[validate(...)]` attributes to declare validation rules.

**Feature flag:** `derive`

```rust
use domainstack::prelude::*;

#[derive(Validate)]
struct User {
    #[validate(length(min = 1, max = 50))]
    name: String,

    #[validate(range(min = 18, max = 120))]
    age: u8,
}

// Validate automatically
let user = User { name: "Alice".to_string(), age: 25 };
user.validate()?;  // [ok] Valid
```

## Basic Attributes

### #[validate(length)]

Validate string length with `min`, `max`, or both:

```rust
#[derive(Validate)]
struct User {
    #[validate(length(min = 1, max = 50))]
    name: String,

    #[validate(length(min = 8, max = 128))]
    password: String,
}
```

**Shortcuts:**
- `#[validate(min_len = 3)]` - Minimum length only
- `#[validate(max_len = 50)]` - Maximum length only

### #[validate(range)]

Validate numeric ranges:

```rust
#[derive(Validate)]
struct Adult {
    #[validate(range(min = 18, max = 120))]
    age: u8,
}
```

**Shortcuts:**
- `#[validate(min = 0)]` - Minimum value only
- `#[validate(max = 100)]` - Maximum value only

### String Format Rules

```rust
#[derive(Validate)]
struct Contact {
    #[validate(email)]
    email: String,

    #[validate(url)]
    website: String,

    #[validate(alphanumeric)]
    username: String,
}
```

**Available rules:**
- `email` - Email format
- `url` - URL format
- `alphanumeric` - Letters and numbers only
- `ascii` - ASCII characters only
- `alpha_only` - Letters only
- `numeric_string` - Numeric string
- `non_empty` - Not empty
- `non_blank` - Not empty/whitespace
- `no_whitespace` - No whitespace

### String Pattern Rules

```rust
#[derive(Validate)]
struct Document {
    #[validate(contains = "draft")]
    status: String,

    #[validate(starts_with = "PRE-")]
    prefix_code: String,

    #[validate(ends_with = ".pdf")]
    filename: String,

    #[validate(matches_regex = r"^[A-Z]{2}\d{4}$")]
    license_plate: String,
}
```

### Numeric Rules

```rust
#[derive(Validate)]
struct Measurement {
    #[validate(positive)]
    height: f64,

    #[validate(non_zero)]
    divisor: i32,

    #[validate(multiple_of = 5)]
    quantity: u32,
}
```

**Available rules:**
- `positive` - Greater than zero
- `negative` - Less than zero
- `non_zero` - Not equal to zero
- `finite` - Finite number (floats)
- `multiple_of = N` - Multiple of N
- `equals = N` - Equal to N
- `not_equals = N` - Not equal to N

## Nested Validation

Validate nested structs with `#[validate(nested)]`:

```rust
#[derive(Validate)]
struct Email {
    #[validate(length(min = 5, max = 255))]
    #[validate(email)]
    value: String,
}

#[derive(Validate)]
struct User {
    #[validate(nested)]
    email: Email,

    #[validate(nested)]
    address: Address,
}

// Error paths include nesting: "email.value", "address.city"
```

**Optional nested types:**

```rust
#[derive(Validate)]
struct Profile {
    #[validate(nested)]
    bio: Option<Bio>,  // Validates only if Some
}
```

## Collection Validation

> **Comprehensive guide:** See [COLLECTION_VALIDATION.md](COLLECTION_VALIDATION.md) for complete collection validation patterns, including manual validation, error paths, and complex examples.

### Nested Collections

Validate each item in a collection with `each(nested)`:

```rust
#[derive(Validate)]
struct Team {
    #[validate(each(nested))]
    members: Vec<User>,
}

// Error paths: "members[0].name", "members[1].email.value"
```

### Primitive Collections with each(rule)

Apply any validation rule to collection items:

```rust
#[derive(Validate)]
struct BlogPost {
    // Validate string length for each tag
    #[validate(each(length(min = 1, max = 50)))]
    tags: Vec<String>,

    // Validate each email format
    #[validate(each(email))]
    author_emails: Vec<String>,

    // Validate each URL
    #[validate(each(url))]
    related_links: Vec<String>,

    // Validate each keyword is alphanumeric
    #[validate(each(alphanumeric))]
    keywords: Vec<String>,

    // Validate each rating is in range
    #[validate(each(range(min = 1, max = 5)))]
    ratings: Vec<u8>,

    // Combine with collection-level rules
    #[validate(each(min_len = 3))]
    #[validate(min_items = 1)]
    #[validate(max_items = 10)]
    category_names: Vec<String>,
}

// Error paths include indices:
// "tags[0]", "author_emails[2]", "related_links[1]", "ratings[0]"
```

**Supported each() rules:**
- **String:** `email`, `url`, `min_len`, `max_len`, `length`, `alphanumeric`, `ascii`, `alpha_only`, `numeric_string`, `non_empty`, `non_blank`, `no_whitespace`, `contains`, `starts_with`, `ends_with`, `matches_regex`
- **Numeric:** `range`, `min`, `max`, `positive`, `negative`, `non_zero`, `finite`, `multiple_of`, `equals`, `not_equals`
- **Nested:** `nested` (for complex types)

### Collection-Level Rules

Validate collection size and uniqueness:

```rust
#[derive(Validate)]
struct Playlist {
    #[validate(min_items = 1)]
    #[validate(max_items = 100)]
    songs: Vec<Song>,

    #[validate(unique)]
    #[validate(min_items = 1)]
    tags: Vec<String>,
}
```

### Non-Empty Collection Items

The `non_empty_items` rule validates that all string items are non-empty:

```rust
#[derive(Validate)]
struct Article {
    // Tags must exist, be unique, and all non-empty
    #[validate(min_items = 1)]
    #[validate(max_items = 20)]
    #[validate(unique)]
    #[validate(non_empty_items)]
    tags: Vec<String>,

    // Keywords must be non-empty and alphanumeric
    #[validate(each(alphanumeric))]
    #[validate(non_empty_items)]
    keywords: Vec<String>,
}
```

**Common use cases:**
- Tags where empty strings are invalid
- Category names
- Keywords for search
- User-provided lists

## Cross-Field Validation

> **Comprehensive guide:** See [CROSS_FIELD_VALIDATION.md](CROSS_FIELD_VALIDATION.md) for complete cross-field validation patterns, including date ranges, password confirmation, conditional validation, and complex business rules.

Validate relationships between fields using struct-level `#[validate(...)]` attributes.

### Basic Cross-Field Rules

Use `check` parameter to specify the validation condition:

```rust
use domainstack::prelude::*;
use chrono::{DateTime, Utc, Duration};

#[derive(Validate)]
#[validate(
    check = "self.end_date > self.start_date",
    code = "invalid_date_range",
    message = "End date must be after start date"
)]
struct DateRange {
    #[validate(future)]
    start_date: DateTime<Utc>,

    #[validate(future)]
    end_date: DateTime<Utc>,
}

// Usage
let range = DateRange {
    start_date: Utc::now() + Duration::days(1),
    end_date: Utc::now() + Duration::days(30),
};
range.validate()?;  // [ok] Valid

let invalid = DateRange {
    start_date: Utc::now() + Duration::days(30),
    end_date: Utc::now() + Duration::days(1),  // Before start!
};
invalid.validate()?;  // [error] Error: invalid_date_range
```

### Multiple Cross-Field Rules

Stack multiple struct-level validations:

```rust
#[derive(Validate)]
#[validate(
    check = "self.end_date > self.start_date",
    code = "invalid_date_range",
    message = "End date must be after start date"
)]
#[validate(
    check = "self.total >= self.minimum_order",
    code = "below_minimum",
    message = "Order total below minimum"
)]
struct Order {
    start_date: DateTime<Utc>,
    end_date: DateTime<Utc>,
    total: f64,
    minimum_order: f64,
}
```

### Conditional Cross-Field Validation

Use `when` parameter for conditional validation:

```rust
#[derive(Validate)]
#[validate(
    check = "self.total >= self.minimum_order",
    code = "below_minimum",
    message = "Order total below minimum",
    when = "self.requires_minimum"  // Only validate if this is true
)]
struct FlexibleOrder {
    total: f64,
    minimum_order: f64,
    requires_minimum: bool,
}

// Usage
let order = FlexibleOrder {
    total: 50.0,
    minimum_order: 100.0,
    requires_minimum: false,  // Validation skipped!
};
order.validate()?;  // [ok] Valid - condition is false

let required = FlexibleOrder {
    total: 50.0,
    minimum_order: 100.0,
    requires_minimum: true,  // Validation runs!
};
required.validate()?;  // [error] Error: below_minimum
```

### Password Confirmation Example

```rust
#[derive(Validate)]
#[validate(
    check = "self.password == self.password_confirmation",
    code = "password_mismatch",
    message = "Passwords do not match"
)]
struct PasswordChange {
    #[validate(length(min = 8, max = 128))]
    #[validate(matches_regex = r"[A-Z]")]  // At least one uppercase
    #[validate(matches_regex = r"[0-9]")]  // At least one digit
    password: String,

    password_confirmation: String,
}
```

### Manual Implementation Alternative

For complex cross-field logic, implement `Validate` manually:

```rust
impl Validate for DateRange {
    fn validate(&self) -> Result<(), ValidationError> {
        let mut err = ValidationError::new();

        // Field-level validation first
        if let Err(e) = validate("start_date", &self.start_date, &rules::future()) {
            err.extend(e);
        }
        if let Err(e) = validate("end_date", &self.end_date, &rules::future()) {
            err.extend(e);
        }

        // Cross-field validation
        if self.end_date <= self.start_date {
            err.push(
                "end_date",
                "invalid_range",
                "End date must be after start date"
            );
        }

        // Check minimum duration (e.g., at least 1 day)
        let duration = self.end_date.signed_duration_since(self.start_date);
        if duration.num_days() < 1 {
            err.push(
                "end_date",
                "duration_too_short",
                "Event must be at least 1 day long"
            );
        }

        if err.is_empty() { Ok(()) } else { Err(err) }
    }
}
```

**When to use manual implementation:**
- Complex business logic that doesn't fit in a single check expression
- Need to compute intermediate values (like duration calculations)
- Multiple related cross-field checks with shared logic
- Dynamic error messages based on calculation results

## Custom Validation

Define custom validation functions and reference them with `#[validate(custom = "...")]`:

```rust
fn validate_even(value: &u8) -> Result<(), ValidationError> {
    if *value % 2 == 0 {
        Ok(())
    } else {
        Err(ValidationError::single(
            Path::root(),
            "not_even",
            "Must be even"
        ))
    }
}

#[derive(Validate)]
struct EvenNumber {
    #[validate(range(min = 0, max = 100))]
    #[validate(custom = "validate_even")]
    value: u8,
}
```

**Reusable validation functions:**

```rust
fn validate_positive(value: &i32) -> Result<(), ValidationError> {
    if *value > 0 {
        Ok(())
    } else {
        Err(ValidationError::single(
            Path::root(),
            "not_positive",
            "Must be positive"
        ))
    }
}

#[derive(Validate)]
struct Balance {
    #[validate(custom = "validate_positive")]
    amount: i32,
}

#[derive(Validate)]
struct Credit {
    #[validate(custom = "validate_positive")]
    points: i32,
}
```

**Custom validation with context:**

```rust
fn validate_strong_password(value: &String) -> Result<(), ValidationError> {
    let mut err = ValidationError::new();

    if !value.chars().any(|c| c.is_uppercase()) {
        err.push("", "no_uppercase", "Must contain uppercase letter");
    }
    if !value.chars().any(|c| c.is_lowercase()) {
        err.push("", "no_lowercase", "Must contain lowercase letter");
    }
    if !value.chars().any(|c| c.is_numeric()) {
        err.push("", "no_digit", "Must contain digit");
    }

    if err.is_empty() { Ok(()) } else { Err(err) }
}

#[derive(Validate)]
struct Password {
    #[validate(length(min = 8, max = 128))]
    #[validate(custom = "validate_strong_password")]
    value: String,
}
```

## Multiple Attributes

Stack multiple `#[validate(...)]` attributes to combine rules:

```rust
#[derive(Validate)]
struct Password {
    #[validate(length(min = 8, max = 128))]
    #[validate(custom = "validate_strong_password")]
    value: String,
}

#[derive(Validate)]
struct Product {
    #[validate(min_len = 1)]
    #[validate(max_len = 100)]
    #[validate(alphanumeric)]
    sku: String,

    #[validate(positive)]
    #[validate(max = 10000)]
    #[validate(custom = "validate_price_precision")]
    price: f64,
}

#[derive(Validate)]
struct Tags {
    #[validate(each(min_len = 1))]
    #[validate(each(max_len = 50))]
    #[validate(min_items = 1)]
    #[validate(max_items = 20)]
    #[validate(unique)]
    #[validate(non_empty_items)]
    values: Vec<String>,
}
```

**Execution order:**
1. Field-level attributes run top to bottom
2. All field validations complete before cross-field validations
3. Struct-level `#[validate(check = "...")]` attributes run last
4. All violations accumulate (fail-slow by default)

## Related Derive Macros

The `domainstack-derive` crate provides additional derive macros:

| Macro | Purpose | Feature |
|-------|---------|---------|
| `#[derive(Validate)]` | Runtime validation | `derive` |
| `#[derive(ToSchema)]` | OpenAPI 3.0 schema generation | `schema` |
| `#[derive(ToJsonSchema)]` | JSON Schema (Draft 2020-12) generation | `schema` |
| `#[derive(ValidateOnDeserialize)]` | Validate during serde deserialization | `serde` |

```rust
// Use all together for complete type-safe validation with schema generation
#[derive(Validate, ToSchema, ToJsonSchema)]
struct User {
    #[validate(email)]
    email: String,
}
```

## See Also

**Specialized Guides:**
- [Cross-Field Validation](CROSS_FIELD_VALIDATION.md) - Date ranges, password confirmation, conditional validation
- [Collection Validation](COLLECTION_VALIDATION.md) - Arrays, vectors, `each()` patterns
- [Conditional Validation](CONDITIONAL_VALIDATION.md) - Runtime-determined validation rules

**Schema Generation:**
- [JSON Schema Generation](JSON_SCHEMA.md) - Auto-generate JSON Schema from validation rules
- [OpenAPI Schema Generation](OPENAPI_SCHEMA.md) - Auto-generate OpenAPI schemas from validation rules

**Integration:**
- [Serde Integration](SERDE_INTEGRATION.md) - Validate on deserialize with `ValidateOnDeserialize`
- [HTTP Integration](HTTP_INTEGRATION.md) - Framework adapters for Axum, Actix-web, Rocket

**Reference:**
- [Rules Reference](RULES.md) - Complete list of 37 built-in validation rules
- [Manual Validation](MANUAL_VALIDATION.md) - Implementing `Validate` trait manually
- [Core Concepts](CORE_CONCEPTS.md) - Foundation principles and patterns

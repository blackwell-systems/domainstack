# API Guide

Complete guide to using domainstack for domain validation.

## Table of Contents

- [Core Concepts](#core-concepts)
- [Manual Validation](#manual-validation)
- [Derive Macro](#derive-macro)
- [Error Handling](#error-handling)
- [HTTP Integration](#http-integration)
- [Advanced Patterns](#advanced-patterns)

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

### Basic Attributes

#### #[validate(length)]

```rust
#[derive(Validate)]
struct User {
    #[validate(length(min = 1, max = 50))]
    name: String,
}
```

#### #[validate(range)]

```rust
#[derive(Validate)]
struct Adult {
    #[validate(range(min = 18, max = 120))]
    age: u8,
}
```

### Nested Validation

```rust
#[derive(Validate)]
struct Email {
    #[validate(length(min = 5, max = 255))]
    value: String,
}

#[derive(Validate)]
struct User {
    #[validate(nested)]
    email: Email,
}

// Error paths: "email.value"
```

### Collection Validation

#### Nested Collections

```rust
#[derive(Validate)]
struct Team {
    #[validate(each(nested))]
    members: Vec<User>,
}

// Error paths: "members[0].name", "members[1].email.value"
```

#### Primitive Collections with `each(rule)`

**Any validation rule can be used with `each()` to validate collection items:**

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

// Error paths include array indices:
// - "tags[0]" - "Must be at least 1 character"
// - "author_emails[2]" - "Invalid email format"
// - "related_links[1]" - "Invalid URL format"
// - "keywords[3]" - "Must be alphanumeric"
// - "ratings[0]" - "Must be between 1 and 5"
```

**Supported `each()` rules:**
- String: `email`, `url`, `min_len`, `max_len`, `length`, `alphanumeric`, `ascii`, `alpha_only`, `numeric_string`, `non_empty`, `non_blank`, `no_whitespace`, `contains`, `starts_with`, `ends_with`, `matches_regex`
- Numeric: `range`, `min`, `max`, `positive`, `negative`, `non_zero`, `finite`, `multiple_of`, `equals`, `not_equals`
- Nested: `nested` (for complex types)

#### Non-Empty Collection Items

The `non_empty_items()` rule validates that all string items in a collection are non-empty:

```rust
use domainstack::prelude::*;

// Manual validation
let tag_rule = rules::min_items(1)
    .and(rules::unique())
    .and(rules::non_empty_items());

let tags = vec!["rust".to_string(), "validation".to_string(), "domain".to_string()];
validate("tags", &tags, &tag_rule)?;  // ✓ Valid

// Invalid - contains empty string
let invalid_tags = vec!["rust".to_string(), "".to_string(), "domain".to_string()];
match validate("tags", &invalid_tags, &tag_rule) {
    Ok(_) => {},
    Err(e) => {
        // Error: empty_item
        // Meta: {"empty_count": "1", "indices": "[1]"}
        println!("Found {} empty items at indices {}",
            e.violations[0].meta.get("empty_count").unwrap(),
            e.violations[0].meta.get("indices").unwrap());
    }
}

// With derive macro
#[derive(Validate)]
struct Article {
    #[validate(min_len = 1)]
    #[validate(max_len = 200)]
    title: String,

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

### Custom Validation

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

### Multiple Attributes

You can stack multiple validations:

```rust
#[derive(Validate)]
struct Password {
    #[validate(length(min = 8, max = 128))]
    #[validate(custom = "validate_strong_password")]
    value: String,
}
```

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

## HTTP Integration

### Using error-envelope

```rust
use domainstack_envelope::IntoEnvelopeError;

async fn create_user(
    Json(user): Json<User>
) -> Result<Json<User>, Error> {
    // One-line conversion to HTTP error
    user.validate()
        .map_err(|e| e.into_envelope_error())?;
    
    // Save user...
    Ok(Json(user))
}
```

### Error Response Format

```json
{
  "code": "VALIDATION",
  "status": 400,
  "message": "Validation failed with 2 errors",
  "retryable": false,
  "details": {
    "fields": {
      "guest.email.value": [
        {
          "code": "invalid_email",
          "message": "Invalid email format",
          "meta": {"max": 255}
        }
      ],
      "rooms[1].adults": [
        {
          "code": "out_of_range",
          "message": "Must be between 1 and 4",
          "meta": {"min": 1, "max": 4}
        }
      ]
    }
  }
}
```

### Client-Side Error Handling

The structured format makes client-side rendering easy:

```typescript
// TypeScript example
interface FieldErrors {
  [fieldPath: string]: Array<{
    code: string;
    message: string;
    meta?: Record<string, string>;
  }>;
}

function displayErrors(response: ErrorResponse) {
  const fields = response.details.fields;
  
  for (const [path, errors] of Object.entries(fields)) {
    errors.forEach(err => {
      showErrorAtField(path, err.message);
    });
  }
}
```

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

## Advanced Patterns

### Cross-Field Validation

```rust
impl Validate for DateRange {
    fn validate(&self) -> Result<(), ValidationError> {
        let mut err = ValidationError::new();
        
        if self.end_date < self.start_date {
            err.push(
                "end_date",
                "invalid_range",
                "End date must be after start date"
            );
        }
        
        if err.is_empty() { Ok(()) } else { Err(err) }
    }
}
```

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

### Reusable Validation Functions

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

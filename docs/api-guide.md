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

#### Primitive Collections

```rust
#[derive(Validate)]
struct Tags {
    #[validate(each(length(min = 3, max = 20)))]
    tags: Vec<String>,
}

// Error paths: "tags[0]", "tags[1]"
```

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

- [Rules Reference](./rules.md) - Complete list of built-in rules
- [Examples](../domainstack/examples/) - Runnable code examples
- [API Documentation](https://docs.rs/domainstack) - Full API reference

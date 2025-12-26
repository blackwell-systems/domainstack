# Error Handling

**Working with ValidationError, Violation, and structured error responses.**

## Table of Contents

- [ValidationError Construction](#validationerror-construction)
- [Adding Violations](#adding-violations)
- [Violation Structure](#violation-structure)
- [Error Accumulation](#error-accumulation)
- [Extracting Error Information](#extracting-error-information)
- [Message Transformation](#message-transformation)
- [HTTP Integration](#http-integration)
- [Best Practices](#best-practices)

## ValidationError Construction

### Creating Empty Errors

```rust
use domainstack::prelude::*;

// Create empty error accumulator
let mut err = ValidationError::new();

// Or use Default
let err = ValidationError::default();

// Check if empty
if err.is_empty() {
    println!("No errors!");
}
```

### Single Error Constructor

For cases where you have exactly one error:

```rust
let err = ValidationError::single(
    "email",
    "invalid_email",
    "Invalid email format"
);

// Equivalent to:
let mut err = ValidationError::new();
err.push("email", "invalid_email", "Invalid email format");
```

### Incremental Construction

Build up errors by validating multiple fields:

```rust
impl Validate for User {
    fn validate(&self) -> Result<(), ValidationError> {
        let mut err = ValidationError::new();

        // Validate each field
        if let Err(e) = validate("name", &self.name, &rules::min_len(1)) {
            err.extend(e);
        }

        if let Err(e) = validate("email", &self.email, &rules::email()) {
            err.extend(e);
        }

        if let Err(e) = validate("age", &self.age, &rules::range(18, 120)) {
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

## Adding Violations

### Basic push()

Add violations with field path, error code, and message:

```rust
let mut err = ValidationError::new();

// String field path
err.push("email", "invalid_email", "Invalid email format");

// Nested path
err.push("guest.email", "required", "Email is required");

// Array index path
err.push("rooms[0].adults", "out_of_range", "Must be between 1 and 4");
```

### Path Types

The `push()` method accepts anything that implements `Into<Path>`:

```rust
// String slice
err.push("email", "invalid", "Invalid email");

// String
err.push("email".to_string(), "invalid", "Invalid email");

// Path API (type-safe)
let path = Path::root().field("rooms").index(0).field("adults");
err.push(path, "out_of_range", "Must be between 1 and 4");
```

### Error Codes

Error codes are `&'static str` for zero-allocation:

```rust
// Built-in rule codes
err.push("email", "invalid_email", "Invalid email format");
err.push("age", "out_of_range", "Must be between 18 and 120");

// Custom codes
err.push("password", "weak_password", "Password too weak");
err.push("username", "username_taken", "Username already exists");
```

**Common error codes:**
- `required` - Missing required field
- `invalid_email` - Email format invalid
- `invalid_url` - URL format invalid
- `min_length` - String too short
- `max_length` - String too long
- `out_of_range` - Number outside valid range
- `pattern_mismatch` - Regex pattern failed
- `min_items` - Collection too small
- `max_items` - Collection too large

See [RULES.md](RULES.md) for complete list of rule codes.

## Violation Structure

Each violation contains:

```rust
pub struct Violation {
    pub path: Path,           // Field path (e.g., "guest.email")
    pub code: &'static str,   // Machine-readable code
    pub message: String,      // Human-readable message
    pub meta: Meta,           // Additional context
}
```

### Accessing Violations

```rust
let err = ValidationError::single("email", "invalid_email", "Invalid email");

for v in &err.violations {
    println!("Path: {}", v.path);          // "email"
    println!("Code: {}", v.code);          // "invalid_email"
    println!("Message: {}", v.message);    // "Invalid email"
    println!("Meta: {:?}", v.meta);        // Meta { ... }
}
```

### Meta Fields

Violations can include additional context:

```rust
// Rules automatically populate meta fields
let rule = rules::range(18, 120);
// On failure, includes: meta.insert("min", 18); meta.insert("max", 120);

let rule = rules::max_len(255);
// On failure, includes: meta.insert("max", 255);
```

**Accessing meta:**

```rust
for v in &err.violations {
    if let Some(min) = v.meta.get("min") {
        println!("Minimum value: {}", min);
    }

    if let Some(max) = v.meta.get("max") {
        println!("Maximum value: {}", max);
    }
}
```

**Common meta keys:**
- `min`, `max` - Range bounds
- `actual` - Actual value that failed
- `expected` - Expected value
- `pattern` - Regex pattern that failed

## Error Accumulation

### extend() - Same-Level Errors

Use `extend()` when merging errors from the **same level** (no path change):

```rust
impl Validate for User {
    fn validate(&self) -> Result<(), ValidationError> {
        let mut err = ValidationError::new();

        // Validate name (path: "name")
        if let Err(e) = validate("name", &self.name, &name_rule) {
            err.extend(e);  // Path stays "name"
        }

        // Validate age (path: "age")
        if let Err(e) = validate("age", &self.age, &age_rule) {
            err.extend(e);  // Path stays "age"
        }

        if err.is_empty() { Ok(()) } else { Err(err) }
    }
}
```

### merge_prefixed() - Nested Errors

Use `merge_prefixed()` when merging errors from **nested types** (add path prefix):

```rust
// Email validates itself with path "value"
impl Validate for Email {
    fn validate(&self) -> Result<(), ValidationError> {
        validate("value", self.0.as_str(), &rules::email())
        // Error path: "value"
    }
}

// User needs to prefix "email" when merging
impl Validate for User {
    fn validate(&self) -> Result<(), ValidationError> {
        let mut err = ValidationError::new();

        if let Err(e) = self.email.validate() {
            err.merge_prefixed("email", e);
            // "value" becomes "email.value"
        }

        if err.is_empty() { Ok(()) } else { Err(err) }
    }
}
```

### prefixed() - Transform All Paths

Transform an entire error by adding a prefix:

```rust
// Validate booking
let booking_err = booking.validate();  // Paths: "check_in", "check_out"

// Add prefix to all paths
let prefixed_err = booking_err.prefixed("booking");
// Paths become: "booking.check_in", "booking.check_out"
```

**When to use:**
- Converting DTO → Domain at HTTP boundary
- Nesting validation in parent structures
- Scoping errors to subsystems

### Collection Validation Pattern

Combine array indices with nested errors:

```rust
impl Validate for Team {
    fn validate(&self) -> Result<(), ValidationError> {
        let mut err = ValidationError::new();

        for (i, member) in self.members.iter().enumerate() {
            if let Err(e) = member.validate() {
                // Build path with index
                let path = Path::root().field("members").index(i);
                err.merge_prefixed(path, e);
                // Produces: "members[0].email", "members[1].name", etc.
            }
        }

        if err.is_empty() { Ok(()) } else { Err(err) }
    }
}
```

## Extracting Error Information

### field_violations_map() - Recommended

Get violations grouped by field path (preserves codes and meta):

```rust
let mut err = ValidationError::new();
err.push("email", "invalid_email", "Invalid email format");
err.push("email", "max_length", "Email too long");
err.push("age", "out_of_range", "Age out of range");

let map = err.field_violations_map();
// BTreeMap<String, Vec<&Violation>>

for (field, violations) in map {
    println!("Field: {}", field);
    for v in violations {
        println!("  - [{}] {}", v.code, v.message);
    }
}

// Output:
// Field: age
//   - [out_of_range] Age out of range
// Field: email
//   - [invalid_email] Invalid email format
//   - [max_length] Email too long
```

**Why use this:**
- ✅ Preserves error codes (needed for client-side handling)
- ✅ Preserves meta fields (needed for context)
- ✅ Enables proper error classification
- ✅ Supports internationalization

### field_errors_map() - Deprecated

Get only error messages grouped by field (loses codes and meta):

```rust
#[allow(deprecated)]
let map = err.field_errors_map();
// BTreeMap<String, Vec<String>>

for (field, messages) in map {
    println!("{}: {:?}", field, messages);
}

// Output:
// email: ["Invalid email format", "Email too long"]
// age: ["Age out of range"]
```

**⚠️ Warning:** This method only returns messages and **loses error codes and metadata**. Use `field_violations_map()` instead for complete error information.

### Direct Violation Access

Iterate over violations directly:

```rust
for v in &err.violations {
    match v.code {
        "invalid_email" => handle_email_error(v),
        "out_of_range" => handle_range_error(v),
        _ => handle_generic_error(v),
    }
}
```

## Message Transformation

### map_messages() - Internationalization

Transform all violation messages (useful for i18n):

```rust
let mut err = ValidationError::new();
err.push("email", "invalid_email", "Invalid email");
err.push("age", "out_of_range", "Must be 18+");

// Translate to Spanish
let err = err.map_messages(|msg| {
    match msg.as_str() {
        "Invalid email" => "Email inválido".to_string(),
        "Must be 18+" => "Debe tener 18 años o más".to_string(),
        _ => msg,
    }
});

assert_eq!(err.violations[0].message, "Email inválido");
assert_eq!(err.violations[1].message, "Debe tener 18 años o más");
```

### Code-Based Translation

Use error codes for proper i18n:

```rust
fn translate(err: ValidationError, lang: &str) -> ValidationError {
    err.map_messages(|_msg| {
        // Ignore English message, translate from code
        format!("Translated to {}", lang)
    })
}

// Better: Use error codes + meta for contextual translation
fn translate_with_context(v: &Violation, lang: &str) -> String {
    match (v.code, lang) {
        ("out_of_range", "es") => {
            let min = v.meta.get("min").unwrap();
            let max = v.meta.get("max").unwrap();
            format!("Debe estar entre {} y {}", min, max)
        }
        ("invalid_email", "es") => "Email inválido".to_string(),
        _ => v.message.clone(),
    }
}
```

### filter() - Conditional Errors

Remove certain violations based on criteria:

```rust
let mut err = ValidationError::new();
err.push("email", "warning", "Email format questionable");
err.push("age", "invalid", "Age is required");
err.push("name", "warning", "Name seems unusual");

// Remove warnings, keep only errors
let err = err.filter(|v| v.code != "warning");

assert_eq!(err.violations.len(), 1);
assert_eq!(err.violations[0].code, "invalid");
```

**Use cases:**
- Separating warnings from errors
- Filtering by severity level
- Removing certain error types conditionally

## HTTP Integration

### error-envelope Integration

Convert ValidationError to structured HTTP responses:

```rust
use domainstack_envelope::IntoEnvelopeError;
use axum::{Json, http::StatusCode};

async fn create_user(
    Json(dto): Json<UserDto>
) -> Result<Json<User>, ErrorResponse> {
    // Convert DTO → Domain
    let user = User::try_from(dto)
        .map_err(|e| e.into_envelope_error())?;

    // Save user...
    Ok(Json(user))
}
```

### RFC 9457 Response Format

Validation errors are serialized to RFC 9457 problem details:

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

### Client-Side Handling

The structured format maps cleanly to frontend forms:

```typescript
// TypeScript example
interface ErrorResponse {
  code: string;
  status: number;
  message: string;
  details: {
    fields: {
      [fieldPath: string]: Array<{
        code: string;
        message: string;
        meta?: Record<string, string>;
      }>;
    };
  };
}

function displayErrors(response: ErrorResponse) {
  const fields = response.details.fields;

  for (const [path, errors] of Object.entries(fields)) {
    // Map directly to form fields
    const inputElement = document.querySelector(`[name="${path}"]`);

    errors.forEach(err => {
      showErrorAtField(inputElement, err.message);

      // Use error codes for custom handling
      if (err.code === "invalid_email") {
        highlightEmailField(inputElement);
      }
    });
  }
}
```

**React Hook Form example:**

```typescript
const { setError } = useFormContext();

// Map validation errors to form fields
for (const [path, errors] of Object.entries(response.details.fields)) {
  setError(path, {
    type: errors[0].code,
    message: errors[0].message,
  });
}
```

### Custom Error Responses

Build custom error responses for your API:

```rust
use serde::Serialize;

#[derive(Serialize)]
struct ApiError {
    success: false,
    errors: BTreeMap<String, Vec<ErrorDetail>>,
}

#[derive(Serialize)]
struct ErrorDetail {
    code: &'static str,
    message: String,
}

fn to_api_error(err: ValidationError) -> ApiError {
    let mut errors = BTreeMap::new();

    for (path, violations) in err.field_violations_map() {
        let details = violations.iter().map(|v| ErrorDetail {
            code: v.code,
            message: v.message.clone(),
        }).collect();

        errors.insert(path, details);
    }

    ApiError { success: false, errors }
}
```

## Best Practices

### 1. Always Use Error Codes

Error codes enable proper client-side handling:

```rust
// ✅ GOOD: Include error codes
err.push("email", "invalid_email", "Invalid email format");

// ❌ BAD: Generic or missing codes
err.push("email", "error", "Something went wrong");
```

### 2. extend() vs merge_prefixed()

Choose the right method for error merging:

```rust
// ✅ Use extend() for same-level validation
if let Err(e) = validate("name", &self.name, &name_rule) {
    err.extend(e);  // Path stays "name"
}

// ✅ Use merge_prefixed() for nested types
if let Err(e) = self.email.validate() {
    err.merge_prefixed("email", e);  // Adds "email." prefix
}
```

### 3. Preserve Complete Error Information

Use `field_violations_map()` instead of deprecated `field_errors_map()`:

```rust
// ✅ GOOD: Preserves codes and meta
let map = err.field_violations_map();
for (field, violations) in map {
    for v in violations {
        log_error(field, v.code, &v.message, &v.meta);
    }
}

// ❌ BAD: Loses codes and meta
#[allow(deprecated)]
let map = err.field_errors_map();  // Only messages!
```

### 4. Use Path API for Complex Paths

Type-safe path building prevents typos:

```rust
// ✅ GOOD: Type-safe path building
let path = Path::root()
    .field("booking")
    .field("rooms")
    .index(0)
    .field("guest")
    .field("email");
err.push(path, "invalid_email", "Invalid email");

// ❌ BAD: String concatenation (error-prone)
err.push("booking.rooms[0].guest.email", "invalid_email", "Invalid email");
```

### 5. Include Context in Meta Fields

Help clients display better error messages:

```rust
// ✅ GOOD: Include useful context
let mut err = ValidationError::new();
let mut violation = Violation {
    path: "age".into(),
    code: "out_of_range",
    message: "Must be between 18 and 120".to_string(),
    meta: Meta::new(),
};
violation.meta.insert("min", "18");
violation.meta.insert("max", "120");
violation.meta.insert("actual", &self.age.to_string());
err.violations.push(violation);

// ❌ BAD: No context
err.push("age", "out_of_range", "Invalid age");
```

### 6. Test Error Paths

Verify error paths match your frontend expectations:

```rust
#[test]
fn test_nested_validation_paths() {
    let mut booking = Booking::new();
    booking.rooms[0].adults = 10;  // Invalid

    let err = booking.validate().unwrap_err();

    assert_eq!(err.violations[0].path.to_string(), "rooms[0].adults");
    assert_eq!(err.violations[0].code, "out_of_range");
}
```

### 7. Fail-Slow Accumulation

Collect **all** errors before returning:

```rust
// ✅ GOOD: Fail-slow - collect all errors
impl Validate for User {
    fn validate(&self) -> Result<(), ValidationError> {
        let mut err = ValidationError::new();

        if let Err(e) = validate("name", &self.name, &name_rule) {
            err.extend(e);
        }

        if let Err(e) = validate("email", &self.email, &email_rule) {
            err.extend(e);
        }

        if let Err(e) = validate("age", &self.age, &age_rule) {
            err.extend(e);
        }

        if err.is_empty() { Ok(()) } else { Err(err) }
    }
}

// ❌ BAD: Fail-fast - only first error
impl Validate for User {
    fn validate(&self) -> Result<(), ValidationError> {
        validate("name", &self.name, &name_rule)?;  // Stops here on error!
        validate("email", &self.email, &email_rule)?;
        validate("age", &self.age, &age_rule)?;
        Ok(())
    }
}
```

**Why fail-slow matters:**
- Users see **all** validation errors at once
- Better UX - fix all issues in one attempt
- Reduces frustration from incremental error discovery

### 8. Display Implementation

ValidationError implements `Display` for logging:

```rust
let err = ValidationError::single("email", "invalid", "Invalid email");

// Automatic formatting
println!("{}", err);  // "Validation error: Invalid email"

// Multiple errors
let mut err = ValidationError::new();
err.push("email", "invalid", "Invalid email");
err.push("age", "required", "Age required");

println!("{}", err);  // "Validation failed with 2 errors"
```

## See Also

- [Core Concepts](CORE_CONCEPTS.md) - Valid-by-construction types and domain modeling
- [Manual Validation](MANUAL_VALIDATION.md) - Implementing Validate trait manually
- [HTTP Integration](HTTP_INTEGRATION.md) - Framework adapters and error responses
- [Rules Reference](RULES.md) - Complete list of validation rules and error codes

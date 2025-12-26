# Domain Modeling Patterns

**Recommended patterns for building valid-by-construction domain types with domainstack.**

## Table of Contents

- [Valid-by-Construction Types](#valid-by-construction-types)
- [DTO → Domain Conversion](#dto--domain-conversion)
- [Smart Constructors](#smart-constructors)
- [Private Fields Pattern](#private-fields-pattern)
- [Manual vs Derive](#manual-vs-derive)
- [HTTP Integration](#http-integration)

## Valid-by-Construction Types

The core principle: **domain types that can only exist in valid states**.

### The Pattern

```rust
use domainstack::prelude::*;
use serde::Deserialize;

// DTO - Public, for deserialization (untrusted)
#[derive(Deserialize)]
pub struct UserDto {
    pub name: String,
    pub age: u8,
    pub email: String,
}

// Domain - Private fields, enforced validity (trusted)
#[derive(Debug, Validate)]
pub struct User {
    #[validate(length(min = 2, max = 50))]
    name: String,     // Private - can't be set directly!

    #[validate(range(min = 18, max = 120))]
    age: u8,

    #[validate(nested)]
    email: Email,
}

impl User {
    // Smart constructor - validation enforced here
    pub fn new(name: String, age: u8, email_raw: String) -> Result<Self, ValidationError> {
        let email = Email::new(email_raw).map_err(|e| e.prefixed("email"))?;

        let user = Self { name, age, email };
        user.validate()?;  // One line - validates all fields!
        Ok(user)
    }

    // Getters only - no setters
    pub fn name(&self) -> &str { &self.name }
    pub fn age(&self) -> u8 { self.age }
    pub fn email(&self) -> &Email { &self.email }
}
```

### Key Benefits

- **Type safety** - Can't accidentally use unvalidated data
- **Single validation point** - Validate once at construction
- **Self-documenting** - Type signature requires `User`, not raw values
- **Compiler-enforced** - Can't bypass validation
- **Impossible states** - Invalid users literally cannot exist in memory

## DTO → Domain Conversion

Use `TryFrom` to enforce validation at boundaries:

### Basic Conversion

```rust
impl TryFrom<UserDto> for User {
    type Error = ValidationError;

    fn try_from(dto: UserDto) -> Result<Self, Self::Error> {
        User::new(dto.name, dto.age, dto.email)
    }
}

// Usage in HTTP handler
async fn create_user(Json(dto): Json<UserDto>) -> Result<Json<User>, Error> {
    let user = User::try_from(dto)
        .map_err(|e| e.into_envelope_error())?;
    // user is GUARANTEED valid here!
    Ok(Json(user))
}
```

### Why TryFrom?

**Benefits:**
- Standard library trait - idiomatic Rust
- Type signature enforces validation
- Works with `?` operator
- Self-documenting conversion

**[x] Alternatives to avoid:**
```rust
// BAD: Direct construction bypasses validation
let user = User {
    name: dto.name,
    age: dto.age,
    email: Email(dto.email),  // No validation!
};
```

## Smart Constructors

Functions that return `Result<T, ValidationError>` and enforce invariants.

### Newtype Pattern

```rust
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
```

### Multi-Field Constructor

```rust
pub struct BookingRequest {
    check_in: NaiveDate,
    check_out: NaiveDate,
    rooms: Vec<Room>,
}

impl BookingRequest {
    pub fn new(
        check_in: NaiveDate,
        check_out: NaiveDate,
        rooms: Vec<Room>,
    ) -> Result<Self, ValidationError> {
        let booking = Self { check_in, check_out, rooms };
        booking.validate()?;  // Validates all fields + cross-field rules
        Ok(booking)
    }

    // Getters
    pub fn check_in(&self) -> NaiveDate { self.check_in }
    pub fn check_out(&self) -> NaiveDate { self.check_out }
    pub fn rooms(&self) -> &[Room] { &self.rooms }
}
```

### With Derive Macro

Use `#[derive(Validate)]` to eliminate boilerplate:

```rust
#[derive(Validate)]
#[validate(
    check = "self.check_out > self.check_in",
    message = "Check-out must be after check-in"
)]
pub struct BookingRequest {
    check_in: NaiveDate,
    check_out: NaiveDate,

    #[validate(min_items = 1, max_items = 10)]
    #[validate(each_nested)]
    rooms: Vec<Room>,
}

impl BookingRequest {
    pub fn new(
        check_in: NaiveDate,
        check_out: NaiveDate,
        rooms: Vec<Room>,
    ) -> Result<Self, ValidationError> {
        let booking = Self { check_in, check_out, rooms };
        booking.validate()?;  // Validates fields + cross-field check!
        Ok(booking)
    }
}
```

## Private Fields Pattern

### Why Private Fields Matter

```rust
// [x] BAD: Public fields allow invalid states
pub struct User {
    pub email: String,  // Anyone can set this!
    pub age: u8,
}

// Nothing prevents this:
let mut user = User { email: "valid@example.com".to_string(), age: 25 };
user.email = "not-an-email".to_string();  // Now invalid!
user.age = 200;  // Also invalid!
```

```rust
// GOOD: Private fields enforce invariants
#[derive(Validate)]
pub struct User {
    #[validate(email, max_len = 255)]
    email: String,  // Private!

    #[validate(range(min = 18, max = 120))]
    age: u8,
}

impl User {
    pub fn new(email: String, age: u8) -> Result<Self, ValidationError> {
        let user = Self { email, age };
        user.validate()?;
        Ok(user)
    }

    // Read-only access
    pub fn email(&self) -> &str { &self.email }
    pub fn age(&self) -> u8 { self.age }

    // Validated mutation (if needed)
    pub fn update_email(&mut self, new_email: String) -> Result<(), ValidationError> {
        validate("email", new_email.as_str(), &rules::email())?;
        self.email = new_email;
        Ok(())
    }
}
```

### Benefits

- **Encapsulation** - Hide implementation details
- **Invariant preservation** - Can't be modified without validation
- **API evolution** - Change internal representation without breaking consumers
- **Forced validation** - Only way to create instance is through constructor

## Manual vs Derive

### When to Use Derive

Use `#[derive(Validate)]` for:
- Straightforward field validation
- Cross-field validation with `check` attribute
- Standard error messages
- Less boilerplate

```rust
#[derive(Validate)]
pub struct User {
    #[validate(length(min = 2, max = 50))]
    name: String,

    #[validate(range(min = 18, max = 120))]
    age: u8,

    #[validate(nested)]
    email: Email,
}

impl User {
    pub fn new(name: String, age: u8, email: Email) -> Result<Self, ValidationError> {
        let user = Self { name, age, email };
        user.validate()?;  // ← 1 line validates everything!
        Ok(user)
    }
}
```

### When to Use Manual

Use manual `impl Validate` for:
- Custom error codes and messages
- Conditional validation logic
- Complex business rules
- Early returns for performance

```rust
impl User {
    pub fn new(name: String, age: u8, email: String) -> Result<Self, ValidationError> {
        let mut err = ValidationError::new();

        // Custom error code and message
        let name_rule = rules::min_len(2)
            .and(rules::max_len(50))
            .code("invalid_name")
            .message("Name must be between 2 and 50 characters");
        if let Err(e) = validate("name", name.as_str(), &name_rule) {
            err.extend(e);
        }

        // Conditional validation
        let age_rule = if self.is_enterprise {
            rules::range(21, 120)  // Enterprise: 21+
        } else {
            rules::range(18, 120)  // Standard: 18+
        };
        if let Err(e) = validate("age", &age, &age_rule) {
            err.extend(e);
        }

        // Nested validation with custom prefix
        let email = Email::new(email).map_err(|e| {
            e.prefixed("email")
                .map_messages(|msg| format!("Email error: {}", msg))
        })?;

        if !err.is_empty() {
            return Err(err);
        }

        Ok(Self { name, age, email })
    }
}
```

### Comparison

| Aspect | Derive | Manual |
|--------|--------|--------|
| **Boilerplate** | Low | High |
| **Readability** | High | Medium |
| **Custom errors** | Limited | Full control |
| **Conditional logic** | Limited | Full control |
| **Performance** | Good | Optimizable (early returns) |
| **Type safety** | Compile-time | Runtime |

## HTTP Integration

### With Framework Adapters

Use framework-specific adapters for automatic validation:

```rust
use domainstack_axum::{DomainJson, ErrorResponse};

// Automatic DTO → Domain conversion with validation
async fn create_user(
    DomainJson(dto, user): DomainJson<UserDto, User>
) -> Result<Json<User>, ErrorResponse> {
    // `user` is already validated via TryFrom!
    Ok(Json(save_user(user).await?))
}
```

### With error-envelope

Convert `ValidationError` to structured HTTP responses:

```rust
use domainstack_envelope::IntoEnvelopeError;

async fn create_user(Json(dto): Json<UserDto>) -> Result<Json<User>, Error> {
    let user = User::try_from(dto)
        .map_err(|e| e.into_envelope_error())?;
    Ok(Json(user))
}
```

**Produces structured error responses:**

```json
{
  "code": "VALIDATION",
  "status": 400,
  "message": "Validation failed with 2 errors",
  "details": {
    "fields": {
      "email": [{"code": "invalid_email", "message": "Invalid email format"}],
      "age": [{"code": "out_of_range", "message": "Must be between 18 and 120"}]
    }
  }
}
```

### Manual Error Handling

For custom error responses:

```rust
async fn create_user(Json(dto): Json<UserDto>) -> Result<Json<User>, (StatusCode, Json<ErrorResponse>)> {
    let user = User::try_from(dto).map_err(|e| {
        let field_errors: HashMap<String, Vec<String>> = e
            .field_violations_map()
            .into_iter()
            .map(|(path, violations)| {
                let messages = violations.iter()
                    .map(|v| v.message.clone())
                    .collect();
                (path, messages)
            })
            .collect();

        let response = ErrorResponse {
            error: "Validation failed".to_string(),
            fields: field_errors,
        };

        (StatusCode::BAD_REQUEST, Json(response))
    })?;

    Ok(Json(user))
}
```

## See Also

- **[Core Concepts](CORE_CONCEPTS.md)** - Foundation principles of domainstack
- **[Manual Validation](MANUAL_VALIDATION.md)** - Implementing Validate trait manually
- **[Derive Macro](DERIVE_MACRO.md)** - Using `#[derive(Validate)]`
- **[HTTP Integration](HTTP_INTEGRATION.md)** - Framework adapters and error responses
- **[Error Handling](ERROR_HANDLING.md)** - Working with ValidationError

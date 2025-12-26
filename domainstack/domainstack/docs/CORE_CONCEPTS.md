# Core Concepts

**Foundation principles of domainstack: valid-by-construction types, structured error paths, and composable rules.**

## Table of Contents

- [Valid-by-Construction Types](#valid-by-construction-types)
- [Structured Error Paths](#structured-error-paths)
- [Composable Rules](#composable-rules)
- [Domain vs DTO Separation](#domain-vs-dto-separation)
- [Smart Constructors](#smart-constructors)

## Valid-by-Construction Types

Domain types that enforce validity at construction time—invalid states become impossible to represent.

### Newtype Pattern

The simplest valid-by-construction type is a newtype wrapper with a validated constructor:

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
// [ok] If this succeeds, email is GUARANTEED valid

let invalid = Email::new("not-an-email".to_string());
// [error] Returns ValidationError - invalid email cannot exist!
```

**Key benefits:**
- Type safety: Can't accidentally use unvalidated strings
- Single validation point: Validate once at construction
- Self-documenting: Type signature requires `Email`, not `String`
- Compiler-enforced: Can't bypass validation

### Structs with Private Fields

For complex domain types, use private fields with validated constructors:

```rust
#[derive(Validate)]
pub struct User {
    #[validate(length(min = 2, max = 50))]
    name: String,      // Private - can't be set directly!

    #[validate(nested)]
    email: Email,

    #[validate(range(min = 18, max = 120))]
    age: u8,
}

impl User {
    pub fn new(name: String, email: Email, age: u8) -> Result<Self, ValidationError> {
        let user = Self { name, email, age };
        user.validate()?;  // One line validates all fields!
        Ok(user)
    }

    // Getters only - no setters
    pub fn name(&self) -> &str { &self.name }
    pub fn email(&self) -> &Email { &self.email }
    pub fn age(&self) -> u8 { self.age }
}
```

**Why private fields matter:**
- Prevents bypassing validation
- Forces use of validated constructor
- Makes invalid states unrepresentable
- Encapsulates invariants

## Structured Error Paths

domainstack provides a type-safe `Path` API for building error paths instead of string concatenation.

### Path API

```rust
use domainstack::Path;

// Simple field path
let path = Path::from("email");
// → "email"

// Nested fields
let path = Path::root()
    .field("guest")
    .field("email");
// → "guest.email"

// Collection indices
let path = Path::root()
    .field("rooms")
    .index(0)
    .field("adults");
// → "rooms[0].adults"

// Complex nesting
let path = Path::root()
    .field("booking")
    .field("rooms")
    .index(1)
    .field("guest")
    .field("email");
// → "booking.rooms[1].guest.email"
```

### Why Structured Paths?

**Type safety:**
```rust
// [ok] Type-safe - compiler catches mistakes
let path = Path::root().field("email");

// [error] String concat - typos become runtime bugs
let path = format!("emial");  // Oops!
```

**UI-friendly:**
```json
{
  "fields": {
    "rooms[0].adults": ["Must be between 1 and 4"],
    "rooms[1].children": ["Must be between 0 and 3"],
    "guest.email": ["Invalid email format"]
  }
}
```

Frontend can directly map these paths to form fields:
```typescript
// React example
errors["rooms[0].adults"]  // Highlight specific field
errors["guest.email"]      // Show error on email input
```

### Path Transformations

Paths can be prefixed when merging nested errors:

```rust
// Nested validation
let email_error = email.validate();  // Path: "value"

// Prefix when merging
err.merge_prefixed("guest.email", email_error);
// → "guest.email.value"
```

## Composable Rules

Rules in domainstack are **values**, not attributes. This enables reusable validation logic.

### Rules as Values

```rust
use domainstack::rules::*;

// Rules are just values - assign, pass around, store
let email_rule = email().and(max_len(255));
let name_rule = min_len(1).and(max_len(50));
let age_rule = range(18, 120);

// Use anywhere
validate("email", user_email, &email_rule)?;
validate("name", user_name, &name_rule)?;
validate("age", &user_age, &age_rule)?;
```

### Composition Operators

**`.and()` - Both rules must pass:**

```rust
let password_rule = min_len(8)
    .and(max_len(128))
    .and(matches_regex(r"[A-Z]"))   // Has uppercase
    .and(matches_regex(r"[0-9]"));  // Has digit
```

**`.or()` - At least one rule must pass:**

```rust
let flexible_id = alphanumeric()
    .or(matches_regex(r"^\d{4}-\d{4}$"));  // OR uuid format
```

**`.when()` - Conditional validation:**

```rust
let rule = some_rule.when(|value| should_validate(value));

// Example: only validate if not empty
let optional_url = url().when(|s: &String| !s.is_empty());
```

### Reusable Rule Libraries

Build domain-specific rule libraries:

```rust
// Company validation rules
pub mod company_rules {
    use domainstack::prelude::*;

    pub fn company_email() -> Rule<str> {
        rules::email()
            .and(rules::ends_with("@company.com"))
            .code("invalid_company_email")
            .message("Must use company email address")
    }

    pub fn employee_id() -> Rule<str> {
        rules::matches_regex(r"^EMP-\d{6}$")
            .code("invalid_employee_id")
            .message("Employee ID must be EMP-XXXXXX format")
    }
}

// Use across services
validate("email", email, &company_rules::company_email())?;
validate("id", id, &company_rules::employee_id())?;
```

### Builder Customization

Customize error codes, messages, and metadata:

```rust
let rule = email()
    .code("custom_email_error")
    .message("Please provide a valid email address")
    .meta("hint", "example@domain.com");

// All rules support customization
let age_rule = range(18, 65)
    .code("age_restriction")
    .message("Must be between 18 and 65 for this program")
    .meta("min", "18")
    .meta("max", "65");
```

## Domain vs DTO Separation

domainstack is built for the boundary pattern: **DTO → Domain conversion**.

### The Pattern

```
HTTP/JSON (untrusted)
    ↓
Gate 1: Serde (deserialize)
    ↓
DTO (untrusted struct)
    ↓
Gate 2: Domain (validate + convert)
    ↓
Domain (trusted, valid-by-construction)
```

### DTO at the Boundary

DTOs are public for deserialization:

```rust
use serde::Deserialize;

// DTO - Public fields, for deserialization
#[derive(Deserialize)]
pub struct UserDto {
    pub name: String,
    pub email: String,
    pub age: u8,
}
```

**DTOs are untrusted:**
- Just received from HTTP/JSON/YAML
- Passed Serde validation (type checking)
- NOT passed domain validation (business rules)

### Domain Inside

Domain types are private with validated constructors:

```rust
use domainstack::prelude::*;

// Domain - Private fields, enforced validity
#[derive(Validate)]
pub struct User {
    #[validate(length(min = 2, max = 50))]
    name: String,     // Private!

    #[validate(nested)]
    email: Email,

    #[validate(range(min = 18, max = 120))]
    age: u8,
}
```

**Domain types are trusted:**
- Private fields prevent direct construction
- Only creatable through validated constructors
- Business invariants guaranteed
- Safe to use in business logic

### Boundary Conversion

Use `TryFrom` to enforce validation at the boundary:

```rust
impl TryFrom<UserDto> for User {
    type Error = ValidationError;

    fn try_from(dto: UserDto) -> Result<Self, Self::Error> {
        // Convert untrusted DTO → trusted Domain
        let email = Email::new(dto.email)
            .map_err(|e| e.prefixed("email"))?;

        let user = Self {
            name: dto.name,
            email,
            age: dto.age,
        };

        user.validate()?;  // Validates all fields + cross-field rules
        Ok(user)
    }
}

// In HTTP handlers
let user = User::try_from(dto)?;  // One line - validates everything!
// user is GUARANTEED valid here
```

### Why Separation Matters

**Without separation:**
```rust
// [x] BAD: Same type for DTO and domain
#[derive(Deserialize, Validate)]
pub struct User {  // Public fields - anyone can set!
    pub name: String,
    pub email: String,
    pub age: u8,
}

// Nothing prevents this:
let invalid_user = User {
    name: "".to_string(),      // Empty name!
    email: "not-email",         // Invalid email!
    age: 200,                   // Invalid age!
};
// ↑ Invalid user exists in memory!
```

**With separation:**
```rust
// [ok] GOOD: DTO at boundary, Domain inside
let dto = UserDto { ... };  // From JSON - untrusted
let user = User::try_from(dto)?;  // Validates
// ↑ If this succeeds, user is guaranteed valid
// ↑ Invalid users cannot exist!
```

## Smart Constructors

Smart constructors are functions that return `Result<T, ValidationError>` and enforce invariants.

### Basic Smart Constructor

```rust
pub struct Email(String);

impl Email {
    // Smart constructor - enforces email validation
    pub fn new(raw: String) -> Result<Self, ValidationError> {
        validate("email", raw.as_str(), &rules::email())?;
        Ok(Self(raw))
    }

    // Accessor
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

// Usage
let email = Email::new("user@example.com".to_string())?;  // [ok] Valid
let bad = Email::new("invalid".to_string());               // [error] Returns error
```

### Multi-Field Smart Constructor

```rust
impl User {
    pub fn new(name: String, email_raw: String, age: u8) -> Result<Self, ValidationError> {
        let mut err = ValidationError::new();

        // Validate name
        if let Err(e) = validate("name", name.as_str(), &rules::min_len(2)) {
            err.extend(e);
        }

        // Validate email (nested)
        let email = match Email::new(email_raw) {
            Ok(e) => e,
            Err(e) => {
                err.merge_prefixed("email", e);
                return Err(err);
            }
        };

        // Validate age
        if let Err(e) = validate("age", &age, &rules::range(18, 120)) {
            err.extend(e);
        }

        if !err.is_empty() {
            return Err(err);
        }

        Ok(Self { name, email, age })
    }
}
```

### With Derive Macro

Simplify smart constructors with `#[derive(Validate)]`:

```rust
#[derive(Validate)]
pub struct User {
    #[validate(length(min = 2, max = 50))]
    name: String,

    #[validate(nested)]
    email: Email,

    #[validate(range(min = 18, max = 120))]
    age: u8,
}

impl User {
    pub fn new(name: String, email: Email, age: u8) -> Result<Self, ValidationError> {
        let user = Self { name, email, age };
        user.validate()?;  // One line - validates everything!
        Ok(user)
    }
}
```

### Cross-Field Smart Constructors

Enforce business rules between fields:

```rust
#[derive(Validate)]
#[validate(
    check = "self.check_out > self.check_in",
    message = "Check-out must be after check-in"
)]
pub struct Booking {
    check_in: NaiveDate,
    check_out: NaiveDate,

    #[validate(min_items = 1)]
    rooms: Vec<Room>,
}

impl Booking {
    pub fn new(
        check_in: NaiveDate,
        check_out: NaiveDate,
        rooms: Vec<Room>,
    ) -> Result<Self, ValidationError> {
        let booking = Self { check_in, check_out, rooms };
        booking.validate()?;  // Validates fields + cross-field rule!
        Ok(booking)
    }
}
```

**Benefits of smart constructors:**
- Single validation point
- Type signature enforces use
- Impossible to create invalid instances
- Self-documenting code
- Compiler-enforced correctness

## See Also

- [Manual Validation](MANUAL_VALIDATION.md) - Implementing Validate trait manually
- [Derive Macro](DERIVE_MACRO.md) - Declarative validation with `#[derive(Validate)]`
- [Error Handling](ERROR_HANDLING.md) - Working with ValidationError
- [Rules Reference](RULES.md) - Complete list of 37 built-in rules

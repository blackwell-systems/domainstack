# Type-State Validation

**Compile-time validation guarantees using phantom types. Make invalid states unrepresentable at the type level.**

## Table of Contents

- [Overview](#overview)
- [Why Type-State?](#why-type-state)
- [The Pattern](#the-pattern)
- [Single-Field Type-State](#single-field-type-state)
- [Multi-Field Type-State](#multi-field-type-state)
- [Builder Pattern Integration](#builder-pattern-integration)
- [Database Operations](#database-operations)
- [Workflow States](#workflow-states)
- [Performance](#performance)
- [When to Use Type-State](#when-to-use-type-state)
- [Best Practices](#best-practices)

## Overview

Type-state validation uses Rust's type system to enforce that validation has occurred. Instead of runtime checks, the **compiler** guarantees validated data.

```rust
use domainstack::typestate::{Validated, Unvalidated};

// Function only accepts validated emails
fn send_newsletter(email: Email<Validated>) {
    // Compiler GUARANTEES this email is valid!
    println!("Sending to: {}", email.as_str());
}

// Unvalidated email - cannot pass to send_newsletter
let raw = Email::new("user@example.com".to_string());

// Compile error! Expected Email<Validated>, got Email<Unvalidated>
// send_newsletter(raw);

// Must validate first
let validated = raw.validate()?;
send_newsletter(validated);  // Compiles!
```

## Why Type-State?

### Runtime Validation Problems

```rust
// Runtime approach - easy to forget validation
fn process_user(user: User) {
    // Did someone validate this user?
    // No way to know from the type signature!
    db.insert(user);  // Could insert invalid data!
}

// Defensive validation - wasteful
fn process_user(user: User) -> Result<(), Error> {
    user.validate()?;  // Might be validated already...
    user.validate()?;  // Are we validating twice?
    db.insert(user);
    Ok(())
}
```

### Type-State Solution

```rust
// Type-level guarantee - impossible to misuse
fn process_user(user: User<Validated>) {
    // Type system GUARANTEES user is valid
    // No runtime check needed
    db.insert(user);
}

// Caller must provide validated user
let validated = User::new(dto)?.validate()?;
process_user(validated);  // Compiler enforces this!
```

## The Pattern

### Phantom Type Markers

domainstack provides two marker types:

```rust
// In domainstack::typestate
pub struct Validated;
pub struct Unvalidated;
```

These are **zero-sized types** - they exist only at compile time.

### Basic Structure

```rust
use domainstack::typestate::{Validated, Unvalidated};
use std::marker::PhantomData;

pub struct Email<State = Unvalidated> {
    value: String,
    _state: PhantomData<State>,
}
```

**Key points:**
- `State` defaults to `Unvalidated`
- `PhantomData<State>` is zero-sized (no runtime cost)
- Methods are implemented separately for each state

## Single-Field Type-State

### Email Example

```rust
use domainstack::typestate::{Validated, Unvalidated};
use domainstack::{ValidationError, validate, rules};
use std::marker::PhantomData;

pub struct Email<State = Unvalidated> {
    value: String,
    _state: PhantomData<State>,
}

// Methods for Unvalidated state
impl Email<Unvalidated> {
    /// Create new unvalidated email
    pub fn new(value: String) -> Self {
        Self {
            value,
            _state: PhantomData,
        }
    }

    /// Validate and transition to Validated state
    pub fn validate(self) -> Result<Email<Validated>, ValidationError> {
        // Run validation rules
        validate("email", self.value.as_str(), &rules::email())?;
        validate("email", self.value.as_str(), &rules::max_len(255))?;

        // Transition to validated state
        Ok(Email {
            value: self.value,
            _state: PhantomData,
        })
    }
}

// Methods for Validated state
impl Email<Validated> {
    /// Get the email value (only available after validation)
    pub fn as_str(&self) -> &str {
        &self.value
    }

    /// Get the domain part
    pub fn domain(&self) -> &str {
        self.value.split('@').last().unwrap()
    }
}

// Common methods for both states
impl<State> Email<State> {
    /// Check if this is a specific domain (works for both states)
    pub fn is_domain(&self, domain: &str) -> bool {
        self.value.ends_with(&format!("@{}", domain))
    }
}
```

### Usage

```rust
// Create unvalidated email
let raw = Email::new("user@example.com".to_string());

// Cannot access value yet!
// raw.as_str();  // [x] Compile error - method not available

// Validate and get validated email
let email = raw.validate()?;

// Now we can access value
println!("Email: {}", email.as_str());  // Works!
println!("Domain: {}", email.domain());  // Works!

// Function that requires validated email
fn send_email(email: Email<Validated>) {
    // No need to check - type guarantees validity
    smtp.send(&email.as_str());
}

send_email(email);  // Compiles!
```

## Multi-Field Type-State

For structs with multiple fields:

```rust
use domainstack::typestate::{Validated, Unvalidated};
use domainstack::prelude::*;
use std::marker::PhantomData;

pub struct User<State = Unvalidated> {
    email: String,
    username: String,
    age: u8,
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

        // Validate email
        if let Err(e) = validate("email", &self.email, &rules::email()) {
            err.extend(e);
        }

        // Validate username
        let username_rule = rules::length(3, 50).and(rules::alphanumeric());
        if let Err(e) = validate("username", &self.username, &username_rule) {
            err.extend(e);
        }

        // Validate age
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

impl User<Validated> {
    pub fn email(&self) -> &str {
        &self.email
    }

    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn age(&self) -> u8 {
        self.age
    }
}
```

## Builder Pattern Integration

Type-state works naturally with the builder pattern:

```rust
use domainstack::typestate::{Validated, Unvalidated};
use domainstack::prelude::*;
use std::marker::PhantomData;

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

    /// Validate and transition to Validated state
    pub fn build(self) -> Result<UserBuilder<Validated>, ValidationError> {
        let mut err = ValidationError::new();

        // Check required fields
        let email = match self.email {
            Some(e) => e,
            None => {
                err.push("email", "required", "Email is required");
                return Err(err);
            }
        };

        let username = match self.username {
            Some(u) => u,
            None => {
                err.push("username", "required", "Username is required");
                return Err(err);
            }
        };

        let age = match self.age {
            Some(a) => a,
            None => {
                err.push("age", "required", "Age is required");
                return Err(err);
            }
        };

        // Validate fields
        if let Err(e) = validate("email", &email, &rules::email()) {
            err.extend(e);
        }

        if let Err(e) = validate("username", &username, &rules::min_len(3)) {
            err.extend(e);
        }

        if let Err(e) = validate("age", &age, &rules::range(18, 120)) {
            err.extend(e);
        }

        if !err.is_empty() {
            return Err(err);
        }

        Ok(UserBuilder {
            email: Some(email),
            username: Some(username),
            age: Some(age),
            _state: PhantomData,
        })
    }
}

impl UserBuilder<Validated> {
    /// Convert to User (only available after validation)
    pub fn into_user(self) -> User<Validated> {
        User {
            email: self.email.unwrap(),
            username: self.username.unwrap(),
            age: self.age.unwrap(),
            _state: PhantomData,
        }
    }
}

// Usage
let user = UserBuilder::new()
    .email("user@example.com".to_string())
    .username("johndoe".to_string())
    .age(25)
    .build()?           // Returns UserBuilder<Validated>
    .into_user();       // Returns User<Validated>

insert_user(&db, user).await?;  // Type-safe!
```

## Database Operations

Type-state prevents inserting unvalidated data:

```rust
pub struct UserRepository {
    db: PgPool,
}

impl UserRepository {
    /// Insert only accepts validated users
    pub async fn insert(&self, user: User<Validated>) -> Result<i64, sqlx::Error> {
        let id = sqlx::query!(
            "INSERT INTO users (email, username, age) VALUES ($1, $2, $3) RETURNING id",
            user.email(),
            user.username(),
            user.age() as i32
        )
        .fetch_one(&self.db)
        .await?
        .id;

        Ok(id)
    }

    /// Update only accepts validated users
    pub async fn update(&self, id: i64, user: User<Validated>) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE users SET email = $1, username = $2, age = $3 WHERE id = $4",
            user.email(),
            user.username(),
            user.age() as i32,
            id
        )
        .execute(&self.db)
        .await?;

        Ok(())
    }
}

// Usage
let user = User::new(
    "user@example.com".to_string(),
    "johndoe".to_string(),
    25
);

// Cannot insert unvalidated user!
// repo.insert(user).await?;  // [x] Compile error

// Must validate first
let validated = user.validate()?;
repo.insert(validated).await?;  // Compiles!
```

## Workflow States

Type-state can model complex workflows:

```rust
use std::marker::PhantomData;

// Workflow states
pub struct Draft;
pub struct Submitted;
pub struct Approved;
pub struct Rejected;

pub struct Document<State> {
    id: i64,
    title: String,
    content: String,
    _state: PhantomData<State>,
}

impl Document<Draft> {
    pub fn new(title: String, content: String) -> Self {
        Self {
            id: 0,
            title,
            content,
            _state: PhantomData,
        }
    }

    pub fn submit(self) -> Result<Document<Submitted>, ValidationError> {
        // Validate before submission
        let mut err = ValidationError::new();

        if self.title.is_empty() {
            err.push("title", "required", "Title is required");
        }

        if self.content.len() < 100 {
            err.push("content", "too_short", "Content must be at least 100 characters");
        }

        if !err.is_empty() {
            return Err(err);
        }

        Ok(Document {
            id: self.id,
            title: self.title,
            content: self.content,
            _state: PhantomData,
        })
    }
}

impl Document<Submitted> {
    pub fn approve(self, approver: &User<Validated>) -> Document<Approved> {
        // Log approval
        Document {
            id: self.id,
            title: self.title,
            content: self.content,
            _state: PhantomData,
        }
    }

    pub fn reject(self, reason: String) -> Document<Rejected> {
        Document {
            id: self.id,
            title: self.title,
            content: self.content,
            _state: PhantomData,
        }
    }
}

impl Document<Approved> {
    pub fn publish(&self) -> Result<(), PublishError> {
        // Only approved documents can be published
        publish_to_website(&self.title, &self.content)
    }
}

// Usage - compiler enforces workflow
let doc = Document::new("My Article".to_string(), content);

// Cannot publish draft!
// doc.publish();  // [x] Compile error - method doesn't exist

// Must go through workflow
let submitted = doc.submit()?;
let approved = submitted.approve(&admin);
approved.publish()?;  // Only works for approved documents
```

## Performance

### Zero Runtime Cost

`PhantomData` is a zero-sized type:

```rust
use std::mem::size_of;

// Without type-state
struct Email { value: String }
assert_eq!(size_of::<Email>(), size_of::<String>());

// With type-state
struct EmailTS<S> { value: String, _state: PhantomData<S> }
assert_eq!(size_of::<EmailTS<Validated>>(), size_of::<String>());

// Same size! PhantomData is zero-sized
```

### No Runtime Overhead

```rust
// This code:
let email = Email::new(value).validate()?;
send_email(email);

// Compiles to essentially:
validate(&value)?;
send_email(value);

// The PhantomData disappears completely
```

## When to Use Type-State

### Good Use Cases

| Scenario | Why Type-State? |
|----------|-----------------|
| Database operations | Prevent inserting unvalidated data |
| API boundaries | Enforce validation at entry points |
| Workflows | Model state transitions at type level |
| Sensitive operations | Require validated data for security |
| Builder patterns | Ensure all required fields are set |

### When Not to Use Type-State

| Scenario | Better Alternative |
|----------|-------------------|
| Simple validation | Use `#[derive(Validate)]` |
| Internal code | Trust validated boundaries |
| Performance-critical hot paths | Validate once at boundary |
| Prototyping | Add type-state later |

### Decision Guide

```
Need compile-time guarantees? ──────┐
                                    │
        ┌──── Yes ────┐             │
        ▼             │             │
  Multiple states? ───┘             │
        │                           │
        ├── Yes ── Use Type-State   │
        │                           │
        └── No ─── Use Newtype ─────┘
                                    │
        ┌──── No ─────┐             │
        ▼             │             │
  Use #[derive(Validate)]           │
                                    │
```

## Best Practices

### 1. Default to Unvalidated

```rust
// GOOD: Default is Unvalidated
pub struct Email<State = Unvalidated> { ... }

let email = Email::new(value);  // Email<Unvalidated>
```

### 2. Consume Self in Validation

```rust
// GOOD: Takes ownership, returns new type
pub fn validate(self) -> Result<Email<Validated>, ValidationError> { ... }

// [x] BAD: Doesn't enforce transition
pub fn validate(&self) -> Result<(), ValidationError> { ... }
```

### 3. Restrict Methods by State

```rust
// GOOD: as_str only available after validation
impl Email<Validated> {
    pub fn as_str(&self) -> &str { ... }
}

// [x] BAD: Allows access to potentially invalid value
impl<S> Email<S> {
    pub fn as_str(&self) -> &str { ... }
}
```

### 4. Document State Requirements

```rust
/// Insert a user into the database.
///
/// # Type Safety
///
/// This function only accepts `User<Validated>`, ensuring
/// the user has been validated before insertion.
pub async fn insert_user(user: User<Validated>) -> Result<i64, Error> {
    // ...
}
```

### 5. Use Type Aliases for Clarity

```rust
pub type ValidatedUser = User<Validated>;
pub type UnvalidatedUser = User<Unvalidated>;

// Clear function signatures
fn process(user: ValidatedUser) { ... }
```

### 6. Combine with Other Patterns

```rust
// Type-state + DTO boundary
impl TryFrom<UserDto> for User<Validated> {
    type Error = ValidationError;

    fn try_from(dto: UserDto) -> Result<Self, Self::Error> {
        User::new(dto.email, dto.username, dto.age).validate()
    }
}

// Usage in HTTP handler
async fn create_user(Json(dto): Json<UserDto>) -> Result<Json<User<Validated>>, Error> {
    let user = User::try_from(dto)?;
    Ok(Json(user))
}
```

## See Also

- [Core Concepts](CORE_CONCEPTS.md) - Valid-by-construction philosophy
- [Patterns](PATTERNS.md) - Domain modeling patterns
- [Manual Validation](MANUAL_VALIDATION.md) - Implementing `Validate` trait
- [Advanced Patterns](ADVANCED_PATTERNS.md) - Overview of advanced techniques
- Example: `domainstack/examples/phantom_types.rs`

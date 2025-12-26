# Advanced Patterns

**Advanced validation techniques: async validation, type-state, context-dependent validation, and more.**

## Table of Contents

- [Conditional Validation](#conditional-validation)
- [Validation with Context](#validation-with-context)
- [Async Validation](#async-validation)
- [Type-State Validation](#type-state-validation)
- [Best Practices](#best-practices)

## Conditional Validation

Validate fields based on runtime conditions using manual implementation.

### Basic Conditional Validation

```rust
use domainstack::prelude::*;

pub struct Order {
    pub requires_shipping: bool,
    pub shipping_address: Option<Address>,
    pub items: Vec<Item>,
}

impl Validate for Order {
    fn validate(&self) -> Result<(), ValidationError> {
        let mut err = ValidationError::new();

        // Always validate items
        if self.items.is_empty() {
            err.push("items", "min_items", "Order must have at least one item");
        }

        // Only validate shipping address if shipped
        if self.requires_shipping {
            match &self.shipping_address {
                Some(addr) => {
                    if let Err(e) = addr.validate() {
                        err.merge_prefixed("shipping_address", e);
                    }
                }
                None => {
                    err.push(
                        "shipping_address",
                        "required",
                        "Shipping address required for shipped orders"
                    );
                }
            }
        }

        if err.is_empty() { Ok(()) } else { Err(err) }
    }
}
```

### Conditional Rules

Use the `.when()` combinator for rule-level conditions:

```rust
use domainstack::prelude::*;

let optional_url_rule = rules::url()
    .when(|s: &String| !s.is_empty());

// URL validation only runs if string is not empty
validate("website", &website, &optional_url_rule)?;
```

### Multi-Branch Validation

Different validation based on type or category:

```rust
pub enum PaymentMethod {
    CreditCard { number: String, cvv: String },
    BankTransfer { iban: String },
    PayPal { email: String },
}

impl Validate for PaymentMethod {
    fn validate(&self) -> Result<(), ValidationError> {
        let mut err = ValidationError::new();

        match self {
            PaymentMethod::CreditCard { number, cvv } => {
                // Credit card validation
                if let Err(e) = validate("number", number, &rules::credit_card()) {
                    err.extend(e);
                }
                if let Err(e) = validate("cvv", cvv, &rules::matches_regex(r"^\d{3,4}$")) {
                    err.extend(e);
                }
            }
            PaymentMethod::BankTransfer { iban } => {
                // IBAN validation
                if let Err(e) = validate("iban", iban, &rules::iban()) {
                    err.extend(e);
                }
            }
            PaymentMethod::PayPal { email } => {
                // PayPal email validation
                if let Err(e) = validate("email", email, &rules::email()) {
                    err.extend(e);
                }
            }
        }

        if err.is_empty() { Ok(()) } else { Err(err) }
    }
}
```

## Validation with Context

Validate against external state like existing records, configuration, or runtime data.

### Basic Context Pattern

```rust
use std::collections::HashSet;
use domainstack::prelude::*;

pub struct ValidationContext {
    pub existing_emails: HashSet<String>,
    pub existing_usernames: HashSet<String>,
}

pub struct User {
    pub email: String,
    pub username: String,
    pub age: u8,
}

impl User {
    pub fn validate_with_context(
        &self,
        ctx: &ValidationContext
    ) -> Result<(), ValidationError> {
        let mut err = ValidationError::new();

        // Basic field validation
        if let Err(e) = validate("email", &self.email, &rules::email()) {
            err.extend(e);
        }

        if let Err(e) = validate("username", &self.username, &rules::min_len(3)) {
            err.extend(e);
        }

        if let Err(e) = validate("age", &self.age, &rules::range(18, 120)) {
            err.extend(e);
        }

        // Context-dependent validation
        if ctx.existing_emails.contains(&self.email) {
            err.push("email", "email_taken", "Email already exists");
        }

        if ctx.existing_usernames.contains(&self.username) {
            err.push("username", "username_taken", "Username already taken");
        }

        if err.is_empty() { Ok(()) } else { Err(err) }
    }
}

// Usage
let ctx = ValidationContext {
    existing_emails: vec!["alice@example.com".to_string()].into_iter().collect(),
    existing_usernames: vec!["alice".to_string()].into_iter().collect(),
};

let user = User {
    email: "bob@example.com".to_string(),
    username: "bob".to_string(),
    age: 25,
};

user.validate_with_context(&ctx)?;  // ✓ Valid
```

### Configuration-Based Validation

Adjust validation rules based on configuration:

```rust
pub struct ValidationConfig {
    pub min_password_length: usize,
    pub require_special_chars: bool,
}

pub struct PasswordChange {
    pub new_password: String,
}

impl PasswordChange {
    pub fn validate_with_config(
        &self,
        config: &ValidationConfig
    ) -> Result<(), ValidationError> {
        let mut err = ValidationError::new();

        // Dynamic length validation
        let length_rule = rules::min_len(config.min_password_length);
        if let Err(e) = validate("new_password", &self.new_password, &length_rule) {
            err.extend(e);
        }

        // Conditional special character requirement
        if config.require_special_chars {
            let special_char_rule = rules::matches_regex(r"[!@#$%^&*(),.?:{}|<>]");
            if let Err(e) = validate("new_password", &self.new_password, &special_char_rule) {
                err.extend(e);
            }
        }

        if err.is_empty() { Ok(()) } else { Err(err) }
    }
}
```

## Async Validation

Validate against external resources like databases, APIs, or caches using the `AsyncValidate` trait.

### When to Use Async Validation

Use async validation when you need:
- **Database uniqueness checks** - Email, username, phone number
- **External API validation** - VAT numbers, postal codes, credit cards
- **Rate limiting** - Login attempts, API calls
- **Real-time data checks** - Inventory availability, seat reservations

### AsyncValidate Trait

```rust
use domainstack::{AsyncValidate, ValidationError, ValidationContext, Path};
use async_trait::async_trait;

#[async_trait]
pub trait AsyncValidate {
    async fn validate_async(&self, ctx: &ValidationContext) -> Result<(), ValidationError>;
}
```

### Database Uniqueness Check

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

        // 1. Sync validation first (fast)
        if let Err(e) = self.validate() {
            err.extend(e);
            // Early return if basic validation fails
            return Err(err);
        }

        // 2. Async validation (I/O required)
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

### Using ValidationContext

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

### Axum Integration

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

### External API Validation

Validate data against external services:

```rust
use domainstack::{AsyncValidate, ValidationError, ValidationContext, Path};
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;

#[derive(Deserialize)]
struct VATResponse {
    is_valid: bool,
}

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

### Rate Limiting

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
        let mut redis = ctx.get_resource::<redis::Client>("redis")?
            .get_async_connection()
            .await?;

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

### Combining Sync and Async Validation

**Best practice:** Run synchronous validation first (fast), then async validation (slow):

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

### Error Handling

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

### Benefits of Async Validation

- **Database integrity** - Prevent duplicate records before insertion
- **External validation** - Verify data with third-party APIs
- **Rate limiting** - Protect against abuse
- **Real-time checks** - Validate against live data sources
- **Clean error messages** - Field-level errors just like sync validation

**Performance Tip:** Always run sync validation first to fail fast on basic errors before expensive I/O operations.

## Type-State Validation

Use phantom types for compile-time validation guarantees, preventing use of unvalidated data.

### Basic Type-State Pattern

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

### Multi-Field Type-State

```rust
use domainstack::typestate::{Validated, Unvalidated};
use domainstack::prelude::*;
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

### Builder Pattern Integration

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

    pub fn build(self) -> Result<UserBuilder<Validated>, ValidationError> {
        let mut err = ValidationError::new();

        // Check required fields
        let email = self.email.ok_or_else(|| {
            let mut e = ValidationError::new();
            e.push("email", "required", "Email is required");
            e
        })?;

        let username = self.username.ok_or_else(|| {
            let mut e = ValidationError::new();
            e.push("username", "required", "Username is required");
            e
        })?;

        let age = self.age.ok_or_else(|| {
            let mut e = ValidationError::new();
            e.push("age", "required", "Age is required");
            e
        })?;

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
let builder = UserBuilder::new()
    .email("user@example.com".to_string())
    .username("johndoe".to_string())
    .age(25)
    .build()?;  // Returns UserBuilder<Validated>

let user = builder.into_user();  // User<Validated>
insert_user(&db, user).await?;  // ✓ Type-safe!
```

### Benefits of Type-State Validation

- **Zero runtime cost** - `PhantomData` has size 0, no memory or CPU overhead
- **Compile-time safety** - Type system enforces validation occurred
- **Self-documenting** - Function signatures make validation requirements explicit
- **Builder pattern friendly** - Natural fit with builder APIs
- **Prevents misuse** - Cannot accidentally use unvalidated data

### Use Cases

- Database operations requiring validated data
- Business logic with validation boundaries
- Multi-step workflows with validation gates
- API handlers ensuring data is validated before processing
- Builder patterns with validation as final step

### See Also

- Complete module documentation: `domainstack/src/typestate.rs`
- Example: `domainstack/examples/phantom_types.rs`
- 9 unit tests demonstrating all patterns

## Best Practices

### 1. Use Derive Macro for Simple Cases

Less boilerplate for straightforward validation:

```rust
// ✅ GOOD: Derive for simple field validation
#[derive(Validate)]
pub struct User {
    #[validate(email)]
    email: String,

    #[validate(range(min = 18, max = 120))]
    age: u8,
}

// ❌ BAD: Manual implementation when derive would work
impl Validate for User {
    fn validate(&self) -> Result<(), ValidationError> {
        // ... 15 lines of boilerplate
    }
}
```

### 2. Manual Implementation for Complex Logic

Use manual `Validate` for cross-field or conditional validation:

```rust
// ✅ GOOD: Manual for complex business logic
impl Validate for Booking {
    fn validate(&self) -> Result<(), ValidationError> {
        let mut err = ValidationError::new();

        // Complex logic: discount only if total > $100
        if self.discount > 0.0 && self.total < 100.0 {
            err.push("discount", "invalid_discount",
                "Discount only available for orders over $100");
        }

        if err.is_empty() { Ok(()) } else { Err(err) }
    }
}
```

### 3. Compose Rules

Build reusable validation components:

```rust
// ✅ GOOD: Composable rules
pub mod company_rules {
    use domainstack::prelude::*;

    pub fn company_email() -> Rule<str> {
        rules::email()
            .and(rules::ends_with("@company.com"))
            .code("invalid_company_email")
            .message("Must use company email address")
    }
}

// Use across services
validate("email", email, &company_rules::company_email())?;
```

### 4. Structured Error Paths

Use Path API, not string formatting:

```rust
// ✅ GOOD: Type-safe path building
let path = Path::root()
    .field("booking")
    .field("rooms")
    .index(0)
    .field("guest");
err.merge_prefixed(path, nested_err);

// ❌ BAD: String concatenation (error-prone)
err.merge_prefixed("booking.rooms[0].guest", nested_err);
```

### 5. Framework-Agnostic Core

Keep domain logic separate from HTTP:

```rust
// ✅ GOOD: Domain validation independent of framework
impl Validate for User {
    fn validate(&self) -> Result<(), ValidationError> {
        // Pure domain logic - no Axum/Actix/Rocket dependencies
    }
}

// Framework adapter in HTTP layer
async fn create_user(DomainJson(_, user): DomainJson<UserDto, User>) {
    // Validation already happened in extractor
}
```

### 6. One Validation Point

Validate at domain boundaries, not everywhere:

```rust
// ✅ GOOD: Validate once at boundary
impl TryFrom<UserDto> for User {
    fn try_from(dto: UserDto) -> Result<Self, ValidationError> {
        let user = Self { ... };
        user.validate()?;  // Single validation point
        Ok(user)
    }
}

// ❌ BAD: Validating repeatedly throughout codebase
fn process_user(user: &User) -> Result<()> {
    user.validate()?;  // Why validate again?
    // ...
}
```

### 7. Use error-envelope for HTTP

Automatic structured responses:

```rust
// ✅ GOOD: Structured error responses
use domainstack_envelope::IntoEnvelopeError;

async fn create_user(Json(dto): Json<UserDto>) -> Result<Json<User>, ErrorResponse> {
    let user = User::try_from(dto)
        .map_err(|e| e.into_envelope_error())?;
    Ok(Json(user))
}

// ❌ BAD: Custom error handling per endpoint
async fn create_user(Json(dto): Json<UserDto>) -> Result<Json<User>, String> {
    match User::try_from(dto) {
        Ok(user) => Ok(Json(user)),
        Err(e) => {
            let msg = format!("Validation failed: {}", e);
            Err(msg)  // Lost field-level errors!
        }
    }
}
```

### 8. Custom Functions for Domain Rules

Encapsulate business logic:

```rust
// ✅ GOOD: Reusable domain validation
fn validate_password_strength(password: &str) -> Result<(), ValidationError> {
    let mut err = ValidationError::new();

    if !password.chars().any(|c| c.is_uppercase()) {
        err.push("", "no_uppercase", "Must contain uppercase letter");
    }

    if !password.chars().any(|c| c.is_numeric()) {
        err.push("", "no_digit", "Must contain digit");
    }

    if err.is_empty() { Ok(()) } else { Err(err) }
}

// Use in multiple places
impl Validate for PasswordChange {
    fn validate(&self) -> Result<(), ValidationError> {
        validate_password_strength(&self.new_password)
    }
}
```

## See Also

- [Core Concepts](CORE_CONCEPTS.md) - Foundation principles
- [Manual Validation](MANUAL_VALIDATION.md) - Implementing Validate trait
- [Error Handling](ERROR_HANDLING.md) - Working with ValidationError
- [HTTP Integration](HTTP_INTEGRATION.md) - Framework adapters
- [Examples](../domainstack/examples/) - Runnable code examples

# Async Validation

**Complete guide to async validation: database uniqueness checks, external API validation, rate limiting, and real-time data verification.**

## Table of Contents

- [Overview](#overview)
- [When to Use Async Validation](#when-to-use-async-validation)
- [The AsyncValidate Trait](#the-asyncvalidate-trait)
- [ValidationContext](#validationcontext)
- [Database Uniqueness Checks](#database-uniqueness-checks)
- [External API Validation](#external-api-validation)
- [Rate Limiting](#rate-limiting)
- [Real-Time Data Checks](#real-time-data-checks)
- [Framework Integration](#framework-integration)
- [Error Handling](#error-handling)
- [Performance Best Practices](#performance-best-practices)
- [Testing Async Validation](#testing-async-validation)

## Overview

Async validation extends domainstack's synchronous validation to handle I/O-bound checks:

- **Database queries** - Email uniqueness, username availability
- **External APIs** - VAT number validation, postal code lookup
- **Rate limiting** - Login attempts, API call limits
- **Real-time checks** - Inventory availability, seat reservations

```rust
use domainstack::{AsyncValidate, ValidationContext, ValidationError};

#[async_trait]
impl AsyncValidate for User {
    async fn validate_async(&self, ctx: &ValidationContext) -> Result<(), ValidationError> {
        // Sync validation first (fast)
        self.validate()?;

        // Async validation (I/O)
        let db = ctx.get_resource::<PgPool>("db")?;
        check_email_unique(db, &self.email).await?;

        Ok(())
    }
}
```

## When to Use Async Validation

**Use async validation when you need:**

| Scenario | Example | Why Async? |
|----------|---------|------------|
| Database uniqueness | Email, username | Requires DB query |
| External API validation | VAT numbers, addresses | Requires HTTP call |
| Rate limiting | Login attempts | Requires cache lookup |
| Real-time availability | Inventory, seats | Requires live data |
| Authorization checks | Permissions, quotas | Requires user lookup |

**Don't use async for:**

- Format validation (email regex, string length)
- Range checks (age between 18-120)
- Cross-field validation (password confirmation)
- Pattern matching (alphanumeric, URL format)

These are synchronous operations - use `#[derive(Validate)]` or `impl Validate`.

## The AsyncValidate Trait

```rust
use domainstack::{AsyncValidate, ValidationError, ValidationContext};
use async_trait::async_trait;

#[async_trait]
pub trait AsyncValidate {
    async fn validate_async(&self, ctx: &ValidationContext) -> Result<(), ValidationError>;
}
```

### Basic Implementation

```rust
use domainstack::{AsyncValidate, ValidationError, ValidationContext, Path, Validate};
use async_trait::async_trait;

pub struct User {
    pub email: String,
    pub username: String,
    pub age: u8,
}

// Implement sync validation with derive or manual
impl Validate for User {
    fn validate(&self) -> Result<(), ValidationError> {
        let mut err = ValidationError::new();

        if let Err(e) = validate("email", &self.email, &rules::email()) {
            err.extend(e);
        }
        if let Err(e) = validate("username", &self.username, &rules::min_len(3)) {
            err.extend(e);
        }
        if let Err(e) = validate("age", &self.age, &rules::range(18, 120)) {
            err.extend(e);
        }

        if err.is_empty() { Ok(()) } else { Err(err) }
    }
}

// Implement async validation for I/O checks
#[async_trait]
impl AsyncValidate for User {
    async fn validate_async(&self, ctx: &ValidationContext) -> Result<(), ValidationError> {
        let mut err = ValidationError::new();

        // 1. Run sync validation first (fast, catches obvious errors)
        if let Err(e) = self.validate() {
            return Err(e);  // Early return - no point hitting DB with invalid data
        }

        // 2. Run async validation (I/O required)
        let db = ctx.get_resource::<PgPool>("db")?;

        // Check email uniqueness
        if email_exists(db, &self.email).await? {
            err.push(
                Path::from("email"),
                "email_taken",
                "This email is already registered"
            );
        }

        // Check username uniqueness
        if username_exists(db, &self.username).await? {
            err.push(
                Path::from("username"),
                "username_taken",
                "This username is already taken"
            );
        }

        if err.is_empty() { Ok(()) } else { Err(err) }
    }
}
```

## ValidationContext

`ValidationContext` is a type-safe container for external resources (database pools, HTTP clients, caches).

### Creating a Context

```rust
use domainstack::ValidationContext;
use sqlx::PgPool;
use reqwest::Client;
use redis::Client as RedisClient;

// Create context with resources
let mut ctx = ValidationContext::new();
ctx.insert_resource("db", db_pool);
ctx.insert_resource("http", reqwest::Client::new());
ctx.insert_resource("cache", redis_client);

// Run async validation
user.validate_async(&ctx).await?;
```

### Accessing Resources

```rust
#[async_trait]
impl AsyncValidate for User {
    async fn validate_async(&self, ctx: &ValidationContext) -> Result<(), ValidationError> {
        // Get typed resource (returns Result)
        let db: &PgPool = ctx.get_resource("db")?;
        let http: &Client = ctx.get_resource("http")?;
        let cache: &RedisClient = ctx.get_resource("cache")?;

        // Use resources...
        Ok(())
    }
}
```

### Resource Type Safety

Resources are stored by key and type. Mismatched types return an error:

```rust
ctx.insert_resource("db", PgPool::connect(...).await?);

// Correct type
let db: &PgPool = ctx.get_resource("db")?;

// [x] Wrong type - returns error
let wrong: &MySqlPool = ctx.get_resource("db")?;  // Error!
```

## Database Uniqueness Checks

The most common async validation pattern.

### Email Uniqueness

```rust
use sqlx::{PgPool, query_scalar};

async fn email_exists(db: &PgPool, email: &str) -> Result<bool, sqlx::Error> {
    let exists: bool = query_scalar!(
        "SELECT EXISTS(SELECT 1 FROM users WHERE email = $1)",
        email
    )
    .fetch_one(db)
    .await?
    .unwrap_or(false);

    Ok(exists)
}

#[async_trait]
impl AsyncValidate for CreateUserRequest {
    async fn validate_async(&self, ctx: &ValidationContext) -> Result<(), ValidationError> {
        let mut err = ValidationError::new();

        // Sync validation first
        if let Err(e) = self.validate() {
            return Err(e);
        }

        let db = ctx.get_resource::<PgPool>("db")?;

        if email_exists(db, &self.email).await? {
            err.push(
                Path::from("email"),
                "email_taken",
                "This email address is already registered"
            );
        }

        if err.is_empty() { Ok(()) } else { Err(err) }
    }
}
```

### Multiple Uniqueness Checks

Run checks in parallel for better performance:

```rust
use futures::future::try_join;

#[async_trait]
impl AsyncValidate for CreateUserRequest {
    async fn validate_async(&self, ctx: &ValidationContext) -> Result<(), ValidationError> {
        let mut err = ValidationError::new();

        if let Err(e) = self.validate() {
            return Err(e);
        }

        let db = ctx.get_resource::<PgPool>("db")?;

        // Run both checks in parallel
        let (email_exists, username_exists) = try_join!(
            email_exists(db, &self.email),
            username_exists(db, &self.username)
        )?;

        if email_exists {
            err.push(Path::from("email"), "email_taken", "Email already registered");
        }

        if username_exists {
            err.push(Path::from("username"), "username_taken", "Username already taken");
        }

        if err.is_empty() { Ok(()) } else { Err(err) }
    }
}
```

### Update Scenario (Exclude Current User)

When updating, exclude the current record from uniqueness checks:

```rust
#[async_trait]
impl AsyncValidate for UpdateUserRequest {
    async fn validate_async(&self, ctx: &ValidationContext) -> Result<(), ValidationError> {
        let mut err = ValidationError::new();

        let db = ctx.get_resource::<PgPool>("db")?;
        let current_user_id = ctx.get_resource::<i64>("current_user_id")?;

        // Check if email is taken by someone else
        let email_taken: bool = query_scalar!(
            "SELECT EXISTS(SELECT 1 FROM users WHERE email = $1 AND id != $2)",
            self.email,
            current_user_id
        )
        .fetch_one(db)
        .await?
        .unwrap_or(false);

        if email_taken {
            err.push(Path::from("email"), "email_taken", "Email already in use");
        }

        if err.is_empty() { Ok(()) } else { Err(err) }
    }
}
```

## External API Validation

Validate data against third-party services.

### VAT Number Validation

```rust
use reqwest::Client;
use serde::Deserialize;

#[derive(Deserialize)]
struct VATResponse {
    valid: bool,
    country_code: String,
    company_name: Option<String>,
}

async fn validate_vat(client: &Client, country: &str, vat: &str) -> Result<bool, reqwest::Error> {
    let response = client
        .get(&format!("https://api.vatvalidator.eu/{}/{}", country, vat))
        .send()
        .await?
        .json::<VATResponse>()
        .await?;

    Ok(response.valid)
}

#[async_trait]
impl AsyncValidate for BusinessRegistration {
    async fn validate_async(&self, ctx: &ValidationContext) -> Result<(), ValidationError> {
        let mut err = ValidationError::new();

        if let Err(e) = self.validate() {
            return Err(e);
        }

        let http = ctx.get_resource::<Client>("http")?;

        // Validate VAT number with external API
        match validate_vat(http, &self.country, &self.vat_number).await {
            Ok(true) => {},  // Valid
            Ok(false) => {
                err.push(
                    Path::from("vat_number"),
                    "invalid_vat",
                    "VAT number is not valid for the specified country"
                );
            }
            Err(e) => {
                // Log the error but don't fail validation
                tracing::warn!("VAT validation API error: {}", e);
                // Option: Add a warning or skip this check
            }
        }

        if err.is_empty() { Ok(()) } else { Err(err) }
    }
}
```

### Address Verification

```rust
#[derive(Deserialize)]
struct AddressValidationResponse {
    valid: bool,
    standardized: Option<StandardizedAddress>,
    suggestions: Vec<AddressSuggestion>,
}

#[async_trait]
impl AsyncValidate for ShippingAddress {
    async fn validate_async(&self, ctx: &ValidationContext) -> Result<(), ValidationError> {
        let mut err = ValidationError::new();

        let http = ctx.get_resource::<Client>("http")?;

        let response = http
            .post("https://api.address-validator.com/validate")
            .json(&self)
            .send()
            .await?
            .json::<AddressValidationResponse>()
            .await?;

        if !response.valid {
            err.push(
                Path::root(),
                "invalid_address",
                "Could not verify this address. Please check and try again."
            );

            // Add suggestions as metadata if available
            if !response.suggestions.is_empty() {
                // You could include suggestions in the error metadata
            }
        }

        if err.is_empty() { Ok(()) } else { Err(err) }
    }
}
```

## Rate Limiting

Protect against abuse with rate-limited validation.

### Login Attempt Limiting

```rust
use redis::AsyncCommands;

#[async_trait]
impl AsyncValidate for LoginAttempt {
    async fn validate_async(&self, ctx: &ValidationContext) -> Result<(), ValidationError> {
        let mut err = ValidationError::new();

        // Get Redis connection
        let redis = ctx.get_resource::<redis::Client>("redis")?;
        let mut conn = redis.get_async_connection().await?;

        // Rate limit key (by IP or email)
        let key = format!("login_attempts:{}", self.ip_address);

        // Check current attempt count
        let attempts: i32 = conn.get(&key).await.unwrap_or(0);

        if attempts >= 5 {
            err.push(
                Path::root(),
                "rate_limited",
                "Too many login attempts. Please try again in 15 minutes."
            );
            return Err(err);
        }

        // Increment counter and set expiry
        let _: () = conn.incr(&key, 1).await?;
        let _: () = conn.expire(&key, 900).await?;  // 15 minutes

        // Continue with normal validation
        if let Err(e) = self.validate() {
            err.extend(e);
        }

        if err.is_empty() { Ok(()) } else { Err(err) }
    }
}
```

### API Call Limiting

```rust
#[async_trait]
impl AsyncValidate for APIRequest {
    async fn validate_async(&self, ctx: &ValidationContext) -> Result<(), ValidationError> {
        let mut err = ValidationError::new();

        let redis = ctx.get_resource::<redis::Client>("redis")?;
        let mut conn = redis.get_async_connection().await?;

        // Daily limit per API key
        let key = format!("api_calls:{}:{}", self.api_key, today_date());
        let calls: i32 = conn.get(&key).await.unwrap_or(0);

        let daily_limit = 10000;  // Could be dynamic per plan
        if calls >= daily_limit {
            err.push(
                Path::root(),
                "daily_limit_exceeded",
                format!("Daily API limit of {} calls exceeded", daily_limit)
            );
            return Err(err);
        }

        // Increment counter
        let _: () = conn.incr(&key, 1).await?;
        if calls == 0 {
            // Set expiry on first call
            let _: () = conn.expire(&key, 86400).await?;  // 24 hours
        }

        Ok(())
    }
}
```

## Real-Time Data Checks

Validate against live data sources.

### Inventory Availability

```rust
#[async_trait]
impl AsyncValidate for CartCheckout {
    async fn validate_async(&self, ctx: &ValidationContext) -> Result<(), ValidationError> {
        let mut err = ValidationError::new();

        let db = ctx.get_resource::<PgPool>("db")?;

        // Check inventory for each item
        for (i, item) in self.items.iter().enumerate() {
            let available: i32 = query_scalar!(
                "SELECT quantity FROM inventory WHERE product_id = $1",
                item.product_id
            )
            .fetch_one(db)
            .await?
            .unwrap_or(0);

            if item.quantity > available as u32 {
                err.push(
                    Path::root().field("items").index(i).field("quantity"),
                    "insufficient_stock",
                    format!(
                        "Only {} units available (requested {})",
                        available, item.quantity
                    )
                );
            }
        }

        if err.is_empty() { Ok(()) } else { Err(err) }
    }
}
```

### Seat Reservation

```rust
#[async_trait]
impl AsyncValidate for SeatReservation {
    async fn validate_async(&self, ctx: &ValidationContext) -> Result<(), ValidationError> {
        let mut err = ValidationError::new();

        let db = ctx.get_resource::<PgPool>("db")?;

        // Check each seat
        for (i, seat_id) in self.seat_ids.iter().enumerate() {
            let is_available: bool = query_scalar!(
                r#"
                SELECT NOT EXISTS(
                    SELECT 1 FROM reservations
                    WHERE seat_id = $1
                    AND event_id = $2
                    AND status IN ('reserved', 'confirmed')
                )
                "#,
                seat_id,
                self.event_id
            )
            .fetch_one(db)
            .await?
            .unwrap_or(false);

            if !is_available {
                err.push(
                    Path::root().field("seat_ids").index(i),
                    "seat_unavailable",
                    format!("Seat {} is no longer available", seat_id)
                );
            }
        }

        if err.is_empty() { Ok(()) } else { Err(err) }
    }
}
```

## Framework Integration

### Axum

```rust
use axum::{extract::State, Json};
use domainstack::{AsyncValidate, ValidationContext};
use domainstack_axum::ErrorResponse;
use sqlx::PgPool;

async fn create_user(
    State(db): State<PgPool>,
    Json(request): Json<CreateUserRequest>
) -> Result<Json<User>, ErrorResponse> {
    // Create validation context
    let mut ctx = ValidationContext::new();
    ctx.insert_resource("db", db.clone());

    // Run async validation
    request.validate_async(&ctx)
        .await
        .map_err(ErrorResponse::from)?;

    // User is valid and unique - proceed with creation
    let user = insert_user(&db, request).await?;
    Ok(Json(user))
}
```

### Actix-web

```rust
use actix_web::{web, HttpResponse};
use domainstack::{AsyncValidate, ValidationContext};
use domainstack_envelope::IntoEnvelopeError;
use sqlx::PgPool;

async fn create_user(
    db: web::Data<PgPool>,
    request: web::Json<CreateUserRequest>
) -> Result<HttpResponse, actix_web::Error> {
    let mut ctx = ValidationContext::new();
    ctx.insert_resource("db", db.get_ref().clone());

    request.validate_async(&ctx)
        .await
        .map_err(|e| {
            let envelope = e.into_envelope_error();
            actix_web::error::InternalError::from_response(
                envelope.clone(),
                HttpResponse::BadRequest().json(envelope)
            )
        })?;

    let user = insert_user(&db, request.into_inner()).await?;
    Ok(HttpResponse::Ok().json(user))
}
```

## Error Handling

### Combining Sync and Async Errors

```rust
#[async_trait]
impl AsyncValidate for User {
    async fn validate_async(&self, ctx: &ValidationContext) -> Result<(), ValidationError> {
        let mut err = ValidationError::new();

        // 1. Sync validation (always run first)
        if let Err(e) = self.validate() {
            // Option A: Return immediately (fail-fast for invalid format)
            return Err(e);

            // Option B: Accumulate with async errors (fail-slow)
            // err.extend(e);
        }

        // 2. Async validation
        let db = ctx.get_resource::<PgPool>("db")?;

        if email_exists(db, &self.email).await? {
            err.push(Path::from("email"), "email_taken", "Email already registered");
        }

        if err.is_empty() { Ok(()) } else { Err(err) }
    }
}
```

### Handling External Service Failures

```rust
#[async_trait]
impl AsyncValidate for VATRegistration {
    async fn validate_async(&self, ctx: &ValidationContext) -> Result<(), ValidationError> {
        let http = ctx.get_resource::<Client>("http")?;

        match validate_vat_external(http, &self.vat_number).await {
            Ok(is_valid) => {
                if !is_valid {
                    return Err(ValidationError::single(
                        Path::from("vat_number"),
                        "invalid_vat",
                        "VAT number is not valid"
                    ));
                }
            }
            Err(e) => {
                // Log the error
                tracing::error!("VAT validation service unavailable: {}", e);

                // Options:
                // 1. Skip validation (risky)
                // return Ok(());

                // 2. Return a specific error
                return Err(ValidationError::single(
                    Path::from("vat_number"),
                    "validation_unavailable",
                    "VAT validation service is temporarily unavailable. Please try again."
                ));

                // 3. Add warning but continue (with flag in context)
            }
        }

        Ok(())
    }
}
```

## Performance Best Practices

### 1. Run Sync Validation First

```rust
// GOOD: Check format before hitting database
if let Err(e) = self.validate() {
    return Err(e);  // Don't waste DB query on invalid email format
}

// Now check database
if email_exists(db, &self.email).await? { ... }
```

### 2. Parallelize Independent Checks

```rust
use futures::future::try_join3;

// GOOD: Run checks in parallel
let (email_exists, username_exists, phone_exists) = try_join3(
    check_email(db, &self.email),
    check_username(db, &self.username),
    check_phone(db, &self.phone)
).await?;
```

### 3. Use Connection Pooling

```rust
// GOOD: Reuse connections from pool
let db = ctx.get_resource::<PgPool>("db")?;

// [x] BAD: Create new connection per request
let conn = PgConnection::connect(&db_url).await?;
```

### 4. Cache Where Appropriate

```rust
// For data that doesn't change often (e.g., country codes, zip codes)
async fn validate_country_code(cache: &RedisClient, code: &str) -> bool {
    // Check cache first
    if let Ok(exists) = cache.get(&format!("country:{}", code)).await {
        return exists;
    }

    // Fallback to database/API
    let exists = check_country_external(code).await;

    // Cache result
    cache.set(&format!("country:{}", code), exists, 3600).await;

    exists
}
```

## Testing Async Validation

### Unit Testing with Mock Context

```rust
#[tokio::test]
async fn test_email_uniqueness() {
    // Setup test database
    let pool = setup_test_db().await;

    // Insert existing user
    sqlx::query!("INSERT INTO users (email) VALUES ($1)", "existing@example.com")
        .execute(&pool)
        .await
        .unwrap();

    // Create context
    let mut ctx = ValidationContext::new();
    ctx.insert_resource("db", pool);

    // Test: existing email should fail
    let user = CreateUserRequest {
        email: "existing@example.com".to_string(),
        username: "newuser".to_string(),
    };

    let result = user.validate_async(&ctx).await;
    assert!(result.is_err());

    let err = result.unwrap_err();
    assert_eq!(err.violations[0].code, "email_taken");

    // Test: new email should pass
    let new_user = CreateUserRequest {
        email: "new@example.com".to_string(),
        username: "newuser".to_string(),
    };

    let result = new_user.validate_async(&ctx).await;
    assert!(result.is_ok());
}
```

### Integration Testing

```rust
#[tokio::test]
async fn test_full_validation_flow() {
    let app = setup_test_app().await;

    // Test validation failure
    let response = app
        .post("/users")
        .json(&json!({
            "email": "existing@example.com",  // Already exists
            "username": "taken",               // Already exists
            "age": 25
        }))
        .await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body: serde_json::Value = response.json().await;
    assert!(body["details"]["fields"]["email"].is_array());
    assert!(body["details"]["fields"]["username"].is_array());
}
```

## See Also

- [Conditional Validation](CONDITIONAL_VALIDATION.md) - Runtime-determined validation
- [HTTP Integration](HTTP_INTEGRATION.md) - Framework adapters
- [Error Handling](ERROR_HANDLING.md) - Working with `ValidationError`
- [Advanced Patterns](ADVANCED_PATTERNS.md) - Overview of advanced techniques

# domainstack-axum

**Axum extractors for the [domainstack](https://crates.io/crates/domainstack) full-stack validation ecosystem**

[![Crates.io](https://img.shields.io/crates/v/domainstack-axum.svg)](https://crates.io/crates/domainstack-axum)
[![Documentation](https://docs.rs/domainstack-axum/badge.svg)](https://docs.rs/domainstack-axum)

Turn Axum handlers into one-line DTO→Domain conversions with automatic error responses.

## Quick Example

```rust
use domainstack::prelude::*;
use domainstack_axum::{DomainJson, ErrorResponse};
use axum::{routing::post, Router, Json};

// Define your type alias for clean handlers
type UserJson = DomainJson<User, UserDto>;

async fn create_user(
    UserJson { domain: user, .. }: UserJson
) -> Result<Json<User>, ErrorResponse> {
    // user is GUARANTEED valid here - no need to check!
    Ok(Json(save_user(user).await?))
}

let app = Router::new().route("/users", post(create_user));
```

## Installation

```toml
[dependencies]
domainstack-axum = "1.0"
domainstack = { version = "1.0", features = ["derive"] }
```

## What You Get

### `DomainJson<T, Dto>` Extractor

Deserializes JSON → validates DTO → converts to domain type → returns 400 on failure.

**Before (manual):**
```rust
async fn create_user(Json(dto): Json<UserDto>) -> Result<Json<User>, StatusCode> {
    // 20+ lines of validation boilerplate
    let user = User::try_from(dto).map_err(|_| StatusCode::BAD_REQUEST)?;
    Ok(Json(user))
}
```

**After (domainstack-axum):**
```rust
type UserJson = DomainJson<User, UserDto>;

async fn create_user(
    UserJson { domain: user, .. }: UserJson
) -> Result<Json<User>, ErrorResponse> {
    Ok(Json(user))  // user is valid!
}
```

### `ErrorResponse` Wrapper

Implements `IntoResponse` for `error_envelope::Error` with structured field-level errors.

**Error Response Format:**
```json
{
  "code": "VALIDATION",
  "message": "Validation failed with 2 errors",
  "status": 400,
  "retryable": false,
  "details": {
    "fields": {
      "name": [
        {
          "code": "min_length",
          "message": "Must be at least 2 characters",
          "meta": {"min": "2"}
        }
      ],
      "age": [
        {
          "code": "out_of_range",
          "message": "Must be between 18 and 120",
          "meta": {"min": "18", "max": "120"}
        }
      ]
    }
  }
}
```

## When to Use Which Extractor

### `DomainJson<T, Dto>` - Domain-First (Recommended)

**Validation happens during DTO→Domain conversion** (`TryFrom`).

```rust
type UserJson = DomainJson<User, UserDto>;

async fn create_user(
    UserJson { domain: user, .. }: UserJson
) -> Result<Json<User>, ErrorResponse> {
    // user is valid domain object
    Ok(Json(save_user(user).await?))
}
```

**Use when:**
- You have domain models with business invariants
- You want valid-by-construction types  
- You need domain logic separate from HTTP concerns
- You'll add async validation later (database checks, uniqueness, etc.)

**DTOs don't need `#[derive(Validate)]`** - validation lives in `TryFrom`.

### `ValidatedJson<Dto>` - DTO-First

**Validation happens on the DTO immediately after deserialization**.

```rust
use domainstack::Validate;

#[derive(Deserialize, Validate)]
struct QuickDto {
    #[validate(length(min = 2, max = 50))]
    name: String,
}

async fn quick_endpoint(
    ValidatedJson(dto): ValidatedJson<QuickDto>
) -> Json<QuickDto> {
    // dto has been validated, but not converted to domain
    Json(dto)
}
```

**Use when:**
- You're building simple CRUD endpoints
- You don't need domain conversion
- You want request-shape validation only
- DTOs are your domain (for simple services)

**DTOs must `#[derive(Validate)]`** - validation attributes drive behavior.

## Usage Patterns

### 1. Type Alias Pattern (Recommended)

Define one alias per endpoint/command:

```rust
use domainstack_axum::DomainJson;

type CreateUserJson = DomainJson<CreateUserCommand, CreateUserDto>;
type UpdateUserJson = DomainJson<UpdateUserCommand, UpdateUserDto>;

async fn create_user(
    CreateUserJson { domain: cmd, .. }: CreateUserJson
) -> Result<Json<User>, ErrorResponse> {
    Ok(Json(user_service.create(cmd).await?))
}
```

### 2. Inline Pattern

For one-off handlers:

```rust
async fn create_user(
    DomainJson { domain: user, .. }: DomainJson<User, UserDto>
) -> Result<Json<User>, ErrorResponse> {
    Ok(Json(user))
}
```

### 3. ErrorResponse Conversion

`ErrorResponse` implements `From<ValidationError>` and `From<error_envelope::Error>`:

```rust
async fn handler() -> Result<Json<Data>, ErrorResponse> {
    let validated = validate_something()?;  // ValidationError
    let data = fetch_data().await?;         // error_envelope::Error
    Ok(Json(data))
}
```

## Domain Modeling

### DTO → Domain Conversion

```rust
use domainstack::prelude::*;
use serde::Deserialize;

// DTO: Public, for deserialization
#[derive(Deserialize)]
pub struct CreateUserDto {
    pub name: String,
    pub email: String,
    pub age: u8,
}

// Domain: Private fields, valid-by-construction
pub struct CreateUserCommand {
    name: String,      // Private!
    email: Email,      // Validated newtype
    age: u8,
}

impl TryFrom<CreateUserDto> for CreateUserCommand {
    type Error = ValidationError;

    fn try_from(dto: CreateUserDto) -> Result<Self, Self::Error> {
        let mut err = ValidationError::new();

        // Validate name
        if let Err(e) = validate("name", dto.name.as_str(), 
                                 &rules::min_len(1).and(rules::max_len(50))) {
            err.extend(e);
        }

        // Convert email (newtype with validation)
        let email = Email::new(dto.email)
            .map_err(|e| e.prefixed("email"))?;

        // Validate age
        if let Err(e) = validate("age", &dto.age, &rules::range(18, 120)) {
            err.extend(e);
        }

        if !err.is_empty() {
            return Err(err);
        }

        Ok(Self {
            name: dto.name,
            email,
            age: dto.age,
        })
    }
}
```

### Handler

```rust
type CreateUserJson = DomainJson<CreateUserCommand, CreateUserDto>;

async fn create_user(
    CreateUserJson { domain: cmd, .. }: CreateUserJson
) -> Result<Json<UserId>, ErrorResponse> {
    let user_id = user_service.create(cmd).await?;
    Ok(Json(user_id))
}
```

## Design Notes

### Why `DomainJson<T, Dto>` instead of `DomainJson<T>`?

**Problem:** Rust cannot reliably infer the DTO type when multiple `TryFrom` implementations exist:

```rust
impl TryFrom<UserDto> for User { ... }
impl<T> TryFrom<T> for User where T: Into<User> { ... }  // Ambiguous!
```

**Solution:** Explicit DTO type parameter keeps compilation deterministic:

```rust
DomainJson<User, UserDto>  // Unambiguous
```

**Ergonomics:** Use type aliases to keep handlers clean:

```rust
type UserJson = DomainJson<User, UserDto>;
```

This is a deliberate design choice, not a compromise. It avoids:
- Marker trait boilerplate
- Coherence/inference games
- Unpredictable compilation errors

### Why `ErrorResponse` wrapper?

**Problem:** Rust's orphan rules prevent implementing foreign traits on foreign types:

```rust
impl IntoResponse for error_envelope::Error { ... }  // [x] Not allowed
//   ^^^^^^^^^^^^     ^^^^^^^^^^^^^^^^^^^^
//   foreign trait    foreign type
```

**Solution:** Newtype wrapper implements `IntoResponse`:

```rust
pub struct ErrorResponse(pub error_envelope::Error);

impl IntoResponse for ErrorResponse { ... }  // Allowed
impl From<error_envelope::Error> for ErrorResponse { ... }
impl From<ValidationError> for ErrorResponse { ... }
```

This is the canonical Rust pattern for working around orphan rules.

### Why no DTO field in `DomainJson`?

The DTO is consumed during validation via `TryFrom`. Keeping it would require:

1. **Cloning** - Extra allocations for every request
2. **Security risk** - Encourages reading unvalidated fields
3. **Conceptual confusion** - Breaks the DTO→Domain boundary

If you need the original request data for logging/debugging, capture it before the extractor:

```rust
async fn create_user(
    req: Request,  // Capture first if needed
    UserJson { domain: user, .. }: UserJson,
) -> Result<Json<User>, ErrorResponse> {
    tracing::debug!("Request: {:?}", req);
    Ok(Json(user))
}
```

## Error Handling

### Automatic 400 Responses

Validation errors automatically return HTTP 400 with structured field details:

```rust
POST /users {"name": "", "age": 200}

// Response: 400 Bad Request
{
  "code": "VALIDATION",
  "message": "Validation failed with 2 errors",
  "details": {
    "fields": {
      "name": [{"code": "min_length", "message": "Must be at least 1 characters"}],
      "age": [{"code": "out_of_range", "message": "Must be between 18 and 120"}]
    }
  }
}
```

### Malformed JSON

```rust
POST /users {invalid json

// Response: 400 Bad Request
{
  "code": "BAD_REQUEST",
  "message": "Invalid JSON: ..."
}
```

## Testing

```rust
use axum::routing::post;
use axum_test::TestServer;

#[tokio::test]
async fn test_validation() {
    let app = Router::new().route("/", post(create_user));
    let server = TestServer::new(app).unwrap();

    let response = server
        .post("/")
        .json(&serde_json::json!({"name": "", "age": 200}))
        .await;

    response.assert_status_bad_request();

    let body: serde_json::Value = response.json();
    assert_eq!(body["message"], "Validation failed with 2 errors");
}
```

## License

Apache 2.0

## Author

Dayna Blackwell - [blackwellsystems@protonmail.com](mailto:blackwellsystems@protonmail.com)

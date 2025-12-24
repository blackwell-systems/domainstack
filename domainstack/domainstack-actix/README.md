# domainstack-actix

**Actix-web extractors for the domainstack validation framework**

[![Crates.io](https://img.shields.io/crates/v/domainstack-actix.svg)](https://crates.io/crates/domainstack-actix)
[![Documentation](https://docs.rs/domainstack-actix/badge.svg)](https://docs.rs/domainstack-actix)

Turn Actix-web handlers into one-line DTO→Domain conversions with automatic error responses.

## Quick Example

```rust
use domainstack::prelude::*;
use domainstack_actix::{DomainJson, ErrorResponse};
use actix_web::{post, web, App, HttpServer};

// Define your type alias for clean handlers
type UserJson = DomainJson<User, UserDto>;

#[post("/users")]
async fn create_user(
    UserJson { domain: user, .. }: UserJson
) -> Result<web::Json<User>, ErrorResponse> {
    // user is GUARANTEED valid here - no need to check!
    Ok(web::Json(save_user(user).await?))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new().service(create_user)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
```

## Installation

```toml
[dependencies]
domainstack-actix = "0.4"
domainstack = { version = "0.3", features = ["derive"] }
```

## What You Get

### `DomainJson<T, Dto>` Extractor

Deserializes JSON → validates DTO → converts to domain type → returns 400 on failure.

**Before (manual):**
```rust
#[post("/users")]
async fn create_user(dto: web::Json<UserDto>) -> Result<web::Json<User>, actix_web::Error> {
    // 20+ lines of validation boilerplate
    let user = User::try_from(dto.into_inner())
        .map_err(|_| actix_web::error::ErrorBadRequest("Validation failed"))?;
    Ok(web::Json(user))
}
```

**After (domainstack-actix):**
```rust
type UserJson = DomainJson<User, UserDto>;

#[post("/users")]
async fn create_user(
    UserJson { domain: user, .. }: UserJson
) -> Result<web::Json<User>, ErrorResponse> {
    Ok(web::Json(user))  // user is valid!
}
```

### `ErrorResponse` Wrapper

Implements `ResponseError` for `error_envelope::Error` with structured field-level errors.

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

#[post("/users")]
async fn create_user(
    UserJson { domain: user, .. }: UserJson
) -> Result<web::Json<User>, ErrorResponse> {
    // user is valid domain object
    Ok(web::Json(save_user(user).await?))
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

#[post("/quick")]
async fn quick_endpoint(
    ValidatedJson(dto): ValidatedJson<QuickDto>
) -> web::Json<QuickDto> {
    // dto has been validated, but not converted to domain
    web::Json(dto)
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
use domainstack_actix::DomainJson;

type CreateUserJson = DomainJson<CreateUserCommand, CreateUserDto>;
type UpdateUserJson = DomainJson<UpdateUserCommand, UpdateUserDto>;

#[post("/users")]
async fn create_user(
    CreateUserJson { domain: cmd, .. }: CreateUserJson
) -> Result<web::Json<User>, ErrorResponse> {
    Ok(web::Json(user_service.create(cmd).await?))
}
```

### 2. Inline Pattern

For one-off handlers:

```rust
#[post("/users")]
async fn create_user(
    DomainJson { domain: user, .. }: DomainJson<User, UserDto>
) -> Result<web::Json<User>, ErrorResponse> {
    Ok(web::Json(user))
}
```

### 3. ErrorResponse Conversion

`ErrorResponse` implements `From<ValidationError>` and `From<error_envelope::Error>`:

```rust
#[post("/data")]
async fn handler() -> Result<web::Json<Data>, ErrorResponse> {
    let validated = validate_something()?;  // ValidationError
    let data = fetch_data().await?;         // error_envelope::Error
    Ok(web::Json(data))
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

#[post("/users")]
async fn create_user(
    CreateUserJson { domain: cmd, .. }: CreateUserJson
) -> Result<web::Json<UserId>, ErrorResponse> {
    let user_id = user_service.create(cmd).await?;
    Ok(web::Json(user_id))
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
impl ResponseError for error_envelope::Error { ... }  // ❌ Not allowed
//   ^^^^^^^^^^^^^     ^^^^^^^^^^^^^^^^^^^^
//   foreign trait     foreign type
```

**Solution:** Newtype wrapper implements `ResponseError`:

```rust
pub struct ErrorResponse(pub error_envelope::Error);

impl ResponseError for ErrorResponse { ... }  // ✅ Allowed
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
#[post("/users")]
async fn create_user(
    req: HttpRequest,  // Capture first if needed
    UserJson { domain: user, .. }: UserJson,
) -> Result<web::Json<User>, ErrorResponse> {
    tracing::debug!("Request: {:?}", req);
    Ok(web::Json(user))
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
use actix_web::{test, web, App};

#[actix_rt::test]
async fn test_validation() {
    let app = test::init_service(
        App::new().route("/", web::post().to(create_user))
    ).await;

    let req = test::TestRequest::post()
        .uri("/")
        .set_json(&serde_json::json!({"name": "", "age": 200}))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["message"], "Validation failed with 2 errors");
}
```

## Complete Example

```rust
use actix_web::{get, post, web, App, HttpServer};
use domainstack::prelude::*;
use domainstack_actix::{DomainJson, ErrorResponse};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct CreateBookingDto {
    pub guest_name: String,
    pub email: String,
    pub nights: u8,
}

#[derive(Debug, Clone)]
pub struct Email(String);

impl Email {
    pub fn new(raw: String) -> Result<Self, ValidationError> {
        validate("value", raw.as_str(), &rules::email())?;
        Ok(Self(raw))
    }
}

pub struct CreateBookingCommand {
    guest_name: String,
    email: Email,
    nights: u8,
}

impl TryFrom<CreateBookingDto> for CreateBookingCommand {
    type Error = ValidationError;

    fn try_from(dto: CreateBookingDto) -> Result<Self, Self::Error> {
        let mut err = ValidationError::new();

        if let Err(e) = validate("guest_name", dto.guest_name.as_str(),
                                 &rules::min_len(1)) {
            err.extend(e);
        }

        let email = match Email::new(dto.email) {
            Ok(email) => Some(email),
            Err(e) => {
                err.extend(e.prefixed("email"));
                None
            }
        };

        if let Err(e) = validate("nights", &dto.nights, &rules::range(1, 30)) {
            err.extend(e);
        }

        if !err.is_empty() {
            return Err(err);
        }

        Ok(Self {
            guest_name: dto.guest_name,
            email: email.unwrap(),
            nights: dto.nights,
        })
    }
}

#[derive(Serialize)]
pub struct Booking {
    pub id: String,
    pub guest_name: String,
}

type CreateBookingJson = DomainJson<CreateBookingCommand, CreateBookingDto>;

#[post("/bookings")]
async fn create_booking(
    CreateBookingJson { domain: cmd, .. }: CreateBookingJson,
) -> Result<web::Json<Booking>, ErrorResponse> {
    let booking = Booking {
        id: "123".to_string(),
        guest_name: cmd.guest_name,
    };
    Ok(web::Json(booking))
}

#[get("/health")]
async fn health() -> &'static str {
    "OK"
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(health)
            .service(create_booking)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
```

## License

Apache 2.0

## Author

Dayna Blackwell - [blackwellsystems@protonmail.com](mailto:blackwellsystems@protonmail.com)

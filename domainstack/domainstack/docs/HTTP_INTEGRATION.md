# HTTP Integration

**Complete guide to integrating domainstack validation with web frameworks: Axum, Actix-web, and Rocket.**

## Table of Contents

- [Overview](#overview)
- [Error Response Format](#error-response-format)
- [Axum Integration](#axum-integration)
- [Actix-web Integration](#actix-web-integration)
- [Rocket Integration](#rocket-integration)
- [Framework Comparison](#framework-comparison)
- [Domain Modeling for HTTP](#domain-modeling-for-http)
- [Error Customization](#error-customization)
- [Client-Side Error Handling](#client-side-error-handling)
- [Testing HTTP Endpoints](#testing-http-endpoints)

## Overview

domainstack provides framework adapters that convert validation errors to structured HTTP responses automatically:

| Framework | Crate | Extractor |
|-----------|-------|-----------|
| **Axum** | `domainstack-axum` | `DomainJson<T, Dto>`, `ValidatedJson<Dto>` |
| **Actix-web** | `domainstack-actix` | `DomainJson<T, Dto>`, `ValidatedJson<Dto>` |
| **Rocket** | `domainstack-rocket` | `DomainJson<T, Dto>`, `ValidatedJson<Dto>` |

**All adapters provide:**
- Automatic JSON deserialization
- DTO → Domain conversion with validation
- Structured error responses (400 Bad Request)
- Field-level error paths for UI integration

## Error Response Format

All frameworks return the same structured error format:

```json
{
  "code": "VALIDATION",
  "status": 400,
  "message": "Validation failed with 3 errors",
  "retryable": false,
  "details": {
    "fields": {
      "email": [
        {
          "code": "invalid_email",
          "message": "Invalid email format"
        }
      ],
      "rooms[0].adults": [
        {
          "code": "out_of_range",
          "message": "Must be between 1 and 4",
          "meta": {"min": "1", "max": "4"}
        }
      ],
      "rooms[1].children": [
        {
          "code": "out_of_range",
          "message": "Must be between 0 and 3",
          "meta": {"min": "0", "max": "3"}
        }
      ]
    }
  }
}
```

**Key features:**
- **`code`**: Machine-readable error code for programmatic handling
- **`message`**: Human-readable message for display
- **`meta`**: Additional context (validation limits, patterns, etc.)
- **Field paths**: Include array indices (`rooms[0].adults`) for precise UI targeting

## Axum Integration

### Installation

```toml
[dependencies]
domainstack-axum = "1.0"
domainstack = { version = "1.0", features = ["derive"] }
axum = "0.7"
```

### DomainJson Extractor

The recommended approach - validates during DTO→Domain conversion:

```rust
use axum::{routing::post, Router, Json};
use domainstack::prelude::*;
use domainstack_axum::{DomainJson, ErrorResponse};
use serde::Deserialize;

// DTO for deserialization
#[derive(Deserialize)]
struct CreateUserDto {
    name: String,
    email: String,
    age: u8,
}

// Domain type with validation
#[derive(Validate, serde::Serialize)]
struct User {
    #[validate(length(min = 2, max = 50))]
    name: String,

    #[validate(email)]
    email: String,

    #[validate(range(min = 18, max = 120))]
    age: u8,
}

impl TryFrom<CreateUserDto> for User {
    type Error = ValidationError;

    fn try_from(dto: CreateUserDto) -> Result<Self, Self::Error> {
        let user = Self {
            name: dto.name,
            email: dto.email,
            age: dto.age,
        };
        user.validate()?;
        Ok(user)
    }
}

// Type alias for cleaner handlers
type CreateUserJson = DomainJson<User, CreateUserDto>;

async fn create_user(
    CreateUserJson { domain: user, .. }: CreateUserJson
) -> Result<Json<User>, ErrorResponse> {
    // user is GUARANTEED valid here!
    Ok(Json(user))
}

let app = Router::new().route("/users", post(create_user));
```

### ValidatedJson Extractor

For simpler cases where the DTO is your domain type:

```rust
use domainstack::Validate;
use domainstack_axum::ValidatedJson;

#[derive(Deserialize, Validate)]
struct QuickRequest {
    #[validate(length(min = 1, max = 100))]
    query: String,

    #[validate(range(min = 1, max = 100))]
    limit: u32,
}

async fn search(
    ValidatedJson(request): ValidatedJson<QuickRequest>
) -> Json<SearchResults> {
    // request is validated
    perform_search(request.query, request.limit).await
}
```

### Complete Axum Example

```rust
use axum::{
    routing::{get, post, put},
    Router, Json,
    extract::{Path, State},
};
use domainstack::prelude::*;
use domainstack_axum::{DomainJson, ErrorResponse};
use sqlx::PgPool;

// Types
type CreateBookingJson = DomainJson<Booking, CreateBookingDto>;
type UpdateBookingJson = DomainJson<UpdateBooking, UpdateBookingDto>;

// Handlers
async fn create_booking(
    State(db): State<PgPool>,
    CreateBookingJson { domain: booking, .. }: CreateBookingJson
) -> Result<Json<Booking>, ErrorResponse> {
    let saved = save_booking(&db, booking).await?;
    Ok(Json(saved))
}

async fn update_booking(
    State(db): State<PgPool>,
    Path(id): Path<i64>,
    UpdateBookingJson { domain: update, .. }: UpdateBookingJson
) -> Result<Json<Booking>, ErrorResponse> {
    let updated = update_booking_in_db(&db, id, update).await?;
    Ok(Json(updated))
}

// Router
let app = Router::new()
    .route("/bookings", post(create_booking))
    .route("/bookings/:id", put(update_booking))
    .with_state(db_pool);
```

**Full documentation:** [domainstack-axum README](../../domainstack-axum/README.md)

## Actix-web Integration

### Installation

```toml
[dependencies]
domainstack-actix = "1.0"
domainstack = { version = "1.0", features = ["derive"] }
actix-web = "4"
```

### DomainJson Extractor

```rust
use actix_web::{post, web, App, HttpServer};
use domainstack::prelude::*;
use domainstack_actix::{DomainJson, ErrorResponse};
use serde::Deserialize;

#[derive(Deserialize)]
struct CreateUserDto {
    name: String,
    email: String,
    age: u8,
}

#[derive(Validate, serde::Serialize)]
struct User {
    #[validate(length(min = 2, max = 50))]
    name: String,

    #[validate(email)]
    email: String,

    #[validate(range(min = 18, max = 120))]
    age: u8,
}

impl TryFrom<CreateUserDto> for User {
    type Error = ValidationError;

    fn try_from(dto: CreateUserDto) -> Result<Self, Self::Error> {
        let user = Self {
            name: dto.name,
            email: dto.email,
            age: dto.age,
        };
        user.validate()?;
        Ok(user)
    }
}

type UserJson = DomainJson<User, CreateUserDto>;

#[post("/users")]
async fn create_user(
    UserJson { domain: user, .. }: UserJson
) -> Result<web::Json<User>, ErrorResponse> {
    Ok(web::Json(user))
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

### ValidatedJson Extractor

```rust
use domainstack::Validate;
use domainstack_actix::ValidatedJson;

#[derive(Deserialize, Validate)]
struct SearchRequest {
    #[validate(length(min = 1, max = 100))]
    query: String,
}

#[post("/search")]
async fn search(
    ValidatedJson(request): ValidatedJson<SearchRequest>
) -> web::Json<SearchResults> {
    web::Json(perform_search(request.query).await)
}
```

### Complete Actix-web Example

```rust
use actix_web::{web, App, HttpServer, post, put};
use domainstack::prelude::*;
use domainstack_actix::{DomainJson, ErrorResponse};
use sqlx::PgPool;

type CreateBookingJson = DomainJson<Booking, CreateBookingDto>;

#[post("/bookings")]
async fn create_booking(
    db: web::Data<PgPool>,
    CreateBookingJson { domain: booking, .. }: CreateBookingJson
) -> Result<web::Json<Booking>, ErrorResponse> {
    let saved = save_booking(&db, booking).await?;
    Ok(web::Json(saved))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let pool = PgPool::connect(&db_url).await.unwrap();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .service(create_booking)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
```

**Note:** Actix extractor uses `block_on()` for synchronous validation in async context. This is the standard pattern for Actix-web 4.x extractors.

**Full documentation:** [domainstack-actix README](../../domainstack-actix/README.md)

## Rocket Integration

### Installation

```toml
[dependencies]
domainstack-rocket = "1.0"
domainstack = { version = "1.0", features = ["derive"] }
rocket = "0.5"
```

### DomainJson Request Guard

```rust
use rocket::{post, routes, serde::json::Json};
use domainstack::prelude::*;
use domainstack_rocket::{DomainJson, ErrorResponse};
use serde::Deserialize;

#[derive(Deserialize)]
struct CreateUserDto {
    name: String,
    email: String,
    age: u8,
}

#[derive(Validate, serde::Serialize)]
struct User {
    #[validate(length(min = 2, max = 50))]
    name: String,

    #[validate(email)]
    email: String,

    #[validate(range(min = 18, max = 120))]
    age: u8,
}

impl TryFrom<CreateUserDto> for User {
    type Error = ValidationError;

    fn try_from(dto: CreateUserDto) -> Result<Self, Self::Error> {
        let user = Self {
            name: dto.name,
            email: dto.email,
            age: dto.age,
        };
        user.validate()?;
        Ok(user)
    }
}

#[post("/users", data = "<user>")]
fn create_user(
    user: DomainJson<User, CreateUserDto>
) -> Result<Json<User>, ErrorResponse> {
    Ok(Json(user.domain))
}
```

### Error Catcher (Required)

Rocket requires an error catcher for proper error handling:

```rust
use rocket::{catch, catchers, Request};
use domainstack_rocket::ErrorResponse;

#[catch(400)]
fn validation_catcher(req: &Request) -> ErrorResponse {
    req.local_cache(|| None::<ErrorResponse>)
        .clone()
        .unwrap_or_else(|| {
            ErrorResponse(Box::new(error_envelope::Error::bad_request("Bad Request")))
        })
}

#[rocket::main]
async fn main() {
    rocket::build()
        .mount("/", routes![create_user])
        .register("/", catchers![validation_catcher])  // Required!
        .launch()
        .await
        .unwrap();
}
```

**Full documentation:** [domainstack-rocket README](../../domainstack-rocket/README.md)

## Framework Comparison

| Feature | Axum | Actix-web | Rocket |
|---------|------|-----------|--------|
| **DomainJson** | ✅ | ✅ | ✅ |
| **ValidatedJson** | ✅ | ✅ | ✅ |
| **Automatic ErrorResponse** | ✅ | ✅ | ✅ (needs catcher) |
| **Async Validation** | ✅ Native | ⚠️ `block_on()` | ✅ Native |
| **Setup Complexity** | Low | Low | Medium |
| **Type Safety** | High | High | High |

### When to Use Each

**Axum**: Modern, tower-based, excellent for new projects. Best async story.

**Actix-web**: Battle-tested, high performance. Uses `block_on()` for sync validation in async context.

**Rocket**: Developer-friendly, macro-heavy. Requires error catcher registration.

## Domain Modeling for HTTP

### DTO → Domain Pattern

```rust
// DTO: Public fields for deserialization
#[derive(Deserialize)]
pub struct CreateBookingDto {
    pub guest_email: String,
    pub check_in: String,
    pub check_out: String,
    pub rooms: Vec<RoomDto>,
}

// Domain: Private fields, business invariants
#[derive(Validate)]
#[validate(
    check = "self.check_out > self.check_in",
    message = "Check-out must be after check-in"
)]
pub struct Booking {
    #[validate(email)]
    guest_email: String,

    check_in: NaiveDate,
    check_out: NaiveDate,

    #[validate(min_items = 1, max_items = 5)]
    #[validate(each(nested))]
    rooms: Vec<Room>,
}

impl TryFrom<CreateBookingDto> for Booking {
    type Error = ValidationError;

    fn try_from(dto: CreateBookingDto) -> Result<Self, Self::Error> {
        // Parse dates
        let check_in = NaiveDate::parse_from_str(&dto.check_in, "%Y-%m-%d")
            .map_err(|_| ValidationError::single("check_in", "invalid_date", "Invalid date format"))?;

        let check_out = NaiveDate::parse_from_str(&dto.check_out, "%Y-%m-%d")
            .map_err(|_| ValidationError::single("check_out", "invalid_date", "Invalid date format"))?;

        // Convert rooms
        let rooms: Result<Vec<Room>, _> = dto.rooms
            .into_iter()
            .enumerate()
            .map(|(i, r)| Room::try_from(r).map_err(|e| e.prefixed(format!("rooms[{}]", i))))
            .collect();

        let booking = Self {
            guest_email: dto.guest_email,
            check_in,
            check_out,
            rooms: rooms?,
        };

        booking.validate()?;
        Ok(booking)
    }
}
```

### Type Aliases for Clean Handlers

```rust
// Define once
type CreateBookingJson = DomainJson<Booking, CreateBookingDto>;
type UpdateBookingJson = DomainJson<UpdateBooking, UpdateBookingDto>;
type CancelBookingJson = DomainJson<CancelBooking, CancelBookingDto>;

// Use in handlers
async fn create_booking(CreateBookingJson { domain: booking, .. }: CreateBookingJson) { ... }
async fn update_booking(UpdateBookingJson { domain: update, .. }: UpdateBookingJson) { ... }
async fn cancel_booking(CancelBookingJson { domain: cancel, .. }: CancelBookingJson) { ... }
```

## Error Customization

### Using error-envelope Directly

```rust
use domainstack_envelope::IntoEnvelopeError;

async fn create_user(
    Json(dto): Json<CreateUserDto>
) -> Result<Json<User>, ErrorResponse> {
    let user = User::try_from(dto)
        .map_err(|e| ErrorResponse::from(e.into_envelope_error()))?;

    Ok(Json(user))
}
```

### Custom Error Responses

```rust
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Serialize)]
struct ApiError {
    success: bool,
    errors: BTreeMap<String, Vec<FieldError>>,
}

#[derive(Serialize)]
struct FieldError {
    code: String,
    message: String,
}

fn to_api_error(err: ValidationError) -> ApiError {
    let mut errors = BTreeMap::new();

    for (path, violations) in err.field_violations_map() {
        let field_errors = violations.iter().map(|v| FieldError {
            code: v.code.to_string(),
            message: v.message.clone(),
        }).collect();

        errors.insert(path, field_errors);
    }

    ApiError { success: false, errors }
}
```

## Client-Side Error Handling

### TypeScript Interface

```typescript
interface ValidationErrorResponse {
  code: string;
  status: number;
  message: string;
  retryable: boolean;
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
```

### React Hook Form Integration

```typescript
import { useForm } from 'react-hook-form';

function BookingForm() {
  const { setError, handleSubmit } = useForm();

  const onSubmit = async (data: BookingFormData) => {
    try {
      await api.createBooking(data);
    } catch (error) {
      if (error.status === 400) {
        const response = error.data as ValidationErrorResponse;

        // Map errors to form fields
        for (const [path, errors] of Object.entries(response.details.fields)) {
          setError(path, {
            type: errors[0].code,
            message: errors[0].message,
          });
        }
      }
    }
  };

  return <form onSubmit={handleSubmit(onSubmit)}>...</form>;
}
```

### Handling Array Field Errors

```typescript
// Handle array indices in paths
function displayArrayErrors(path: string, errors: FieldError[]) {
  // path might be "rooms[0].adults" or "rooms[2].children"
  const match = path.match(/(\w+)\[(\d+)\]\.(\w+)/);

  if (match) {
    const [, arrayName, index, fieldName] = match;
    // Highlight specific item in array form
    highlightArrayItem(arrayName, parseInt(index), fieldName, errors);
  }
}
```

## Testing HTTP Endpoints

### Axum Testing

```rust
use axum::routing::post;
use axum_test::TestServer;

#[tokio::test]
async fn test_validation_error() {
    let app = Router::new().route("/users", post(create_user));
    let server = TestServer::new(app).unwrap();

    let response = server
        .post("/users")
        .json(&json!({
            "name": "",
            "email": "invalid",
            "age": 200
        }))
        .await;

    response.assert_status_bad_request();

    let body: serde_json::Value = response.json();
    assert_eq!(body["code"], "VALIDATION");
    assert!(body["details"]["fields"]["name"].is_array());
    assert!(body["details"]["fields"]["email"].is_array());
    assert!(body["details"]["fields"]["age"].is_array());
}

#[tokio::test]
async fn test_valid_request() {
    let app = Router::new().route("/users", post(create_user));
    let server = TestServer::new(app).unwrap();

    let response = server
        .post("/users")
        .json(&json!({
            "name": "Alice",
            "email": "alice@example.com",
            "age": 25
        }))
        .await;

    response.assert_status_ok();
}
```

### Actix-web Testing

```rust
use actix_web::{test, App};

#[actix_web::test]
async fn test_validation_error() {
    let app = test::init_service(App::new().service(create_user)).await;

    let req = test::TestRequest::post()
        .uri("/users")
        .set_json(&json!({
            "name": "",
            "email": "invalid",
            "age": 200
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);
}
```

## See Also

- **Framework READMEs:**
  - [domainstack-axum](../../domainstack-axum/README.md) - Full Axum documentation
  - [domainstack-actix](../../domainstack-actix/README.md) - Full Actix-web documentation
  - [domainstack-rocket](../../domainstack-rocket/README.md) - Full Rocket documentation

- **Related Guides:**
  - [Async Validation](ASYNC_VALIDATION.md) - Database and API checks
  - [Error Handling](ERROR_HANDLING.md) - Working with `ValidationError`
  - [Serde Integration](SERDE_INTEGRATION.md) - Validate on deserialize
  - [Patterns](PATTERNS.md) - DTO → Domain patterns

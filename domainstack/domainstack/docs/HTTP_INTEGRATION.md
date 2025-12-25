# HTTP Integration

Complete guide to integrating domainstack validation with web frameworks.

## Table of Contents

- [Using error-envelope](#using-error-envelope)
- [Error Response Format](#error-response-format)
- [Client-Side Error Handling](#client-side-error-handling)
- [Axum](#axum)
- [Actix-web](#actix-web)
- [Rocket](#rocket)
- [Framework Comparison](#framework-comparison)

## Using error-envelope

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

## Error Response Format

Structured RFC 9457-compliant error responses:

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

## Client-Side Error Handling

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

## Axum

**Crate:** `domainstack-axum`

### DomainJson Extractor

```rust
use domainstack_axum::{DomainJson, ErrorResponse};

// Type alias for cleaner signatures
type Result<T> = std::result::Result<T, ErrorResponse>;

async fn create_user(
    DomainJson(request, user): DomainJson<CreateUserRequest, User>
) -> Result<Json<User>> {
    // `user` is guaranteed valid - automatic validation!
    let created = db.insert_user(user).await?;
    Ok(Json(created))
}
```

**How it works:**
1. Deserializes JSON into `CreateUserRequest` (DTO)
2. Validates the DTO
3. Converts to `User` (domain type) via `TryFrom`
4. Returns `ErrorResponse` automatically on validation failure

### ValidatedJson Extractor

For simpler cases where DTO = Domain type:

```rust
use domainstack_axum::ValidatedJson;

async fn create_user(
    ValidatedJson(user): ValidatedJson<User>
) -> Result<Json<User>> {
    // `user` validated automatically
    Ok(Json(user))
}
```

## Actix-web

**Crate:** `domainstack-actix`

### DomainJson Extractor

```rust
use domainstack_actix::{DomainJson, ErrorResponse};
use actix_web::{post, web::Json};

type Result<T> = std::result::Result<T, ErrorResponse>;

#[post("/users")]
async fn create_user(
    DomainJson(request, user): DomainJson<CreateUserRequest, User>
) -> Result<Json<User>> {
    // Guaranteed valid user
    let created = db.insert_user(user).await?;
    Ok(Json(created))
}
```

**Note:** Actix extractor uses `block_on()` for synchronous validation in async context. This is the standard pattern for Actix-web 4.x extractors.

## Rocket

**Crate:** `domainstack-rocket`

### DomainJson Guard

```rust
use domainstack_rocket::{DomainJson, ErrorResponse};
use rocket::{post, serde::json::Json};

type Result<T> = std::result::Result<T, ErrorResponse>;

#[post("/users", data = "<request>")]
async fn create_user(
    DomainJson(request, user): DomainJson<CreateUserRequest, User>
) -> Result<Json<User>> {
    // Guaranteed valid user
    let created = db.insert_user(user).await?;
    Ok(Json(created))
}
```

### Error Catchers

Rocket requires registering error catchers for proper error handling:

```rust
use rocket::catch;
use domainstack_rocket::ErrorResponse;

#[catch(400)]
fn bad_request(req: &Request) -> ErrorResponse {
    ErrorResponse::from_request(req)
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![create_user])
        .register("/", catchers![bad_request])
}
```

## Framework Comparison

| Feature | Axum | Actix | Rocket |
|---------|------|-------|--------|
| DomainJson | ✅ | ✅ | ✅ |
| ValidatedJson | ✅ | ✅ | ✅ |
| Auto ErrorResponse | ✅ | ✅ | ✅ |
| Async Validation | ✅ | ⚠️ Requires `block_on()` | ✅ |
| Setup Complexity | Low | Low | Medium (catchers) |

## See Also

- [Serde Integration](SERDE_INTEGRATION.md) - Validate on deserialize
- [Error Handling](api-guide.md#error-handling) - Custom error messages
- Main guide: [API Guide](api-guide.md)

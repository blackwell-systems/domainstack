# domainstack-rocket

[![Blackwell Systems™](https://raw.githubusercontent.com/blackwell-systems/blackwell-docs-theme/main/badge-trademark.svg)](https://github.com/blackwell-systems)
[![Crates.io](https://img.shields.io/crates/v/domainstack-rocket.svg)](https://crates.io/crates/domainstack-rocket)
[![Documentation](https://docs.rs/domainstack-rocket/badge.svg)](https://docs.rs/domainstack-rocket)
[![License: Apache 2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://github.com/blackwell-systems/domainstack/blob/main/LICENSE)

**Rocket request guards for the [domainstack](https://crates.io/crates/domainstack) full-stack validation ecosystem**

One-line DTO→Domain extraction with automatic structured error responses. Define validation once, get type-safe handlers and UI-friendly errors.

## Hero Example

```rust
use domainstack::prelude::*;
use domainstack_derive::Validate;
use domainstack_rocket::{DomainJson, ErrorResponse};
use rocket::{launch, post, routes, serde::json::Json};
use serde::{Deserialize, Serialize};

// DTO: What the client sends
#[derive(Deserialize)]
struct CreateBookingDto {
    guest_email: String,
    rooms: u8,
    nights: u8,
    promo_code: Option<String>,
}

// Domain: Valid-by-construction with derive macro
#[derive(Debug, Serialize, Validate)]
#[validate(check = "self.rooms > 0 || self.nights > 0", message = "Booking must have rooms or nights")]
struct Booking {
    #[validate(email)]
    #[validate(max_len = 255)]
    guest_email: String,

    #[validate(range(min = 1, max = 10))]
    rooms: u8,

    #[validate(range(min = 1, max = 30))]
    nights: u8,

    #[validate(alphanumeric)]
    #[validate(length(min = 4, max = 20))]
    promo_code: Option<String>,
}

impl TryFrom<CreateBookingDto> for Booking {
    type Error = ValidationError;
    fn try_from(dto: CreateBookingDto) -> Result<Self, Self::Error> {
        let booking = Self {
            guest_email: dto.guest_email,
            rooms: dto.rooms,
            nights: dto.nights,
            promo_code: dto.promo_code,
        };
        booking.validate()?;
        Ok(booking)
    }
}

// Handler: ONE LINE - extraction, validation, conversion all handled
#[post("/bookings", data = "<booking>")]
fn create_booking(
    booking: DomainJson<Booking, CreateBookingDto>,
) -> Result<Json<Booking>, ErrorResponse> {
    // booking.domain is GUARANTEED valid - use with confidence!
    Ok(Json(booking.domain))
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![create_booking])
}
```

**Send invalid data:**
```bash
curl -X POST http://localhost:8000/bookings \
  -H "Content-Type: application/json" \
  -d '{"guest_email": "bad", "rooms": 0, "nights": 50}'
```

**Get structured, UI-friendly errors:**
```json
{
  "code": "VALIDATION",
  "status": 400,
  "message": "Validation failed with 3 errors",
  "details": {
    "fields": {
      "guest_email": [{"code": "invalid_email", "message": "Invalid email format"}],
      "rooms": [{"code": "out_of_range", "message": "Must be between 1 and 10"}],
      "nights": [{"code": "out_of_range", "message": "Must be between 1 and 30"}]
    }
  }
}
```

**Your frontend can map these directly to form fields. No parsing. No guessing.**

## Installation

```toml
[dependencies]
domainstack-rocket = "1.0"
domainstack = { version = "1.0", features = ["derive", "regex"] }
domainstack-derive = "1.0"
serde = { version = "1", features = ["derive"] }
rocket = { version = "0.5", features = ["json"] }
```

## Features

### `DomainJson<T, Dto>`

Request guard that:
1. Deserializes JSON body to DTO
2. Converts DTO to domain type via `TryFrom`
3. Returns structured validation errors on failure

```rust
#[post("/users", data = "<user>")]
fn create_user(user: DomainJson<User, CreateUserDto>) -> Result<Json<User>, ErrorResponse> {
    Ok(Json(user.domain))  // domain is guaranteed valid
}
```

### `ValidatedJson<Dto>`

Request guard for DTO validation without domain conversion:

```rust
use domainstack::Validate;

#[derive(Deserialize, Validate)]
struct UpdateUserDto {
    #[validate(length(min = 2, max = 50))]
    name: String,
}

#[post("/users/<id>", data = "<dto>")]
fn update_user(id: u64, dto: ValidatedJson<UpdateUserDto>) -> Json<UpdateUserDto> {
    Json(dto.0)  // dto is guaranteed valid
}
```

### Error Handling

Validation errors are automatically converted to structured JSON responses:

**Important:** You need to register an error catcher to properly handle validation errors:

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
        .register("/", catchers![validation_catcher])  // Register the catcher
        .launch()
        .await
        .unwrap();
}
```

**Error response format:**

```json
{
  "code": "VALIDATION",
  "status": 400,
  "message": "Validation failed with 2 errors",
  "details": {
    "fields": {
      "name": [{
        "code": "min_length",
        "message": "Must be at least 2 characters",
        "meta": { "min": "2" }
      }],
      "email": [{
        "code": "invalid_email",
        "message": "Invalid email format"
      }]
    }
  }
}
```

## Documentation

For more details, see the [main domainstack documentation](https://docs.rs/domainstack).

## License

Licensed under Apache-2.0.

# domainstack-rocket

Rocket request guards for [domainstack](https://crates.io/crates/domainstack) validation.

## Overview

`domainstack-rocket` provides Rocket request guards for automatic JSON validation and domain conversion with RFC 9457 compliant error responses.

## Installation

```toml
[dependencies]
domainstack-rocket = "0.4.0"
```

## Quick Start

```rust
use domainstack::prelude::*;
use domainstack_rocket::{DomainJson, ErrorResponse};
use rocket::{post, routes, serde::json::Json};
use serde::Deserialize;

#[derive(Deserialize)]
struct CreateUserDto {
    name: String,
    email: String,
    age: u8,
}

struct User {
    name: String,
    email: String,
    age: u8,
}

impl TryFrom<CreateUserDto> for User {
    type Error = domainstack::ValidationError;

    fn try_from(dto: CreateUserDto) -> Result<Self, Self::Error> {
        validate("name", &dto.name, &rules::min_len(2).and(rules::max_len(50)))?;
        validate("email", &dto.email, &rules::email())?;
        validate("age", &dto.age, &rules::range(18, 120))?;
        Ok(Self { name: dto.name, email: dto.email, age: dto.age })
    }
}

#[post("/users", data = "<user>")]
fn create_user(user: DomainJson<User, CreateUserDto>) -> Result<Json<String>, ErrorResponse> {
    // user.domain is guaranteed valid here!
    Ok(Json(format!("Created user: {}", user.domain.name)))
}

#[rocket::main]
async fn main() {
    rocket::build()
        .mount("/", routes![create_user])
        .launch()
        .await
        .unwrap();
}
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

Validation errors are automatically converted to RFC 9457 compliant JSON responses:

**Important:** You need to register an error catcher to properly handle validation errors:

```rust
use rocket::{catch, catchers, Request};
use domainstack_rocket::ErrorResponse;

#[catch(400)]
fn validation_catcher(req: &Request) -> ErrorResponse {
    req.local_cache(|| None::<ErrorResponse>)
        .clone()
        .unwrap_or_else(|| ErrorResponse(error_envelope::Error::bad_request("Bad Request")))
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

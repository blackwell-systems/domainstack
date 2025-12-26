# Serde Integration

Complete guide to using domainstack with serde for automatic validation during deserialization.

## Validate on Deserialize

**Feature flag:** `serde`

Automatically validate during JSON/YAML/etc. deserialization:

```rust
use domainstack::ValidateOnDeserialize;
use serde::Deserialize;

#[derive(Deserialize, ValidateOnDeserialize)]
struct User {
    #[validate(email)]
    #[validate(max_len = 255)]
    email: String,

    #[validate(range(min = 18, max = 120))]
    age: u8,

    #[validate(url)]
    website: Option<String>,
}

// Single step: deserialize + validate automatically
let user: User = serde_json::from_str(json)?;
// ↑ Returns ValidationError if invalid, not serde::Error
```

## Benefits

- Eliminates `dto.validate()` boilerplate
- Better error messages: "age must be between 18 and 120" vs "expected u8"
- Type safety: if you have `User`, it's guaranteed valid
- Works with all serde attributes: `#[serde(rename)]`, `#[serde(default)]`, etc.
- ~5% overhead vs two-step approach

## How it Works

Two-phase deserialization:
1. Deserialize into intermediate type (standard serde)
2. Validate all fields
3. Return validated type or ValidationError

```rust
// Before (two steps)
let dto: UserDto = serde_json::from_str(json)?;  // Step 1: deserialize
dto.validate()?;                                  // Step 2: validate

// After (one step)
let user: User = serde_json::from_str(json)?;    // Deserialize + validate!
```

## Example: API Request Handler

```rust
#[derive(Deserialize, ValidateOnDeserialize)]
struct CreateUserRequest {
    #[validate(email)]
    email: String,

    #[validate(length(min = 3, max = 50))]
    #[validate(alphanumeric)]
    username: String,

    #[validate(length(min = 8, max = 128))]
    password: String,
}

// In your handler - guaranteed valid on successful deserialization
async fn create_user(
    Json(request): Json<CreateUserRequest>
) -> Result<Json<User>, ErrorResponse> {
    // request.email is guaranteed to be a valid email!
    // No manual .validate() call needed
    let user = db.insert_user(request).await?;
    Ok(Json(user))
}
```

## Optional Field Handling

Validation runs only for `Some(_)` values:

```rust
#[derive(ValidateOnDeserialize)]
struct Profile {
    #[validate(url)]
    website: Option<String>,  // Validates URL if present, allows None

    #[validate(length(min = 10, max = 500))]
    bio: Option<String>,  // Validates length if present
}
```

## See Also

- Example: `domainstack/examples/serde_validation.rs`
- Tests: 12 comprehensive integration tests in serde_integration.rs
- Main guide: [Core Concepts](CORE_CONCEPTS.md)

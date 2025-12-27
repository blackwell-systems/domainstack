# Serde Integration

Complete guide to using domainstack with serde for automatic validation during deserialization.

## Two Validation Gates

domainstack offers two approaches for validating data:

| Approach | When Validation Runs | Derive Macro |
|----------|---------------------|--------------|
| **Integrated** | During deserialization | `#[derive(ValidateOnDeserialize)]` |
| **Separate** | Explicit `.validate()` call | `#[derive(Validate)]` |

Both are valid patterns with different tradeoffs. Choose based on your use case.

---

## Gate 1: Validate on Deserialize (Integrated)

**Feature flag:** `serde`

Validation runs automatically during deserialization:

```rust
use domainstack::ValidateOnDeserialize;
use serde::Deserialize;

#[derive(Deserialize, ValidateOnDeserialize)]
struct User {
    #[validate(email)]
    email: String,

    #[validate(range(min = 18, max = 120))]
    age: u8,
}

// Single step: deserialize + validate
let user: User = serde_json::from_str(json)?;
// If you have a User, it's guaranteed valid
```

### When to Use Integrated Validation

**Best for:**

- **API boundaries** - Incoming requests should be validated immediately
- **Type-safe guarantees** - "If I have a `User`, it's valid" invariant
- **Reducing boilerplate** - No forgotten `.validate()` calls
- **Fail-fast** - Reject bad data at the earliest possible point

**Use cases:**

```rust
// REST API handlers - validate on entry
async fn create_user(Json(user): Json<User>) -> Response {
    // user is guaranteed valid - no .validate() needed
    db.insert(user).await
}

// Config file loading - fail fast on startup
let config: AppConfig = serde_json::from_str(&file)?;
// Invalid config = immediate startup failure

// Message queue consumers - reject malformed messages
let event: OrderEvent = serde_json::from_slice(&payload)?;
// Invalid event never enters your system
```

---

## Gate 2: Separate Validation Step

Deserialize first, validate later:

```rust
use domainstack::Validate;
use serde::Deserialize;

#[derive(Deserialize, Validate)]
struct User {
    #[validate(email)]
    email: String,

    #[validate(range(min = 18, max = 120))]
    age: u8,
}

// Two steps: deserialize, then validate
let user: User = serde_json::from_str(json)?;  // Step 1
user.validate()?;                               // Step 2
```

### When to Use Separate Validation

**Best for:**

- **Partial/draft data** - Save incomplete forms, validate on submit
- **Multi-stage validation** - Different rules at different stages
- **Conditional validation** - Rules depend on runtime context
- **Migration/import** - Log invalid records instead of failing
- **Testing** - Create invalid instances for testing error paths

**Use cases:**

```rust
// Draft documents - save now, validate on publish
#[derive(Deserialize, Validate)]
struct BlogPost {
    #[validate(non_empty)]
    title: String,
    #[validate(length(min = 100))]
    content: String,
}

fn save_draft(post: BlogPost) -> Result<(), Error> {
    // Don't validate - drafts can be incomplete
    db.save_draft(post)
}

fn publish(post: BlogPost) -> Result<(), Error> {
    post.validate()?;  // NOW validate before publishing
    db.publish(post)
}

// Batch import with error collection
fn import_users(records: Vec<UserRecord>) -> ImportResult {
    let mut valid = vec![];
    let mut errors = vec![];

    for (i, record) in records.into_iter().enumerate() {
        match record.validate() {
            Ok(()) => valid.push(record),
            Err(e) => errors.push((i, e)),  // Log, don't fail
        }
    }
    ImportResult { imported: valid.len(), errors }
}

// Context-dependent validation
fn validate_order(order: Order, user: &User) -> Result<(), ValidationErrors> {
    order.validate()?;  // Basic validation

    // Additional context-aware checks
    if order.total > user.credit_limit {
        return Err(/* credit limit error */);
    }
    Ok(())
}
```

---

## Tradeoffs Comparison

| Aspect | Integrated (`ValidateOnDeserialize`) | Separate (`Validate`) |
|--------|--------------------------------------|----------------------|
| **Guarantees** | Type = valid data | Type = parsed data |
| **Boilerplate** | None | Must call `.validate()` |
| **Flexibility** | Fixed rules | Context-aware rules |
| **Partial data** | Not possible | Fully supported |
| **Performance** | <2% overhead ([benchmarked](./SERDE_BENCHMARK.md)) | Zero overhead until called |
| **Error type** | `serde::Error` wrapper | Native `ValidationErrors` |
| **Testing** | Can't create invalid instances | Full control |

### The Forgotten `.validate()` Problem

Separate validation has one major risk:

```rust
// BUG: Forgot to validate!
fn process_user(json: &str) -> Result<(), Error> {
    let user: User = serde_json::from_str(json)?;
    db.insert(user).await?;  // Invalid data in DB!
    Ok(())
}
```

Integrated validation eliminates this class of bugs entirely.

### The Inflexibility Problem

Integrated validation can be too strict:

```rust
// Can't save a draft with empty title
#[derive(ValidateOnDeserialize)]
struct Post {
    #[validate(non_empty)]  // Always enforced!
    title: String,
}

let draft: Post = serde_json::from_str(r#"{"title": ""}"#)?;
// Error! But we wanted to save a draft...
```

---

## Hybrid Approach

Use both patterns where appropriate:

```rust
// Strict type for API boundaries
#[derive(Deserialize, ValidateOnDeserialize)]
struct CreateUserRequest {
    #[validate(email)]
    email: String,
}

// Flexible type for internal processing
#[derive(Deserialize, Validate)]
struct UserDraft {
    #[validate(email)]
    email: String,
}

// API handler uses strict type
async fn create_user(Json(req): Json<CreateUserRequest>) -> Response {
    // Guaranteed valid
}

// Internal tool uses flexible type
fn import_from_csv(records: Vec<UserDraft>) -> ImportResult {
    // Validate selectively
}
```

---

## How ValidateOnDeserialize Works

Three-phase deserialization:

1. Deserialize into intermediate type (standard serde)
2. Construct the final struct
3. Validate and return, or return error

```rust
// Conceptually equivalent to:
impl<'de> Deserialize<'de> for User {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> {
        // Phase 1: Deserialize into hidden intermediate struct
        let intermediate = UserIntermediate::deserialize(deserializer)?;

        // Phase 2: Construct final struct
        let user = User {
            email: intermediate.email,
            age: intermediate.age,
        };

        // Phase 3: Validate
        user.validate()
            .map_err(|e| serde::de::Error::custom(
                format!("Validation failed: {}", e)
            ))?;

        Ok(user)
    }
}
```

The macro also generates a `Validate` impl, so you can still call `.validate()` manually if needed.

---

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

---

## See Also

- Example: `domainstack/examples/serde_validation.rs`
- Tests: 12 comprehensive integration tests in serde_integration.rs
- Main guide: [Core Concepts](CORE_CONCEPTS.md)

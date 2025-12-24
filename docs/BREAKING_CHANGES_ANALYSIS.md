# Breaking Changes Analysis: v1.0.0 Opportunities

**What if we allowed breaking changes?**

This document explores improvements that would be possible in a major version bump (v1.0.0) where breaking changes are acceptable.

---

## Current Constraints

In v0.4.0+, we maintain **100% backward compatibility**. This means:
- No API changes to existing functions
- No signature changes
- No behavior changes
- Only additions allowed

**Result:** Safe, but limits some optimizations.

---

## Proposed Breaking Changes for v1.0.0

### Category 1: API Simplification

#### 1.1 **Unify Error Handling** 🔥

**Current Problem:**
```rust
// Different error types across modules
pub struct ValidationError { ... }
pub struct Violation { ... }
pub struct Meta { ... }

// Meta is awkward - Vec<(Key, Value)>
pub type Meta = Vec<(&'static str, String)>;
```

**v1.0.0 Proposal:**
```rust
// Single, unified error type
pub struct ValidationError {
    pub violations: Vec<Violation>,
}

pub struct Violation {
    pub path: Path,
    pub code: &'static str,
    pub message: String,
    pub metadata: HashMap<&'static str, String>,  // Use HashMap instead
}
```

**Benefits:**
- More idiomatic Rust (HashMap for key-value pairs)
- Better performance for metadata lookups
- Clearer API

**Breaking Change:** Yes - `Meta` type removed, `Violation` struct changed

---

#### 1.2 **Simplify Path API** 🔥

**Current Problem:**
```rust
// Path has public Vec<PathSegment>
pub struct Path(pub Vec<PathSegment>);

// Exposes internal implementation
let mut path = Path(Vec::new());
path.0.push(PathSegment::Field("name"));  // Awkward!
```

**v1.0.0 Proposal:**
```rust
// Private implementation
pub struct Path {
    segments: Vec<PathSegment>,  // Private!
}

impl Path {
    pub fn segments(&self) -> &[PathSegment] {
        &self.segments
    }

    pub fn push_field(&mut self, name: &'static str) {
        self.segments.push(PathSegment::Field(name));
    }

    pub fn push_index(&mut self, idx: usize) {
        self.segments.push(PathSegment::Index(idx));
    }
}
```

**Benefits:**
- Encapsulation (can change implementation later)
- Better API ergonomics
- More idiomatic Rust

**Breaking Change:** Yes - `Path(Vec<...>)` no longer accessible

---

### Category 2: Type Safety Improvements

#### 2.1 **Const Generics for String Length** 🔥

**Current Problem:**
```rust
// Length checks happen at runtime
validate("email", email.as_str(), &rules::min_len(5))?;
```

**v1.0.0 Proposal:**
```rust
// Compile-time length guarantees for fixed strings
#[derive(Debug)]
pub struct BoundedString<const MIN: usize, const MAX: usize> {
    value: String,
}

impl<const MIN: usize, const MAX: usize> BoundedString<MIN, MAX> {
    pub fn new(value: String) -> Result<Self, ValidationError> {
        if value.len() < MIN || value.len() > MAX {
            return Err(ValidationError::single(
                Path::root(),
                "length_invalid",
                format!("Must be {}-{} characters", MIN, MAX)
            ));
        }
        Ok(Self { value })
    }

    pub fn as_str(&self) -> &str {
        &self.value
    }
}

// Usage
type Email = BoundedString<5, 255>;
type Username = BoundedString<3, 20>;

let email = Email::new("user@example.com".to_string())?;
```

**Benefits:**
- Type-level documentation (Email is 5-255 chars)
- Better compile-time guarantees
- Self-documenting APIs

**Breaking Change:** New pattern, not a replacement

---

#### 2.2 **Phantom Types for Validated State** 🔥🔥

**Current Problem:**
```rust
// No type-level guarantee that validation occurred
pub struct User {
    name: String,
    email: String,
}

// Did someone forget to validate?
let user = User { name: "Alice", email: "invalid" };
```

**v1.0.0 Proposal:**
```rust
use std::marker::PhantomData;

pub struct Validated;
pub struct Unvalidated;

pub struct User<State = Unvalidated> {
    name: String,
    email: String,
    _state: PhantomData<State>,
}

impl User<Unvalidated> {
    pub fn new(name: String, email: String) -> Self {
        Self {
            name,
            email,
            _state: PhantomData,
        }
    }

    pub fn validate(self) -> Result<User<Validated>, ValidationError> {
        // Perform validation...

        Ok(User {
            name: self.name,
            email: self.email,
            _state: PhantomData,
        })
    }
}

// Only validated users can be saved
async fn save_user(user: User<Validated>) -> Result<(), Error> {
    // Compile-time guarantee that user is validated!
}

// This won't compile:
let user = User::new("Alice", "invalid");
save_user(user).await?;  // ERROR: expected User<Validated>, found User<Unvalidated>

// Must validate first:
let user = User::new("Alice", "alice@example.com")
    .validate()?;
save_user(user).await?;  // ✓ Compiles!
```

**Benefits:**
- Compile-time validation guarantees
- Cannot accidentally use unvalidated data
- Self-documenting API

**Breaking Change:** Yes - all domain types need phantom parameter

---

### Category 3: Performance Optimizations

#### 3.1 **SmallVec for Violations** 🔥

**Current Problem:**
```rust
// Most errors have 1-3 violations, but we allocate Vec
pub struct ValidationError {
    pub violations: Vec<Violation>,
}
```

**v1.0.0 Proposal:**
```rust
use smallvec::{SmallVec, smallvec};

pub struct ValidationError {
    pub violations: SmallVec<[Violation; 4]>,  // Stack-allocated for ≤4 violations
}
```

**Benefits:**
- No heap allocation for common case (1-3 errors)
- Better cache locality
- ~30% performance improvement for validation

**Breaking Change:** Minimal - SmallVec has Vec-compatible API

---

### Category 4: Advanced Features

#### 4.1 **Async Validation** 🔥🔥🔥

**Current Limitation:**
```rust
// Cannot do database checks
pub fn validate_email_unique(email: &str) -> Result<(), ValidationError> {
    // Can't call async fn here!
}
```

**v1.0.0 Proposal:**
```rust
use async_trait::async_trait;

#[async_trait]
pub trait AsyncValidate {
    async fn validate_async(&self, ctx: &ValidationContext) -> Result<(), ValidationError>;
}

pub struct ValidationContext {
    db: Arc<dyn Database>,
    cache: Arc<dyn Cache>,
}

// Example usage
#[derive(AsyncValidate)]
struct User {
    #[validate(length(min = 5, max = 255))]  // Sync check
    #[validate_async(unique = "users.email")]  // Async check
    email: String,
}

// In handler
async fn create_user(user: User, ctx: ValidationContext) -> Result<(), Error> {
    user.validate_async(&ctx).await?;  // Checks DB for uniqueness
    // ...
}
```

**Benefits:**
- Database uniqueness checks
- External API validation
- Rate limiting integration
- Most-requested feature

**Breaking Change:** New trait, optional adoption

---

### Category 5: Developer Experience

#### 5.1 **Schema Generation** 🔥🔥

**Current Limitation:**
```rust
// No way to generate OpenAPI/JSON Schema from domain types
```

**v1.0.0 Proposal:**
```rust
use schemars::{JsonSchema, schema_for};

#[derive(Validate, JsonSchema)]
struct User {
    #[validate(length(min = 5, max = 255))]
    #[schemars(regex = "^[^@]+@[^@]+\\.[^@]+$")]
    email: String,

    #[validate(range(min = 18, max = 120))]
    #[schemars(range(min = 18, max = 120))]
    age: u8,
}

// Generate OpenAPI schema
let schema = schema_for!(User);
```

**Benefits:**
- Auto-generated API documentation
- Frontend validation rules
- Contract-first development

**Breaking Change:** New feature, opt-in

---

## Prioritization Matrix

| Change | Impact | Complexity | User Demand | Priority | Status |
|--------|--------|------------|-------------|----------|--------|
| Async validation | 🔥🔥🔥 | High | Very High | **P0** | 📋 Planned |
| Phantom types for validation state | 🔥🔥 | Low | Medium | **P2** | 📋 Planned |
| Schema generation | 🔥🔥 | High | Medium | **P2** | 📋 Planned |
| SmallVec optimization | 🔥 | Low | Low | **P3** | 📋 Planned |
| Const generics | 🔥 | Medium | Low | **P3** | 📋 Planned |
| HashMap for metadata | 🔥 | Low | Low | **P3** | 📋 Planned |

---

## Recommended v1.0.0 Feature Set

#### Must Have (P0)

1. **Async Validation**
   - `AsyncValidate` trait
   - Database uniqueness checks
   - External API validation
   - Validation context passing

#### Nice to Have (P2)

2. **Phantom Types**
   - Compile-time validation guarantees
   - Type-safe state tracking
   - Optional adoption

3. **Schema Generation**
   - OpenAPI integration
   - JSON Schema export
   - TypeScript type generation

---

## Migration Path

### Option 1: Big Bang (v1.0.0)

**Pros:**
- Clean slate
- Optimal API design
- All improvements at once

**Cons:**
- Breaking all existing users
- High migration cost
- Risky

### Option 2: Gradual (Recommended)

**Recommended Approach:**

#### v0.6.0 - Async Validation
- Add `AsyncValidate` trait
- Add validation context
- **Breaking:** None (new feature)

#### v0.7.0 - Additional domain helpers
- Performance optimizations
- Additional utilities
- **Breaking:** None (new features)

#### v0.9.0 - Deprecations
- Deprecate old APIs if needed
- Add migration warnings
- **Breaking:** None (warnings only)

#### v1.0.0 - Stabilize
- Finalize stable API
- Clean up any deprecated items
- **Breaking:** Only removes deprecated items (if any)

---

## Ecosystem Comparison

| Feature | domainstack (current) | validator | garde | nutype |
|---------|----------------------|-----------|-------|--------|
| Domain-First | ✅ | ❌ | ❌ | ⚠️ |
| Composable Rules | ✅ + Builders | ❌ | ⚠️ | ⚠️ |
| Async Validation | 📋 Planned | ❌ | ❌ | ❌ |
| Cross-Field | ✅ v0.5 | ⚠️ Limited | ❌ | ❌ |
| Conditional Validation | ✅ v0.5 | ❌ | ❌ | ❌ |
| Custom Messages | ✅ v0.4 | ⚠️ Attributes only | ⚠️ | ✅ |
| Type-Safe State | 📋 Planned | ❌ | ❌ | ✅ |
| Framework Adapters | ✅ v0.4 | ❌ | ❌ | ❌ |
| Schema Generation | 📋 Planned | ❌ | ❌ | ❌ |
| Zero Dependencies | ✅ Core only | ❌ | ❌ | ✅ |

**Note:** domainstack already implements most desired features without breaking changes.

---

## Recommendation

### Current Status
- Focus on adding features without breaking changes
- Maintain backward compatibility
- Gather feedback from users

### For v1.0.0
- Finalize stable API
- Minimal to no breaking changes expected
- Declare production-ready

**Philosophy:** Move fast, but carry users with you. Most desired features can be added without breaking changes.

---

## Conclusion

The architecture is flexible enough to add most desired features without breaking changes. Breaking changes should be reserved for true architectural improvements that provide significant value and cannot be done compatibly.

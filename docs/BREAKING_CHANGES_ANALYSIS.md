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

### Category 4: Developer Experience

#### 4.1 **Schema Generation** 🔥🔥

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
| Phantom types for validation state | 🔥🔥 | Low | Medium | **P1** | 📋 Planned |
| Schema generation | 🔥🔥 | High | Medium | **P1** | 📋 Planned |
| SmallVec optimization | 🔥 | Low | Low | **P2** | 📋 Planned |
| Const generics | 🔥 | Medium | Low | **P2** | 📋 Planned |
| HashMap for metadata | 🔥 | Low | Low | **P3** | 📋 Planned |

---

## Recommended v1.0.0 Feature Set

#### High Priority (P1)

1. **Phantom Types**
   - Compile-time validation guarantees
   - Type-safe state tracking
   - Optional adoption

2. **Schema Generation**
   - OpenAPI integration
   - JSON Schema export
   - TypeScript type generation

#### Medium Priority (P2)

3. **SmallVec Optimization**
   - Stack-allocated violations for common case
   - Performance improvements

4. **Const Generics**
   - Type-level string length constraints
   - Better compile-time guarantees

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

#### v0.5.0 - Completed ✅
- ✅ Path API encapsulation
- ✅ Async validation support
- ✅ Extended rule library (31 rules)
- ✅ Cross-field validation
- **Breaking:** Path API internal structure private

#### v0.6.0 - Type Safety
- Phantom types for validated state
- Optional adoption pattern
- **Breaking:** None (new pattern)

#### v0.7.0 - Developer Experience
- Schema generation
- OpenAPI integration
- **Breaking:** None (new features)

#### v0.9.0 - Optimizations & Deprecations
- Performance optimizations (SmallVec, etc.)
- Deprecate old APIs if needed
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
| Async Validation | ✅ v0.5 | ❌ | ❌ | ❌ |
| Cross-Field | ✅ v0.5 | ⚠️ Limited | ❌ | ❌ |
| Conditional Validation | ✅ v0.5 | ❌ | ❌ | ❌ |
| Custom Messages | ✅ v0.4 | ⚠️ Attributes only | ⚠️ | ✅ |
| Type-Safe State | 📋 Planned | ❌ | ❌ | ✅ |
| Framework Adapters | ✅ v0.4 | ❌ | ❌ | ❌ |
| Schema Generation | 📋 Planned | ❌ | ❌ | ❌ |
| Zero Dependencies | ✅ Core only | ❌ | ❌ | ✅ |

**Note:** domainstack has already implemented most P0 features (async validation, cross-field validation, Path API encapsulation) without breaking changes in v0.5.

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

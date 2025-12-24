# Breaking Changes Analysis: v1.0.0 Opportunities

**What if we allowed breaking changes?**

This document explores improvements that would be possible in a major version bump (v1.0.0) where breaking changes are acceptable.

---

## ✅ Implementation Progress (v0.4.0 - v0.5.0)

### Already Implemented WITHOUT Breaking Changes! 🎉

1. **✅ Builder Pattern for Rules (v0.4.0)**
   - Implemented via `.with_message()`, `.with_code()`, `.with_meta()`
   - Fully backward compatible - rules still work without customization
   - No breaking changes required!

2. **✅ Cross-Field Validation (v0.5.0)**
   - Implemented via `#[validate(check = "expr", code = "...", message = "...")]`
   - Conditional validation via `when = "predicate"`
   - Examples in `examples/v5_cross_field_validation.rs`
   - Added as new feature, zero breaking changes!

3. **✅ Better Error Context (v0.3.0+)**
   - `RuleContext` already exists and provides field paths
   - Context-aware error messages working
   - This was built into the system from the start!

4. **✅ Rule System Improvements (v0.4.0)**
   - **Fixed:** NaN handling with new `finite()` rule + `FiniteCheck` trait
   - **Fixed:** Added `non_zero()` rule for better numeric validation
   - **Fixed:** Regex caching with `once_cell` - EMAIL_REGEX and URL_REGEX are static
   - **Fixed:** Feature flags unified under `regex` (removed `email` alias)
   - All improvements done without breaking changes!

### Current Status

**v0.4.0 Achievements:**
- Builder-style customization ✅
- Float edge case handling ✅
- Regex performance optimization ✅
- Feature flag consistency ✅
- Zero breaking changes ✅

**v0.5.0 Achievements:**
- Cross-field validation ✅
- Conditional validation ✅
- Password confirmation patterns ✅
- Date range validation ✅

**What This Means:**
- We've implemented 3 of the P0/P1 features WITHOUT breaking changes
- The architecture is more flexible than initially thought
- Breaking changes may not be needed for most desired features!

---

## Current Constraints

In v0.4.0, we maintained **100% backward compatibility** with v0.3.0. This means:
- ✅ No API changes to existing functions
- ✅ No signature changes
- ✅ No behavior changes
- ✅ Only additions allowed

**Result:** Safe, but not optimal.

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

#### 1.3 **Builder Pattern for Rules** 🔥🔥

**Current Problem:**
```rust
// No way to customize error messages
let rule = rules::min_len(5);  // Error message is hardcoded

// Have to create custom rule for custom message
let rule = Rule::new(move |value: &str| {
    if value.len() < 5 {
        ValidationError::single(Path::root(), "min_length", "Email too short")
    } else {
        ValidationError::default()
    }
});
```

**v1.0.0 Proposal:**
```rust
// Builder pattern for all rules
let rule = rules::min_len(5)
    .message("Email too short")
    .code("email_too_short");

// Or use defaults
let rule = rules::min_len(5);  // Still works

// Fluent API
let rule = rules::email()
    .message("Invalid email format")
    .code("invalid_email")
    .meta("hint", "Use format: user@domain.com");
```

**Implementation:**
```rust
pub struct MinLenBuilder {
    min: usize,
    code: &'static str,
    message: Option<String>,
    metadata: HashMap<&'static str, String>,
}

impl MinLenBuilder {
    pub fn message(mut self, msg: impl Into<String>) -> Self {
        self.message = Some(msg.into());
        self
    }

    pub fn code(mut self, code: &'static str) -> Self {
        self.code = code;
        self
    }

    pub fn meta(mut self, key: &'static str, value: impl Into<String>) -> Self {
        self.metadata.insert(key, value.into());
        self
    }

    pub fn build(self) -> Rule<str> {
        let min = self.min;
        let code = self.code;
        let message = self.message.unwrap_or_else(|| format!("Must be at least {} characters", min));
        let mut metadata = self.metadata;
        metadata.insert("min", min.to_string());

        Rule::new(move |value: &str| {
            if value.len() < min {
                let mut err = ValidationError::single(Path::root(), code, &message);
                err.violations[0].metadata = metadata.clone();
                err
            } else {
                ValidationError::default()
            }
        })
    }
}

pub fn min_len(min: usize) -> MinLenBuilder {
    MinLenBuilder {
        min,
        code: "min_length",
        message: None,
        metadata: HashMap::new(),
    }
}

// Auto-conversion so existing code still works
impl From<MinLenBuilder> for Rule<str> {
    fn from(builder: MinLenBuilder) -> Self {
        builder.build()
    }
}
```

**Benefits:**
- Custom error messages (most requested feature)
- Custom error codes
- Additional metadata
- Backward compatible pattern (builder → Rule conversion)

**Breaking Change:** Technically yes, but migration is trivial

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

#### 3.1 **Remove Box::leak() from Path** 🔥🔥🔥

**Current Problem:**
```rust
// Path::parse() uses Box::leak() for 'static lifetime
pub fn parse(s: &str) -> Self {
    // ...
    segments.push(PathSegment::Field(Box::leak(current.into_boxed_str())));
}
```

**v1.0.0 Proposal:**
```rust
// Use Cow<'static, str> or Arc<str> instead
pub enum PathSegment {
    Field(Arc<str>),  // Reference-counted, no leaks!
    Index(usize),
}

pub fn parse(s: &str) -> Self {
    // ...
    segments.push(PathSegment::Field(Arc::from(current.as_str())));
}
```

**Alternative - Lifetime Parameter:**
```rust
pub struct Path<'a> {
    segments: Vec<PathSegment<'a>>,
}

pub enum PathSegment<'a> {
    Field(&'a str),  // Borrowed
    Index(usize),
}
```

**Benefits:**
- No memory leaks
- More Rust-idiomatic
- Better memory profile for long-running services

**Breaking Change:** Yes - Path API changes significantly

---

#### 3.2 **SmallVec for Violations** 🔥

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

#### 4.2 **Cross-Field Validation** 🔥🔥

**Current Limitation:**
```rust
// Cannot validate relationships between fields
struct DateRange {
    start: Date,
    end: Date,  // Must be >= start, but no way to express this!
}
```

**v1.0.0 Proposal:**
```rust
#[derive(Validate)]
#[validate(check = "end_after_start")]
struct DateRange {
    start: Date,
    end: Date,
}

impl DateRange {
    fn end_after_start(&self) -> Result<(), ValidationError> {
        if self.end < self.start {
            return Err(ValidationError::single(
                Path::root().field("end"),
                "invalid_range",
                "End date must be after start date"
            ));
        }
        Ok(())
    }
}
```

**Benefits:**
- Validate field relationships
- Business rule enforcement
- Password confirmation matching

**Breaking Change:** New derive macro attribute

---

#### 4.3 **Conditional Validation** 🔥

**Current Limitation:**
```rust
// Cannot conditionally apply rules based on other fields
struct PaymentMethod {
    method_type: String,  // "card" or "bank"
    card_number: Option<String>,  // Required if method_type == "card"
    routing_number: Option<String>,  // Required if method_type == "bank"
}
```

**v1.0.0 Proposal:**
```rust
#[derive(Validate)]
struct PaymentMethod {
    method_type: String,

    #[validate(when = "is_card", non_empty)]
    card_number: Option<String>,

    #[validate(when = "is_bank", non_empty)]
    routing_number: Option<String>,
}

impl PaymentMethod {
    fn is_card(&self) -> bool {
        self.method_type == "card"
    }

    fn is_bank(&self) -> bool {
        self.method_type == "bank"
    }
}
```

**Benefits:**
- Conditional validation logic
- Polymorphic domain models
- Cleaner business rules

**Breaking Change:** New derive macro features

---

### Category 5: Developer Experience

#### 5.1 **Better Error Messages** 🔥🔥🔥

**Current Problem:**
```rust
// Generic error messages
"Must be at least 5 characters"
"Invalid email format"
```

**v1.0.0 Proposal:**
```rust
// Context-aware error messages
pub struct RuleContext {
    field_name: &'static str,
    field_value: Option<String>,  // For debugging
    parent_path: Path,
}

// Rules receive context
pub fn min_len(min: usize) -> Rule<str> {
    Rule::new(move |value: &str, ctx: &RuleContext| {
        if value.len() < min {
            ValidationError::single(
                ctx.parent_path.field(ctx.field_name),
                "min_length",
                format!("Field '{}' must be at least {} characters (got {})",
                    ctx.field_name, min, value.len())
            )
        } else {
            ValidationError::default()
        }
    })
}

// Generates:
// "Field 'email' must be at least 5 characters (got 3)"
```

**Benefits:**
- More helpful error messages
- Better debugging experience
- Easier for end users

**Breaking Change:** Yes - Rule signature changes

---

#### 5.2 **Schema Generation** 🔥🔥

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
| ~~Builder pattern for rules~~ | 🔥🔥🔥 | Medium | Very High | **P0** | ✅ v0.4.0 |
| Async validation | 🔥🔥🔥 | High | Very High | **P0** | 📋 Planned |
| ~~Cross-field validation~~ | 🔥🔥 | Medium | High | **P1** | ✅ v0.5.0 |
| Remove Box::leak() | 🔥🔥 | High | Medium | **P1** | 📋 Planned |
| Phantom types for validation state | 🔥🔥 | Low | Medium | **P2** | 📋 Planned |
| ~~Conditional validation~~ | 🔥🔥 | Medium | Medium | **P2** | ✅ v0.5.0 |
| ~~Better error messages~~ | 🔥🔥🔥 | Medium | High | **P1** | ✅ v0.3.0+ |
| Schema generation | 🔥🔥 | High | Medium | **P2** | 📋 Planned |
| SmallVec optimization | 🔥 | Low | Low | **P3** | 📋 Planned |
| Const generics | 🔥 | Medium | Low | **P3** | 📋 Planned |
| HashMap for metadata | 🔥 | Low | Low | **P3** | 📋 Planned |

**Legend:**
- ✅ = Implemented (no breaking changes needed!)
- 📋 = Planned for future versions

---

## Recommended v1.0.0 Feature Set

### ✅ Already Completed (v0.4.0 - v0.5.0)

1. **~~Builder Pattern for Rules~~** (v0.4.0)
   - ✅ Custom error messages via `.with_message()`
   - ✅ Custom error codes via `.with_code()`
   - ✅ Additional metadata via `.with_meta()`
   - ✅ Backward compatible - no breaking changes!

2. **~~Cross-Field Validation~~** (v0.5.0)
   - ✅ `#[validate(check = "expr")]` attribute
   - ✅ Field relationship enforcement
   - ✅ Business rule validation

3. **~~Better Error Messages~~** (v0.3.0+)
   - ✅ Context-aware messages via `RuleContext`
   - ✅ Includes field names and paths
   - ✅ Built into system from start

4. **~~Conditional Validation~~** (v0.5.0)
   - ✅ `#[validate(when = "predicate")]`
   - ✅ Polymorphic validation
   - ✅ Business logic support

### Still Needed for v1.0.0

#### Must Have (P0)

1. **Async Validation**
   - `AsyncValidate` trait
   - Database uniqueness checks
   - External API validation
   - Validation context passing

#### Should Have (P1)

2. **Remove Box::leak()**
   - Use `Arc<str>` for path segments
   - Better memory profile
   - More idiomatic Rust

#### Nice to Have (P2)

3. **Phantom Types**
   - Compile-time validation guarantees
   - Type-safe state tracking
   - Optional adoption

4. **Schema Generation**
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

### Option 2: Gradual (v0.5 → v0.9 → v1.0)

**Recommended Approach:**

#### v0.5.0 - Async Validation
- Add `AsyncValidate` trait
- Add validation context
- **Breaking:** None (new feature)

#### v0.6.0 - Builder Pattern
- Add builders for all rules
- Keep existing functions working
- **Breaking:** None (parallel APIs)

#### v0.7.0 - Cross-Field Validation
- Add `#[validate(check = "fn")]` attribute
- **Breaking:** None (new feature)

#### v0.8.0 - Better Error Context
- Add `RuleContext` to rules
- **Breaking:** Optional (use new API or old)

#### v0.9.0 - Deprecations
- Deprecate old APIs
- Add migration warnings
- **Breaking:** None (warnings only)

#### v1.0.0 - Clean Up
- Remove deprecated APIs
- Finalize stable API
- **Breaking:** Only removes deprecated items

---

## Ecosystem Comparison with Breaking Changes

| Feature | domainstack v1.0 | validator | garde | nutype |
|---------|------------------|-----------|-------|--------|
| Domain-First | ✅ | ❌ | ❌ | ⚠️ |
| Composable Rules | ✅ + Builders | ❌ | ⚠️ | ⚠️ |
| Async Validation | ✅ | ❌ | ❌ | ❌ |
| Cross-Field | ✅ | ⚠️ Limited | ❌ | ❌ |
| Conditional Validation | ✅ | ❌ | ❌ | ❌ |
| Custom Messages | ✅ | ⚠️ Attributes only | ⚠️ | ✅ |
| Type-Safe State | ✅ Phantom types | ❌ | ❌ | ✅ |
| Framework Adapters | ✅ | ❌ | ❌ | ❌ |
| Schema Generation | ✅ | ❌ | ❌ | ❌ |
| Zero Dependencies | ✅ Core only | ❌ | ❌ | ✅ |

**Result:** domainstack v1.0 would be **undisputed leader** in domain validation.

---

## Recommendation

### For v0.4.0 (Now)
✅ Ship as-is with no breaking changes
✅ Focus on adoption and feedback

### For v0.5.0-v0.9.0 (Next 6-12 months)
✅ Add new features gradually (async, builders, cross-field)
✅ Keep backward compatibility
✅ Gather feedback from users

### For v1.0.0 (12+ months out)
✅ Clean up deprecated APIs
✅ Stabilize with breaking changes if needed
✅ Declare production-ready

**Philosophy:** Move fast, but carry users with you. Breaking changes are acceptable in v1.0, but only after proving value in v0.x releases.

---

## Conclusion

**Update (v0.5.0):** It turns out we didn't need breaking changes! We've successfully implemented:
- ✅ Builder pattern for rules (v0.4.0)
- ✅ Cross-field validation (v0.5.0)
- ✅ Conditional validation (v0.5.0)
- ✅ Better error context (built-in)
- ✅ Rule system improvements (v0.4.0)

**All without breaking backward compatibility!** 🎉

### Revised Strategy

1. **v0.4.0** - ✅ Done! Builder pattern, rule improvements, zero breaking changes
2. **v0.5.0** - ✅ Done! Cross-field validation, conditional validation
3. **v0.6.0** - Collection rules, choice/membership, string semantics
4. **v0.7.0** - Async validation (new trait, no breaking changes)
5. **v1.0.0** - Stabilize, maybe fix Box::leak() (only real breaking change needed)

**Key Insight:** Most desired features can be added without breaking changes. The architecture is more flexible than initially thought. Breaking changes should be reserved for true architectural improvements (like Box::leak removal) that can't be done compatibly.

**The features are exciting, AND we kept user trust.**

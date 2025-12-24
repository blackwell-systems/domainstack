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

### Category 1: Type Safety Improvements

#### 1.1 **Const Generics for String Length** 🔥

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

### Category 2: Developer Experience

#### 2.1 **Schema Generation** 🔥🔥

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
| Schema generation | 🔥🔥 | High | Medium | **P1** | 📋 Planned |
| Const generics | 🔥 | Medium | Low | **P2** | 📋 Planned |

---

## Recommended v1.0.0 Feature Set

#### High Priority (P1)

1. **Schema Generation**
   - OpenAPI integration
   - JSON Schema export
   - TypeScript type generation

#### Medium Priority (P2)

2. **Const Generics**
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

#### v0.7.0 - Developer Experience
- Schema generation
- OpenAPI integration
- **Breaking:** None (new features)

#### v0.9.0 - Optimizations & Deprecations
- Performance optimizations
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
| Type-Safe State | ✅ v0.6 | ❌ | ❌ | ✅ |
| Framework Adapters | ✅ v0.4 | ❌ | ❌ | ❌ |
| Schema Generation | 📋 Planned | ❌ | ❌ | ❌ |
| Zero Dependencies | ✅ Core only | ❌ | ❌ | ✅ |

**Note:** domainstack has already implemented most P0/P1 features (async validation, cross-field validation, Path API encapsulation in v0.5, phantom types in v0.6) without breaking changes.

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

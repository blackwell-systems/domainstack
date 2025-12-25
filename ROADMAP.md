# domainstack Roadmap

This roadmap outlines potential future features for domainstack, ranked by impact and alignment with the library's philosophy of "write once, derive everything."

## Status: v1.0.0 Released + domainstack-cli v0.1.0 ✅

The core library is production-ready with:
- ✅ Derive macro for validation
- ✅ 37+ built-in validation rules
- ✅ OpenAPI 3.0 schema generation
- ✅ Framework adapters (Axum, Actix, Rocket)
- ✅ Async validation with context
- ✅ Type-state validation
- ✅ Nested validation with path tracking
- ✅ Serde integration (validate on deserialize)
- ✅ **NEW:** Code generation CLI (`domainstack-cli`)
  - TypeScript/Zod schema generation
  - 26+ validation rules supported
  - Unified CLI architecture for future generators

---

## Future Features (Ranked by Priority)

### 🔥 Tier 1: High Impact, Core Extensions

#### 1. Serde Integration (Validate on Deserialize) ✅

**Status**: ✅ **Implemented in v1.0.0**
**Impact**: 🔥🔥🔥 Very High
**Effort**: Medium
**Feature Flag**: `serde`

Automatically validate during JSON/YAML/etc. deserialization:

```rust
#[derive(Deserialize, ValidateOnDeserialize)]
struct User {
    #[validate(email)]
    email: String,

    #[validate(range(min = 18, max = 120))]
    age: u8,
}

// Validation happens automatically during parsing
let user: User = serde_json::from_str(json)?;
// ↑ Returns ValidationError if invalid, not just serde::Error
```

**Benefits:**
- Eliminates `dto.validate()` boilerplate
- Guaranteed valid after deserialization
- Better error messages (field-level validation errors vs "unexpected value")
- Single step: parse + validate

**Technical Approach:**
- Custom serde deserializer that runs validation rules during parsing
- Integrates with existing `#[validate(...)]` attributes
- Returns `ValidationError` with field paths matching serde's error reporting

**See**: [Serde Integration Deep Dive](#serde-integration-deep-dive) below

---

#### 2. Code Generation CLI (TypeScript/Zod) ✅

**Status**: ✅ **Phase 1 Complete - v0.1.0**
**Impact**: 🔥🔥🔥 Very High
**Effort**: 6 days for MVP - **COMPLETED**
**Crate**: `domainstack-cli`

Generate TypeScript Zod schemas from Rust validation rules:

```bash
# Install the CLI
cargo install domainstack-cli

# Generate Zod schemas
domainstack zod --input src --output frontend/schemas.ts
```

**From Rust:**
```rust
#[derive(Validate)]
struct User {
    #[validate(email)]
    #[validate(max_len = 255)]
    email: String,

    #[validate(range(min = 18, max = 120))]
    age: u8,

    #[validate(url)]
    profile_url: Option<String>,
}
```

**Generates TypeScript/Zod:**
```typescript
export const UserSchema = z.object({
  email: z.string().email().max(255),
  age: z.number().min(18).max(120),
  profile_url: z.string().url().optional(),
});

export type User = z.infer<typeof UserSchema>;
```

**Implemented Features:**
- ✅ Unified CLI with subcommand architecture
- ✅ Zod schema generation with 26+ validation rules
- ✅ All string validations (email, url, length, patterns)
- ✅ All numeric validations (range, min/max, positive/negative)
- ✅ Optional fields with correct `.optional()` ordering
- ✅ Arrays and nested types
- ✅ Custom type references
- ✅ Precision warnings for large integers (u64, i128, etc.)
- ✅ Auto-generated headers with timestamps
- ✅ Comprehensive test coverage (32 unit tests)

**Benefits:**
- ✅ Single source of truth for validation
- ✅ Frontend/backend validation stays in sync automatically
- ✅ No duplicate validation logic
- ✅ Type-safe schemas with Zod's type inference
- ✅ Zero maintenance - regenerate when Rust types change

**Future Generators (Planned):**
- 📋 Yup schemas (`domainstack yup`)
- 📋 GraphQL SDL (`domainstack graphql`)
- 📋 Prisma schemas (`domainstack prisma`)
- 📋 JSON Schema (`domainstack json-schema`)

**See:** `domainstack/domainstack-cli/README.md` for full documentation

---

#### 3. Property-Based Test Data Generation

**Status**: Research
**Impact**: 🔥🔥 High
**Effort**: Medium
**RFC**: TBD

Auto-generate test data from validation rules:

```rust
#[derive(Validate, Arbitrary)]
struct User {
    #[validate(email)]
    email: String,

    #[validate(range(min = 18, max = 120))]
    age: u8,

    #[validate(length(min = 3, max = 20))]
    #[validate(alphanumeric)]
    username: String,
}

// Auto-generates:
// - Valid users (random emails, ages 18-120, alphanumeric usernames)
// - Invalid users for each validation rule
// - Edge cases (age=18, age=120, username length 3, etc.)
```

**Integration with:**
- `proptest` - Property-based testing
- `quickcheck` - Random test generation
- `arbitrary` - Arbitrary trait implementation

**Benefits:**
- Comprehensive test coverage without manual test data
- Finds edge cases automatically
- Tests validation rules are correct
- Mutation testing for validation logic

---

### 🚀 Tier 2: High Value, Ecosystem Integration

#### 4. Database Constraint Generation (SQL DDL)

**Status**: Planned
**Impact**: 🔥🔥 High
**Effort**: Medium

Generate SQL constraints from validation rules:

```rust
#[derive(Validate, ToMigration)]
struct User {
    #[validate(email)]
    #[validate(max_len = 255)]
    email: String,

    #[validate(range(min = 18, max = 120))]
    age: u8,
}
```

**Generates SQL:**
```sql
CREATE TABLE users (
    email VARCHAR(255) NOT NULL
        CHECK (email ~ '^[^@]+@[^@]+\.[^@]+$'),
    age INTEGER NOT NULL
        CHECK (age >= 18 AND age <= 120)
);
```

**Integration with:**
- `sqlx` - Compile-time verified queries
- `diesel` - ORM integration
- `sea-orm` - Async ORM integration

**Benefits:**
- Database enforces same rules as application
- Prevents invalid data at multiple layers
- Single source of truth for constraints

---

#### 5. JSON Schema Generation

**Status**: Planned
**Impact**: 🔥 Medium-High
**Effort**: Low

Generate JSON Schema (Draft 2020-12) from validation rules:

```rust
#[derive(Validate, ToJsonSchema)]
struct User {
    #[validate(email)]
    email: String,
}
```

**Benefits:**
- Works with JSON Schema validators in any language
- API gateway integration (Kong, AWS API Gateway)
- Schema registries (Confluent, Apicurio)
- Cross-language validation

**Similar to OpenAPI but:**
- Broader ecosystem support
- Used for validation, not just documentation
- Can validate events, configs, etc. (not just API requests)

---

### 📊 Tier 3: Developer Experience & Tooling

#### 6. Localization/i18n Support

**Status**: Research
**Impact**: 🔥 Medium
**Effort**: Medium

Multi-language validation error messages:

```rust
#[derive(Validate)]
struct User {
    #[validate(email)]
    #[message(en = "Invalid email format", es = "Formato de correo inválido", fr = "Format d'email invalide")]
    email: String,
}

// Runtime locale switching
let error = user.validate_with_locale("es")?;
```

**Alternative approach:**
```rust
// Auto-generate translation keys
// error.email.invalid_format -> load from i18n files
```

---

#### 7. Validation Metrics/Observability

**Status**: Planned
**Impact**: Medium
**Effort**: Low

Auto-generate metrics for validation failures:

```rust
#[derive(Validate)]
#[metrics(namespace = "user_service")]
struct User {
    #[validate(email)]
    email: String,
}
```

**Generates Prometheus metrics:**
```
validation_failures_total{field="email", code="invalid_email"} 42
validation_duration_seconds{type="User"} 0.001
```

**Benefits:**
- Identify data quality issues
- Track which fields fail most
- Performance monitoring
- Alert on validation failure spikes

---

#### 8. GraphQL Schema Generation

**Status**: Research
**Impact**: Medium
**Effort**: Low

Generate GraphQL schemas from validation rules:

```rust
#[derive(Validate, ToGraphQL)]
struct User {
    #[validate(email)]
    email: String,
}
```

**Generates:**
```graphql
type User {
  email: String! @constraint(format: "email")
}
```

**Integration with:**
- `async-graphql` - GraphQL server
- `juniper` - GraphQL library

---

#### 9. Validation Coverage Tracking

**Status**: Research
**Impact**: Medium
**Effort**: Medium

Track which validation rules are tested:

```bash
cargo test --features validation-coverage

Coverage Report:
✓ User.email (email format): 15 tests
✓ User.age (range): 12 tests
⚠ User.nickname (length): 0 tests  ← Not tested!
```

**Benefits:**
- Ensure all validation rules have tests
- Identify untested edge cases
- CI/CD integration

---

#### 10. Contract Testing Generator

**Status**: Research
**Impact**: Medium
**Effort**: High

Generate contract tests from validation rules:

```rust
#[derive(Validate, ContractTest)]
struct CreateUserRequest { ... }
```

**Generates:**
- Pact contract tests
- Invalid input test cases
- Boundary condition tests
- Property-based tests

---

## 🧪 Experimental / Future Research

### WebAssembly Validation

Compile validation rules to WASM for browser-side validation.

### Machine Learning Rule Inference

Suggest validation rules based on sample data analysis.

### Visual Rule Builder

GUI tool for building complex validation rules visually.

### Real-time Validation Streaming

Stream validation results for large datasets.

---

## Serde Integration Deep Dive

### Problem Statement

Current workflow requires two steps:

```rust
// Step 1: Deserialize (can fail)
let dto: UserDto = serde_json::from_str(json)?;

// Step 2: Validate (can fail)
dto.validate()?;
```

**Issues:**
- Boilerplate (always need both steps)
- Two different error types to handle
- Invalid data gets deserialized before being rejected
- Confusing error messages for users ("expected integer" vs "age must be between 18 and 120")

### Solution: ValidateOnDeserialize

```rust
#[derive(Deserialize, ValidateOnDeserialize)]
struct User {
    #[validate(email)]
    email: String,

    #[validate(range(min = 18, max = 120))]
    age: u8,
}

// Single step: deserialize + validate
let user: User = serde_json::from_str(json)?;
// ↑ Already validated! Returns ValidationError if invalid
```

### How It Works

#### Technical Implementation

1. **Derive Macro Analysis:**
   - Scans `#[validate(...)]` attributes
   - Generates a custom `Deserialize` implementation
   - Wraps serde's deserializer with validation layer

2. **Two-Phase Deserialization:**
   ```rust
   // Phase 1: Deserialize into intermediate type
   let intermediate: UserIntermediate = Deserialize::deserialize(deserializer)?;

   // Phase 2: Validate and convert
   let user = User::try_from_intermediate(intermediate)?;
   ```

3. **Error Mapping:**
   - serde errors → ValidationError with field paths
   - Validation errors → already have field paths
   - Consistent error format for API consumers

#### Generated Code Example

```rust
// What the macro generates
impl<'de> Deserialize<'de> for User {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // Intermediate struct without validation
        #[derive(Deserialize)]
        struct UserIntermediate {
            email: String,
            age: u8,
        }

        // Deserialize first
        let intermediate = UserIntermediate::deserialize(deserializer)?;

        // Then validate
        let user = User {
            email: intermediate.email,
            age: intermediate.age,
        };

        // Run validation rules
        user.validate()
            .map_err(|e| serde::de::Error::custom(e.to_string()))?;

        Ok(user)
    }
}
```

### Benefits in Detail

#### 1. Single Step API
```rust
// Before
let dto: UserDto = serde_json::from_str(json)?;
let domain = dto.try_into()?;

// After
let domain: User = serde_json::from_str(json)?;
```

#### 2. Better Error Messages
```rust
// Before (serde error)
"invalid type: string "abc", expected u8 at line 1 column 10"

// After (validation error)
"age: Must be between 18 and 120"
```

#### 3. Fail Fast
```rust
// Before: Deserializes whole object, then validates
// Memory allocated for invalid data

// After: Validates fields as they're deserialized
// Can fail early, less memory waste
```

#### 4. Type Safety
```rust
// If you have User, it's GUARANTEED valid
fn process_user(user: User) {
    // No need to check user.age is valid
    // Type system guarantees it
}
```

### Edge Cases & Design Decisions

#### Optional Fields
```rust
#[derive(ValidateOnDeserialize)]
struct User {
    #[validate(email)]
    email: String,

    // How to validate Option<T>?
    #[validate(min_len = 1)]
    nickname: Option<String>,  // Validate if present
}
```

**Decision:** Validation runs only if value is `Some(_)`

#### Default Values
```rust
#[derive(ValidateOnDeserialize)]
struct User {
    #[serde(default = "default_age")]
    #[validate(range(min = 18, max = 120))]
    age: u8,
}
```

**Decision:** Validate default values (prevent invalid defaults)

#### Custom Deserializers
```rust
#[derive(ValidateOnDeserialize)]
struct User {
    #[serde(deserialize_with = "custom_deser")]
    #[validate(email)]
    email: String,
}
```

**Decision:** Run validation after custom deserializer

### Performance Considerations

**Overhead:**
- Minimal: Validation happens in-place during deserialization
- No extra allocations
- Same memory usage as two-step approach

**Benchmark (estimated):**
```
Two-step (deserialize + validate):  1000 ns
ValidateOnDeserialize:              1050 ns  (~5% overhead)
```

The 5% overhead is from the intermediate struct, but you save the manual validation call.

### Migration Path

**Opt-in, not breaking:**
```rust
// Existing code still works
#[derive(Deserialize, Validate)]
struct User { ... }

// New opt-in feature
#[derive(Deserialize, ValidateOnDeserialize)]
struct User { ... }
```

**Feature flag:**
```toml
domainstack = { version = "1.0", features = ["serde-integration"] }
```

### Related Work

**Similar to:**
- Pydantic (Python) - Validates during parsing
- Go validator tags - Validates during JSON unmarshal
- JSON Schema validators - Validates during parsing

**Different from:**
- Most Rust validators - Separate validate step
- serde itself - No validation, only deserialization

---

## Contributing

Interested in helping with any of these features?

1. Check if an RFC exists for the feature
2. If not, open an issue to discuss the design
3. We'll create an RFC for major features
4. Implementation PRs welcome after RFC approval

**Maintainer:** Dayna Blackwell (blackwellsystems@protonmail.com)

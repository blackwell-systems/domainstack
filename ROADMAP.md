# domainstack Roadmap

This roadmap outlines future features for domainstack, ranked by impact and alignment with the library's philosophy of "write once, derive everything."

## Status: v1.0.0 Released

The core library is production-ready with:
- Derive macro for validation (`Validate`, `ToSchema`, `ValidateOnDeserialize`)
- 37+ built-in validation rules
- OpenAPI 3.0 schema generation
- Framework adapters (Axum, Actix, Rocket)
- Async validation with context
- Type-state validation
- Nested validation with path tracking
- Serde integration (validate on deserialize)
- Code generation CLI (`domainstack-cli`) with Zod support
- WASM browser validation (`domainstack-wasm`)

---

## Future Features (Ranked by Priority)

### 🔥 Tier 1: High Impact, Core Extensions

#### 1. Enum and Tuple Struct Support for Derive Macros

**Status**: Planned
**Impact**: 🔥🔥🔥 Very High
**Effort**: Medium

Currently `#[derive(Validate)]` only supports structs with named fields. Adding support for:

**Enum Validation:**
```rust
#[derive(Validate)]
enum PaymentMethod {
    Card {
        #[validate(length(min = 16, max = 19))]
        number: String,
        #[validate(matches_regex = r"^\d{3,4}$")]
        cvv: String,
    },
    BankTransfer {
        #[validate(alphanumeric)]
        account_number: String,
    },
    Cash,
}
```

**Tuple Struct / Newtype Validation:**
```rust
#[derive(Validate)]
struct Email(#[validate(email)] String);

#[derive(Validate)]
struct Age(#[validate(range(min = 0, max = 150))] u8);

// Enables the newtype pattern with derive
let email = Email("user@example.com".to_string());
email.validate()?;
```

**Benefits:**
- Enables type-safe newtype patterns with derive
- Supports sum types (enums) in domain modeling
- More idiomatic Rust validation

---

#### 2. CLI Additional Generators

**Status**: Planned (Architecture ready)
**Impact**: 🔥🔥🔥 Very High
**Effort**: Medium per generator

The CLI architecture supports multiple generators. Planned additions:

```bash
# Yup schemas (popular React form library)
domainstack yup --input src --output frontend/schemas.ts

# GraphQL SDL with validation directives
domainstack graphql --input src --output schema.graphql

# Prisma schema generation
domainstack prisma --input src --output prisma/schema.prisma

# JSON Schema (Draft 2020-12)
domainstack json-schema --input src --output schemas/
```

**Benefits:**
- Single source of truth across all platforms
- Support for major frontend validation libraries
- API gateway integration (JSON Schema)
- Database schema generation (Prisma)

---

#### 3. CLI Watch Mode

**Status**: Planned (flag exists, not implemented)
**Impact**: 🔥🔥 High
**Effort**: Low

Implement the `--watch` flag for automatic regeneration:

```bash
# Regenerate schemas when Rust files change
domainstack zod --input src --output frontend/schemas.ts --watch

# Watch mode with specific file patterns
domainstack zod --input src --output schemas.ts --watch --pattern "**/*.rs"
```

**Benefits:**
- Faster development workflow
- Automatic sync during development
- No manual regeneration needed

---

### 🚀 Tier 2: Documentation & Examples

#### 4. WASM Integration Example Project

**Status**: Planned
**Impact**: 🔥🔥 High
**Effort**: Low-Medium

Create a complete example showing WASM browser validation:

```
examples/
└── wasm-react-demo/
    ├── rust/
    │   └── src/lib.rs        # Types with validation
    ├── frontend/
    │   ├── src/App.tsx       # React form with WASM validation
    │   └── package.json
    └── README.md             # Step-by-step setup guide
```

**Demonstrates:**
- Building WASM module with `wasm-pack`
- Registering types for validation
- Calling validation from JavaScript/TypeScript
- Displaying field-level errors in React
- Same error structure as server responses

---

#### 5. CLI Step-by-Step Tutorial

**Status**: Planned
**Impact**: 🔥🔥 High
**Effort**: Low

Create comprehensive CLI documentation:

```markdown
# CLI Tutorial: From Rust to TypeScript

## Step 1: Install the CLI
cargo install domainstack-cli

## Step 2: Add validation rules to your Rust types
...

## Step 3: Generate TypeScript schemas
...

## Step 4: Use in your frontend
...
```

**Contents:**
- Installation and setup
- Basic usage examples
- Handling custom types
- CI/CD integration
- Troubleshooting common issues

---

### 📊 Tier 3: Framework Improvements

#### 6. Actix Adapter Async Improvement

**Status**: Research
**Impact**: 🔥 Medium
**Effort**: Medium

The Actix adapter currently uses `futures::executor::block_on()` due to Actix-web's sync extractor pattern. Research options:

**Current (works, documented limitation):**
```rust
fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
    ready(match futures::executor::block_on(json_fut) { ... })
}
```

**Potential improvements:**
1. Document the limitation more prominently in README
2. Provide alternative truly-async pattern in documentation
3. Track Actix-web updates for potential native async extractors

**Workaround documentation:**
```rust
// For truly async extraction, use this pattern:
async fn create_user(
    Json(dto): Json<CreateUserDto>
) -> Result<Json<User>, ErrorResponse> {
    let user = domainstack_http::into_domain::<User, _>(dto)?;
    Ok(Json(user))
}
```

---

#### 7. Property-Based Test Data Generation

**Status**: Research
**Impact**: 🔥🔥 High
**Effort**: Medium

Auto-generate test data from validation rules:

```rust
#[derive(Validate, Arbitrary)]
struct User {
    #[validate(email)]
    email: String,

    #[validate(range(min = 18, max = 120))]
    age: u8,
}

// Auto-generates:
// - Valid users (random emails, ages 18-120)
// - Invalid users for each validation rule
// - Edge cases (age=18, age=120, etc.)
```

**Integration with:**
- `proptest` - Property-based testing
- `quickcheck` - Random test generation
- `arbitrary` - Arbitrary trait implementation

---

#### 8. Database Constraint Generation (SQL DDL)

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

---

### 🧪 Tier 4: Advanced Features

#### 9. Localization/i18n Support

**Status**: Research
**Impact**: 🔥 Medium
**Effort**: Medium

Multi-language validation error messages:

```rust
#[derive(Validate)]
struct User {
    #[validate(email)]
    #[message(en = "Invalid email format", es = "Formato de correo inválido")]
    email: String,
}

// Runtime locale switching
let error = user.validate_with_locale("es")?;
```

---

#### 10. Validation Metrics/Observability

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

---

#### 11. Validation Coverage Tracking

**Status**: Research
**Impact**: Medium
**Effort**: Medium

Track which validation rules are tested:

```bash
cargo test --features validation-coverage

Coverage Report:
[ok] User.email (email format): 15 tests
[ok] User.age (range): 12 tests
⚠ User.nickname (length): 0 tests  ← Not tested!
```

---

## 🧪 Experimental / Future Research

- **Machine Learning Rule Inference** - Suggest validation rules based on sample data
- **Visual Rule Builder** - GUI tool for building complex validation rules
- **Real-time Validation Streaming** - Stream validation results for large datasets
- **Contract Testing Generator** - Generate Pact/contract tests from validation rules

---

## Contributing

Interested in helping with any of these features?

1. Check if an RFC exists for the feature
2. If not, open an issue to discuss the design
3. We'll create an RFC for major features
4. Implementation PRs welcome after RFC approval

**Maintainer:** Dayna Blackwell (blackwellsystems@protonmail.com)

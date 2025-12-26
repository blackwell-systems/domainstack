# OpenAPI Schema Generation

**Auto-generate OpenAPI 3.0 schemas from validation rules**

Generate OpenAPI schemas automatically from your validation rules, eliminating duplication between validation logic and API documentation.

## Table of Contents

- [Quick Start](#quick-start)
- [Why Auto-Derivation](#why-auto-derivation)
- [Rule Mapping Reference](#rule-mapping-reference)
- [Nested Types](#nested-types)
- [Collections and Arrays](#collections-and-arrays)
- [Optional Fields](#optional-fields)
- [Custom Validators](#custom-validators)
- [Schema Hints](#schema-hints)
- [Advanced Usage](#advanced-usage)
- [Migration Guide](#migration-guide)

## Quick Start

```rust
use domainstack::prelude::*;
use domainstack_derive::{Validate, ToSchema};
use domainstack_schema::OpenApiBuilder;

// Write validation rules ONCE, get BOTH runtime validation AND OpenAPI schemas!
#[derive(Validate, ToSchema)]
struct User {
    #[validate(email)]
    #[validate(max_len = 255)]
    #[schema(description = "User's email address", example = "alice@example.com")]
    email: String,

    #[validate(range(min = 18, max = 120))]
    #[schema(description = "User's age")]
    age: u8,
}

// Runtime validation works
let user = User { email, age };
user.validate()?;  // [ok] Validates email format, length, age range

// Schema generation works
let schema = User::schema();
// [ok] Automatically includes:
//   - email: format="email", maxLength=255
//   - age: minimum=18, maximum=120
//   - required=["email", "age"]

// OpenAPI spec generation works
let spec = OpenApiBuilder::new("My API", "1.0.0")
    .register::<User>()
    .build();
```

## Why Auto-Derivation

**The Problem:** Without auto-derivation, you write validation constraints twice—once for runtime validation, once for OpenAPI schemas. This creates duplication, drift, and maintenance burden.

**The Solution:** With `#[derive(Validate, ToSchema)]`, you write validation rules **once** and get **both** runtime validation AND OpenAPI schemas:

```rust
#[derive(Validate, ToSchema)]
struct CreateUser {
    #[validate(email)]
    #[validate(max_len = 255)]
    #[schema(description = "User's email address", example = "alice@example.com")]
    email: String,

    #[validate(range(min = 18, max = 120))]
    #[schema(description = "User's age in years")]
    age: u8,

    #[validate(min_len = 2)]
    #[validate(max_len = 50)]
    #[schema(description = "User's full name")]
    name: String,
}

// [ok] Runtime validation works automatically
let user = CreateUser::new(email, age, name)?;  // Validates all rules

// [ok] Schema generation works automatically
let schema = CreateUser::schema();  // Includes all constraints
```

**Generated OpenAPI schema:**
```json
{
  "CreateUser": {
    "type": "object",
    "required": ["email", "age", "name"],
    "properties": {
      "email": {
        "type": "string",
        "format": "email",
        "maxLength": 255,
        "description": "User's email address",
        "example": "alice@example.com"
      },
      "age": {
        "type": "integer",
        "minimum": 18,
        "maximum": 120,
        "description": "User's age in years"
      },
      "name": {
        "type": "string",
        "minLength": 2,
        "maxLength": 50,
        "description": "User's full name"
      }
    }
  }
}
```

**Benefits:**
- Write validation rules **once**
- Schema **always matches** validation
- Less boilerplate
- Single source of truth
- Impossible for docs to drift from validation

## Rule Mapping Reference

The derive macro automatically maps validation rules to OpenAPI constraints:

### String Rules

| Validation Rule | OpenAPI Constraint | Example |
|----------------|-------------------|---------|
| `email()` | `format: "email"` | `#[validate(email)]` → `"format": "email"` |
| `url()` | `format: "uri"` | `#[validate(url)]` → `"format": "uri"` |
| `min_len(n)` | `minLength: n` | `#[validate(min_len = 3)]` → `"minLength": 3` |
| `max_len(n)` | `maxLength: n` | `#[validate(max_len = 255)]` → `"maxLength": 255` |
| `length(min, max)` | `minLength, maxLength` | `#[validate(length(min = 3, max = 20))]` → both |
| `len_chars(min, max)` | `minLength, maxLength` | `#[validate(len_chars(3, 20))]` → both |
| `matches_regex(p)` | `pattern: p` | `#[validate(matches_regex = "^[A-Z].*")]` → `"pattern": "^[A-Z].*"` |
| `ascii()` | `pattern: "^[\\x00-\\x7F]*$"` | Auto-generated pattern |
| `alphanumeric()` | `pattern: "^[a-zA-Z0-9]*$"` | Auto-generated pattern |
| `alpha_only()` | `pattern: "^[a-zA-Z]*$"` | Auto-generated pattern |
| `numeric_string()` | `pattern: "^[0-9]*$"` | Auto-generated pattern |

### Numeric Rules

| Validation Rule | OpenAPI Constraint | Example |
|----------------|-------------------|---------|
| `min(n)` | `minimum: n` | `#[validate(min = 0)]` → `"minimum": 0` |
| `max(n)` | `maximum: n` | `#[validate(max = 100)]` → `"maximum": 100` |
| `range(min, max)` | `minimum, maximum` | `#[validate(range(min = 18, max = 120))]` → both |
| `positive()` | `minimum: 0, exclusiveMinimum: true` | Greater than zero |
| `negative()` | `maximum: 0, exclusiveMaximum: true` | Less than zero |
| `non_zero()` | `not: {enum: [0]}` | Not equal to zero |
| `finite()` | *(no mapping)* | Use `#[schema(...)]` hint |
| `multiple_of(n)` | `multipleOf: n` | `#[validate(multiple_of = 5)]` → `"multipleOf": 5` |

### Choice Rules

| Validation Rule | OpenAPI Constraint | Example |
|----------------|-------------------|---------|
| `one_of([...])` | `enum: [...]` | `#[validate(one_of = ["US", "CA", "UK"])]` → enum |
| `equals(v)` | `const: v` | `#[validate(equals = "active")]` → `"const": "active"` |
| `not_equals(v)` | `not: {const: v}` | Negation constraint |

### Collection Rules

| Validation Rule | OpenAPI Constraint | Example |
|----------------|-------------------|---------|
| `min_items(n)` | `minItems: n` | `#[validate(min_items = 1)]` → `"minItems": 1` |
| `max_items(n)` | `maxItems: n` | `#[validate(max_items = 10)]` → `"maxItems": 10` |
| `unique()` | `uniqueItems: true` | All array items must be unique |

### Composite Rules

| Validation Rule | OpenAPI Constraint | Notes |
|----------------|-------------------|-------|
| `.and()` | Both constraints applied | `min_len(3).and(max_len(20))` → both minLength and maxLength |
| `.or()` | `anyOf: [...]` | Alternative constraints |
| `.when()` | *(no direct mapping)* | Use schema composition or hints |

## Nested Types

Nested validation automatically includes referenced schemas:

```rust
#[derive(Validate, ToSchema)]
struct Email {
    #[validate(email)]
    #[validate(max_len = 255)]
    value: String,
}

#[derive(Validate, ToSchema)]
struct Guest {
    #[validate(min_len = 2)]
    #[validate(max_len = 50)]
    name: String,

    #[validate(nested)]  // ← Automatically references Email schema
    email: Email,
}
```

**Generated schema:**
```json
{
  "Guest": {
    "type": "object",
    "required": ["name", "email"],
    "properties": {
      "name": {
        "type": "string",
        "minLength": 2,
        "maxLength": 50
      },
      "email": {
        "$ref": "#/components/schemas/Email"
      }
    }
  },
  "Email": {
    "type": "object",
    "required": ["value"],
    "properties": {
      "value": {
        "type": "string",
        "format": "email",
        "maxLength": 255
      }
    }
  }
}
```

## Collections and Arrays

### Nested Collections with `each(nested)`

Array validation for nested types using `#[validate(each(nested))]`:

```rust
#[derive(Validate, ToSchema)]
struct Team {
    #[validate(min_len = 1, max_len = 50)]
    team_name: String,

    #[validate(each(nested))]
    #[validate(min_items = 1)]
    #[validate(max_items = 10)]
    members: Vec<User>,
}
```

**Generated schema:**
```json
{
  "Team": {
    "type": "object",
    "required": ["team_name", "members"],
    "properties": {
      "team_name": {
        "type": "string",
        "minLength": 1,
        "maxLength": 50
      },
      "members": {
        "type": "array",
        "items": {
          "$ref": "#/components/schemas/User"
        },
        "minItems": 1,
        "maxItems": 10
      }
    }
  }
}
```

### Primitive Collections with `each(rule)`

Any validation rule can be used with `each()` to validate items in collections:

```rust
#[derive(Validate, ToSchema)]
struct BlogPost {
    // Validate each email in the list
    #[validate(each(email))]
    #[validate(min_items = 1, max_items = 5)]
    author_emails: Vec<String>,

    // Validate each tag's length
    #[validate(each(length(min = 1, max = 50)))]
    tags: Vec<String>,

    // Validate each rating is in range
    #[validate(each(range(min = 1, max = 5)))]
    ratings: Vec<u8>,
}
```

**Generated schema** applies constraints to array items:
- `author_emails`: `items: { type: "string", format: "email" }, minItems: 1, maxItems: 5`
- `tags`: `items: { type: "string", minLength: 1, maxLength: 50 }`
- `ratings`: `items: { type: "integer", minimum: 1, maximum: 5 }`

**Error paths** include array indices: `author_emails[0]`, `tags[2]`, `ratings[1]`

## Optional Fields

Optional fields (using `Option<T>`) are not included in the `required` array:

```rust
#[derive(Validate, ToSchema)]
struct UpdateUser {
    #[validate(email)]
    #[validate(max_len = 255)]
    email: Option<String>,  // ← Optional, not in "required"

    #[validate(range(min = 18, max = 120))]
    age: Option<u8>,
}
```

**Generated schema:**
```json
{
  "UpdateUser": {
    "type": "object",
    "properties": {
      "email": {
        "type": "string",
        "format": "email",
        "maxLength": 255
      },
      "age": {
        "type": "integer",
        "minimum": 18,
        "maximum": 120
      }
    }
  }
}
```

Note: `email` and `age` are **not** in the `required` array.

## Custom Validators

For custom validation functions, use `#[schema(...)]` hints:

```rust
fn validate_strong_password(value: &str) -> Result<(), ValidationError> {
    // Complex password validation logic
    // ...
}

#[derive(Validate, ToSchema)]
struct UserAccount {
    #[validate(custom = "validate_strong_password")]
    #[schema(
        description = "Must contain uppercase, lowercase, digit, and special character",
        pattern = "^(?=.*[a-z])(?=.*[A-Z])(?=.*\\d)(?=.*[@$!%*?&])[A-Za-z\\d@$!%*?&]{8,}$",
        min_length = 8
    )]
    password: String,
}
```

**Why hints are needed:**
Custom validators contain arbitrary logic that can't be automatically converted to OpenAPI constraints. Use `#[schema(...)]` to manually specify the constraints for documentation.

## Schema Hints

The `#[schema(...)]` attribute provides additional metadata:

### Available Attributes

```rust
#[derive(Validate, ToSchema)]
struct Product {
    #[validate(min_len = 1, max_len = 100)]
    #[schema(
        description = "Product name",
        example = "Acme Widget",
        deprecated = false,
        read_only = false,
        write_only = false
    )]
    name: String,

    #[validate(range(min = 0, max = 1000000))]
    #[schema(
        description = "Price in cents",
        example = 1999,
        default = 0
    )]
    price: i32,

    #[validate(one_of = ["draft", "published", "archived"])]
    #[schema(
        description = "Publication status",
        example = "draft"
    )]
    status: String,
}
```

### Struct-Level Attributes

```rust
#[derive(Validate, ToSchema)]
#[schema(
    description = "User creation request",
    example = r#"{"email": "user@example.com", "age": 25}"#
)]
struct CreateUserRequest {
    #[validate(email)]
    email: String,

    #[validate(range(min = 18, max = 120))]
    age: u8,
}
```

## Advanced Usage

### Combining Auto-Derivation with Manual Overrides

For complex cases, you can mix auto-derived schemas with manual enhancements:

```rust
#[derive(Validate, ToSchema)]
#[schema(
    description = "Complex user registration with custom business rules",
    example = r#"{
        "email": "alice@example.com",
        "age": 25,
        "username": "alice_123",
        "referral_code": "FRIEND2024"
    }"#
)]
struct UserRegistration {
    #[validate(email)]
    #[validate(max_len = 255)]
    #[schema(description = "Must be unique in the system")]
    email: String,

    #[validate(range(min = 18, max = 120))]
    #[schema(description = "Must be 18+ for GDPR compliance")]
    age: u8,

    #[validate(custom = "validate_unique_username")]
    #[schema(
        description = "Alphanumeric username, must be unique",
        pattern = "^[a-zA-Z0-9_]{3,20}$",
        min_length = 3,
        max_length = 20
    )]
    username: String,

    #[validate(custom = "validate_referral_code")]
    #[schema(
        description = "Optional referral code from existing user",
        pattern = "^[A-Z0-9]{8,12}$"
    )]
    referral_code: Option<String>,
}
```

### Cross-Field Validation

Cross-field validation (struct-level `#[validate(...)]`) doesn't map to OpenAPI directly. Use schema description:

```rust
#[derive(Validate, ToSchema)]
#[validate(
    check = "self.password == self.password_confirmation",
    code = "passwords_mismatch",
    message = "Passwords must match"
)]
#[schema(description = "Passwords must match (validated on submission)")]
struct RegisterForm {
    #[validate(min_len = 8)]
    password: String,

    password_confirmation: String,
}
```

### Conditional Validation

For `.when()` conditional validation, document the condition in schema description:

```rust
#[derive(Validate, ToSchema)]
struct ConditionalForm {
    #[validate(email)]
    #[schema(description = "Required if notification_enabled is true")]
    email: Option<String>,

    #[schema(description = "Enable email notifications")]
    notification_enabled: bool,
}
```

## Migration Guide

### From Manual ToSchema to Auto-Derived

**Before:**
```rust
#[derive(Validate)]
struct User {
    #[validate(email)]
    #[validate(max_len = 255)]
    email: String,

    #[validate(range(min = 18, max = 120))]
    age: u8,
}

impl ToSchema for User {
    fn schema_name() -> &'static str { "User" }

    fn schema() -> Schema {
        Schema::object()
            .property("email", Schema::string().format("email").max_length(255))
            .property("age", Schema::integer().minimum(18).maximum(120))
            .required(&["email", "age"])
    }
}
```

**After:**
```rust
#[derive(Validate, ToSchema)]
struct User {
    #[validate(email)]
    #[validate(max_len = 255)]
    email: String,

    #[validate(range(min = 18, max = 120))]
    age: u8,
}
```

**Steps:**
1. Add `ToSchema` to the derive list
2. Remove manual `impl ToSchema`
3. Add `#[schema(...)]` hints for custom validators
4. Test that generated schema matches expectations

### Gradual Migration

You can migrate incrementally:

1. **Keep existing manual impls** for complex types
2. **Use auto-derivation** for new simple types
3. **Migrate one type at a time** when refactoring

Auto-derived and manual impls can coexist in the same codebase.

## Best Practices

### 1. Always Add Descriptions

```rust
#[derive(Validate, ToSchema)]
struct User {
    #[validate(email)]
    #[schema(description = "User's primary email address")]  // ← Always describe!
    email: String,
}
```

### 2. Provide Examples

```rust
#[derive(Validate, ToSchema)]
struct CreateUser {
    #[validate(email)]
    #[schema(example = "alice@example.com")]  // ← Help API consumers!
    email: String,
}
```

### 3. Document Custom Validators

```rust
#[derive(Validate, ToSchema)]
struct Account {
    #[validate(custom = "validate_iban")]
    #[schema(
        description = "International Bank Account Number",
        pattern = "^[A-Z]{2}[0-9]{2}[A-Z0-9]+$",
        example = "DE89370400440532013000"
    )]
    iban: String,
}
```

### 4. Use Struct-Level Examples for Complex Types

```rust
#[derive(Validate, ToSchema)]
#[schema(example = r#"{
    "name": "Alice Johnson",
    "email": "alice@example.com",
    "age": 25
}"#)]
struct User {
    #[validate(min_len = 2)]
    name: String,

    #[validate(email)]
    email: String,

    #[validate(range(min = 18, max = 120))]
    age: u8,
}
```

## Troubleshooting

### "Schema constraint not generated"

**Problem:** A validation rule isn't showing up in the schema.

**Solutions:**
- Check the [Rule Mapping Reference](#rule-mapping-reference) to see if the rule maps to OpenAPI
- For custom validators, add `#[schema(...)]` hints manually
- For `.when()` conditionals, document in schema description

### "Conflicting constraints"

**Problem:** Multiple validation rules create conflicting schema constraints.

**Solution:** The derive macro applies all constraints. Ensure your validation rules are compatible:

```rust
// This is fine:
#[validate(min_len = 3)]
#[validate(max_len = 20)]  // minLength: 3, maxLength: 20

// This creates conflict:
#[validate(email)]
#[validate(matches_regex = "^custom$")]  // format: email AND pattern: ^custom$
// Solution: Use one or the other, or combine with .or()
```

### "Nested type not found"

**Problem:** Schema references a type that doesn't implement `ToSchema`.

**Solution:** Ensure all nested types also derive `ToSchema`:

```rust
#[derive(Validate, ToSchema)]  // ← Both must derive ToSchema
struct Email {
    #[validate(email)]
    value: String,
}

#[derive(Validate, ToSchema)]  // ← Both must derive ToSchema
struct User {
    #[validate(nested)]
    email: Email,
}
```

## Building OpenAPI Specs

Register your types to generate a complete OpenAPI specification:

```rust
use domainstack_schema::{OpenApiBuilder, ToSchema};

#[derive(Validate, ToSchema)]
struct CreateUserRequest {
    #[validate(email, max_len = 255)]
    #[schema(example = "alice@example.com")]
    email: String,

    #[validate(range(min = 18, max = 120))]
    age: u8,
}

#[derive(ToSchema)]
struct User {
    id: String,
    email: String,
    age: u8,
}

// Generate OpenAPI 3.0 spec
let spec = OpenApiBuilder::new("User API", "1.0.0")
    .description("User management API")
    .register::<CreateUserRequest>()
    .register::<User>()
    .build();

// Export as JSON or YAML
let json = spec.to_json()?;
let yaml = spec.to_yaml()?;  // Requires "yaml" feature
```

## See Also

- [API Guide](api-guide.md) - Core validation patterns
- [HTTP Integration](HTTP_INTEGRATION.md) - Framework adapters
- [RULES.md](RULES.md) - Complete validation rules reference

# OpenAPI Schema Generation

Complete guide to generating OpenAPI 3.0 schemas from domainstack validation rules.

## Auto-Derive Schemas

**Feature flag:** `schema`

Generate OpenAPI 3.0 schemas from validation rules:

```rust
use domainstack::{Validate, ToSchema};

#[derive(Validate, ToSchema)]
#[schema(description = "User registration data")]
struct User {
    #[validate(email)]
    #[validate(max_len = 255)]
    #[schema(example = "user@example.com")]
    email: String,

    #[validate(range(min = 18, max = 120))]
    #[schema(description = "User's age in years")]
    age: u8,

    #[validate(url)]
    #[schema(example = "https://example.com")]
    website: Option<String>,
}
```

## Generated OpenAPI Schema

```json
{
  "type": "object",
  "description": "User registration data",
  "required": ["email", "age"],
  "properties": {
    "email": {
      "type": "string",
      "format": "email",
      "maxLength": 255,
      "example": "user@example.com"
    },
    "age": {
      "type": "integer",
      "minimum": 18,
      "maximum": 120,
      "description": "User's age in years"
    },
    "website": {
      "type": "string",
      "format": "uri",
      "example": "https://example.com"
    }
  }
}
```

## Validation Rule → Schema Mappings

| Validation Rule | OpenAPI Property |
|----------------|------------------|
| `email()` | `format: "email"` |
| `url()` | `format: "uri"` |
| `min_len(n)` | `minLength: n` |
| `max_len(n)` | `maxLength: n` |
| `range(min, max)` | `minimum: min, maximum: max` |
| `positive()` | `minimum: 0, exclusiveMinimum: true` |
| `min_items(n)` | `minItems: n` |
| `max_items(n)` | `maxItems: n` |
| `unique()` | `uniqueItems: true` |
| `alphanumeric()` | `pattern: "^[a-zA-Z0-9]*$"` |
| `Option<T>` | Excluded from `required` array |

## Schema Attributes

```rust
#[schema(description = "Field description")]   // Field/type description
#[schema(example = "value")]                    // Example value
#[schema(deprecated = true)]                    // Mark as deprecated
#[schema(read_only = true)]                     // Response-only field
#[schema(write_only = true)]                    // Request-only field
```

## Building OpenAPI Specs

```rust
use domainstack_schema::{OpenApiBuilder, ToSchema};

let spec = OpenApiBuilder::new("My API", "1.0.0")
    .description("API for user management")
    .register::<User>()
    .register::<Post>()
    .register::<Comment>()
    .build();

// Export as JSON
let json = spec.to_json()?;

// Or YAML (with "yaml" feature)
let yaml = spec.to_yaml()?;
```

## Benefits

- **Single source of truth** - Validation rules = API documentation
- **Always in sync** - Change validation, schema updates automatically
- **Type-safe** - Compile-time guarantees for schema generation
- **Zero maintenance** - No manual schema writing

## See Also

- Complete guide: [SCHEMA_DERIVATION.md](SCHEMA_DERIVATION.md)
- Example: `domainstack-schema/examples/auto_derive.rs`
- Tests: `domainstack-derive/tests/schema_derive.rs`
- Main guide: [API Guide](api-guide.md)

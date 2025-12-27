# JSON Schema Generation

**Auto-generate JSON Schema (Draft 2020-12) from validation rules**

Generate JSON Schema automatically from your validation rules, enabling frontend validation, API gateway validation, and cross-language schema sharing.

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
- [CLI Alternative](#cli-alternative)

## Quick Start

```rust
use domainstack::prelude::*;
use domainstack_derive::{Validate, ToJsonSchema};
use domainstack_schema::JsonSchemaBuilder;

// Write validation rules ONCE, get BOTH runtime validation AND JSON Schema!
#[derive(Validate, ToJsonSchema)]
struct User {
    #[validate(email)]
    #[validate(max_len = 255)]
    email: String,

    #[validate(range(min = 18, max = 120))]
    age: u8,
}

// Runtime validation works
let user = User { email, age };
user.validate()?;  // Validates email format, length, age range

// Schema generation works
let schema = User::json_schema();
// Automatically includes:
//   - email: format="email", maxLength=255
//   - age: minimum=18, maximum=120
//   - required=["email", "age"]

// Build complete JSON Schema document
let doc = JsonSchemaBuilder::new()
    .title("My API Schema")
    .register::<User>()
    .build();

let json = serde_json::to_string_pretty(&doc)?;
```

## Why Auto-Derivation

**The Problem:** Without auto-derivation, you write validation constraints twice—once for runtime validation, once for JSON Schema. This creates duplication, drift, and maintenance burden.

**The Solution:** With `#[derive(Validate, ToJsonSchema)]`, you write validation rules **once** and get **both** runtime validation AND JSON Schema:

```rust
#[derive(Validate, ToJsonSchema)]
struct CreateUser {
    #[validate(email)]
    #[validate(max_len = 255)]
    email: String,

    #[validate(range(min = 18, max = 120))]
    age: u8,

    #[validate(min_len = 2)]
    #[validate(max_len = 50)]
    name: String,
}

// Runtime validation works automatically
let user = CreateUser::new(email, age, name)?;  // Validates all rules

// Schema generation works automatically
let schema = CreateUser::json_schema();  // Includes all constraints
```

**Generated JSON Schema:**
```json
{
  "type": "object",
  "title": "CreateUser",
  "required": ["email", "age", "name"],
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
    },
    "name": {
      "type": "string",
      "minLength": 2,
      "maxLength": 50
    }
  },
  "additionalProperties": false
}
```

**Benefits:**
- Write validation rules **once**
- Schema **always matches** validation
- Less boilerplate
- Single source of truth
- Impossible for docs to drift from validation

## Rule Mapping Reference

The derive macro automatically maps validation rules to JSON Schema constraints:

### String Rules

| Validation Rule | JSON Schema Constraint | Example |
|----------------|------------------------|---------|
| `email()` | `format: "email"` | `#[validate(email)]` → `"format": "email"` |
| `url()` | `format: "uri"` | `#[validate(url)]` → `"format": "uri"` |
| `min_len(n)` | `minLength: n` | `#[validate(min_len = 3)]` → `"minLength": 3` |
| `max_len(n)` | `maxLength: n` | `#[validate(max_len = 255)]` → `"maxLength": 255` |
| `length(min, max)` | `minLength, maxLength` | `#[validate(length(min = 3, max = 20))]` → both |
| `non_empty` | `minLength: 1` | Ensures non-empty string |
| `non_blank` | `minLength: 1, pattern` | Non-whitespace start |
| `matches_regex(p)` | `pattern: p` | `#[validate(matches_regex = "^[A-Z].*")]` → `"pattern": "^[A-Z].*"` |
| `ascii()` | `pattern: "^[\\x00-\\x7F]*$"` | ASCII characters only |
| `alphanumeric()` | `pattern: "^[a-zA-Z0-9]*$"` | Letters and digits only |
| `alpha_only()` | `pattern: "^[a-zA-Z]*$"` | Letters only |
| `numeric_string()` | `pattern: "^[0-9]*$"` | Digits only |
| `no_whitespace` | `pattern: "^\\S*$"` | No whitespace |
| `starts_with(s)` | `pattern: "^prefix.*"` | Prefix pattern |
| `ends_with(s)` | `pattern: ".*suffix$"` | Suffix pattern |
| `contains(s)` | `pattern: ".*needle.*"` | Contains pattern |

### Numeric Rules

| Validation Rule | JSON Schema Constraint | Example |
|----------------|------------------------|---------|
| `min(n)` | `minimum: n` | `#[validate(min = 0)]` → `"minimum": 0` |
| `max(n)` | `maximum: n` | `#[validate(max = 100)]` → `"maximum": 100` |
| `range(min, max)` | `minimum, maximum` | `#[validate(range(min = 18, max = 120))]` → both |
| `positive()` | `exclusiveMinimum: 0` | Greater than zero |
| `negative()` | `exclusiveMaximum: 0` | Less than zero |
| `non_zero()` | `not: {const: 0}` | Not equal to zero |
| `multiple_of(n)` | `multipleOf: n` | `#[validate(multiple_of = 5)]` → `"multipleOf": 5` |

### Collection Rules

| Validation Rule | JSON Schema Constraint | Example |
|----------------|------------------------|---------|
| `min_items(n)` | `minItems: n` | `#[validate(min_items = 1)]` → `"minItems": 1` |
| `max_items(n)` | `maxItems: n` | `#[validate(max_items = 10)]` → `"maxItems": 10` |
| `unique()` | `uniqueItems: true` | All array items must be unique |

## Nested Types

Nested validation automatically includes referenced schemas:

```rust
#[derive(Validate, ToJsonSchema)]
struct Email {
    #[validate(email)]
    #[validate(max_len = 255)]
    value: String,
}

#[derive(Validate, ToJsonSchema)]
struct Guest {
    #[validate(min_len = 2)]
    #[validate(max_len = 50)]
    name: String,

    #[validate(nested)]  // Automatically references Email schema
    email: Email,
}
```

**Generated schema:**
```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$defs": {
    "Guest": {
      "type": "object",
      "title": "Guest",
      "required": ["name", "email"],
      "properties": {
        "name": {
          "type": "string",
          "minLength": 2,
          "maxLength": 50
        },
        "email": {
          "$ref": "#/$defs/Email"
        }
      },
      "additionalProperties": false
    },
    "Email": {
      "type": "object",
      "title": "Email",
      "required": ["value"],
      "properties": {
        "value": {
          "type": "string",
          "format": "email",
          "maxLength": 255
        }
      },
      "additionalProperties": false
    }
  }
}
```

## Collections and Arrays

### Nested Collections with `each(nested)`

Array validation for nested types using `#[validate(each(nested))]`:

```rust
#[derive(Validate, ToJsonSchema)]
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
          "$ref": "#/$defs/User"
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
#[derive(Validate, ToJsonSchema)]
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

## Optional Fields

Optional fields (using `Option<T>`) are not included in the `required` array:

```rust
#[derive(Validate, ToJsonSchema)]
struct UpdateUser {
    #[validate(email)]
    #[validate(max_len = 255)]
    email: Option<String>,  // Optional, not in "required"

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
}

#[derive(Validate, ToJsonSchema)]
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
Custom validators contain arbitrary logic that can't be automatically converted to JSON Schema constraints. Use `#[schema(...)]` to manually specify the constraints for documentation.

## Schema Hints

The `#[schema(...)]` attribute provides additional metadata:

### Available Attributes

```rust
#[derive(Validate, ToJsonSchema)]
struct Product {
    #[validate(min_len = 1, max_len = 100)]
    #[schema(
        description = "Product name",
        example = "Acme Widget"
    )]
    name: String,

    #[validate(range(min = 0, max = 1000000))]
    #[schema(
        description = "Price in cents",
        default = 0
    )]
    price: i32,
}
```

## Advanced Usage

### Building Complete Schema Documents

```rust
use domainstack_schema::{JsonSchemaBuilder, ToJsonSchema};

#[derive(Validate, ToJsonSchema)]
struct User {
    #[validate(email, max_len = 255)]
    email: String,

    #[validate(range(min = 18, max = 120))]
    age: u8,
}

#[derive(Validate, ToJsonSchema)]
struct Order {
    #[validate(positive)]
    total: f64,

    #[validate(nested)]
    customer: User,
}

// Generate complete JSON Schema document
let doc = JsonSchemaBuilder::new()
    .title("My API Schema")
    .description("Auto-generated from validation rules")
    .register::<User>()
    .register::<Order>()
    .build();

// Export as JSON
let json = serde_json::to_string_pretty(&doc)?;
```

### Using with Frontend Validation (Ajv)

```typescript
import Ajv from 'ajv';
import schema from './schema.json';

const ajv = new Ajv();
const validate = ajv.getSchema('#/$defs/UserRegistration');

const valid = validate(formData);
if (!valid) {
  console.log(validate.errors);
}
```

### Using with Python Validation

```python
from jsonschema import validate
import json

with open('schema.json') as f:
    schema = json.load(f)

user_schema = schema['$defs']['User']
validate(instance=user_data, schema=user_schema)
```

## CLI Alternative

For build-time codegen without implementing traits, use `domainstack-cli`:

```bash
# Generate JSON Schema from Rust source files
domainstack json-schema --input src --output schema.json

# With verbose output
domainstack json-schema --input src --output schema.json --verbose

# Watch mode for development
domainstack json-schema --input src --output schema.json --watch
```

The CLI parses `#[validate(...)]` attributes from source files and generates JSON Schema without requiring trait implementations.

## Type Mappings

### Primitive Types

| Rust Type | JSON Schema Type | Notes |
|-----------|-----------------|-------|
| `String` | `"type": "string"` | |
| `bool` | `"type": "boolean"` | |
| `u8`, `u16`, `u32`, `i8`, `i16`, `i32` | `"type": "integer"` | Safe integer range |
| `u64`, `u128`, `i64`, `i128` | `"type": "integer"` | May exceed JS safe integer |
| `f32`, `f64` | `"type": "number"` | Floating point |

### Compound Types

| Rust Type | JSON Schema Type | Notes |
|-----------|-----------------|-------|
| `Option<T>` | Same as `T` | Field not in `required` array |
| `Vec<T>` | `"type": "array", "items": {...}` | Array with item schema |
| Custom struct | `"$ref": "#/$defs/TypeName"` | Reference to definition |

## Limitations

Some validation rules don't map directly to JSON Schema:

1. **Cross-field validation** - Cannot express field dependencies
2. **Conditional validation** - `.when()` clauses not mapped
3. **Async validation** - Database checks have no JSON Schema equivalent

For these cases, use vendor extensions:
```rust
#[schema(x_validation = "end_date > start_date")]
```

## See Also

- [OpenAPI Schema](OPENAPI_SCHEMA.md) - OpenAPI 3.0 schema generation
- [CLI Guide](CLI_GUIDE.md) - Full CLI documentation
- [RULES.md](RULES.md) - Complete validation rules reference

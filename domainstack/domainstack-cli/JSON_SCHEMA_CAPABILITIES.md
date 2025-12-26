# JSON Schema Capabilities Reference

**Version:** domainstack-cli v1.0.0
**JSON Schema Version:** Draft 2020-12
**Last Updated:** 2025-12-26

This document provides a comprehensive reference of JSON Schema features supported by `domainstack json-schema`.

---

## Table of Contents

1. [Feature Coverage Matrix](#feature-coverage-matrix)
2. [Validation Rule Mappings](#validation-rule-mappings)
3. [Type Mappings](#type-mappings)
4. [Generated Schema Structure](#generated-schema-structure)
5. [Examples](#examples)
6. [Limitations](#limitations)
7. [Best Practices](#best-practices)

---

## Feature Coverage Matrix

### JSON Schema Draft 2020-12 Keywords

| Keyword | Support | Notes |
|---------|---------|-------|
| **$schema** | Full | Set to `https://json-schema.org/draft/2020-12/schema` |
| **$id** | Full | Set to `https://example.com/schemas/generated.json` |
| **$defs** | Full | All types placed in `$defs` |
| **$ref** | Full | Custom types referenced via `#/$defs/TypeName` |
| **title** | Full | Type name becomes title |
| **description** | Full | Auto-generated from context |
| **type** | Full | `string`, `integer`, `number`, `boolean`, `array`, `object` |
| **properties** | Full | All struct fields mapped |
| **required** | Full | Non-`Option` fields are required |
| **additionalProperties** | Full | Set to `false` for strict schemas |

### String Keywords

| Keyword | Support | Mapped From |
|---------|---------|-------------|
| **minLength** | Full | `min_len`, `length(min, max)` |
| **maxLength** | Full | `max_len`, `length(min, max)` |
| **pattern** | Full | `matches_regex`, `alphanumeric`, `alpha_only`, etc. |
| **format** | Full | `email`, `url` |

### Numeric Keywords

| Keyword | Support | Mapped From |
|---------|---------|-------------|
| **minimum** | Full | `min`, `range(min, max)` |
| **maximum** | Full | `max`, `range(min, max)` |
| **exclusiveMinimum** | Full | `positive` |
| **exclusiveMaximum** | Full | `negative` |
| **multipleOf** | None | Not currently mapped |

### Array Keywords

| Keyword | Support | Mapped From |
|---------|---------|-------------|
| **items** | Full | `Vec<T>` element type |
| **minItems** | None | Not currently mapped |
| **maxItems** | None | Not currently mapped |
| **uniqueItems** | None | Not currently mapped |

### Object Keywords

| Keyword | Support | Mapped From |
|---------|---------|-------------|
| **properties** | Full | Struct fields |
| **required** | Full | Non-`Option<T>` fields |
| **additionalProperties** | Full | Always `false` |

---

## Validation Rule Mappings

### String Validations

| Rust Rule | JSON Schema Output | Example |
|-----------|-------------------|---------|
| `#[validate(email)]` | `"format": "email"` | Email format validation |
| `#[validate(url)]` | `"format": "uri"` | URI format validation |
| `#[validate(min_len = N)]` | `"minLength": N` | Minimum string length |
| `#[validate(max_len = N)]` | `"maxLength": N` | Maximum string length |
| `#[validate(length(min = M, max = N))]` | `"minLength": M, "maxLength": N` | Length range |
| `#[validate(non_empty)]` | `"minLength": 1` | Non-empty string |
| `#[validate(matches_regex = "pattern")]` | `"pattern": "pattern"` | Regex pattern |
| `#[validate(alphanumeric)]` | `"pattern": "^[a-zA-Z0-9]*$"` | Alphanumeric only |
| `#[validate(alpha_only)]` | `"pattern": "^[a-zA-Z]*$"` | Letters only |
| `#[validate(numeric_string)]` | `"pattern": "^[0-9]*$"` | Digits only |
| `#[validate(ascii)]` | `"pattern": "^[\\x00-\\x7F]*$"` | ASCII characters |
| `#[validate(no_whitespace)]` | `"pattern": "^\\S*$"` | No whitespace |
| `#[validate(starts_with = "prefix")]` | `"pattern": "^prefix.*"` | Prefix pattern |
| `#[validate(ends_with = "suffix")]` | `"pattern": ".*suffix$"` | Suffix pattern |
| `#[validate(contains = "needle")]` | `"pattern": ".*needle.*"` | Contains pattern |

### Numeric Validations

| Rust Rule | JSON Schema Output | Example |
|-----------|-------------------|---------|
| `#[validate(range(min = M, max = N))]` | `"minimum": M, "maximum": N` | Numeric range |
| `#[validate(min = N)]` | `"minimum": N` | Minimum value |
| `#[validate(max = N)]` | `"maximum": N` | Maximum value |
| `#[validate(positive)]` | `"exclusiveMinimum": 0` | Greater than zero |
| `#[validate(negative)]` | `"exclusiveMaximum": 0` | Less than zero |
| `#[validate(non_zero)]` | `"not": {"const": 0}` | Not zero |

### Custom Validations

| Rust Rule | JSON Schema Output | Notes |
|-----------|-------------------|-------|
| `#[validate(custom = "rule_name")]` | `"x-custom-validation": "rule_name"` | Vendor extension |

---

## Type Mappings

### Primitive Types

| Rust Type | JSON Schema Type | Notes |
|-----------|-----------------|-------|
| `String` | `"type": "string"` | |
| `&str` | `"type": "string"` | |
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

---

## Generated Schema Structure

### Root Schema

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://example.com/schemas/generated.json",
  "title": "Generated Schemas",
  "description": "Auto-generated JSON Schema from domainstack validation rules",
  "$defs": {
    "TypeName": { ... },
    "AnotherType": { ... }
  }
}
```

### Type Schema

```json
{
  "type": "object",
  "title": "User",
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
    "website": {
      "type": "string",
      "format": "uri"
    }
  },
  "required": ["email", "age"],
  "additionalProperties": false
}
```

---

## Examples

### Example 1: User Registration

**Rust Input:**

```rust
#[derive(Validate)]
struct UserRegistration {
    #[validate(email)]
    #[validate(max_len = 255)]
    email: String,

    #[validate(length(min = 8, max = 128))]
    password: String,

    #[validate(length(min = 1, max = 100))]
    name: String,

    #[validate(range(min = 13, max = 120))]
    age: u8,

    #[validate(url)]
    website: Option<String>,
}
```

**Generated JSON Schema:**

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$defs": {
    "UserRegistration": {
      "type": "object",
      "title": "UserRegistration",
      "properties": {
        "email": {
          "type": "string",
          "format": "email",
          "maxLength": 255
        },
        "password": {
          "type": "string",
          "minLength": 8,
          "maxLength": 128
        },
        "name": {
          "type": "string",
          "minLength": 1,
          "maxLength": 100
        },
        "age": {
          "type": "integer",
          "minimum": 13,
          "maximum": 120
        },
        "website": {
          "type": "string",
          "format": "uri"
        }
      },
      "required": ["email", "password", "name", "age"],
      "additionalProperties": false
    }
  }
}
```

### Example 2: Product Catalog

**Rust Input:**

```rust
#[derive(Validate)]
struct Product {
    #[validate(length(min = 1, max = 200))]
    name: String,

    #[validate(max_len = 2000)]
    description: Option<String>,

    #[validate(range(min = 0.0, max = 1000000.0))]
    price: f64,

    #[validate(positive)]
    quantity: i32,

    tags: Vec<String>,
}
```

**Generated JSON Schema:**

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$defs": {
    "Product": {
      "type": "object",
      "title": "Product",
      "properties": {
        "name": {
          "type": "string",
          "minLength": 1,
          "maxLength": 200
        },
        "description": {
          "type": "string",
          "maxLength": 2000
        },
        "price": {
          "type": "number",
          "minimum": 0.0,
          "maximum": 1000000.0
        },
        "quantity": {
          "type": "integer",
          "exclusiveMinimum": 0
        },
        "tags": {
          "type": "array",
          "items": {
            "type": "string"
          }
        }
      },
      "required": ["name", "price", "quantity", "tags"],
      "additionalProperties": false
    }
  }
}
```

### Example 3: Nested Types

**Rust Input:**

```rust
#[derive(Validate)]
struct Address {
    #[validate(length(min = 1, max = 100))]
    street: String,

    #[validate(length(min = 2, max = 50))]
    city: String,

    #[validate(matches_regex = "^[0-9]{5}(-[0-9]{4})?$")]
    postal_code: String,
}

#[derive(Validate)]
struct Customer {
    #[validate(email)]
    email: String,

    #[validate(nested)]
    billing_address: Address,

    #[validate(nested)]
    shipping_address: Option<Address>,
}
```

**Generated JSON Schema:**

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$defs": {
    "Address": {
      "type": "object",
      "title": "Address",
      "properties": {
        "street": {
          "type": "string",
          "minLength": 1,
          "maxLength": 100
        },
        "city": {
          "type": "string",
          "minLength": 2,
          "maxLength": 50
        },
        "postal_code": {
          "type": "string",
          "pattern": "^[0-9]{5}(-[0-9]{4})?$"
        }
      },
      "required": ["street", "city", "postal_code"],
      "additionalProperties": false
    },
    "Customer": {
      "type": "object",
      "title": "Customer",
      "properties": {
        "email": {
          "type": "string",
          "format": "email"
        },
        "billing_address": {
          "$ref": "#/$defs/Address"
        },
        "shipping_address": {
          "$ref": "#/$defs/Address"
        }
      },
      "required": ["email", "billing_address"],
      "additionalProperties": false
    }
  }
}
```

---

## Limitations

### Not Currently Supported

1. **Collection constraints** - `min_items`, `max_items`, `unique` rules are not mapped
2. **Cross-field validation** - Cannot express field dependencies in JSON Schema
3. **Conditional validation** - `when` clauses not mapped
4. **Async validation** - Database checks have no JSON Schema equivalent
5. **multipleOf** - Numeric divisibility rule not mapped
6. **Enum variants** - Rust enums are validated but not fully represented

### Workarounds

**For collection constraints:**
```rust
// Validate at runtime, document in description
/// List of tags (1-10 items required)
#[validate(min_items = 1)]
#[validate(max_items = 10)]
tags: Vec<String>,
```

**For cross-field validation:**
Use vendor extensions to document:
```json
{
  "x-domainstack-validation": {
    "cross_field": "end_date > start_date"
  }
}
```

---

## Best Practices

### 1. Use Descriptive Field Names

```rust
// GOOD: Clear field names
#[derive(Validate)]
struct User {
    email_address: String,
    date_of_birth: String,
}

// BAD: Abbreviated names
#[derive(Validate)]
struct User {
    e: String,
    dob: String,
}
```

### 2. Apply Appropriate Constraints

```rust
// GOOD: Realistic constraints
#[validate(length(min = 1, max = 255))]
email: String,

// BAD: No constraints or unrealistic
email: String,  // No validation!

#[validate(max_len = 1000000)]  // Too permissive
email: String,
```

### 3. Use Option for Optional Fields

```rust
// GOOD: Explicit optionality
#[validate(url)]
website: Option<String>,  // Not required

// The JSON Schema will not include "website" in required array
```

### 4. Combine Validation Rules

```rust
// GOOD: Multiple complementary rules
#[validate(email)]
#[validate(max_len = 255)]
#[validate(non_empty)]
email: String,

// Generates:
// "format": "email", "maxLength": 255, "minLength": 1
```

### 5. Document Custom Types

```rust
/// User authentication credentials
#[derive(Validate)]
struct LoginRequest {
    /// Email address for login
    #[validate(email)]
    email: String,

    /// Password (8-128 characters)
    #[validate(length(min = 8, max = 128))]
    password: String,
}
```

---

## CLI Usage Reference

### Generate JSON Schema

```bash
# Basic usage
domainstack json-schema --input src --output schema.json

# With verbose output
domainstack json-schema --input src --output schema.json --verbose

# Watch mode for development
domainstack json-schema --input src --output schema.json --watch

# Combined options
domainstack json-schema -i src/models -o api/schema.json -v --watch
```

### Options

| Option | Short | Description | Default |
|--------|-------|-------------|---------|
| `--input` | `-i` | Input directory with Rust files | `src` |
| `--output` | `-o` | Output JSON file path | (required) |
| `--verbose` | `-v` | Show processing details | false |
| `--watch` | `-w` | Watch for changes | false |

---

## Use Cases

### 1. API Gateway Validation

Generate schemas for AWS API Gateway, Kong, or other gateways:

```bash
domainstack json-schema --input src/api --output openapi/schemas.json
```

### 2. Frontend Form Validation

Use with [Ajv](https://ajv.js.org/) in JavaScript/TypeScript:

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

### 3. Python Validation

Use with [jsonschema](https://python-jsonschema.readthedocs.io/):

```python
from jsonschema import validate
import json

with open('schema.json') as f:
    schema = json.load(f)

user_schema = schema['$defs']['User']
validate(instance=user_data, schema=user_schema)
```

### 4. Documentation

Include in API documentation for language-agnostic validation specs.

---

## Version Compatibility

| domainstack-cli | JSON Schema Draft | Rust MSRV |
|-----------------|-------------------|-----------|
| 1.0.0 | 2020-12 | 1.76+ |

---

## Related Documentation

- [CLI Guide](../domainstack/docs/CLI_GUIDE.md) - Full CLI documentation
- [RULES.md](../domainstack/docs/RULES.md) - All validation rules
- [domainstack-schema](../domainstack-schema/) - OpenAPI schema generation

For issues: https://github.com/blackwell-systems/domainstack/issues

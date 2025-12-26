# CLI Guide

**Generate TypeScript/Zod schemas and JSON Schema from Rust validation rules**

The `domainstack-cli` tool generates frontend validation schemas from your Rust code, creating a single source of truth for validation across your entire stack.

## Table of Contents

- [Quick Start](#quick-start)
- [Installation](#installation)
- [TypeScript/Zod Generation](#typescriptzod-generation)
- [JSON Schema Generation](#json-schema-generation)
- [Watch Mode](#watch-mode)
- [CLI Usage](#cli-usage)
- [Integration](#integration)
- [Rule Mapping](#rule-mapping)
- [Examples](#examples)
- [CI/CD Integration](#cicd-integration)
- [Troubleshooting](#troubleshooting)

## Quick Start

```bash
# Install the CLI
cargo install domainstack-cli

# Generate Zod schemas from Rust types
domainstack zod --input src --output frontend/src/schemas.ts
```

**From this Rust code:**

```rust
#[derive(Validate)]
struct User {
    #[validate(email)]
    #[validate(max_len = 255)]
    email: String,

    #[validate(range(min = 18, max = 120))]
    age: u8,

    #[validate(url)]
    website: Option<String>,
}
```

**Generates this TypeScript:**

```typescript
// frontend/src/schemas.ts (AUTO-GENERATED)
import { z } from "zod";

export const UserSchema = z.object({
  email: z.string().email().max(255),
  age: z.number().min(18).max(120),
  website: z.string().url().optional(),
});

export type User = z.infer<typeof UserSchema>;
```

## Installation

### Install the CLI

```bash
cargo install domainstack-cli
```

### Verify installation

```bash
domainstack --version
```

### Install Zod (frontend dependency)

```bash
# npm
npm install zod

# yarn
yarn add zod

# pnpm
pnpm add zod
```

## TypeScript/Zod Generation

### Why Generate Frontend Schemas?

**The Problem:** Maintaining separate validation rules on frontend and backend leads to:
- Duplication of validation logic
- Frontend/backend drift over time
- Manual synchronization errors
- Inconsistent error messages

**The Solution:** Generate frontend schemas from backend Rust code:
- **Single source of truth** - Change validation once, regenerate schemas
- **Frontend/backend in sync** - Guaranteed consistency
- **Zero maintenance** - No manual schema writing
- **Type-safe** - Zod's type inference works automatically

### Supported Validation Rules

**26+ validation rules are automatically converted:**

| Category | Rules |
|----------|-------|
| **String** | email, url, min_len, max_len, length, alphanumeric, matches_regex, contains, starts_with, ends_with |
| **Numeric** | range, min, max, positive, negative, non_zero, multiple_of |
| **Collections** | min_items, max_items, unique |
| **Types** | Optional fields, Arrays, Nested types |

### String Validation Examples

```rust
#[derive(Validate)]
struct Contact {
    #[validate(email)]
    email: String,

    #[validate(url)]
    website: String,

    #[validate(length(min = 3, max = 50))]
    name: String,

    #[validate(matches_regex = r"^\d{3}-\d{3}-\d{4}$")]
    phone: String,
}
```

**Generates:**

```typescript
export const ContactSchema = z.object({
  email: z.string().email(),
  website: z.string().url(),
  name: z.string().min(3).max(50),
  phone: z.string().regex(/^\d{3}-\d{3}-\d{4}$/),
});
```

### Numeric Validation Examples

```rust
#[derive(Validate)]
struct Product {
    #[validate(range(min = 0, max = 1000000))]
    price: i32,

    #[validate(min = 1)]
    quantity: u32,

    #[validate(positive)]
    weight: f64,
}
```

**Generates:**

```typescript
export const ProductSchema = z.object({
  price: z.number().min(0).max(1000000),
  quantity: z.number().min(1),
  weight: z.number().positive(),
});
```

### Optional Fields

```rust
#[derive(Validate)]
struct Profile {
    #[validate(url)]
    website: Option<String>,

    #[validate(length(min = 10, max = 500))]
    bio: Option<String>,
}
```

**Generates:**

```typescript
export const ProfileSchema = z.object({
  website: z.string().url().optional(),
  bio: z.string().min(10).max(500).optional(),
});
```

### Arrays and Collections

```rust
#[derive(Validate)]
struct BlogPost {
    #[validate(each(length(min = 1, max = 50)))]
    #[validate(min_items = 1)]
    #[validate(max_items = 10)]
    tags: Vec<String>,

    #[validate(each(range(min = 1, max = 5)))]
    ratings: Vec<u8>,
}
```

**Generates:**

```typescript
export const BlogPostSchema = z.object({
  tags: z.array(z.string().min(1).max(50)).min(1).max(10),
  ratings: z.array(z.number().min(1).max(5)),
});
```

### Nested Types

```rust
#[derive(Validate)]
struct Address {
    #[validate(length(min = 1, max = 100))]
    street: String,

    #[validate(length(min = 2, max = 2))]
    country_code: String,
}

#[derive(Validate)]
struct User {
    #[validate(email)]
    email: String,

    #[validate(nested)]
    address: Address,
}
```

**Generates:**

```typescript
export const AddressSchema = z.object({
  street: z.string().min(1).max(100),
  country_code: z.string().min(2).max(2),
});

export const UserSchema = z.object({
  email: z.string().email(),
  address: AddressSchema,
});
```

## JSON Schema Generation

Generate JSON Schema (Draft 2020-12) from your Rust validation rules. JSON Schema is widely supported by API gateways, OpenAPI tools, and validation libraries across all languages.

### Quick Start

```bash
# Generate JSON Schema
domainstack json-schema --input src --output schemas/types.json
```

**From this Rust code:**

```rust
#[derive(Validate)]
struct User {
    #[validate(email)]
    #[validate(max_len = 255)]
    email: String,

    #[validate(range(min = 18, max = 120))]
    age: u8,

    #[validate(url)]
    website: Option<String>,
}
```

**Generates this JSON Schema:**

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://example.com/schemas/generated.json",
  "title": "Generated Schemas",
  "$defs": {
    "User": {
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
  }
}
```

### JSON Schema Command

```bash
domainstack json-schema [OPTIONS]

Options:
  -i, --input <PATH>     Input directory containing Rust source files
                         [default: src]

  -o, --output <PATH>    Output JSON file path
                         [required]

  -w, --watch            Watch for changes and regenerate automatically

  -v, --verbose          Enable verbose output

  -h, --help             Print help information
```

### JSON Schema Rule Mappings

| Rust Rule | JSON Schema Property | Example |
|-----------|---------------------|---------|
| `email` | `"format": "email"` | Email format validation |
| `url` | `"format": "uri"` | URI format validation |
| `min_len(n)` | `"minLength": n` | Minimum string length |
| `max_len(n)` | `"maxLength": n` | Maximum string length |
| `matches_regex(p)` | `"pattern": "p"` | Regex pattern |
| `alphanumeric` | `"pattern": "^[a-zA-Z0-9]*$"` | Alphanumeric pattern |
| `range(min, max)` | `"minimum": m, "maximum": n` | Numeric range |
| `min(n)` | `"minimum": n` | Minimum value |
| `max(n)` | `"maximum": n` | Maximum value |
| `positive` | `"exclusiveMinimum": 0` | Greater than zero |
| `negative` | `"exclusiveMaximum": 0` | Less than zero |
| `Option<T>` | Not in `required` array | Optional field |
| `Vec<T>` | `"type": "array", "items": {...}` | Array type |
| Custom type | `"$ref": "#/$defs/TypeName"` | Type reference |

### Use Cases

**API Gateway Validation:**
```bash
# Generate schema for AWS API Gateway, Kong, etc.
domainstack json-schema --input src/api --output openapi/schemas.json
```

**Form Validation Libraries:**
JSON Schema is supported by [Ajv](https://ajv.js.org/), [jsonschema](https://python-jsonschema.readthedocs.io/), and many other validators across languages.

**Documentation:**
Include generated schemas in your API documentation for language-agnostic validation specs.

---

## Watch Mode

Both `zod` and `json-schema` commands support watch mode for automatic regeneration when files change.

```bash
# Watch mode with Zod
domainstack zod --input src --output schemas.ts --watch

# Watch mode with JSON Schema
domainstack json-schema --input src --output schema.json --watch
```

Watch mode:
- Monitors the input directory for `.rs` file changes
- Debounces rapid changes (500ms) to avoid excessive regeneration
- Shows real-time feedback on file changes and regeneration

---

## CLI Usage

### Basic Command

```bash
domainstack zod --output schemas.ts
```

### All Options

```bash
domainstack zod [OPTIONS]

Options:
  -i, --input <PATH>     Input directory containing Rust source files
                         [default: src]

  -o, --output <PATH>    Output TypeScript file path
                         [required]

  -w, --watch            Watch for changes and regenerate automatically

  -v, --verbose          Enable verbose output showing processed files

  -h, --help             Print help information
```

### Common Usage Patterns

```bash
# Generate from default src/ directory
domainstack zod --output frontend/src/schemas.ts

# Custom input directory
domainstack zod --input backend/models --output schemas.ts

# Verbose mode (shows processing details)
domainstack zod -i src -o schemas.ts -v

# Watch mode for development
domainstack zod -i src -o schemas.ts --watch

# Use in npm scripts
npm run codegen  # calls: domainstack zod -o src/schemas.ts
```

## Integration

### Full-Stack Example

**Backend (Rust):**

```rust
// src/models.rs
use domainstack::prelude::*;

#[derive(Validate, Serialize, Deserialize)]
pub struct CreateUserRequest {
    #[validate(email)]
    #[validate(max_len = 255)]
    pub email: String,

    #[validate(length(min = 3, max = 50))]
    #[validate(alphanumeric)]
    pub username: String,

    #[validate(range(min = 18, max = 120))]
    pub age: u8,
}
```

**Frontend (TypeScript):**

```typescript
// Generated automatically: frontend/src/schemas.ts
import { z } from "zod";

export const CreateUserRequestSchema = z.object({
  email: z.string().email().max(255),
  username: z.string().min(3).max(50).regex(/^[a-zA-Z0-9]*$/),
  age: z.number().min(18).max(120),
});

export type CreateUserRequest = z.infer<typeof CreateUserRequestSchema>;
```

**Using in React:**

```typescript
import { CreateUserRequestSchema } from "./schemas";

function UserForm() {
  const handleSubmit = (formData: unknown) => {
    // Validate with generated schema
    const result = CreateUserRequestSchema.safeParse(formData);

    if (result.success) {
      // Type-safe validated data
      const request: CreateUserRequest = result.data;
      await api.createUser(request);
    } else {
      // Display field-level errors
      result.error.errors.forEach(err => {
        showError(err.path.join("."), err.message);
      });
    }
  };
}
```

### NPM Script Integration

Add to your `package.json`:

```json
{
  "scripts": {
    "codegen": "domainstack zod --input ../backend/src --output src/schemas.ts",
    "codegen:watch": "nodemon --watch ../backend/src --exec npm run codegen",
    "prebuild": "npm run codegen"
  }
}
```

### Monorepo Setup

```bash
my-project/
├── backend/
│   ├── src/
│   │   └── models.rs       # Rust validation models
│   └── Cargo.toml
├── frontend/
│   ├── src/
│   │   └── schemas.ts      # Generated Zod schemas (auto-generated)
│   └── package.json
└── package.json            # Root package.json with codegen script
```

**Root package.json:**

```json
{
  "scripts": {
    "codegen": "cd backend && domainstack zod --output ../frontend/src/schemas.ts"
  }
}
```

## Rule Mapping

### String Rule Mappings

| Rust Rule | Zod Equivalent | Example |
|-----------|----------------|---------|
| `email` | `.email()` | `z.string().email()` |
| `url` | `.url()` | `z.string().url()` |
| `min_len(n)` | `.min(n)` | `z.string().min(3)` |
| `max_len(n)` | `.max(n)` | `z.string().max(50)` |
| `length(min, max)` | `.min(m).max(n)` | `z.string().min(3).max(50)` |
| `matches_regex(p)` | `.regex(p)` | `z.string().regex(/^[A-Z]/)` |
| `alphanumeric` | `.regex(/^[a-zA-Z0-9]*$/)` | Auto-generated pattern |
| `contains(s)` | `.includes(s)` | `z.string().includes("@")` |
| `starts_with(s)` | `.startsWith(s)` | `z.string().startsWith("pre")` |
| `ends_with(s)` | `.endsWith(s)` | `z.string().endsWith(".com")` |

### Numeric Rule Mappings

| Rust Rule | Zod Equivalent | Example |
|-----------|----------------|---------|
| `range(min, max)` | `.min(m).max(n)` | `z.number().min(0).max(100)` |
| `min(n)` | `.min(n)` | `z.number().min(18)` |
| `max(n)` | `.max(n)` | `z.number().max(120)` |
| `positive` | `.positive()` | `z.number().positive()` |
| `negative` | `.negative()` | `z.number().negative()` |
| `non_zero` | `.refine(n => n !== 0)` | Custom refinement |
| `multiple_of(n)` | `.multipleOf(n)` | `z.number().multipleOf(5)` |

### Collection Rule Mappings

| Rust Rule | Zod Equivalent | Example |
|-----------|----------------|---------|
| `Vec<T>` | `z.array(T)` | `z.array(z.string())` |
| `min_items(n)` | `.min(n)` | `z.array(T).min(1)` |
| `max_items(n)` | `.max(n)` | `z.array(T).max(10)` |
| `each(rule)` | `z.array(T.rule())` | `z.array(z.string().email())` |
| `Option<T>` | `.optional()` | `z.string().optional()` |

## Examples

### Example 1: API Request/Response Types

```rust
// Backend
#[derive(Validate)]
pub struct CreatePostRequest {
    #[validate(length(min = 1, max = 200))]
    pub title: String,

    #[validate(length(min = 10, max = 5000))]
    pub content: String,

    #[validate(each(length(min = 1, max = 50)))]
    #[validate(min_items = 1)]
    #[validate(max_items = 10)]
    pub tags: Vec<String>,

    #[validate(one_of = ["draft", "published"])]
    pub status: String,
}
```

```typescript
// Frontend (generated)
export const CreatePostRequestSchema = z.object({
  title: z.string().min(1).max(200),
  content: z.string().min(10).max(5000),
  tags: z.array(z.string().min(1).max(50)).min(1).max(10),
  status: z.enum(["draft", "published"]),
});
```

### Example 2: Complex Nested Types

```rust
#[derive(Validate)]
pub struct PaymentMethod {
    #[validate(length(min = 16, max = 16))]
    pub card_number: String,

    #[validate(range(min = 1, max = 12))]
    pub exp_month: u8,

    #[validate(range(min = 2024, max = 2034))]
    pub exp_year: u16,
}

#[derive(Validate)]
pub struct Order {
    #[validate(nested)]
    pub payment: PaymentMethod,

    #[validate(each(nested))]
    #[validate(min_items = 1)]
    pub items: Vec<OrderItem>,

    #[validate(positive)]
    pub total: f64,
}
```

```typescript
export const PaymentMethodSchema = z.object({
  card_number: z.string().min(16).max(16),
  exp_month: z.number().min(1).max(12),
  exp_year: z.number().min(2024).max(2034),
});

export const OrderSchema = z.object({
  payment: PaymentMethodSchema,
  items: z.array(OrderItemSchema).min(1),
  total: z.number().positive(),
});
```

## CI/CD Integration

### GitHub Actions

```yaml
# .github/workflows/codegen.yml
name: Code Generation

on:
  push:
    paths:
      - 'backend/src/**/*.rs'

jobs:
  generate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Install domainstack-cli
        run: cargo install domainstack-cli

      - name: Generate Zod schemas
        run: |
          domainstack zod \
            --input backend/src \
            --output frontend/src/schemas.ts

      - name: Check for uncommitted changes
        run: |
          git diff --exit-code frontend/src/schemas.ts || \
            (echo "[x] Schemas out of date! Run: npm run codegen" && exit 1)

      - name: Commit generated schemas
        if: github.ref == 'refs/heads/main'
        run: |
          git config user.name "GitHub Actions"
          git config user.email "actions@github.com"
          git add frontend/src/schemas.ts
          git commit -m "chore: regenerate Zod schemas" || exit 0
          git push
```

### Pre-commit Hook

```bash
# .git/hooks/pre-commit
#!/bin/bash

# Regenerate schemas
npm run codegen

# Stage generated files
git add frontend/src/schemas.ts
```

### Make it executable:

```bash
chmod +x .git/hooks/pre-commit
```

## Troubleshooting

### "Command not found: domainstack"

**Solution:** Ensure the CLI is installed and in your PATH:

```bash
cargo install domainstack-cli
export PATH="$HOME/.cargo/bin:$PATH"
```

### "No Rust files found"

**Problem:** CLI can't find Rust source files.

**Solution:** Check the input directory path:

```bash
# Verify files exist
ls -la backend/src

# Use correct path
domainstack zod --input backend/src --output schemas.ts
```

### "Unsupported validation rule"

**Problem:** A validation rule doesn't have a Zod equivalent.

**Solution:**
- Check the [Rule Mapping](#rule-mapping) table for supported rules
- For custom validators, manually add refinements in TypeScript after generation
- File an issue for commonly needed rules

### Generated schema has wrong types

**Problem:** Numeric types generating as strings, or vice versa.

**Solution:** Ensure your Rust types match Zod expectations:
- `u8`, `u16`, `u32`, `i32`, `f32`, `f64` → `z.number()`
- `String` → `z.string()`
- `bool` → `z.boolean()`
- `Vec<T>` → `z.array(T)`
- `Option<T>` → `T.optional()`

## Available Generators

| Generator | Status | Description |
|-----------|--------|-------------|
| `domainstack zod` | ✅ Available | TypeScript/Zod schemas |
| `domainstack json-schema` | ✅ Available | JSON Schema (Draft 2020-12) |
| `domainstack yup` | 📋 Planned | Yup schemas for React ecosystem |
| `domainstack graphql` | 📋 Planned | GraphQL SDL generation |
| `domainstack prisma` | 📋 Planned | Prisma schemas with validation |

## See Also

- [Core Concepts](CORE_CONCEPTS.md) - Foundation principles and patterns
- [RULES.md](RULES.md) - Complete validation rules reference
- [DERIVE_MACRO.md](DERIVE_MACRO.md) - Derive macro usage
- CLI Repository: `domainstack-cli/` - Source code and issue tracker

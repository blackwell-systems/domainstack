# domainstack-cli

[![Crates.io](https://img.shields.io/crates/v/domainstack-cli.svg)](https://crates.io/crates/domainstack-cli)
[![License: Apache 2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://github.com/blackwell-systems/domainstack/blob/main/LICENSE)

Code generation CLI for the [domainstack](https://crates.io/crates/domainstack) full-stack validation ecosystem. Generate TypeScript/Zod validators from your Rust `#[validate(...)]` attributes.

## Overview

`domainstack-cli` is a command-line tool that transforms Rust types annotated with domainstack validation rules into equivalent schemas for other languages and frameworks. This ensures your validation logic stays synchronized across your entire stack.

```bash
# Single source of truth in Rust
#[derive(Validate)]
struct User {
    #[validate(email)]
    #[validate(max_len = 255)]
    email: String,

    #[validate(range(min = 18, max = 120))]
    age: u8,
}

# Generate TypeScript/Zod schemas automatically
domainstack zod --input src --output frontend/schemas.ts
```

## Installation

### From crates.io (when published)

```bash
cargo install domainstack-cli
```

### From source

```bash
# Clone the repository
git clone https://github.com/blackwell-systems/domainstack
cd domainstack/domainstack/domainstack-cli

# Build and install
cargo install --path .
```

### Verify installation

```bash
domainstack --version
```

## Quick Start

### 1. Define your Rust types with validation rules

```rust
// src/models.rs
use domainstack::Validate;

#[derive(Validate)]
struct User {
    #[validate(email)]
    #[validate(max_len = 255)]
    email: String,

    #[validate(length(min = 3, max = 50))]
    #[validate(alphanumeric)]
    username: String,

    #[validate(range(min = 18, max = 120))]
    age: u8,

    #[validate(url)]
    profile_url: Option<String>,
}
```

### 2. Generate Zod schemas

```bash
domainstack zod --input src --output frontend/src/schemas.ts
```

### 3. Use the generated schemas in TypeScript

```typescript
// frontend/src/schemas.ts (auto-generated)
import { z } from "zod";

export const UserSchema = z.object({
  email: z.string().email().max(255),
  username: z.string().min(3).max(50).regex(/^[a-zA-Z0-9]*$/),
  age: z.number().min(18).max(120),
  profile_url: z.string().url().optional(),
});

export type User = z.infer<typeof UserSchema>;

// Use it in your application
const result = UserSchema.safeParse(formData);
if (result.success) {
  // Type-safe validated data
  const user: User = result.data;
}
```

## Commands

### `domainstack zod`

Generate Zod validation schemas from Rust types.

```bash
domainstack zod [OPTIONS]
```

**Options:**

- `-i, --input <PATH>` - Input directory containing Rust source files (default: `src`)
- `-o, --output <PATH>` - Output TypeScript file (required)
- `-v, --verbose` - Enable verbose output
- `-h, --help` - Print help information

**Examples:**

```bash
# Basic usage
domainstack zod --output schemas.ts

# Specify input directory
domainstack zod --input backend/src --output frontend/schemas.ts

# Verbose output
domainstack zod -i src -o schemas.ts -v
```

## Supported Validation Rules

### String Validations

| Rust Attribute | Zod Output | Description |
|---------------|------------|-------------|
| `#[validate(email)]` | `.email()` | Valid email address |
| `#[validate(url)]` | `.url()` | Valid URL |
| `#[validate(min_len = N)]` | `.min(N)` | Minimum string length |
| `#[validate(max_len = N)]` | `.max(N)` | Maximum string length |
| `#[validate(length(min = N, max = M))]` | `.min(N).max(M)` | String length range |
| `#[validate(non_empty)]` | `.min(1)` | Non-empty string |
| `#[validate(non_blank)]` | `.trim().min(1)` | Non-blank string (after trim) |
| `#[validate(alphanumeric)]` | `.regex(/^[a-zA-Z0-9]*$/)` | Alphanumeric only |
| `#[validate(alpha_only)]` | `.regex(/^[a-zA-Z]*$/)` | Letters only |
| `#[validate(numeric_string)]` | `.regex(/^[0-9]*$/)` | Digits only |
| `#[validate(ascii)]` | `.regex(/^[\x00-\x7F]*$/)` | ASCII characters only |
| `#[validate(starts_with = "prefix")]` | `.startsWith("prefix")` | Must start with prefix |
| `#[validate(ends_with = "suffix")]` | `.endsWith("suffix")` | Must end with suffix |
| `#[validate(contains = "substring")]` | `.includes("substring")` | Must contain substring |
| `#[validate(matches_regex = "pattern")]` | `.regex(/pattern/)` | Custom regex pattern |
| `#[validate(no_whitespace)]` | `.regex(/^\S*$/)` | No whitespace allowed |

### Numeric Validations

| Rust Attribute | Zod Output | Description |
|---------------|------------|-------------|
| `#[validate(range(min = N, max = M))]` | `.min(N).max(M)` | Numeric range |
| `#[validate(min = N)]` | `.min(N)` | Minimum value |
| `#[validate(max = N)]` | `.max(N)` | Maximum value |
| `#[validate(positive)]` | `.positive()` | Must be positive (> 0) |
| `#[validate(negative)]` | `.negative()` | Must be negative (< 0) |
| `#[validate(non_zero)]` | `.refine(n => n !== 0, ...)` | Cannot be zero |
| `#[validate(multiple_of = N)]` | `.multipleOf(N)` | Must be multiple of N |
| `#[validate(finite)]` | `.finite()` | Must be finite (not NaN/Infinity) |

### Type Mappings

| Rust Type | Zod Type | Notes |
|-----------|----------|-------|
| `String` | `z.string()` | |
| `bool` | `z.boolean()` | |
| `u8, u16, u32, i8, i16, i32, f32, f64` | `z.number()` | |
| `u64, u128, i64, i128` | `z.number()` | With precision warning comment |
| `Option<T>` | `T.optional()` | Validations applied to inner type |
| `Vec<T>` | `z.array(T)` | |
| Custom types | `CustomTypeSchema` | References generated schema |

## Examples

### Multiple Validation Rules

You can apply multiple validation rules to a single field:

```rust
#[derive(Validate)]
struct Account {
    #[validate(email)]
    #[validate(max_len = 255)]
    #[validate(non_empty)]
    email: String,

    #[validate(min_len = 8)]
    #[validate(max_len = 128)]
    #[validate(matches_regex = "^(?=.*[A-Z])(?=.*[a-z])(?=.*[0-9])")]
    password: String,
}
```

Generates:

```typescript
export const AccountSchema = z.object({
  email: z.string().email().max(255).min(1),
  password: z.string().min(8).max(128).regex(/(?=.*[A-Z])(?=.*[a-z])(?=.*[0-9])/),
});
```

### Optional Fields with Validations

Optional fields have `.optional()` applied AFTER all validations:

```rust
#[derive(Validate)]
struct Profile {
    #[validate(url)]
    website: Option<String>,

    #[validate(length(min = 10, max = 500))]
    bio: Option<String>,
}
```

Generates:

```typescript
export const ProfileSchema = z.object({
  website: z.string().url().optional(),  // Correct order
  bio: z.string().min(10).max(500).optional(),
});
```

### Arrays and Collections

```rust
#[derive(Validate)]
struct Post {
    tags: Vec<String>,

    #[validate(min = 1)]
    #[validate(max = 100)]
    scores: Vec<u8>,
}
```

Generates:

```typescript
export const PostSchema = z.object({
  tags: z.array(z.string()),
  scores: z.array(z.number()).min(1).max(100),
});
```

## Architecture

### Unified CLI Design

`domainstack-cli` is designed as a **unified code generation tool** with a single binary and multiple subcommands:

```
domainstack
├── zod        Generate Zod schemas (v0.1.0)
├── yup        📋 Generate Yup schemas (planned)
├── graphql    📋 Generate GraphQL schemas (planned)
└── prisma     📋 Generate Prisma schemas (planned)
```

**Benefits:**
- Single installation, multiple generators
- Shared parsing infrastructure (efficient, consistent)
- Consistent CLI interface across all generators
- Easy to add new generators

### Internal Structure

```
domainstack-cli/
├── src/
│   ├── main.rs              # CLI entry point with clap
│   ├── commands/            # Subcommand implementations
│   │   └── zod.rs
│   ├── parser/              # Shared parsing infrastructure
│   │   ├── mod.rs           # Directory walking
│   │   ├── ast.rs           # Rust AST parsing
│   │   └── validation.rs    # Validation rule extraction
│   └── generators/          # Language-specific generators
│       └── zod.rs
```

The parser module (`parser/`) is shared across all generators, ensuring consistent interpretation of Rust validation rules. Each generator (`generators/`) contains language-specific transformation logic.

## Future Generators

The roadmap includes support for:

### Yup (TypeScript validation)
```bash
domainstack yup --input src --output schemas.ts
```

### GraphQL Schema Definition Language
```bash
domainstack graphql --input src --output schema.graphql
```

### Prisma Schema
```bash
domainstack prisma --input src --output schema.prisma
```

### JSON Schema
```bash
domainstack json-schema --input src --output schema.json
```

## Contributing

### Adding a New Generator

To add a new generator (e.g., Yup):

1. Create generator file: `src/generators/yup.rs`
2. Implement generation logic using shared parser types
3. Create command file: `src/commands/yup.rs`
4. Add subcommand to `src/main.rs`

The shared parser infrastructure (`parser::ParsedType`, `parser::ValidationRule`) makes adding new generators straightforward - focus only on the output format transformation.

### Development Setup

```bash
# Clone repository
git clone https://github.com/blackwell-systems/domainstack
cd domainstack

# Build CLI
cargo build -p domainstack-cli

# Run tests
cargo test -p domainstack-cli

# Install locally
cargo install --path domainstack/domainstack-cli
```

## Troubleshooting

### "No types found with validation rules"

Make sure your Rust types:
1. Have `#[derive(Validate)]` attribute
2. Contain at least one `#[validate(...)]` attribute
3. Are in `.rs` files within the input directory

### Generated schemas don't match expectations

Run with `--verbose` flag to see parsing details:

```bash
domainstack zod --input src --output schemas.ts --verbose
```

### Large integer precision warnings

JavaScript numbers cannot safely represent integers larger than `Number.MAX_SAFE_INTEGER` (2^53 - 1). Types like `u64`, `i64`, `u128`, `i128` will generate schemas with inline warning comments:

```typescript
big_number: z.number() /* Warning: Large integers may lose precision in JavaScript */
```

Consider using strings for large integers in your TypeScript schemas if precision is critical.

## License

This project is part of the domainstack workspace. See the root LICENSE file for details.

## Related Projects

- [domainstack](../domainstack/) - Core validation library
- [zod](https://github.com/colinhacks/zod) - TypeScript-first schema validation
- [ts-rs](https://github.com/Aleph-Alpha/ts-rs) - TypeScript type generation (no validation)
- [typeshare](https://github.com/1Password/typeshare) - Cross-language type sharing

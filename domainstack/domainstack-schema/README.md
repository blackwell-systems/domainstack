# domainstack-schema

[![Blackwell Systems™](https://raw.githubusercontent.com/blackwell-systems/blackwell-docs-theme/main/badge-trademark.svg)](https://github.com/blackwell-systems)
[![Crates.io](https://img.shields.io/crates/v/domainstack-schema.svg)](https://crates.io/crates/domainstack-schema)
[![Documentation](https://docs.rs/domainstack-schema/badge.svg)](https://docs.rs/domainstack-schema)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg)](https://github.com/blackwell-systems/domainstack/blob/main/LICENSE-MIT)

OpenAPI and JSON Schema generation for the [domainstack](https://crates.io/crates/domainstack) full-stack validation ecosystem.

## Overview

`domainstack-schema` provides tools to generate **OpenAPI 3.0 schemas** and **JSON Schema (Draft 2020-12)** from your domainstack domain types. This enables automatic API documentation and validation schemas that stay in sync with your validation rules.

## Two Approaches

| Approach | OpenAPI | JSON Schema |
|----------|---------|-------------|
| **Trait** | `ToSchema` | `ToJsonSchema` |
| **CLI** | `domainstack openapi` | `domainstack json-schema` |

Use the **trait approach** for programmatic generation and type-safe composition.
Use the **CLI approach** for build-time codegen from source files.

## Installation

```toml
[dependencies]
domainstack-schema = "1.0.0"
```

## Quick Start: OpenAPI

```rust
use domainstack_schema::{OpenApiBuilder, Schema, ToSchema};

struct User {
    email: String,
    age: u8,
}

impl ToSchema for User {
    fn schema_name() -> &'static str {
        "User"
    }

    fn schema() -> Schema {
        Schema::object()
            .property("email", Schema::string().format("email"))
            .property("age", Schema::integer().minimum(18).maximum(120))
            .required(&["email", "age"])
    }
}

fn main() {
    let spec = OpenApiBuilder::new("My API", "1.0.0")
        .description("User management API")
        .register::<User>()
        .build();

    let json = spec.to_json().expect("Failed to serialize");
    println!("{}", json);
}
```

## Quick Start: JSON Schema

```rust
use domainstack_schema::{JsonSchema, JsonSchemaBuilder, ToJsonSchema};

struct User {
    email: String,
    age: u8,
}

impl ToJsonSchema for User {
    fn schema_name() -> &'static str {
        "User"
    }

    fn json_schema() -> JsonSchema {
        JsonSchema::object()
            .property("email", JsonSchema::string().format("email"))
            .property("age", JsonSchema::integer().minimum(0).maximum(150))
            .required(&["email", "age"])
    }
}

fn main() {
    let doc = JsonSchemaBuilder::new()
        .title("My Schema")
        .register::<User>()
        .build();

    let json = serde_json::to_string_pretty(&doc).expect("Failed to serialize");
    println!("{}", json);
}
```

## Schema Constraints

**Supports all field-level validation rules that have direct OpenAPI Schema constraint mappings:**

| Validation Rule | OpenAPI Constraint | Example |
|----------------|-------------------|---------|
| `length(min, max)` | `minLength`, `maxLength` | `.min_length(3).max_length(50)` |
| `range(min, max)` | `minimum`, `maximum` | `.minimum(0).maximum(100)` |
| `email()` | `format: "email"` | `.format("email")` |
| `one_of(...)` | `enum` | `.enum_values(&["a", "b"])` |
| `numeric_string()` | `pattern: "^[0-9]+$"` | `.pattern("^[0-9]+$")` |
| `min_items(n)` | `minItems` | `.min_items(1)` |
| `max_items(n)` | `maxItems` | `.max_items(10)` |

**Note:** Cross-field validations, conditional rules, and business logic validations (e.g., database uniqueness) don't have direct OpenAPI equivalents. For these, use vendor extensions (see below).

## v0.8 Features

### Schema Composition

Combine schemas using `anyOf`, `allOf`, or `oneOf`:

```rust
// Union type (anyOf): string OR integer
let flexible = Schema::any_of(vec![
    Schema::string(),
    Schema::integer(),
]);

// Composition (allOf): extends base schema
let admin_user = Schema::all_of(vec![
    Schema::reference("User"),
    Schema::object().property("admin", Schema::boolean()),
]);

// Discriminated union (oneOf): exactly one match
let payment = Schema::one_of(vec![
    Schema::object().property("type", Schema::string().enum_values(&["card"])),
    Schema::object().property("type", Schema::string().enum_values(&["cash"])),
]);
```

### Metadata & Documentation

Add defaults, examples, and documentation:

```rust
let theme = Schema::string()
    .enum_values(&["light", "dark", "auto"])
    .default(json!("auto"))          // Default value
    .example(json!("dark"))           // Single example
    .examples(vec![                   // Multiple examples
        json!("light"),
        json!("dark"),
    ])
    .description("UI theme preference");
```

### Request/Response Modifiers

Mark fields as read-only, write-only, or deprecated:

```rust
let user = Schema::object()
    .property("id",
        Schema::string()
            .read_only(true)         // Response only
            .description("Auto-generated ID")
    )
    .property("password",
        Schema::string()
            .format("password")
            .write_only(true)        // Request only
            .min_length(8)
    )
    .property("old_field",
        Schema::string()
            .deprecated(true)        // Mark as deprecated
            .description("Use 'new_field' instead")
    );
```

## Schema Types

Build schemas using the fluent API:

```rust
use domainstack_schema::Schema;

// String with constraints
let name = Schema::string()
    .min_length(1)
    .max_length(100)
    .description("User's full name");

// Integer with range
let age = Schema::integer()
    .minimum(0)
    .maximum(150);

// Enum
let status = Schema::string()
    .enum_values(&["active", "pending", "inactive"]);

// Array
let tags = Schema::array(Schema::string())
    .min_items(1)
    .max_items(10);

// Object with properties
let user = Schema::object()
    .property("name", name)
    .property("age", age)
    .required(&["name", "age"]);

// Reference to another schema
let team_member = Schema::reference("User");
```

## Building OpenAPI Specs

```rust
use domainstack_schema::OpenApiBuilder;

let spec = OpenApiBuilder::new("User API", "1.0.0")
    .description("API for managing users")
    .register::<User>()
    .register::<Address>()
    .register::<Team>()
    .build();

// Export as JSON
let json = spec.to_json()?;
println!("{}", json);
```

## Features

- **Type-safe schema generation**: Implement `ToSchema` or `ToJsonSchema` traits
- **Fluent API**: Chainable methods for building schemas
- **OpenAPI 3.0 compliant**: Generates valid OpenAPI specifications
- **JSON Schema Draft 2020-12**: Standards-compliant JSON Schema generation
- **No runtime overhead**: Schema generation happens at build time
- **Framework agnostic**: Works with any Rust web framework

## Examples

### Basic Usage

See `examples/user_api.rs` for a complete example demonstrating:
- Multiple schema types (User, Address, Team)
- Validation constraint mapping
- Schema references
- Array constraints

```bash
cargo run --example user_api
```

### v0.8 Features

See `examples/v08_features.rs` for advanced features:
- Schema composition (anyOf/allOf/oneOf)
- Metadata (default/example/examples)
- Request/response modifiers (readOnly/writeOnly/deprecated)
- Vendor extensions for non-mappable validations

```bash
cargo run --example v08_features
```

## Scope & Positioning

**What this crate does:**
- Generates **OpenAPI 3.0 component schemas** for domain types
- Generates **JSON Schema (Draft 2020-12)** documents
- Maps field-level validations to schema constraints
- Provides type-safe schema builders
- Exports to JSON/YAML

**What this crate does NOT do:**
- API paths/operations (GET /users, POST /users, etc.)
- Request/response body definitions
- Security schemes or authentication
- Full API documentation generation

**Positioning:** `domainstack-schema` focuses on **schema generation** for domain types. Full OpenAPI spec generation (paths, operations, security) is intentionally out of scope and may be addressed in a separate crate.

## Handling Non-Mappable Validations

Some validation rules don't map cleanly to OpenAPI Schema constraints:

```rust
// Cross-field validation - no OpenAPI equivalent
#[validate(check = "self.end_date > self.start_date")]

// Conditional validation - no OpenAPI equivalent
#[validate(when = "self.requires_card", rule = "...")]

// Business logic - no OpenAPI equivalent
async fn validate_email_unique(&self, db: &Database) -> Result<()>
```

**Solution:** Use vendor extensions to preserve validation metadata:

```rust
Schema::object()
    .property("end_date", Schema::string().format("date"))
    .extension("x-domainstack-validations", json!({
        "cross_field": ["end_date > start_date"]
    }))
```

This maintains a single source of truth while acknowledging OpenAPI's expressiveness limits.

## Related Crates

| Crate | Purpose |
|-------|---------|
| [`domainstack`](https://crates.io/crates/domainstack) | Core validation library |
| [`domainstack-derive`](https://crates.io/crates/domainstack-derive) | `#[derive(Validate, ToSchema)]` macros |
| [`domainstack-cli`](https://crates.io/crates/domainstack-cli) | CLI for Zod, JSON Schema, OpenAPI generation |

## CLI Alternative

For build-time codegen from source files (without implementing traits), use `domainstack-cli`:

```bash
# Generate JSON Schema from Rust source files
domainstack json-schema --input src --output schemas/types.json

# Generate OpenAPI spec from Rust source files
domainstack openapi --input src --output api/openapi.json

# Generate Zod schemas for TypeScript
domainstack zod --input src --output frontend/schemas.ts
```

## Documentation

- **[JSON Schema Capabilities](../domainstack-cli/JSON_SCHEMA_CAPABILITIES.md)** - Complete JSON Schema feature reference
- **[CLI Guide](../domainstack/docs/CLI_GUIDE.md)** - CLI codegen for Zod, JSON Schema, OpenAPI
- **[API Docs](https://docs.rs/domainstack-schema)** - Full API documentation

## License

Licensed under MIT OR Apache-2.0.

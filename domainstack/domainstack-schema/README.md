# domainstack-schema

OpenAPI schema generation for [domainstack](https://crates.io/crates/domainstack) validation types.

## Overview

`domainstack-schema` provides tools to generate OpenAPI 3.0 schemas from your domainstack domain types. This enables automatic API documentation that stays in sync with your validation rules.

## Installation

```toml
[dependencies]
domainstack-schema = "0.7.0"
```

## Quick Start

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

## Schema Constraints

Map validation rules to OpenAPI constraints:

| Validation Rule | OpenAPI Constraint | Example |
|----------------|-------------------|---------|
| `length(min, max)` | `minLength`, `maxLength` | `.min_length(3).max_length(50)` |
| `range(min, max)` | `minimum`, `maximum` | `.minimum(0).maximum(100)` |
| `email()` | `format: "email"` | `.format("email")` |
| `one_of(...)` | `enum` | `.enum_values(&["a", "b"])` |
| `numeric_string()` | `pattern: "^[0-9]+$"` | `.pattern("^[0-9]+$")` |
| `min_items(n)` | `minItems` | `.min_items(1)` |
| `max_items(n)` | `maxItems` | `.max_items(10)` |

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

- **Type-safe schema generation**: Implement `ToSchema` trait for your types
- **Fluent API**: Chainable methods for building schemas
- **OpenAPI 3.0 compliant**: Generates valid OpenAPI specifications
- **No runtime overhead**: Schema generation happens at build time
- **Framework agnostic**: Works with any Rust web framework

## Examples

See `examples/user_api.rs` for a complete example demonstrating:
- Multiple schema types (User, Address, Team)
- Validation constraint mapping
- Schema references
- Array constraints

Run the example:

```bash
cargo run --example user_api
```

## Documentation

For more details, see the [main domainstack documentation](https://docs.rs/domainstack).

## License

Licensed under Apache-2.0.

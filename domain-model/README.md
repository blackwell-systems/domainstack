# domain-model

A Rust validation framework for domain-driven design with derive macro support.

## Overview

This workspace provides a complete validation solution for building domain models in Rust. It consists of three crates:

- **[domain-model](./domain-model/)** - Core validation library
- **[domain-model-derive](./domain-model-derive/)** - Derive macro for `#[derive(Validate)]`
- **[examples](./examples/)** - Comprehensive examples demonstrating both manual and derive-based validation

## Features

- **Valid-by-construction types** - Invalid states can't exist
- **Derive macro support** - `#[derive(Validate)]` with 5 attributes
- **Composable rules** - Combine validation logic with `and`, `or`, `when`
- **Structured error paths** - Field-level error reporting (e.g., `guest.email.value`, `rooms[1].adults`)
- **Zero dependencies** - Core crate uses only std (regex optional for email validation)

## Quick Start

### With Derive Macro (v0.2+)

```rust
use domain_model::prelude::*;
use domain_model_derive::Validate;

#[derive(Debug, Validate)]
struct User {
    #[validate(length(min = 1, max = 50))]
    name: String,
    
    #[validate(range(min = 18, max = 120))]
    age: u8,
}

#[derive(Debug, Validate)]
struct Booking {
    #[validate(nested)]
    guest: User,
    
    #[validate(each(nested))]
    rooms: Vec<Room>,
}
```

### Manual Validation

```rust
use domain_model::prelude::*;

struct Email(String);

impl Email {
    pub fn new(raw: String) -> Result<Self, ValidationError> {
        let rule = rules::email();
        validate("email", raw.as_str(), &rule)?;
        Ok(Self(raw))
    }
}
```

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
# Core library only
domain-model = "0.2"

# With derive macro support
domain-model = { version = "0.2", features = ["derive"] }

# Optional: enable regex-based email validation
domain-model = { version = "0.2", features = ["derive", "email"] }
```

## Derive Attributes

The `#[derive(Validate)]` macro supports 5 validation attributes:

### 1. Length Validation
```rust
#[validate(length(min = 1, max = 50))]
name: String,
```

### 2. Range Validation
```rust
#[validate(range(min = 18, max = 120))]
age: u8,
```

### 3. Nested Validation
```rust
#[validate(nested)]
guest: Guest,  // Guest must implement Validate
```

Errors are automatically prefixed: `guest.email.value`

### 4. Collection Validation
```rust
#[validate(each(nested))]
rooms: Vec<Room>,

#[validate(each(length(min = 3, max = 10)))]
tags: Vec<String>,
```

Errors include indices: `rooms[1].adults`, `tags[2]`

### 5. Custom Validation
```rust
#[validate(custom = "validate_even")]
value: u8,
```

Custom function signature:
```rust
fn validate_even(value: &u8) -> Result<(), ValidationError> {
    if *value % 2 == 0 {
        Ok(())
    } else {
        Err(ValidationError::single(Path::root(), "not_even", "Must be even"))
    }
}
```

## Built-in Rules

### String Rules
- `email()` - Email format validation
- `non_empty()` - Non-empty string
- `min_len(min)` - Minimum length
- `max_len(max)` - Maximum length
- `length(min, max)` - Length range

### Numeric Rules
- `range(min, max)` - Value range
- `min(min)` - Minimum value
- `max(max)` - Maximum value

### Rule Composition
- `rule1.and(rule2)` - Both rules must pass
- `rule1.or(rule2)` - Either rule must pass
- `rule.not(code, msg)` - Negate the rule
- `rule.when(predicate)` - Conditional validation
- `rule.map_path(prefix)` - Prefix error paths

## Running Examples

```bash
# v0.1 examples (manual validation)
cargo run --example email_primitive
cargo run --example age_primitive
cargo run --example booking_aggregate

# v0.2 examples (derive macro)
cargo run --example v2_basic
cargo run --example v2_nested
cargo run --example v2_collections
cargo run --example v2_custom
```

## Testing

```bash
# Run all tests (core + derive macro)
cargo test --all

# Test with all features
cargo test --all --all-features

# Test specific crate
cargo test -p domain-model
cargo test -p domain-model-derive
```

## Error Handling

```rust
match user.validate() {
    Ok(_) => println!("Valid!"),
    Err(e) => {
        println!("Validation failed with {} errors:", e.violations.len());
        for v in &e.violations {
            println!("  [{}] {} - {}", v.path, v.code, v.message);
        }
        
        // For API responses
        let field_errors = e.field_errors_map();
        // { "name": ["Must be at least 1 characters"],
        //   "age": ["Must be between 18 and 120"] }
    }
}
```

## Documentation

- [Core Library Documentation](./domain-model/README.md) - Detailed API documentation
- [Examples](./examples/) - Runnable code examples
- [CONCEPT.md](../CONCEPT.md) - Design philosophy and architecture

## Workspace Structure

```
domain-model/
├── domain-model/           # Core validation library
│   ├── src/
│   │   ├── error.rs       # ValidationError
│   │   ├── path.rs        # Structured error paths
│   │   ├── rule.rs        # Rule<T> and composition
│   │   ├── rules/         # Built-in validation rules
│   │   └── validate.rs    # Validate trait
│   └── examples/          # v0.1 manual validation examples
├── domain-model-derive/    # Derive macro implementation
│   ├── src/lib.rs         # #[derive(Validate)] proc macro
│   └── tests/             # Macro integration tests
└── examples/               # v0.2 derive macro examples
```

## Version History

- **v0.2.0** - Derive macro with 5 attributes, workspace structure
- **v0.1.0** - Core validation library with manual Validate trait

## License

MIT

## Author

Dayna Blackwell <blackwellsystems@protonmail.com>

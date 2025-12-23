# domain-model

A Rust validation framework for domain-driven design.

## Features

- **Valid-by-construction types** - Invalid states can't exist
- **Composable rules** - Combine validation logic with `and`, `or`, `when`
- **Structured error paths** - Field-level error reporting for APIs
- **Zero dependencies** - Core crate uses only std (regex optional for email validation)

## Quick Start

```rust
use domain_model::prelude::*;

struct Email(String);

impl Email {
    pub fn new(raw: String) -> Result<Self, ValidationError> {
        validate("email", raw.as_str(), rules::email())?;
        Ok(Self(raw))
    }
}
```

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
domain-model = "0.1"

# Optional: enable regex-based email validation
domain-model = { version = "0.1", features = ["email"] }
```

## Examples

### Domain Primitive (Email)

```rust
use domain_model::prelude::*;

#[derive(Debug, Clone)]
pub struct Email(String);

impl Email {
    pub fn new(raw: impl Into<String>) -> Result<Self, ValidationError> {
        let raw = raw.into();
        validate("email", raw.as_str(), rules::email().and(rules::max_len(255)))?;
        Ok(Self(raw))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

// Usage
let email = Email::new("user@example.com")?;
```

### Domain Aggregate (Booking)

```rust
use domain_model::prelude::*;

#[derive(Debug)]
pub struct Guest {
    pub name: String,
    pub email: Email,
}

impl Validate for Guest {
    fn validate(&self) -> Result<(), ValidationError> {
        let mut err = ValidationError::default();

        if let Err(e) = validate("name", self.name.as_str(), 
            rules::min_len(1).and(rules::max_len(50))) {
            err.extend(e);
        }

        if let Err(e) = self.email.validate() {
            err.merge_prefixed("email", e);
        }

        if err.is_empty() { Ok(()) } else { Err(err) }
    }
}

#[derive(Debug)]
pub struct BookingRequest {
    pub guest: Guest,
    pub guests_count: u8,
}

impl Validate for BookingRequest {
    fn validate(&self) -> Result<(), ValidationError> {
        let mut err = ValidationError::default();

        if let Err(e) = validate("guests_count", &self.guests_count, 
            rules::range(1, 10)) {
            err.extend(e);
        }

        if let Err(e) = self.guest.validate() {
            err.merge_prefixed("guest", e);
        }

        if err.is_empty() { Ok(()) } else { Err(err) }
    }
}
```

### Error Handling

```rust
match BookingRequest::new(guest, 15) {
    Ok(booking) => println!("Valid: {:?}", booking),
    Err(e) => {
        println!("Validation failed with {} errors:", e.violations.len());
        for v in &e.violations {
            println!("  [{} {}] {}", v.path, v.code, v.message);
        }
        
        // Field errors map for API responses
        let field_errors = e.field_errors_map();
        // { "guests_count": ["Must be between 1 and 10"], 
        //   "guest.name": ["Must be at least 1 characters"] }
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
cargo run --example email_primitive
cargo run --example age_primitive
cargo run --example booking_aggregate
```

## Testing

```bash
cargo test
cargo test --all-features  # Test with email feature enabled
```

## License

Apache 2.0

## Author

Dayna Blackwell <blackwellsystems@protonmail.com>

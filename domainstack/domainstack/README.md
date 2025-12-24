# domainstack

A Rust validation framework for domain-driven design.

## Features

- **Valid-by-construction types** - Invalid states can't exist
- **Composable rules** - Combine validation logic with `and`, `or`, `when`
- **Structured error paths** - Field-level error reporting for APIs
- **Zero dependencies** - Core crate uses only std (regex optional for email/URL validation)
- **31 built-in rules** - String (17), Numeric (8), Choice (3), Collection (3)

## Quick Start

```rust
use domainstack::prelude::*;

struct Username(String);

impl Username {
    pub fn new(raw: String) -> Result<Self, ValidationError> {
        let rule = rules::min_len(3).and(rules::max_len(20));
        validate("username", raw.as_str(), &rule)?;
        Ok(Self(raw))
    }
}
```

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
domainstack = "1.0"

# Optional: enable regex-based validation (email, url, matches_regex)
domainstack = { version = "1.0", features = ["regex"] }

# Optional: enable derive macro
domainstack = { version = "1.0", features = ["derive"] }
```

## Examples

### Domain Primitive (Age)

```rust
use domainstack::prelude::*;

#[derive(Debug, Clone)]
pub struct Age(u8);

impl Age {
    pub fn new(value: u8) -> Result<Self, ValidationError> {
        let rule = rules::range(0, 120);
        validate("age", &value, &rule)?;
        Ok(Self(value))
    }

    pub fn value(&self) -> u8 {
        self.0
    }
}

// Usage
let age = Age::new(25)?;
```

### Using Derive Macro

```rust
use domainstack::prelude::*;

#[derive(Debug, Validate)]
pub struct CreateUser {
    #[validate(min_len = 3, max_len = 20)]
    pub username: String,

    #[validate(range = "18..=120")]
    pub age: u8,

    #[validate(email, max_len = 255)]
    pub email: String,
}

// Usage
let user = CreateUser {
    username: "alice".to_string(),
    age: 25,
    email: "alice@example.com".to_string(),
};

user.validate()?;
```

### Error Handling

```rust
match user.validate() {
    Ok(_) => println!("Valid!"),
    Err(e) => {
        println!("Validation failed with {} errors:", e.violations.len());
        for v in &e.violations {
            println!("  [{} {}] {}", v.path, v.code, v.message);
        }

        // Field errors map for API responses
        let field_errors = e.field_errors_map();
        // { "username": ["Must be at least 3 characters"],
        //   "age": ["Must be between 18 and 120"] }
    }
}
```

## Built-in Rules

### String Rules (17)

- `email()`† - Email format validation
- `url()`† - URL format validation
- `matches_regex(pattern)`† - Custom regex matching
- `non_empty()` - Non-empty string
- `non_blank()` - Not empty or whitespace-only
- `no_whitespace()` - No whitespace characters
- `ascii()` - ASCII-only characters
- `min_len(min)` - Minimum byte length
- `max_len(max)` - Maximum byte length
- `length(exact)` - Exact byte length
- `len_chars(min, max)` - Character count range
- `alphanumeric()` - Only letters and digits
- `alpha_only()` - Only letters
- `numeric_string()` - Only digits
- `contains(substring)` - Contains substring
- `starts_with(prefix)` - Starts with prefix
- `ends_with(suffix)` - Ends with suffix

### Numeric Rules (8)

- `range(min, max)` - Value range
- `min(min)` - Minimum value
- `max(max)` - Maximum value
- `positive()` - Greater than zero
- `negative()` - Less than zero
- `non_zero()` - Not equal to zero
- `finite()` - Finite floating-point (not NaN/Infinity)
- `multiple_of(n)` - Divisible by n

### Choice/Membership Rules (3)

- `equals(value)` - Equal to value
- `not_equals(value)` - Not equal to value
- `one_of(set)` - Member of set

### Collection Rules (3)

- `min_items(min)` - Minimum items
- `max_items(max)` - Maximum items
- `unique()` - All items unique

†Requires `regex` feature

### Rule Composition

- `rule1.and(rule2)` - Both rules must pass
- `rule1.or(rule2)` - Either rule must pass
- `rule.not(code, msg)` - Negate the rule
- `rule.when(predicate)` - Conditional validation
- `rule.map_path(prefix)` - Prefix error paths
- `rule.code(code)` - Override error code
- `rule.message(msg)` - Override error message
- `rule.meta(key, value)` - Add metadata

## Running Examples

```bash
# Basic examples
cargo run --example age_primitive

# Examples requiring regex feature
cargo run --example email_primitive --features regex
cargo run --example booking_aggregate --features regex

# Package examples
cargo run -p domainstack-examples --example v2_basic
```

## Testing

```bash
cargo test
cargo test --features regex    # Test with regex feature
cargo test --all-features       # Test all features
```

## Documentation

For complete documentation, see:

- [Rules Reference](../../docs/RULES.md) - All 31 validation rules
- [API Guide](../../docs/api-guide.md) - Complete API usage
- [Architecture](../../docs/architecture.md) - System design

## License

Apache 2.0

## Author

Dayna Blackwell <blackwellsystems@protonmail.com>

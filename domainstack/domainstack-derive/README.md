# domainstack-derive

[![Crates.io](https://img.shields.io/crates/v/domainstack-derive.svg)](https://crates.io/crates/domainstack-derive)
[![Documentation](https://docs.rs/domainstack-derive/badge.svg)](https://docs.rs/domainstack-derive)
[![License: Apache 2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://github.com/blackwell-systems/domainstack/blob/main/LICENSE)

Derive macros for the [domainstack](https://crates.io/crates/domainstack) full-stack validation ecosystem.

Provides two derive macros that share the same unified rich syntax:
- `#[derive(Validate)]` - Runtime validation
- `#[derive(ToSchema)]` - OpenAPI schema generation

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
domainstack = { version = "1.0", features = ["derive"] }  # Includes Validate
domainstack-derive = "1.0"   # Adds ToSchema derive
domainstack-schema = "1.0"   # Schema builder utilities
```

### Validation Only

```rust
use domainstack::Validate;

#[derive(Validate)]
struct User {
    #[validate(email)]
    #[validate(max_len = 255)]
    email: String,

    #[validate(range(min = 18, max = 120))]
    age: u8,

    #[validate(min_len = 3)]
    #[validate(max_len = 50)]
    name: String,
}
```

### Validation + Schema Generation (Unified Syntax)

**NEW**: Write validation rules ONCE, get BOTH runtime validation AND OpenAPI schemas:

```rust
use domainstack_derive::{Validate, ToSchema};

#[derive(Validate, ToSchema)]
struct User {
    #[validate(email)]          // [ok] Works for both validation and schema
    #[validate(max_len = 255)]
    #[schema(description = "User's email", example = "alice@example.com")]
    email: String,

    #[validate(range(min = 18, max = 120))]
    #[schema(description = "User's age")]
    age: u8,

    #[validate(min_len = 3)]
    #[validate(max_len = 50)]
    name: String,
}

// Runtime validation works
user.validate()?;

// Schema generation works
let schema = User::schema();
// → email: { type: "string", format: "email", maxLength: 255, ... }
// → age: { type: "integer", minimum: 18, maximum: 120 }
```

## Available Attributes

### Validation Rules (Unified Rich Syntax)

Both `Validate` and `ToSchema` support these validation rules:

**String Rules:**
- `#[validate(email)]` - Email format
- `#[validate(url)]` - URL format
- `#[validate(min_len = n)]` - Minimum length
- `#[validate(max_len = n)]` - Maximum length
- `#[validate(alphanumeric)]` - Alphanumeric only
- `#[validate(ascii)]` - ASCII only
- `#[validate(alpha_only)]` - Letters only
- `#[validate(numeric_string)]` - Digits only
- `#[validate(non_empty)]` - Not empty
- `#[validate(non_blank)]` - Not blank (no whitespace)
- `#[validate(matches_regex = "pattern")]` - Custom regex
- Plus: `contains`, `starts_with`, `ends_with`, `no_whitespace`

**Numeric Rules:**
- `#[validate(range(min = a, max = b))]` - Range validation
- `#[validate(positive)]` - Positive numbers
- `#[validate(negative)]` - Negative numbers
- `#[validate(non_zero)]` - Not zero
- `#[validate(multiple_of = n)]` - Multiple of n
- Plus: `min`, `max`, `finite`, `equals`, `not_equals`

**Collection Rules:**
- `#[validate(min_items = n)]` - Minimum items
- `#[validate(max_items = n)]` - Maximum items
- `#[validate(unique)]` - All items unique

**Composite Rules:**
- `#[validate(nested)]` - Validate nested struct
- `#[validate(each(nested))]` - Validate each item in collection (for nested types)
- `#[validate(each(rule))]` - Validate each item with any rule (for primitives)
- `#[validate(custom = "function")]` - Custom validation function

**Collection Item Validation:**

The `each(rule)` syntax allows validating each item in a collection:

```rust
#[derive(Validate)]
struct BlogPost {
    // Validate each email in the list
    #[validate(each(email))]
    author_emails: Vec<String>,

    // Validate each tag length
    #[validate(each(length(min = 1, max = 50)))]
    tags: Vec<String>,

    // Validate each URL
    #[validate(each(url))]
    related_links: Vec<String>,

    // Validate each item is alphanumeric
    #[validate(each(alphanumeric))]
    keywords: Vec<String>,
}
```

Error paths include array indices: `tags[0]`, `author_emails[1]`, etc.

**Legacy Syntax (Still Supported):**
- `#[validate(length(min = a, max = b))]` - String length (prefer `min_len`/`max_len`)

### Schema Hints

For `ToSchema`, add documentation metadata:

```rust
#[schema(description = "Field description")]
#[schema(example = "example value")]
#[schema(deprecated = true)]
#[schema(read_only = true)]
#[schema(write_only = true)]
```

### Struct-Level Validation

```rust
#[derive(Validate)]
#[validate(
    check = "self.password == self.password_confirmation",
    code = "passwords_mismatch",
    message = "Passwords must match"
)]
struct RegisterForm {
    #[validate(min_len = 8)]
    password: String,
    password_confirmation: String,
}
```

## Documentation

This is a proc macro implementation crate. For complete documentation, examples, and usage guides, see:

- [domainstack documentation](https://docs.rs/domainstack)
- [GitHub repository](https://github.com/blackwell-systems/domainstack)

## License

Apache 2.0

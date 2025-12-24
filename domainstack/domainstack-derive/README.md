# domainstack-derive

Derive macro for the [domainstack](https://crates.io/crates/domainstack) validation framework.

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
domainstack = { version = "1.0", features = ["derive"] }
```

Use the derive macro:

```rust
use domainstack::Validate;

#[derive(Validate)]
struct User {
    #[validate(email)]
    email: String,

    #[validate(range(min = 18, max = 120))]
    age: u8,

    #[validate(length(min = 3, max = 50))]
    name: String,
}
```

## Available Attributes

**Field-level validation:**
- `#[validate(rule)]` - Apply validation rule (e.g., `email`, `range`, `length`)
- `#[validate(nested)]` - Validate nested struct
- `#[validate(custom = "function")]` - Custom validation function
- `#[validate(each(rule))]` - Validate each item in a collection

**Struct-level validation:**
- `#[validate(check = "expression", code = "...", message = "...")]` - Cross-field validation

**Rule customization:**
```rust
#[validate(
    length(min = 8),
    code = "weak_password",
    message = "Password must be at least 8 characters"
)]
password: String,
```

## Documentation

This is a proc macro implementation crate. For complete documentation, examples, and usage guides, see:

- [domainstack documentation](https://docs.rs/domainstack)
- [GitHub repository](https://github.com/blackwell-systems/domainstack)

## License

Apache 2.0

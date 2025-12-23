# Rules Reference

Complete reference for all validation rules in domainstack.

## String Rules

### `email()`

Validates email format using regex.

```rust
use domainstack::rules::email;

let rule = email();
validate("email", "user@example.com", &rule)?;  // ✓
validate("email", "invalid", &rule)?;            // ✗ invalid_email
```

**Error code**: `invalid_email`  
**Message**: "Invalid email format"  
**Feature**: Requires `email` feature flag (adds regex dependency)

### `non_empty()`

Ensures string is not empty.

```rust
use domainstack::rules::non_empty;

let rule = non_empty();
validate("name", "Alice", &rule)?;     // ✓
validate("name", "", &rule)?;          // ✗ empty_string
```

**Error code**: `empty_string`  
**Message**: "Must not be empty"

### `min_len(min: usize)`

Validates minimum string length.

```rust
use domainstack::rules::min_len;

let rule = min_len(3);
validate("name", "Bob", &rule)?;       // ✓
validate("name", "Al", &rule)?;        // ✗ min_length
```

**Error code**: `min_length`  
**Message**: "Must be at least {min} characters"  
**Meta**: `min: {value}`

### `max_len(max: usize)`

Validates maximum string length.

```rust
use domainstack::rules::max_len;

let rule = max_len(50);
validate("name", "Alice", &rule)?;          // ✓
validate("name", &"x".repeat(100), &rule)?; // ✗ max_length
```

**Error code**: `max_length`  
**Message**: "Must be at most {max} characters"  
**Meta**: `max: {value}`

### `length(min: usize, max: usize)`

Validates string length range.

```rust
use domainstack::rules::length;

let rule = length(3, 50);
validate("name", "Alice", &rule)?;     // ✓
validate("name", "Al", &rule)?;        // ✗ min_length
validate("name", &"x".repeat(100), &rule)?; // ✗ max_length
```

**Error codes**: `min_length` or `max_length`  
**Meta**: `min: {value}`, `max: {value}`

## Numeric Rules

### `range<T: PartialOrd>(min: T, max: T)`

Validates numeric value is within range.

```rust
use domainstack::rules::range;

let rule = range(18, 120);
validate("age", &30, &rule)?;          // ✓
validate("age", &15, &rule)?;          // ✗ out_of_range
validate("age", &150, &rule)?;         // ✗ out_of_range
```

**Error code**: `out_of_range`  
**Message**: "Must be between {min} and {max}"  
**Meta**: `min: {value}`, `max: {value}`

**Works with**: `u8`, `u16`, `u32`, `u64`, `i8`, `i16`, `i32`, `i64`, `f32`, `f64`

### `min<T: PartialOrd>(min: T)`

Validates minimum value.

```rust
use domainstack::rules::min;

let rule = min(0);
validate("balance", &100, &rule)?;     // ✓
validate("balance", &-50, &rule)?;     // ✗ below_minimum
```

**Error code**: `below_minimum`  
**Message**: "Must be at least {min}"  
**Meta**: `min: {value}`

### `max<T: PartialOrd>(max: T)`

Validates maximum value.

```rust
use domainstack::rules::max;

let rule = max(100);
validate("count", &50, &rule)?;        // ✓
validate("count", &150, &rule)?;       // ✗ above_maximum
```

**Error code**: `above_maximum`  
**Message**: "Must be at most {max}"  
**Meta**: `max: {value}`

## Rule Composition

### `and()`

Both rules must pass.

```rust
use domainstack::rules::*;

let rule = min_len(5).and(max_len(255));
validate("email", "user@example.com", &rule)?;  // ✓
validate("email", "abc", &rule)?;                // ✗ min_length
```

### `or()`

At least one rule must pass.

```rust
use domainstack::rules::*;

let rule = email().or(non_empty());
validate("contact", "user@example.com", &rule)?;  // ✓
validate("contact", "some text", &rule)?;         // ✓
validate("contact", "", &rule)?;                  // ✗ (both fail)
```

### `not(code: &'static str, message: impl Into<String>)`

Inverts the rule result.

```rust
use domainstack::rules::*;

let rule = non_empty().not("must_be_empty", "Must be empty");
validate("field", "", &rule)?;         // ✓
validate("field", "text", &rule)?;     // ✗ must_be_empty
```

### `when(predicate: impl Fn(&T) -> bool)`

Conditionally applies the rule.

```rust
use domainstack::rules::*;

let rule = min_len(10).when(|s: &str| s.starts_with("special_"));
validate("code", "short", &rule)?;             // ✓ (not special_)
validate("code", "special_ok", &rule)?;        // ✗ min_length
validate("code", "special_long_enough", &rule)?; // ✓
```

### `map_path(prefix: impl Into<Path>)`

Prefixes error paths.

```rust
use domainstack::prelude::*;

let rule = email().map_path("contact");
// Errors will have path "contact" instead of root
```

## Derive Macro Attributes

### `#[validate(length(min = N, max = M))]`

String length validation.

```rust
#[derive(Validate)]
struct User {
    #[validate(length(min = 1, max = 50))]
    name: String,
}
```

**Parameters**:
- `min` (optional): Minimum length
- `max` (optional): Maximum length

### `#[validate(range(min = N, max = M))]`

Numeric range validation.

```rust
#[derive(Validate)]
struct Adult {
    #[validate(range(min = 18, max = 120))]
    age: u8,
}
```

**Parameters**:
- `min` (optional): Minimum value
- `max` (optional): Maximum value

### `#[validate(nested)]`

Validates nested struct implementing `Validate`.

```rust
#[derive(Validate)]
struct User {
    #[validate(nested)]
    email: Email,
}
```

**Requirements**: Field type must implement `Validate`

### `#[validate(each(...))]`

Validates each element in a Vec.

```rust
// With nested validation
#[derive(Validate)]
struct Team {
    #[validate(each(nested))]
    members: Vec<User>,
}

// With primitive validation
#[derive(Validate)]
struct Tags {
    #[validate(each(length(min = 3, max = 20)))]
    tags: Vec<String>,
}
```

**Supported inner validations**:
- `each(nested)` - Validates nested structs
- `each(length(min = N, max = M))` - String length
- `each(range(min = N, max = M))` - Numeric range

### `#[validate(custom = "function_name")]`

Custom validation function.

```rust
fn validate_even(value: &u8) -> Result<(), ValidationError> {
    if *value % 2 == 0 {
        Ok(())
    } else {
        Err(ValidationError::single(
            Path::root(),
            "not_even",
            "Must be even"
        ))
    }
}

#[derive(Validate)]
struct EvenNumber {
    #[validate(custom = "validate_even")]
    value: u8,
}
```

**Function signature**: `fn(&T) -> Result<(), ValidationError>`

## Custom Rules

Create your own validation rules:

```rust
use domainstack::prelude::*;

pub fn domain_specific_rule() -> Rule<String> {
    Rule::new(|value: &String| {
        if value.contains("@") && value.len() > 5 {
            Ok(())
        } else {
            Err(Violation {
                path: Path::root(),
                code: "invalid_format",
                message: "Invalid format".to_string(),
                meta: Meta::default(),
            })
        }
    })
}

// Usage
let rule = domain_specific_rule();
validate("field", &value, &rule)?;
```

## Error Codes Reference

| Code | Rule | Description |
|------|------|-------------|
| `invalid_email` | `email()` | Invalid email format |
| `empty_string` | `non_empty()` | String is empty |
| `min_length` | `min_len()`, `length()` | String too short |
| `max_length` | `max_len()`, `length()` | String too long |
| `out_of_range` | `range()` | Number outside range |
| `below_minimum` | `min()` | Number below minimum |
| `above_maximum` | `max()` | Number above maximum |

Custom rules can define their own error codes.

## Best Practices

1. **Compose rules** - Use `and()` and `or()` to build complex validations
2. **Consistent codes** - Use standard error codes for common validations
3. **Meaningful meta** - Include min/max/pattern in meta for client use
4. **Reusable rules** - Extract common validation patterns into functions
5. **Type-appropriate** - Use string rules for strings, numeric for numbers
6. **Feature flags** - Use `email` feature only if needed (adds regex dep)

## See Also

- [API Guide](./api-guide.md) - Complete API documentation
- [Examples](../domainstack/examples/) - Runnable code examples
- [Source Code](../domainstack/domainstack/src/rules/) - Rule implementations

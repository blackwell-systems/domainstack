# Validation Rules Reference (v0.4.0)

**Complete reference for all 18 built-in validation rules in domainstack.**

---

## Quick Reference

| Category | Count | Rules |
|----------|-------|-------|
| **String** | 12 | `email`, `non_empty`, `min_len`, `max_len`, `length`, `url`*, `alphanumeric`*, `alpha_only`*, `numeric_string`*, `contains`*, `starts_with`*, `ends_with`*, `matches_regex`** |
| **Numeric** | 6 | `range`, `min`, `max`, `positive`*, `negative`*, `multiple_of`* |
| **Total** | **18** | *New in v0.4 | **Requires `regex` feature |

---

## String Rules (12 rules)

### Core String Rules

#### `email()`
Validates email format.

```rust
use domainstack::prelude::*;

let rule = rules::email();
assert!(rule.apply("user@example.com").is_empty());
assert!(!rule.apply("invalid").is_empty());
```

- **Error Code:** `invalid_email`
- **Message:** `"Invalid email format"`
- **Feature:** Requires `email` feature (adds regex dependency)

---

#### `non_empty()`
Validates that a string is not empty.

```rust
let rule = rules::non_empty();
assert!(rule.apply("hello").is_empty());
assert!(!rule.apply("").is_empty());
```

- **Error Code:** `non_empty`
- **Message:** `"Must not be empty"`

---

#### `min_len(min: usize)`
Validates minimum string length.

```rust
let rule = rules::min_len(5);
assert!(rule.apply("hello").is_empty());
assert!(!rule.apply("hi").is_empty());
```

- **Error Code:** `min_length`
- **Message:** `"Must be at least {min} characters"`
- **Meta:** `{"min": "5"}`

---

#### `max_len(max: usize)`
Validates maximum string length.

```rust
let rule = rules::max_len(10);
assert!(rule.apply("hello").is_empty());
assert!(!rule.apply("hello world!").is_empty());
```

- **Error Code:** `max_length`
- **Message:** `"Must be at most {max} characters"`
- **Meta:** `{"max": "10"}`

---

#### `length(min: usize, max: usize)`
Validates string length within range (combines min_len and max_len).

```rust
let rule = rules::length(3, 10);
assert!(rule.apply("hello").is_empty());
assert!(!rule.apply("hi").is_empty());           // too short
assert!(!rule.apply("hello world!").is_empty()); // too long
```

- **Error Codes:** `min_length` or `max_length`
- **Meta:** `{"min": "3", "max": "10"}`

---

### Pattern Validation (NEW in v0.4)

#### `url()` 🆕
Validates HTTP/HTTPS URLs.

```rust
let rule = rules::url();
assert!(rule.apply("https://example.com").is_empty());
assert!(rule.apply("http://example.com/path").is_empty());
assert!(!rule.apply("example.com").is_empty());  // missing scheme
```

- **Error Code:** `invalid_url`
- **Message:** `"Invalid URL format"`
- **Note:** With `regex` feature: RFC-compliant validation. Without: basic validation.

---

#### `alphanumeric()` 🆕
Validates that a string contains only letters and numbers.

```rust
let rule = rules::alphanumeric();
assert!(rule.apply("User123").is_empty());
assert!(!rule.apply("user-name").is_empty());  // hyphen not allowed
```

- **Error Code:** `not_alphanumeric`
- **Message:** `"Must contain only letters and numbers"`
- **Use Cases:** Usernames, IDs, codes

---

#### `alpha_only()` 🆕
Validates that a string contains only alphabetic characters.

```rust
let rule = rules::alpha_only();
assert!(rule.apply("Hello").is_empty());
assert!(!rule.apply("Hello123").is_empty());  // numbers not allowed
```

- **Error Code:** `not_alpha`
- **Message:** `"Must contain only letters"`
- **Use Cases:** Name fields, text-only inputs

---

#### `numeric_string()` 🆕
Validates that a string contains only numeric characters.

```rust
let rule = rules::numeric_string();
assert!(rule.apply("123456").is_empty());
assert!(!rule.apply("12.34").is_empty());  // decimal not allowed
```

- **Error Code:** `not_numeric`
- **Message:** `"Must contain only numbers"`
- **Use Cases:** Numeric codes, PIN numbers

---

### Substring Matching (NEW in v0.4)

#### `contains(substring: &'static str)` 🆕
Validates that a string contains the specified substring.

```rust
let rule = rules::contains("@example.com");
assert!(rule.apply("user@example.com").is_empty());
assert!(!rule.apply("user@other.com").is_empty());
```

- **Error Code:** `missing_substring`
- **Message:** `"Must contain '{substring}'"`
- **Meta:** `{"substring": "example.com"}`
- **Use Cases:** Domain validation, required keywords

---

#### `starts_with(prefix: &'static str)` 🆕
Validates that a string starts with the specified prefix.

```rust
let rule = rules::starts_with("https://");
assert!(rule.apply("https://example.com").is_empty());
assert!(!rule.apply("http://example.com").is_empty());
```

- **Error Code:** `invalid_prefix`
- **Message:** `"Must start with '{prefix}'"`
- **Meta:** `{"prefix": "https://"}`
- **Use Cases:** URL scheme validation, prefixes

---

#### `ends_with(suffix: &'static str)` 🆕
Validates that a string ends with the specified suffix.

```rust
let rule = rules::ends_with(".com");
assert!(rule.apply("example.com").is_empty());
assert!(!rule.apply("example.org").is_empty());
```

- **Error Code:** `invalid_suffix`
- **Message:** `"Must end with '{suffix}'"`
- **Meta:** `{"suffix": ".com"}`
- **Use Cases:** File extensions, domain validation

---

#### `matches_regex(pattern: &'static str)` 🆕 (requires `regex` feature)
Validates that a string matches the specified regex pattern.

```rust
#[cfg(feature = "regex")]
{
    let rule = rules::matches_regex(r"^\d{3}-\d{4}$");  // Phone: 123-4567
    assert!(rule.apply("123-4567").is_empty());
    assert!(!rule.apply("1234567").is_empty());
}
```

- **Error Code:** `pattern_mismatch`
- **Message:** `"Does not match required pattern"`
- **Meta:** `{"pattern": "regex"}`
- **Feature:** Requires `regex` feature
- **Use Cases:** Custom patterns, complex validation

---

## Numeric Rules (6 rules)

### Range Validation

#### `range<T>(min: T, max: T)`
Validates that a numeric value is within the specified range (inclusive).

```rust
let rule = rules::range(18, 120);
assert!(rule.apply(&25).is_empty());
assert!(rule.apply(&18).is_empty());   // min boundary
assert!(rule.apply(&120).is_empty());  // max boundary
assert!(!rule.apply(&17).is_empty());  // below min
```

- **Error Code:** `out_of_range`
- **Message:** `"Must be between {min} and {max}"`
- **Meta:** `{"min": "18", "max": "120"}`
- **Types:** Works with any `PartialOrd` type (i8, u8, i32, u32, i64, f32, f64, etc.)

---

#### `min<T>(min: T)`
Validates that a numeric value is at least the minimum.

```rust
let rule = rules::min(18);
assert!(rule.apply(&18).is_empty());
assert!(rule.apply(&100).is_empty());
assert!(!rule.apply(&17).is_empty());
```

- **Error Code:** `below_minimum`
- **Message:** `"Must be at least {min}"`
- **Meta:** `{"min": "18"}`

---

#### `max<T>(max: T)`
Validates that a numeric value does not exceed the maximum.

```rust
let rule = rules::max(100);
assert!(rule.apply(&100).is_empty());
assert!(rule.apply(&50).is_empty());
assert!(!rule.apply(&101).is_empty());
```

- **Error Code:** `above_maximum`
- **Message:** `"Must be at most {max}"`
- **Meta:** `{"max": "100"}`

---

### Sign Validation (NEW in v0.4)

#### `positive<T>()` 🆕
Validates that a numeric value is positive (greater than zero).

```rust
let rule = rules::positive();
assert!(rule.apply(&1).is_empty());
assert!(rule.apply(&100).is_empty());
assert!(!rule.apply(&0).is_empty());
assert!(!rule.apply(&-1).is_empty());
```

- **Error Code:** `not_positive`
- **Message:** `"Must be positive (greater than zero)"`
- **Use Cases:** Amounts, quantities, counts

---

#### `negative<T>()` 🆕
Validates that a numeric value is negative (less than zero).

```rust
let rule = rules::negative();
assert!(rule.apply(&-1).is_empty());
assert!(rule.apply(&-100).is_empty());
assert!(!rule.apply(&0).is_empty());
assert!(!rule.apply(&1).is_empty());
```

- **Error Code:** `not_negative`
- **Message:** `"Must be negative (less than zero)"`
- **Use Cases:** Debt, loss, temperature below zero

---

### Divisibility (NEW in v0.4)

#### `multiple_of<T>(divisor: T)` 🆕
Validates that a numeric value is a multiple of the specified number.

```rust
let rule = rules::multiple_of(5);
assert!(rule.apply(&10).is_empty());
assert!(rule.apply(&15).is_empty());
assert!(rule.apply(&0).is_empty());
assert!(!rule.apply(&7).is_empty());
```

- **Error Code:** `not_multiple`
- **Message:** `"Must be a multiple of {divisor}"`
- **Meta:** `{"divisor": "5"}`
- **Use Cases:** Quantity validation (e.g., "packs of 6"), step values

---

## Rule Composition

All rules can be composed using `and()`, `or()`, `not()`, and `when()`:

### Combining Rules with `and()`

```rust
// Email must be 5-255 characters
let rule = rules::min_len(5)
    .and(rules::max_len(255))
    .and(rules::email());

assert!(rule.apply("user@example.com").is_empty());
assert!(!rule.apply("a@b").is_empty());  // too short
```

### Alternative Rules with `or()`

```rust
// Must be alphanumeric OR contain a hyphen
let rule = rules::alphanumeric()
    .or(rules::contains("-"));

assert!(rule.apply("User123").is_empty());
assert!(rule.apply("user-name").is_empty());
```

### Negating Rules with `not()`

```rust
let rule = rules::contains("@").not("no_at_sign", "Must not contain @");
assert!(rule.apply("username").is_empty());
assert!(!rule.apply("user@example.com").is_empty());
```

### Conditional Rules with `when()`

```rust
let is_premium = || true;
let rule = rules::max_len(100).when(is_premium);

// Only validates if is_premium() returns true
```

---

## Migration Guide

### From v0.3 to v0.4

All existing rules remain unchanged. New rules added:

**String Rules:**
- `url()` - URL validation
- `alphanumeric()` - Letters and numbers only
- `alpha_only()` - Letters only
- `numeric_string()` - Numbers only
- `contains(substring)` - Substring matching
- `starts_with(prefix)` - Prefix validation
- `ends_with(suffix)` - Suffix validation
- `matches_regex(pattern)` - Custom regex patterns (feature-gated)

**Numeric Rules:**
- `positive()` - Greater than zero
- `negative()` - Less than zero
- `multiple_of(divisor)` - Divisibility check

**Breaking Changes:** None

---

## Feature Flags

```toml
[dependencies]
domainstack = { version = "0.4", features = ["email", "regex"] }
```

- `email` - Enables `email()` rule with regex validation
- `regex` - Enables `url()` (improved), `matches_regex()`

Without features, core has **zero dependencies**.

---

## Custom Rules

Create your own rules using `Rule::new()`:

```rust
use domainstack::{Rule, ValidationError, Path};

fn lowercase_only() -> Rule<str> {
    Rule::new(|value: &str| {
        if value.chars().all(|c| c.is_lowercase() || !c.is_alphabetic()) {
            ValidationError::default()
        } else {
            ValidationError::single(
                Path::root(),
                "not_lowercase",
                "Must contain only lowercase letters"
            )
        }
    })
}

let rule = lowercase_only();
assert!(rule.apply("hello").is_empty());
assert!(!rule.apply("Hello").is_empty());
```

---

## See Also

- [API Guide](./api-guide.md) - Complete usage documentation
- [Architecture](./architecture.md) - System design
- [Rule System Analysis](./RULE_SYSTEM_ANALYSIS.md) - Expansion strategy

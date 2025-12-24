# Validation Rules Reference

**Complete reference for all 31 built-in validation rules in domainstack.**

---

## Quick Reference

| Category | Count | Rules |
|----------|-------|-------|
| **String** | 17 | `email`†, `non_empty`, `min_len`, `max_len`, `length`, `url`†, `alphanumeric`, `alpha_only`, `numeric_string`, `contains`, `starts_with`, `ends_with`, `matches_regex`†, `non_blank`‡, `no_whitespace`‡, `ascii`‡, `len_chars`‡ |
| **Numeric** | 8 | `range`, `min`, `max`, `positive`, `negative`, `multiple_of`, `finite`‡, `non_zero`‡ |
| **Choice** | 3 | `equals`‡, `not_equals`‡, `one_of`‡ |
| **Collection** | 3 | `min_items`‡, `max_items`‡, `unique`‡ |
| **Total** | **31** | †Requires `regex` feature |

---

## Collection Item Validation with `each(rule)`

**All validation rules can be used with `each()` to validate items in collections (Vec<T>):**

```rust
use domainstack_derive::Validate;

#[derive(Validate)]
struct BlogPost {
    // String rules with each()
    #[validate(each(email))]
    author_emails: Vec<String>,

    #[validate(each(url))]
    related_links: Vec<String>,

    #[validate(each(alphanumeric))]
    keywords: Vec<String>,

    #[validate(each(length(min = 1, max = 50)))]
    tags: Vec<String>,

    // Numeric rules with each()
    #[validate(each(range(min = 1, max = 5)))]
    ratings: Vec<u8>,

    #[validate(each(positive))]
    amounts: Vec<i32>,

    // Nested types with each()
    #[validate(each(nested))]
    comments: Vec<Comment>,
}
```

**Error paths include array indices:**
- `author_emails[0]` - "Invalid email format"
- `tags[2]` - "Must be at most 50 characters"
- `ratings[1]` - "Must be between 1 and 5"

**Supported with `each()`:** All string rules, all numeric rules, all choice rules, and `nested` for complex types.

---

## String Rules (17 rules)

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

### Pattern Validation

#### `url()`
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

#### `alphanumeric()`
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

#### `alpha_only()`
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

#### `numeric_string()`
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

### Substring Matching

#### `contains(substring: &'static str)`
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

#### `starts_with(prefix: &'static str)`
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

#### `ends_with(suffix: &'static str)`
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

#### `matches_regex(pattern: &'static str)` (requires `regex` feature)
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

### String Semantics

#### `non_blank()`
Validates that a string is not empty after trimming whitespace.

```rust
let rule = rules::non_blank();
assert!(rule.apply("  hello  ").is_empty());  // has content
assert!(!rule.apply("   ").is_empty());       // only whitespace
assert!(!rule.apply("").is_empty());           // empty
```

- **Error Code:** `blank_string`
- **Message:** `"Must not be blank (whitespace only)"`
- **Use Cases:** Catching inputs like "   " that pass non_empty() but have no actual content

---

#### `no_whitespace()`
Validates that a string contains no whitespace characters.

```rust
let rule = rules::no_whitespace();
assert!(rule.apply("username").is_empty());
assert!(rule.apply("user_name").is_empty());
assert!(!rule.apply("user name").is_empty());  // contains space
```

- **Error Code:** `contains_whitespace`
- **Message:** `"Must not contain whitespace"`
- **Use Cases:** Usernames, slugs, identifiers

---

#### `ascii()`
Validates that all characters are ASCII (0-127).

```rust
let rule = rules::ascii();
assert!(rule.apply("Hello123").is_empty());
assert!(!rule.apply("Héllo").is_empty());     // contains é
assert!(!rule.apply("Hello🚀").is_empty());   // contains emoji
```

- **Error Code:** `non_ascii`
- **Message:** `"Must contain only ASCII characters"`
- **Use Cases:** Legacy systems, ASCII-only fields

---

#### `len_chars(min: usize, max: usize)`
Validates character count (not byte count) - handles Unicode correctly.

```rust
let rule = rules::len_chars(3, 10);
assert!(rule.apply("🚀🚀🚀").is_empty());    // 3 chars, not 12 bytes
assert!(rule.apply("hello").is_empty());
assert!(!rule.apply("hi").is_empty());        // too few
```

- **Error Codes:** `min_chars` or `max_chars`
- **Message:** `"Must be at least {min} characters"` or `"Must be at most {max} characters"`
- **Meta:** `{"min": "3", "max": "10", "actual": "2"}`
- **Use Cases:** Unicode text validation where byte length differs from character count

---

## Numeric Rules (8 rules)

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

### Sign Validation

#### `positive<T>()`
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

#### `negative<T>()`
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

### Divisibility

#### `multiple_of<T>(divisor: T)`
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

### Special Numeric Validation

#### `finite<T>()`
Validates that a floating-point value is finite (not NaN or infinity).

```rust
let rule = rules::finite();
assert!(rule.apply(&42.0_f64).is_empty());
assert!(rule.apply(&0.0).is_empty());
assert!(!rule.apply(&f64::NAN).is_empty());
assert!(!rule.apply(&f64::INFINITY).is_empty());
```

- **Error Code:** `not_finite`
- **Message:** `"Must be a finite number (not NaN or infinity)"`
- **Use Cases:** Catching NaN/infinity before range checks (NaN slips through PartialOrd comparisons)
- **Important:** Always combine with `range()` for float validation

---

#### `non_zero<T>()`
Validates that a value is not zero (uses `!= T::default()`).

```rust
let rule = rules::non_zero();
assert!(rule.apply(&42).is_empty());
assert!(rule.apply(&-5).is_empty());
assert!(!rule.apply(&0).is_empty());
```

- **Error Code:** `zero_value`
- **Message:** `"Must be non-zero"`
- **Use Cases:** Division inputs, non-zero quantities

---

## Choice/Membership Rules (3 rules)

### `equals<T>(expected: T)`
Validates that a value equals the specified value.

```rust
let rule = rules::equals("active");
assert!(rule.apply(&"active").is_empty());
assert!(!rule.apply(&"inactive").is_empty());

// Works with numbers too
let rule = rules::equals(42);
assert!(rule.apply(&42).is_empty());
```

- **Error Code:** `not_equal`
- **Message:** `"Must equal '{expected}'"`
- **Meta:** `{"expected": "value"}`
- **Use Cases:** Fixed value validation, status checks

---

### `not_equals<T>(forbidden: T)`
Validates that a value does NOT equal the specified value.

```rust
let rule = rules::not_equals("banned");
assert!(rule.apply(&"active").is_empty());
assert!(!rule.apply(&"banned").is_empty());
```

- **Error Code:** `forbidden_value`
- **Message:** `"Must not equal '{forbidden}'"`
- **Meta:** `{"forbidden": "value"}`
- **Use Cases:** Blacklist validation, reserved values

---

### `one_of<T>(allowed: &[T])`
Validates that a value is in the allowed set.

```rust
let rule = rules::one_of(&["active", "pending", "inactive"]);
assert!(rule.apply(&"active").is_empty());
assert!(rule.apply(&"pending").is_empty());
assert!(!rule.apply(&"banned").is_empty());
```

- **Error Code:** `not_in_set`
- **Message:** `"Must be one of: {allowed}"`
- **Meta:** `{"allowed": "[value1, value2, ...]"}`
- **Use Cases:** Enum validation, status codes, role checks

---

## Collection Rules (3 rules)

### `min_items<T>(min: usize)`
Validates that a collection has at least the minimum number of items.

```rust
let rule = rules::min_items(2);
assert!(rule.apply(&[1, 2, 3]).is_empty());
assert!(rule.apply(&[1, 2]).is_empty());     // exactly min
assert!(!rule.apply(&[1]).is_empty());        // too few
```

- **Error Code:** `too_few_items`
- **Message:** `"Must have at least {min} items"`
- **Meta:** `{"min": "2", "actual": "1"}`
- **Use Cases:** Required list items, minimum selections

---

### `max_items<T>(max: usize)`
Validates that a collection has at most the maximum number of items.

```rust
let rule = rules::max_items(3);
assert!(rule.apply(&[1, 2]).is_empty());
assert!(rule.apply(&[1, 2, 3]).is_empty());  // exactly max
assert!(!rule.apply(&[1, 2, 3, 4]).is_empty()); // too many
```

- **Error Code:** `too_many_items`
- **Message:** `"Must have at most {max} items"`
- **Meta:** `{"max": "3", "actual": "4"}`
- **Use Cases:** Limit selections, capacity constraints

---

### `unique<T>()`
Validates that all items in a collection are unique (no duplicates).

```rust
let rule = rules::unique();
assert!(rule.apply(&[1, 2, 3]).is_empty());
assert!(!rule.apply(&[1, 2, 2, 3]).is_empty()); // duplicate 2
```

- **Error Code:** `duplicate_items`
- **Message:** `"All items must be unique (found {count} duplicates)"`
- **Meta:** `{"duplicates": "1"}`
- **Use Cases:** Unique tags, no duplicate selections
- **Performance:** Uses HashSet for O(n) duplicate detection

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

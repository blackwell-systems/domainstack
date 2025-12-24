# Rule System Analysis & Expansion Plan

## Current Rule Coverage

### String Rules (5 rules)
- ✅ `email()` - Email format validation (RFC-compliant with regex feature)
- ✅ `non_empty()` - Not empty string
- ✅ `min_len(min)` - Minimum character length
- ✅ `max_len(max)` - Maximum character length
- ✅ `length(min, max)` - Combined min/max length

### Numeric Rules (3 rules)
- ✅ `range(min, max)` - Numeric range (works with any PartialOrd type)
- ✅ `min(min)` - Minimum value
- ✅ `max(max)` - Maximum value

**Total: 8 built-in rules**

---

## Gap Analysis: What's Missing

### High Priority (Common Use Cases)

#### String Pattern Validation
| Rule | Use Case | Complexity |
|------|----------|------------|
| `url()` | Website URLs | Medium (regex) |
| `alphanumeric()` | Usernames, IDs | Low |
| `alpha_only()` | Name fields | Low |
| `numeric_string()` | Numeric codes | Low |
| `contains(substring)` | Search validation | Low |
| `starts_with(prefix)` | Prefix validation | Low |
| `ends_with(suffix)` | File extensions | Low |
| `matches_regex(pattern)` | Custom patterns | Medium |

#### String Normalization
| Rule | Use Case | Complexity |
|------|----------|------------|
| `trim()` | Remove whitespace | Low |
| `lowercase()` | Case normalization | Low |
| `uppercase()` | Case normalization | Low |

#### Numeric Validation
| Rule | Use Case | Complexity |
|------|----------|------------|
| `positive()` | Amount fields | Low |
| `negative()` | Debt/loss fields | Low |
| `non_zero()` | Division inputs | Low |
| `multiple_of(n)` | Quantity validation | Low |
| `even()` | Pairing validation | Low |
| `odd()` | Odd requirements | Low |

#### Collection Validation
| Rule | Use Case | Complexity |
|------|----------|------------|
| `collection_len(min, max)` | Vec/HashMap size | Low |
| `unique_items()` | Duplicate prevention | Medium |
| `contains_item(val)` | Presence check | Low |

### Medium Priority (Nice to Have)

#### Advanced String Patterns
- `uuid()` - UUID validation
- `phone(region)` - Phone number validation (complex, needs external crate)
- `credit_card()` - Credit card validation (Luhn algorithm)
- `ip_address()` / `ipv4()` / `ipv6()` - IP validation
- `slug()` - URL-safe slugs
- `hex_color()` - Color codes (#RGB, #RRGGBB)

#### Date/Time
- `date_format(fmt)` - Date string validation
- `iso8601()` - ISO date validation
- `before_date(date)` / `after_date(date)` - Date comparison

### Low Priority (Specialized)

- `json_valid()` - Valid JSON string
- `base64()` - Base64 validation
- `semver()` - Semantic version
- `jwt()` - JWT token format
- `file_extension(exts)` - File upload validation

---

## Recommendations

### ✅ Should Implement (High Value, Low Complexity)

**String Rules:**
1. `url()` - Very common, regex-based
2. `alphanumeric()` - Common for usernames
3. `contains(substring)` - Useful for search
4. `matches_regex(pattern)` - Power user feature
5. `trim()` - Whitespace handling

**Numeric Rules:**
1. `positive()` - Common constraint
2. `negative()` - Common constraint
3. `multiple_of(n)` - Quantity validation

**Collection Rules:**
1. `collection_len(min, max)` - Vec validation
2. `unique_items()` - Duplicate prevention

**Total new rules: 10** (brings us to 18 total rules)

### ⚠️ Maybe Implement (Medium Complexity)

- `uuid()` - Could use external crate or simple regex
- `ip_address()` - Standard library has parsing
- `credit_card()` - Luhn algorithm is straightforward
- `slug()` - Regex-based

### ❌ Should NOT Implement (Too Specialized)

- `phone(region)` - Too complex, region-specific
- `date_format(fmt)` - Use chrono crate directly
- `json_valid()` - Use serde_json directly
- `jwt()` - Use jwt crate

---

## Implementation Strategy

### Phase 1: Core String Rules (Immediate)
```rust
// rules/string.rs additions
pub fn url() -> Rule<str>
pub fn alphanumeric() -> Rule<str>
pub fn contains(substring: &'static str) -> Rule<str>
pub fn matches_regex(pattern: &'static str) -> Rule<str>  // feature-gated
pub fn trim() -> Rule<String>  // Note: Rule<String>, not Rule<str>
```

### Phase 2: Core Numeric Rules (Immediate)
```rust
// rules/numeric.rs additions
pub fn positive<T>() -> Rule<T>
pub fn negative<T>() -> Rule<T>
pub fn multiple_of<T>(n: T) -> Rule<T>
```

### Phase 3: Collection Rules (New Module)
```rust
// rules/collection.rs (new file)
pub fn collection_len<T>(min: usize, max: usize) -> Rule<Vec<T>>
pub fn unique_items<T: Eq + Hash>() -> Rule<Vec<T>>
```

### Phase 4: Advanced Rules (Optional)
```rust
// rules/advanced.rs (new file, feature-gated)
pub fn uuid() -> Rule<str>
pub fn ipv4() -> Rule<str>
pub fn ipv6() -> Rule<str>
pub fn slug() -> Rule<str>
```

---

## Feature Gate Strategy

```toml
[features]
default = []
email = ["regex"]  # existing
regex = ["dep:regex"]  # existing
advanced = ["regex", "uuid"]  # new: advanced validation rules
full = ["email", "regex", "advanced"]  # new: all rules
```

**Benefits:**
- Core remains zero-dependency
- Users opt into regex/advanced rules
- `full` feature for convenience

---

## Custom Error Messages

### Current State
Rules hardcode error messages:
```rust
ValidationError::single(Path::root(), "min_length", format!("Must be at least {} characters", min))
```

### Proposed Enhancement
```rust
pub fn min_len(min: usize) -> Rule<str> { ... }
pub fn min_len_with_message(min: usize, message: impl Into<String>) -> Rule<str> { ... }
```

Or use builder pattern:
```rust
rules::min_len(5).with_message("Email too short")
```

### Derive Macro Support
```rust
#[validate(length(min = 5, max = 255, message = "Email must be 5-255 chars"))]
```

This requires:
1. Parsing `message` in derive macro
2. Generating code that passes custom message to rule

---

## Box::leak() Documentation

### Current Issue
`Path::parse()` uses `Box::leak()` for `'static` lifetimes but this isn't documented.

### Proposed Documentation
Add to `path.rs`:
```rust
/// # Memory Considerations
///
/// This implementation uses `Box::leak()` to create `'static` references to field names.
/// This is intentional - field names in validation paths need static lifetime because:
///
/// 1. Paths can be stored in `ValidationError` which must be `'static`
/// 2. Field names are typically known at compile time (from derive macro)
/// 3. The number of unique field paths is bounded by your schema
///
/// **Memory Impact:** Each unique field name is leaked once. For a typical application
/// with ~100 domain types and ~10 fields each, this is ~1000 leaked strings (~50KB).
/// This is negligible for server applications.
///
/// **When to worry:** If you're dynamically generating millions of unique field names
/// at runtime, this could accumulate memory. In that case, use the core validation
/// functions with borrowed strings instead of the path-based API.
```

---

## Testing Strategy

### Current Coverage
- ✅ 67 tests across workspace
- ✅ 100% passing
- ✅ Unit tests for all existing rules

### New Tests Needed
1. **Doctests** - Add to all public API items
2. **Integration tests** - New rules with derive macro
3. **Property tests** - Consider quickcheck/proptest for rule composition
4. **Benchmark tests** - Performance regression detection

---

## Summary

### Immediate Actions (This PR)
1. ✅ Add 10 new high-value rules
2. ✅ Add doctests to all public APIs
3. ✅ Document `Box::leak()` memory behavior
4. ✅ Expose custom error messages in derive macro
5. ✅ Add comprehensive tests for new rules

### Follow-up Work (Future PRs)
- Add collection validation module
- Add advanced rules (feature-gated)
- Add property-based tests
- Add benchmark suite
- Consider async validation (v0.5)

---

**Estimated Impact:**
- **API Surface:** +55% (8 → 18 rules)
- **Code Size:** +~500 LOC (with tests)
- **Compile Time:** Minimal impact (mostly new functions)
- **Runtime Performance:** No change (zero-cost abstractions)
- **Dependency Count:** No change (unless advanced feature enabled)

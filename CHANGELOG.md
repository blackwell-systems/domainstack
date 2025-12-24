# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.4.0] - 2025-12-24

### Added

#### New Validation Rules (10 total)
**String Rules (8 new):**
- `rules::url()` - Validates URL format (requires `regex` feature)
- `rules::alphanumeric()` - Validates alphanumeric-only strings
- `rules::alpha_only()` - Validates alphabetic-only strings
- `rules::numeric_string()` - Validates numeric-only strings
- `rules::contains(needle)` - Validates string contains substring
- `rules::starts_with(prefix)` - Validates string prefix
- `rules::ends_with(suffix)` - Validates string suffix
- `rules::matches_regex(pattern)` - Validates against regex pattern (requires `regex` feature)

**Numeric Rules (3 new):**
- `rules::positive()` - Validates value is greater than zero
- `rules::negative()` - Validates value is less than zero
- `rules::multiple_of(divisor)` - Validates value is evenly divisible

#### Builder-Style Rule Customization
All validation rules now support fluent builder-style customization:

```rust
let rule = rules::email()
    .code("invalid_email")
    .message("Please provide a valid email address")
    .meta("hint", "Format: user@domain.com");
```

**New Methods:**
- `Rule::code(self, code: &'static str)` - Customize error code
- `Rule::message(self, msg: impl Into<String>)` - Customize error message
- `Rule::meta(self, key: &'static str, value: impl Into<String>)` - Add metadata

These methods work uniformly across all built-in and custom rules.

#### Context-Aware Error Messages
Introduced `RuleContext` to provide validation rules with field information for better error messages:

```rust
use domainstack::{Rule, RuleContext, ValidationError};

fn min_len_with_context(min: usize) -> Rule<str> {
    Rule::new(move |value: &str, ctx: &RuleContext| {
        if value.len() < min {
            ValidationError::single(
                ctx.full_path(),
                "min_length",
                format!(
                    "Field '{}' must be at least {} characters (got {})",
                    ctx.field_name.as_ref().map(|s| s.as_ref()).unwrap_or("unknown"),
                    min,
                    value.len()
                )
            )
        } else {
            ValidationError::default()
        }
    })
}
```

**New Type:**
- `RuleContext` - Contains `field_name`, `parent_path`, and `value_debug` for context-aware validation
- Methods: `root()`, `anonymous()`, `child()`, `with_value_debug()`, `full_path()`

**Benefits:**
- More helpful error messages including field names
- Better debugging with contextual information
- Improved user experience with specific, actionable errors

#### Documentation Improvements
- Added 30+ runnable doctests to public APIs (`ValidationError`, `Rule`, `Path`, all rules)
- Documented `Box::leak()` memory behavior in `Path::parse()` with usage guidance
- Created comprehensive rules reference (see `docs/RULES_V04.md`)
- Added `v4_builder_customization.rs` example demonstrating rule customization
- Added rule system analysis documents

### Changed

#### Performance & Memory Improvements
- **Eliminated memory leaks in `Path`**: Replaced `Box::leak()` with `Arc<str>` for field names
  - No more leaked memory from parsed paths
  - Reference-counted field names with proper cleanup
  - More idiomatic Rust memory management
  - Benefits long-running services and applications parsing many dynamic paths

- Improved error messages for all new validation rules
- Enhanced inline documentation across core types
- All existing tests passing (177 total: 128 unit + 39 doctests + framework tests)

### Breaking Changes
- **Rule function signature changed**: All rules now receive `RuleContext` as second parameter
  - Old: `Fn(&T) -> ValidationError`
  - New: `Fn(&T, &RuleContext) -> ValidationError`
  - **Migration**: Add `ctx: &RuleContext` parameter to custom rules, use `ctx.full_path()` instead of `Path::root()`
  - Existing code using `rule.apply()` continues to work (creates anonymous context)
  - Use `rule.apply_with_context()` for field-aware error messages

- `PathSegment::Field` now uses `Arc<str>` instead of `&'static str`
  - Affects code that pattern matches on `PathSegment` directly
  - Most users unaffected (use `Path::field()` API which remains the same)
  - **Migration**: No code changes needed for standard Path API usage

### Technical Details
- **Zero Unsafe Code** - Maintains safety guarantees
- **Zero Dependencies** - Core library remains dependency-free (regex is optional)
- **Zero Warnings** - Clean compile with clippy
- **Pre-1.0 Status** - Breaking changes acceptable before first publish

### Migration from 0.3.x

Update your `Cargo.toml`:

```toml
domainstack = "0.4.0"
```

**If you pattern match on `PathSegment` directly:**

```rust
// Before (v0.3.x)
match segment {
    PathSegment::Field(name) => println!("{}", name), // name: &'static str
    PathSegment::Index(idx) => println!("[{}]", idx),
}

// After (v0.4.0)
match segment {
    PathSegment::Field(name) => println!("{}", name), // name: Arc<str> (still prints fine)
    PathSegment::Index(idx) => println!("[{}]", idx),
}
```

Most code uses `Path::field()` and `Path::to_string()` which work identically. New features are opt-in via builder methods.

## [0.3.0] - Previous Release

Initial release with core validation framework, derive macros, and framework adapters for Axum and Actix-web.

---

## Unreleased Features (Roadmap)

See `docs/BREAKING_CHANGES_ANALYSIS.md` for planned features in future versions:
- v0.5.0: Async validation with database uniqueness checks
- v0.6.0: Cross-field validation
- v0.7.0: Enhanced metadata system
- v1.0.0: API stabilization

[0.4.0]: https://github.com/blackwell-systems/domainstack/compare/v0.3.0...v0.4.0

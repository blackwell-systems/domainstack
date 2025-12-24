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

#### Documentation Improvements
- Added 30+ runnable doctests to public APIs (`ValidationError`, `Rule`, `Path`, all rules)
- Documented `Box::leak()` memory behavior in `Path::parse()` with usage guidance
- Created comprehensive rules reference (see `docs/RULES_V04.md`)
- Added `v4_builder_customization.rs` example demonstrating rule customization
- Added rule system analysis documents

### Changed
- Improved error messages for all new validation rules
- Enhanced inline documentation across core types
- All existing tests passing (130 total: 100 unit + 30 doctests)

### Technical Details
- **Zero Breaking Changes** - Fully backward compatible with v0.3.x
- **Zero Unsafe Code** - Maintains safety guarantees
- **Zero Dependencies** - Core library remains dependency-free (regex is optional)
- **Zero Warnings** - Clean compile with clippy

### Migration from 0.3.x

No breaking changes! Simply update your `Cargo.toml`:

```toml
domainstack = "0.4.0"
```

All existing code continues to work. New features are opt-in via builder methods.

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

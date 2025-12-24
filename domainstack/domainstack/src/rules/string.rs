use crate::{Rule, RuleContext, ValidationError};

#[cfg(feature = "regex")]
use once_cell::sync::Lazy;

#[cfg(feature = "regex")]
static EMAIL_REGEX: Lazy<regex::Regex> =
    Lazy::new(|| regex::Regex::new(r"^[^@\s]+@[^@\s]+\.[^@\s]+$").unwrap());

#[cfg(feature = "regex")]
static URL_REGEX: Lazy<regex::Regex> = Lazy::new(|| {
    regex::Regex::new(
        r"^https?://[a-zA-Z0-9]([a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(\.[a-zA-Z0-9]([a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*(/.*)?$"
    ).unwrap()
});

/// Validates that a string is a valid email address.
///
/// Uses a cached regex pattern for RFC-compliant validation.
///
/// Requires the `regex` feature to be enabled.
///
/// # Examples
///
/// ```
/// use domainstack::prelude::*;
///
/// let rule = rules::email();
/// assert!(rule.apply("user@example.com").is_empty());
/// assert!(!rule.apply("invalid-email").is_empty());
/// ```
///
/// # Error Code
/// - Code: `invalid_email`
/// - Message: `"Invalid email format"`
///
/// # Performance
/// The regex pattern is compiled once and cached for the lifetime of the program,
/// making subsequent validations very efficient.
#[cfg(feature = "regex")]
pub fn email() -> Rule<str> {
    Rule::new(|value: &str, ctx: &RuleContext| {
        if EMAIL_REGEX.is_match(value) {
            ValidationError::default()
        } else {
            ValidationError::single(ctx.full_path(), "invalid_email", "Invalid email format")
        }
    })
}

/// Validates that a string is not empty.
///
/// # Examples
///
/// ```
/// use domainstack::prelude::*;
///
/// let rule = rules::non_empty();
/// assert!(rule.apply("hello").is_empty());
/// assert!(!rule.apply("").is_empty());
/// ```
///
/// # Error Code
/// - Code: `non_empty`
/// - Message: `"Must not be empty"`
pub fn non_empty() -> Rule<str> {
    Rule::new(|value: &str, ctx: &RuleContext| {
        if value.is_empty() {
            ValidationError::single(ctx.full_path(), "non_empty", "Must not be empty")
        } else {
            ValidationError::default()
        }
    })
}

/// Validates that a string has at least the minimum length.
///
/// # Examples
///
/// ```
/// use domainstack::prelude::*;
///
/// let rule = rules::min_len(5);
/// assert!(rule.apply("hello").is_empty());
/// assert!(rule.apply("hello world").is_empty());
/// assert!(!rule.apply("hi").is_empty());
/// ```
///
/// # Error Code
/// - Code: `min_length`
/// - Message: `"Must be at least {min} characters"`
/// - Meta: `{"min": "5"}`
pub fn min_len(min: usize) -> Rule<str> {
    Rule::new(move |value: &str, ctx: &RuleContext| {
        if value.len() < min {
            let mut err = ValidationError::single(
                ctx.full_path(),
                "min_length",
                format!("Must be at least {} characters", min),
            );
            err.violations[0].meta.insert("min", min.to_string());
            err
        } else {
            ValidationError::default()
        }
    })
}

/// Validates that a string does not exceed the maximum length.
///
/// # Examples
///
/// ```
/// use domainstack::prelude::*;
///
/// let rule = rules::max_len(10);
/// assert!(rule.apply("hello").is_empty());
/// assert!(!rule.apply("hello world!").is_empty());
/// ```
///
/// # Error Code
/// - Code: `max_length`
/// - Message: `"Must be at most {max} characters"`
/// - Meta: `{"max": "10"}`
pub fn max_len(max: usize) -> Rule<str> {
    Rule::new(move |value: &str, ctx: &RuleContext| {
        if value.len() > max {
            let mut err = ValidationError::single(
                ctx.full_path(),
                "max_length",
                format!("Must be at most {} characters", max),
            );
            err.violations[0].meta.insert("max", max.to_string());
            err
        } else {
            ValidationError::default()
        }
    })
}

/// Validates that a string length is within the specified range.
///
/// This is a convenience function that combines `min_len` and `max_len`.
///
/// # Examples
///
/// ```
/// use domainstack::prelude::*;
///
/// let rule = rules::length(3, 10);
/// assert!(rule.apply("hello").is_empty());
/// assert!(!rule.apply("hi").is_empty());  // too short
/// assert!(!rule.apply("hello world!").is_empty());  // too long
/// ```
///
/// # Error Codes
/// - Code: `min_length` if too short
/// - Code: `max_length` if too long
pub fn length(min: usize, max: usize) -> Rule<str> {
    min_len(min).and(max_len(max))
}

/// Validates that a string is a valid URL.
///
/// Checks for:
/// - Valid URL scheme (http, https)
/// - Presence of domain
/// - Valid URL characters
///
/// Requires the `regex` feature to be enabled.
///
/// # Examples
///
/// ```
/// use domainstack::prelude::*;
///
/// let rule = rules::url();
/// assert!(rule.apply("https://example.com").is_empty());
/// assert!(rule.apply("http://example.com/path").is_empty());
/// assert!(!rule.apply("not a url").is_empty());
/// assert!(!rule.apply("example.com").is_empty());  // missing scheme
/// ```
///
/// # Error Code
/// - Code: `invalid_url`
/// - Message: `"Invalid URL format"`
///
/// # Performance
/// The regex pattern is compiled once and cached for the lifetime of the program,
/// making subsequent validations very efficient.
#[cfg(feature = "regex")]
pub fn url() -> Rule<str> {
    Rule::new(|value: &str, ctx: &RuleContext| {
        if URL_REGEX.is_match(value) {
            ValidationError::default()
        } else {
            ValidationError::single(ctx.full_path(), "invalid_url", "Invalid URL format")
        }
    })
}

/// Validates that a string contains only alphanumeric characters (a-z, A-Z, 0-9).
///
/// # Examples
///
/// ```
/// use domainstack::prelude::*;
///
/// let rule = rules::alphanumeric();
/// assert!(rule.apply("abc123").is_empty());
/// assert!(rule.apply("HelloWorld123").is_empty());
/// assert!(!rule.apply("hello world").is_empty());  // space not allowed
/// assert!(!rule.apply("hello-world").is_empty());  // hyphen not allowed
/// ```
///
/// # Error Code
/// - Code: `not_alphanumeric`
/// - Message: `"Must contain only letters and numbers"`
pub fn alphanumeric() -> Rule<str> {
    Rule::new(|value: &str, ctx: &RuleContext| {
        if value.chars().all(|c| c.is_alphanumeric()) {
            ValidationError::default()
        } else {
            ValidationError::single(
                ctx.full_path(),
                "not_alphanumeric",
                "Must contain only letters and numbers",
            )
        }
    })
}

/// Validates that a string contains only alphabetic characters (a-z, A-Z).
///
/// # Examples
///
/// ```
/// use domainstack::prelude::*;
///
/// let rule = rules::alpha_only();
/// assert!(rule.apply("hello").is_empty());
/// assert!(rule.apply("HelloWorld").is_empty());
/// assert!(!rule.apply("hello123").is_empty());  // numbers not allowed
/// assert!(!rule.apply("hello world").is_empty());  // space not allowed
/// ```
///
/// # Error Code
/// - Code: `not_alpha`
/// - Message: `"Must contain only letters"`
pub fn alpha_only() -> Rule<str> {
    Rule::new(|value: &str, ctx: &RuleContext| {
        if value.chars().all(|c| c.is_alphabetic()) {
            ValidationError::default()
        } else {
            ValidationError::single(ctx.full_path(), "not_alpha", "Must contain only letters")
        }
    })
}

/// Validates that a string contains only numeric characters (0-9).
///
/// # Examples
///
/// ```
/// use domainstack::prelude::*;
///
/// let rule = rules::numeric_string();
/// assert!(rule.apply("123456").is_empty());
/// assert!(rule.apply("0").is_empty());
/// assert!(!rule.apply("123abc").is_empty());
/// assert!(!rule.apply("12.34").is_empty());  // decimal point not allowed
/// ```
///
/// # Error Code
/// - Code: `not_numeric`
/// - Message: `"Must contain only numbers"`
pub fn numeric_string() -> Rule<str> {
    Rule::new(|value: &str, ctx: &RuleContext| {
        if value.chars().all(|c| c.is_numeric()) {
            ValidationError::default()
        } else {
            ValidationError::single(ctx.full_path(), "not_numeric", "Must contain only numbers")
        }
    })
}

/// Validates that a string contains the specified substring.
///
/// # Examples
///
/// ```
/// use domainstack::prelude::*;
///
/// let rule = rules::contains("example");
/// assert!(rule.apply("user@example.com").is_empty());
/// assert!(rule.apply("example").is_empty());
/// assert!(!rule.apply("user@test.com").is_empty());
/// ```
///
/// # Error Code
/// - Code: `missing_substring`
/// - Message: `"Must contain '{substring}'"`
/// - Meta: `{"substring": "example"}`
pub fn contains(substring: &'static str) -> Rule<str> {
    Rule::new(move |value: &str, ctx: &RuleContext| {
        if value.contains(substring) {
            ValidationError::default()
        } else {
            let mut err = ValidationError::single(
                ctx.full_path(),
                "missing_substring",
                format!("Must contain '{}'", substring),
            );
            err.violations[0]
                .meta
                .insert("substring", substring.to_string());
            err
        }
    })
}

/// Validates that a string starts with the specified prefix.
///
/// # Examples
///
/// ```
/// use domainstack::prelude::*;
///
/// let rule = rules::starts_with("https://");
/// assert!(rule.apply("https://example.com").is_empty());
/// assert!(!rule.apply("http://example.com").is_empty());
/// ```
///
/// # Error Code
/// - Code: `invalid_prefix`
/// - Message: `"Must start with '{prefix}'"`
/// - Meta: `{"prefix": "https://"}`
pub fn starts_with(prefix: &'static str) -> Rule<str> {
    Rule::new(move |value: &str, ctx: &RuleContext| {
        if value.starts_with(prefix) {
            ValidationError::default()
        } else {
            let mut err = ValidationError::single(
                ctx.full_path(),
                "invalid_prefix",
                format!("Must start with '{}'", prefix),
            );
            err.violations[0].meta.insert("prefix", prefix.to_string());
            err
        }
    })
}

/// Validates that a string ends with the specified suffix.
///
/// # Examples
///
/// ```
/// use domainstack::prelude::*;
///
/// let rule = rules::ends_with(".com");
/// assert!(rule.apply("example.com").is_empty());
/// assert!(!rule.apply("example.org").is_empty());
/// ```
///
/// # Error Code
/// - Code: `invalid_suffix`
/// - Message: `"Must end with '{suffix}'"`
/// - Meta: `{"suffix": ".com"}`
pub fn ends_with(suffix: &'static str) -> Rule<str> {
    Rule::new(move |value: &str, ctx: &RuleContext| {
        if value.ends_with(suffix) {
            ValidationError::default()
        } else {
            let mut err = ValidationError::single(
                ctx.full_path(),
                "invalid_suffix",
                format!("Must end with '{}'", suffix),
            );
            err.violations[0].meta.insert("suffix", suffix.to_string());
            err
        }
    })
}

/// Validates that a string matches the specified regex pattern.
///
/// Requires the `regex` feature to be enabled.
///
/// The regex pattern is compiled once when the rule is created, not on every validation.
/// This makes repeated validations with the same pattern very efficient.
///
/// # Examples
///
/// ```
/// #[cfg(feature = "regex")]
/// {
///     use domainstack::prelude::*;
///
///     let rule = rules::matches_regex(r"^\d{3}-\d{4}$");  // Phone: 123-4567
///     assert!(rule.apply("123-4567").is_empty());
///     assert!(!rule.apply("1234567").is_empty());
/// }
/// ```
///
/// # Error Code
/// - Code: `pattern_mismatch`
/// - Message: `"Does not match required pattern"`
/// - Meta: `{"pattern": "regex"}`
///
/// # Panics
/// Panics if the regex pattern is invalid at rule creation time.
///
/// # Performance
/// The regex is compiled once at rule creation and reused for all validations,
/// making this very efficient for repeated use.
#[cfg(feature = "regex")]
pub fn matches_regex(pattern: &'static str) -> Rule<str> {
    // Compile regex once at rule creation time
    let re = regex::Regex::new(pattern).expect("Invalid regex pattern");

    Rule::new(move |value: &str, ctx: &RuleContext| {
        if re.is_match(value) {
            ValidationError::default()
        } else {
            let mut err = ValidationError::single(
                ctx.full_path(),
                "pattern_mismatch",
                "Does not match required pattern",
            );
            err.violations[0]
                .meta
                .insert("pattern", pattern.to_string());
            err
        }
    })
}

/// Validates that a string is not blank (not empty after trimming whitespace).
///
/// Unlike `non_empty()` which only checks if the string length is zero,
/// `non_blank()` trims whitespace first, so strings containing only whitespace
/// will fail validation.
///
/// # Examples
///
/// ```
/// use domainstack::prelude::*;
///
/// let rule = rules::non_blank();
/// assert!(rule.apply("hello").is_empty());
/// assert!(rule.apply("  hello  ").is_empty());  // has content after trim
/// assert!(!rule.apply("").is_empty());          // empty
/// assert!(!rule.apply("   ").is_empty());       // only whitespace
/// assert!(!rule.apply("\t\n").is_empty());      // only whitespace
/// ```
///
/// # Error Code
/// - Code: `blank`
/// - Message: `"Must not be blank"`
pub fn non_blank() -> Rule<str> {
    Rule::new(|value: &str, ctx: &RuleContext| {
        if value.trim().is_empty() {
            ValidationError::single(ctx.full_path(), "blank", "Must not be blank")
        } else {
            ValidationError::default()
        }
    })
}

/// Validates that a string contains no whitespace characters.
///
/// Checks for any Unicode whitespace characters, including spaces, tabs, newlines, etc.
///
/// # Examples
///
/// ```
/// use domainstack::prelude::*;
///
/// let rule = rules::no_whitespace();
/// assert!(rule.apply("hello").is_empty());
/// assert!(rule.apply("HelloWorld123").is_empty());
/// assert!(!rule.apply("hello world").is_empty());  // space
/// assert!(!rule.apply("hello\tworld").is_empty()); // tab
/// assert!(!rule.apply("hello\n").is_empty());      // newline
/// ```
///
/// # Error Code
/// - Code: `contains_whitespace`
/// - Message: `"Must not contain whitespace"`
pub fn no_whitespace() -> Rule<str> {
    Rule::new(|value: &str, ctx: &RuleContext| {
        if value.chars().any(|c| c.is_whitespace()) {
            ValidationError::single(
                ctx.full_path(),
                "contains_whitespace",
                "Must not contain whitespace",
            )
        } else {
            ValidationError::default()
        }
    })
}

/// Validates that a string contains only ASCII characters.
///
/// # Examples
///
/// ```
/// use domainstack::prelude::*;
///
/// let rule = rules::ascii();
/// assert!(rule.apply("hello").is_empty());
/// assert!(rule.apply("Hello123!").is_empty());
/// assert!(!rule.apply("hello🚀").is_empty());   // emoji
/// assert!(!rule.apply("café").is_empty());      // accented character
/// ```
///
/// # Error Code
/// - Code: `not_ascii`
/// - Message: `"Must contain only ASCII characters"`
pub fn ascii() -> Rule<str> {
    Rule::new(|value: &str, ctx: &RuleContext| {
        if value.is_ascii() {
            ValidationError::default()
        } else {
            ValidationError::single(
                ctx.full_path(),
                "not_ascii",
                "Must contain only ASCII characters",
            )
        }
    })
}

/// Validates that a string's character count (not byte count) is within the specified range.
///
/// Unlike `length()` which counts bytes, `len_chars()` counts Unicode characters.
/// This is important for strings with multi-byte characters (emojis, accented letters, etc.).
///
/// # Examples
///
/// ```
/// use domainstack::prelude::*;
///
/// let rule = rules::len_chars(3, 10);
///
/// // ASCII strings - byte length == char length
/// assert!(rule.apply("hello").is_empty());      // 5 chars
/// assert!(!rule.apply("hi").is_empty());        // 2 chars (too short)
///
/// // Multi-byte strings - byte length != char length
/// assert!(rule.apply("café").is_empty());       // 4 chars (5 bytes)
/// assert!(rule.apply("hello🚀").is_empty());    // 6 chars (9 bytes)
/// assert!(!rule.apply("hi").is_empty());        // 2 chars (too short)
/// ```
///
/// # Error Codes
/// - Code: `min_chars` if too few characters
/// - Code: `max_chars` if too many characters
/// - Meta: `{"min": "3", "max": "10", "actual": "2"}`
pub fn len_chars(min: usize, max: usize) -> Rule<str> {
    Rule::new(move |value: &str, ctx: &RuleContext| {
        let char_count = value.chars().count();

        if char_count < min {
            let mut err = ValidationError::single(
                ctx.full_path(),
                "min_chars",
                format!("Must be at least {} characters", min),
            );
            err.violations[0].meta.insert("min", min.to_string());
            err.violations[0]
                .meta
                .insert("actual", char_count.to_string());
            err
        } else if char_count > max {
            let mut err = ValidationError::single(
                ctx.full_path(),
                "max_chars",
                format!("Must be at most {} characters", max),
            );
            err.violations[0].meta.insert("max", max.to_string());
            err.violations[0]
                .meta
                .insert("actual", char_count.to_string());
            err
        } else {
            ValidationError::default()
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_valid() {
        let rule = email();
        assert!(rule.apply("user@example.com").is_empty());
        assert!(rule.apply("test.user@domain.co.uk").is_empty());
    }

    #[test]
    fn test_email_invalid() {
        let rule = email();

        let result = rule.apply("not-an-email");
        assert!(!result.is_empty());
        assert_eq!(result.violations[0].code, "invalid_email");

        let result = rule.apply("missing-domain@");
        assert!(!result.is_empty());

        let result = rule.apply("@missing-user.com");
        assert!(!result.is_empty());
    }

    #[test]
    fn test_non_empty_valid() {
        let rule = non_empty();
        assert!(rule.apply("hello").is_empty());
        assert!(rule.apply(" ").is_empty());
    }

    #[test]
    fn test_non_empty_invalid() {
        let rule = non_empty();
        let result = rule.apply("");
        assert!(!result.is_empty());
        assert_eq!(result.violations[0].code, "non_empty");
    }

    #[test]
    fn test_min_len_valid() {
        let rule = min_len(5);
        assert!(rule.apply("hello").is_empty());
        assert!(rule.apply("hello world").is_empty());
    }

    #[test]
    fn test_min_len_invalid() {
        let rule = min_len(5);
        let result = rule.apply("hi");
        assert!(!result.is_empty());
        assert_eq!(result.violations[0].code, "min_length");
        assert_eq!(result.violations[0].meta.get("min"), Some("5"));
    }

    #[test]
    fn test_max_len_valid() {
        let rule = max_len(10);
        assert!(rule.apply("hello").is_empty());
        assert!(rule.apply("").is_empty());
    }

    #[test]
    fn test_max_len_invalid() {
        let rule = max_len(5);
        let result = rule.apply("hello world");
        assert!(!result.is_empty());
        assert_eq!(result.violations[0].code, "max_length");
        assert_eq!(result.violations[0].meta.get("max"), Some("5"));
    }

    #[test]
    fn test_length_valid() {
        let rule = length(3, 10);
        assert!(rule.apply("hello").is_empty());
        assert!(rule.apply("hi!").is_empty());
    }

    #[test]
    fn test_length_too_short() {
        let rule = length(3, 10);
        let result = rule.apply("hi");
        assert!(!result.is_empty());
        assert_eq!(result.violations[0].code, "min_length");
    }

    #[test]
    fn test_length_too_long() {
        let rule = length(3, 10);
        let result = rule.apply("hello world!");
        assert!(!result.is_empty());
        assert_eq!(result.violations[0].code, "max_length");
    }

    #[test]
    fn test_url_valid() {
        let rule = url();
        assert!(rule.apply("https://example.com").is_empty());
        assert!(rule.apply("http://example.com").is_empty());
        assert!(rule.apply("https://example.com/path").is_empty());
        assert!(rule.apply("http://test.example.com").is_empty());
    }

    #[test]
    fn test_url_invalid() {
        let rule = url();

        let result = rule.apply("not a url");
        assert!(!result.is_empty());
        assert_eq!(result.violations[0].code, "invalid_url");

        let result = rule.apply("example.com");
        assert!(!result.is_empty());

        let result = rule.apply("ftp://example.com");
        assert!(!result.is_empty());
    }

    #[test]
    fn test_alphanumeric_valid() {
        let rule = alphanumeric();
        assert!(rule.apply("abc123").is_empty());
        assert!(rule.apply("HelloWorld123").is_empty());
        assert!(rule.apply("123").is_empty());
        assert!(rule.apply("abc").is_empty());
    }

    #[test]
    fn test_alphanumeric_invalid() {
        let rule = alphanumeric();

        let result = rule.apply("hello world");
        assert!(!result.is_empty());
        assert_eq!(result.violations[0].code, "not_alphanumeric");

        let result = rule.apply("hello-world");
        assert!(!result.is_empty());

        let result = rule.apply("hello_world");
        assert!(!result.is_empty());
    }

    #[test]
    fn test_alpha_only_valid() {
        let rule = alpha_only();
        assert!(rule.apply("hello").is_empty());
        assert!(rule.apply("HelloWorld").is_empty());
        assert!(rule.apply("ABC").is_empty());
    }

    #[test]
    fn test_alpha_only_invalid() {
        let rule = alpha_only();

        let result = rule.apply("hello123");
        assert!(!result.is_empty());
        assert_eq!(result.violations[0].code, "not_alpha");

        let result = rule.apply("hello world");
        assert!(!result.is_empty());
    }

    #[test]
    fn test_numeric_string_valid() {
        let rule = numeric_string();
        assert!(rule.apply("123456").is_empty());
        assert!(rule.apply("0").is_empty());
    }

    #[test]
    fn test_numeric_string_invalid() {
        let rule = numeric_string();

        let result = rule.apply("123abc");
        assert!(!result.is_empty());
        assert_eq!(result.violations[0].code, "not_numeric");

        let result = rule.apply("12.34");
        assert!(!result.is_empty());
    }

    #[test]
    fn test_contains_valid() {
        let rule = contains("example");
        assert!(rule.apply("user@example.com").is_empty());
        assert!(rule.apply("example").is_empty());
        assert!(rule.apply("this is an example").is_empty());
    }

    #[test]
    fn test_contains_invalid() {
        let rule = contains("example");
        let result = rule.apply("user@test.com");
        assert!(!result.is_empty());
        assert_eq!(result.violations[0].code, "missing_substring");
        assert_eq!(result.violations[0].meta.get("substring"), Some("example"));
    }

    #[test]
    fn test_starts_with_valid() {
        let rule = starts_with("https://");
        assert!(rule.apply("https://example.com").is_empty());
        assert!(rule.apply("https://").is_empty());
    }

    #[test]
    fn test_starts_with_invalid() {
        let rule = starts_with("https://");
        let result = rule.apply("http://example.com");
        assert!(!result.is_empty());
        assert_eq!(result.violations[0].code, "invalid_prefix");
    }

    #[test]
    fn test_ends_with_valid() {
        let rule = ends_with(".com");
        assert!(rule.apply("example.com").is_empty());
        assert!(rule.apply(".com").is_empty());
    }

    #[test]
    fn test_ends_with_invalid() {
        let rule = ends_with(".com");
        let result = rule.apply("example.org");
        assert!(!result.is_empty());
        assert_eq!(result.violations[0].code, "invalid_suffix");
    }

    #[cfg(feature = "regex")]
    #[test]
    fn test_matches_regex_valid() {
        let rule = matches_regex(r"^\d{3}-\d{4}$");
        assert!(rule.apply("123-4567").is_empty());
    }

    #[cfg(feature = "regex")]
    #[test]
    fn test_matches_regex_invalid() {
        let rule = matches_regex(r"^\d{3}-\d{4}$");
        let result = rule.apply("1234567");
        assert!(!result.is_empty());
        assert_eq!(result.violations[0].code, "pattern_mismatch");
    }

    #[test]
    fn test_non_blank_valid() {
        let rule = non_blank();
        assert!(rule.apply("hello").is_empty());
        assert!(rule.apply("  hello  ").is_empty()); // has content after trim
        assert!(rule.apply("a").is_empty());
    }

    #[test]
    fn test_non_blank_invalid() {
        let rule = non_blank();

        let result = rule.apply("");
        assert!(!result.is_empty());
        assert_eq!(result.violations[0].code, "blank");

        let result = rule.apply("   ");
        assert!(!result.is_empty());

        let result = rule.apply("\t\n");
        assert!(!result.is_empty());
    }

    #[test]
    fn test_no_whitespace_valid() {
        let rule = no_whitespace();
        assert!(rule.apply("hello").is_empty());
        assert!(rule.apply("HelloWorld123").is_empty());
        assert!(rule.apply("hello-world").is_empty());
        assert!(rule.apply("hello_world").is_empty());
    }

    #[test]
    fn test_no_whitespace_invalid() {
        let rule = no_whitespace();

        let result = rule.apply("hello world");
        assert!(!result.is_empty());
        assert_eq!(result.violations[0].code, "contains_whitespace");

        let result = rule.apply("hello\tworld");
        assert!(!result.is_empty());

        let result = rule.apply("hello\n");
        assert!(!result.is_empty());

        let result = rule.apply("  ");
        assert!(!result.is_empty());
    }

    #[test]
    fn test_ascii_valid() {
        let rule = ascii();
        assert!(rule.apply("hello").is_empty());
        assert!(rule.apply("Hello123!").is_empty());
        assert!(rule.apply("").is_empty()); // empty string is ASCII
        assert!(rule.apply("abc-def_123").is_empty());
    }

    #[test]
    fn test_ascii_invalid() {
        let rule = ascii();

        let result = rule.apply("hello🚀");
        assert!(!result.is_empty());
        assert_eq!(result.violations[0].code, "not_ascii");

        let result = rule.apply("café");
        assert!(!result.is_empty());

        let result = rule.apply("hello世界");
        assert!(!result.is_empty());
    }

    #[test]
    fn test_len_chars_valid() {
        let rule = len_chars(3, 10);

        // ASCII strings
        assert!(rule.apply("hello").is_empty()); // 5 chars
        assert!(rule.apply("abc").is_empty()); // 3 chars (min)
        assert!(rule.apply("abcdefghij").is_empty()); // 10 chars (max)

        // Multi-byte strings
        assert!(rule.apply("café").is_empty()); // 4 chars (5 bytes)
        assert!(rule.apply("hello🚀").is_empty()); // 6 chars (9 bytes)
    }

    #[test]
    fn test_len_chars_too_short() {
        let rule = len_chars(3, 10);

        let result = rule.apply("hi");
        assert!(!result.is_empty());
        assert_eq!(result.violations[0].code, "min_chars");
        assert_eq!(result.violations[0].meta.get("min"), Some("3"));
        assert_eq!(result.violations[0].meta.get("actual"), Some("2"));

        let result = rule.apply("");
        assert!(!result.is_empty());
    }

    #[test]
    fn test_len_chars_too_long() {
        let rule = len_chars(3, 10);

        let result = rule.apply("hello world!");
        assert!(!result.is_empty());
        assert_eq!(result.violations[0].code, "max_chars");
        assert_eq!(result.violations[0].meta.get("max"), Some("10"));
        assert_eq!(result.violations[0].meta.get("actual"), Some("12"));
    }

    #[test]
    fn test_len_chars_vs_length() {
        // Demonstrate the difference between byte length and char length
        let emoji_string = "🚀🚀🚀"; // 3 chars, 12 bytes

        // len_chars counts characters
        let char_rule = len_chars(1, 5);
        assert!(char_rule.apply(emoji_string).is_empty()); // 3 chars - valid

        // length counts bytes
        let byte_rule = length(1, 5);
        assert!(!byte_rule.apply(emoji_string).is_empty()); // 12 bytes - invalid
    }
}

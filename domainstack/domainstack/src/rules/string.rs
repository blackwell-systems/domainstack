use crate::{Rule, RuleContext, ValidationError};

/// Validates that a string is a valid email address.
///
/// With the `email` feature enabled, uses a regex pattern for RFC-compliant validation.
/// Without the feature, performs basic structural validation (checks for @ and domain).
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
pub fn email() -> Rule<str> {
    Rule::new(|value: &str, ctx: &RuleContext| {
        #[cfg(feature = "email")]
        {
            let re = regex::Regex::new(r"^[^@\s]+@[^@\s]+\.[^@\s]+$").unwrap();

            if re.is_match(value) {
                ValidationError::default()
            } else {
                ValidationError::single(ctx.full_path(), "invalid_email", "Invalid email format")
            }
        }

        #[cfg(not(feature = "email"))]
        {
            let parts: Vec<&str> = value.split('@').collect();
            if parts.len() == 2
                && !parts[0].is_empty()
                && parts[1].contains('.')
                && !parts[1].starts_with('.')
            {
                ValidationError::default()
            } else {
                ValidationError::single(ctx.full_path(), "invalid_email", "Invalid email format")
            }
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
/// - Valid URL scheme (http, https, ftp, etc.)
/// - Presence of domain
/// - Valid URL characters
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
pub fn url() -> Rule<str> {
    Rule::new(|value: &str, ctx: &RuleContext| {
        #[cfg(feature = "regex")]
        {
            let re = regex::Regex::new(
                r"^https?://[a-zA-Z0-9]([a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(\.[a-zA-Z0-9]([a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*(/.*)?$"
            ).unwrap();

            if re.is_match(value) {
                ValidationError::default()
            } else {
                ValidationError::single(ctx.full_path(), "invalid_url", "Invalid URL format")
            }
        }

        #[cfg(not(feature = "regex"))]
        {
            // Basic URL validation without regex
            if value.starts_with("http://") || value.starts_with("https://") {
                let without_scheme = value
                    .strip_prefix("http://")
                    .or_else(|| value.strip_prefix("https://"))
                    .unwrap_or("");

                if !without_scheme.is_empty()
                    && without_scheme.contains('.')
                    && !without_scheme.starts_with('.')
                {
                    ValidationError::default()
                } else {
                    ValidationError::single(ctx.full_path(), "invalid_url", "Invalid URL format")
                }
            } else {
                ValidationError::single(ctx.full_path(), "invalid_url", "Invalid URL format")
            }
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
/// Panics if the regex pattern is invalid.
#[cfg(feature = "regex")]
pub fn matches_regex(pattern: &'static str) -> Rule<str> {
    Rule::new(move |value: &str, ctx: &RuleContext| {
        let re = regex::Regex::new(pattern).expect("Invalid regex pattern");
        if re.is_match(value) {
            ValidationError::default()
        } else {
            let mut err = ValidationError::single(
                ctx.full_path(),
                "pattern_mismatch",
                "Does not match required pattern",
            );
            err.violations[0].meta.insert("pattern", pattern.to_string());
            err
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
}

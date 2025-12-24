use crate::{Rule, RuleContext, ValidationError};

/// Validates that a value equals the specified value.
///
/// Works with any type that implements `PartialEq`.
///
/// # Examples
///
/// ```
/// use domainstack::prelude::*;
///
/// // With string literals
/// let rule = rules::equals("active");
/// let value = "active";
/// assert!(rule.apply(&value).is_empty());
///
/// let value = "inactive";
/// assert!(!rule.apply(&value).is_empty());
///
/// // Works with numbers
/// let rule = rules::equals(42);
/// assert!(rule.apply(&42).is_empty());
/// assert!(!rule.apply(&0).is_empty());
/// ```
///
/// # Error Code
/// - Code: `not_equal`
/// - Message: `"Must equal '{expected}'"`
/// - Meta: `{"expected": "value"}`
pub fn equals<T>(expected: T) -> Rule<T>
where
    T: PartialEq + Clone + std::fmt::Display + Send + Sync + 'static,
{
    Rule::new(move |value: &T, ctx: &RuleContext| {
        if *value == expected {
            ValidationError::default()
        } else {
            let mut err = ValidationError::single(
                ctx.full_path(),
                "not_equal",
                format!("Must equal '{}'", expected),
            );
            err.violations[0]
                .meta
                .insert("expected", expected.to_string());
            err
        }
    })
}

/// Validates that a value does not equal the specified value.
///
/// Works with any type that implements `PartialEq`.
///
/// # Examples
///
/// ```
/// use domainstack::prelude::*;
///
/// // With string literals
/// let rule = rules::not_equals("banned");
/// let value = "active";
/// assert!(rule.apply(&value).is_empty());
///
/// let value = "banned";
/// assert!(!rule.apply(&value).is_empty());
///
/// // Works with numbers
/// let rule = rules::not_equals(0);
/// assert!(rule.apply(&42).is_empty());
/// assert!(!rule.apply(&0).is_empty());
/// ```
///
/// # Error Code
/// - Code: `forbidden_value`
/// - Message: `"Must not equal '{forbidden}'"`
/// - Meta: `{"forbidden": "value"}`
pub fn not_equals<T>(forbidden: T) -> Rule<T>
where
    T: PartialEq + Clone + std::fmt::Display + Send + Sync + 'static,
{
    Rule::new(move |value: &T, ctx: &RuleContext| {
        if *value != forbidden {
            ValidationError::default()
        } else {
            let mut err = ValidationError::single(
                ctx.full_path(),
                "forbidden_value",
                format!("Must not equal '{}'", forbidden),
            );
            err.violations[0]
                .meta
                .insert("forbidden", forbidden.to_string());
            err
        }
    })
}

/// Validates that a value is one of the allowed values.
///
/// Works with any type that implements `PartialEq`.
///
/// # Examples
///
/// ```
/// use domainstack::prelude::*;
///
/// // With string literals
/// let rule = rules::one_of(&["active", "pending", "inactive"]);
/// let value = "active";
/// assert!(rule.apply(&value).is_empty());
///
/// let value = "pending";
/// assert!(rule.apply(&value).is_empty());
///
/// let value = "banned";
/// assert!(!rule.apply(&value).is_empty());
///
/// // Works with numbers
/// let rule = rules::one_of(&[1, 2, 3, 5, 8, 13]);
/// assert!(rule.apply(&5).is_empty());
/// assert!(!rule.apply(&4).is_empty());
/// ```
///
/// # Error Code
/// - Code: `not_in_set`
/// - Message: `"Must be one of: {allowed}"`
/// - Meta: `{"allowed": "[value1, value2, ...]"}`
pub fn one_of<T>(allowed: &[T]) -> Rule<T>
where
    T: PartialEq + Clone + std::fmt::Debug + Send + Sync + 'static,
{
    let allowed_vec = allowed.to_vec();

    Rule::new(move |value: &T, ctx: &RuleContext| {
        if allowed_vec.contains(value) {
            ValidationError::default()
        } else {
            let allowed_str = format!("{:?}", allowed_vec);
            let mut err = ValidationError::single(
                ctx.full_path(),
                "not_in_set",
                format!("Must be one of: {}", allowed_str),
            );
            err.violations[0].meta.insert("allowed", allowed_str);
            err
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_equals_valid_string() {
        let rule = equals("active");
        let value = "active";
        assert!(rule.apply(&value).is_empty());
    }

    #[test]
    fn test_equals_invalid_string() {
        let rule = equals("active");
        let value = "inactive";
        let result = rule.apply(&value);
        assert!(!result.is_empty());
        assert_eq!(result.violations[0].code, "not_equal");
        assert_eq!(result.violations[0].meta.get("expected"), Some("active"));
    }

    #[test]
    fn test_equals_valid_number() {
        let rule = equals(42);
        assert!(rule.apply(&42).is_empty());
    }

    #[test]
    fn test_equals_invalid_number() {
        let rule = equals(42);
        let result = rule.apply(&0);
        assert!(!result.is_empty());
        assert_eq!(result.violations[0].code, "not_equal");
    }

    #[test]
    fn test_not_equals_valid_string() {
        let rule = not_equals("banned");
        let value1 = "active";
        let value2 = "pending";
        assert!(rule.apply(&value1).is_empty());
        assert!(rule.apply(&value2).is_empty());
    }

    #[test]
    fn test_not_equals_invalid_string() {
        let rule = not_equals("banned");
        let value = "banned";
        let result = rule.apply(&value);
        assert!(!result.is_empty());
        assert_eq!(result.violations[0].code, "forbidden_value");
        assert_eq!(result.violations[0].meta.get("forbidden"), Some("banned"));
    }

    #[test]
    fn test_not_equals_valid_number() {
        let rule = not_equals(0);
        assert!(rule.apply(&42).is_empty());
        assert!(rule.apply(&-1).is_empty());
    }

    #[test]
    fn test_not_equals_invalid_number() {
        let rule = not_equals(0);
        let result = rule.apply(&0);
        assert!(!result.is_empty());
        assert_eq!(result.violations[0].code, "forbidden_value");
    }

    #[test]
    fn test_one_of_valid_string() {
        let rule = one_of(&["active", "pending", "inactive"]);
        let value1 = "active";
        let value2 = "pending";
        let value3 = "inactive";
        assert!(rule.apply(&value1).is_empty());
        assert!(rule.apply(&value2).is_empty());
        assert!(rule.apply(&value3).is_empty());
    }

    #[test]
    fn test_one_of_invalid_string() {
        let rule = one_of(&["active", "pending", "inactive"]);
        let value = "banned";
        let result = rule.apply(&value);
        assert!(!result.is_empty());
        assert_eq!(result.violations[0].code, "not_in_set");
    }

    #[test]
    fn test_one_of_valid_number() {
        let rule = one_of(&[1, 2, 3, 5, 8, 13]);
        assert!(rule.apply(&1).is_empty());
        assert!(rule.apply(&5).is_empty());
        assert!(rule.apply(&13).is_empty());
    }

    #[test]
    fn test_one_of_invalid_number() {
        let rule = one_of(&[1, 2, 3, 5, 8, 13]);
        let result = rule.apply(&4);
        assert!(!result.is_empty());
        assert_eq!(result.violations[0].code, "not_in_set");
    }

    #[test]
    fn test_one_of_empty_set() {
        let rule: Rule<i32> = one_of(&[]);
        let result = rule.apply(&42);
        assert!(!result.is_empty());
        assert_eq!(result.violations[0].code, "not_in_set");
    }
}

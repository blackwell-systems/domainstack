use crate::{Path, Rule, ValidationError};

/// Validates that a numeric value is within the specified range (inclusive).
///
/// # Examples
///
/// ```
/// use domainstack::prelude::*;
///
/// let rule = rules::range(18, 120);
/// assert!(rule.apply(&25).is_empty());
/// assert!(rule.apply(&18).is_empty());  // min boundary
/// assert!(rule.apply(&120).is_empty()); // max boundary
/// assert!(!rule.apply(&17).is_empty()); // below min
/// assert!(!rule.apply(&121).is_empty()); // above max
/// ```
///
/// # Error Code
/// - Code: `out_of_range`
/// - Message: `"Must be between {min} and {max}"`
/// - Meta: `{"min": "18", "max": "120"}`
pub fn range<T>(min: T, max: T) -> Rule<T>
where
    T: PartialOrd + Copy + std::fmt::Display + Send + Sync + 'static,
{
    Rule::new(move |value: &T| {
        if *value < min || *value > max {
            let mut err = ValidationError::single(
                Path::root(),
                "out_of_range",
                format!("Must be between {} and {}", min, max),
            );
            err.violations[0].meta.insert("min", min.to_string());
            err.violations[0].meta.insert("max", max.to_string());
            err
        } else {
            ValidationError::default()
        }
    })
}

/// Validates that a numeric value is at least the minimum.
///
/// # Examples
///
/// ```
/// use domainstack::prelude::*;
///
/// let rule = rules::min(18);
/// assert!(rule.apply(&18).is_empty());
/// assert!(rule.apply(&100).is_empty());
/// assert!(!rule.apply(&17).is_empty());
/// ```
///
/// # Error Code
/// - Code: `below_minimum`
/// - Message: `"Must be at least {min}"`
/// - Meta: `{"min": "18"}`
pub fn min<T>(min: T) -> Rule<T>
where
    T: PartialOrd + Copy + std::fmt::Display + Send + Sync + 'static,
{
    Rule::new(move |value: &T| {
        if *value < min {
            let mut err = ValidationError::single(
                Path::root(),
                "below_minimum",
                format!("Must be at least {}", min),
            );
            err.violations[0].meta.insert("min", min.to_string());
            err
        } else {
            ValidationError::default()
        }
    })
}

/// Validates that a numeric value does not exceed the maximum.
///
/// # Examples
///
/// ```
/// use domainstack::prelude::*;
///
/// let rule = rules::max(100);
/// assert!(rule.apply(&100).is_empty());
/// assert!(rule.apply(&50).is_empty());
/// assert!(!rule.apply(&101).is_empty());
/// ```
///
/// # Error Code
/// - Code: `above_maximum`
/// - Message: `"Must be at most {max}"`
/// - Meta: `{"max": "100"}`
pub fn max<T>(max: T) -> Rule<T>
where
    T: PartialOrd + Copy + std::fmt::Display + Send + Sync + 'static,
{
    Rule::new(move |value: &T| {
        if *value > max {
            let mut err = ValidationError::single(
                Path::root(),
                "above_maximum",
                format!("Must be at most {}", max),
            );
            err.violations[0].meta.insert("max", max.to_string());
            err
        } else {
            ValidationError::default()
        }
    })
}

/// Validates that a numeric value is positive (greater than zero).
///
/// # Examples
///
/// ```
/// use domainstack::prelude::*;
///
/// let rule = rules::positive();
/// assert!(rule.apply(&1).is_empty());
/// assert!(rule.apply(&100).is_empty());
/// assert!(!rule.apply(&0).is_empty());
/// assert!(!rule.apply(&-1).is_empty());
/// ```
///
/// # Error Code
/// - Code: `not_positive`
/// - Message: `"Must be positive (greater than zero)"`
pub fn positive<T>() -> Rule<T>
where
    T: PartialOrd + Default + Copy + Send + Sync + 'static,
{
    Rule::new(move |value: &T| {
        if *value > T::default() {
            ValidationError::default()
        } else {
            ValidationError::single(
                Path::root(),
                "not_positive",
                "Must be positive (greater than zero)",
            )
        }
    })
}

/// Validates that a numeric value is negative (less than zero).
///
/// # Examples
///
/// ```
/// use domainstack::prelude::*;
///
/// let rule = rules::negative();
/// assert!(rule.apply(&-1).is_empty());
/// assert!(rule.apply(&-100).is_empty());
/// assert!(!rule.apply(&0).is_empty());
/// assert!(!rule.apply(&1).is_empty());
/// ```
///
/// # Error Code
/// - Code: `not_negative`
/// - Message: `"Must be negative (less than zero)"`
pub fn negative<T>() -> Rule<T>
where
    T: PartialOrd + Default + Copy + Send + Sync + 'static,
{
    Rule::new(move |value: &T| {
        if *value < T::default() {
            ValidationError::default()
        } else {
            ValidationError::single(
                Path::root(),
                "not_negative",
                "Must be negative (less than zero)",
            )
        }
    })
}

/// Validates that a numeric value is a multiple of the specified number.
///
/// # Examples
///
/// ```
/// use domainstack::prelude::*;
///
/// let rule = rules::multiple_of(5);
/// assert!(rule.apply(&10).is_empty());
/// assert!(rule.apply(&15).is_empty());
/// assert!(rule.apply(&0).is_empty());
/// assert!(!rule.apply(&7).is_empty());
/// ```
///
/// # Error Code
/// - Code: `not_multiple`
/// - Message: `"Must be a multiple of {divisor}"`
/// - Meta: `{"divisor": "5"}`
pub fn multiple_of<T>(divisor: T) -> Rule<T>
where
    T: std::ops::Rem<Output = T> + PartialEq + Default + Copy + std::fmt::Display + Send + Sync + 'static,
{
    Rule::new(move |value: &T| {
        if *value % divisor == T::default() {
            ValidationError::default()
        } else {
            let mut err = ValidationError::single(
                Path::root(),
                "not_multiple",
                format!("Must be a multiple of {}", divisor),
            );
            err.violations[0]
                .meta
                .insert("divisor", divisor.to_string());
            err
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_range_u8_valid() {
        let rule = range(18u8, 120u8);
        assert!(rule.apply(&18).is_empty());
        assert!(rule.apply(&50).is_empty());
        assert!(rule.apply(&120).is_empty());
    }

    #[test]
    fn test_range_u8_below() {
        let rule = range(18u8, 120u8);
        let result = rule.apply(&17);
        assert!(!result.is_empty());
        assert_eq!(result.violations[0].code, "out_of_range");
        assert_eq!(result.violations[0].meta.get("min"), Some("18"));
        assert_eq!(result.violations[0].meta.get("max"), Some("120"));
    }

    #[test]
    fn test_range_u8_above() {
        let rule = range(18u8, 120u8);
        let result = rule.apply(&121);
        assert!(!result.is_empty());
        assert_eq!(result.violations[0].code, "out_of_range");
    }

    #[test]
    fn test_range_i32() {
        let rule = range(-10i32, 10i32);
        assert!(rule.apply(&0).is_empty());
        assert!(rule.apply(&-10).is_empty());
        assert!(rule.apply(&10).is_empty());

        let result = rule.apply(&-11);
        assert!(!result.is_empty());

        let result = rule.apply(&11);
        assert!(!result.is_empty());
    }

    #[test]
    fn test_range_f64() {
        let rule = range(0.0f64, 1.0f64);
        assert!(rule.apply(&0.5).is_empty());
        assert!(rule.apply(&0.0).is_empty());
        assert!(rule.apply(&1.0).is_empty());

        let result = rule.apply(&-0.1);
        assert!(!result.is_empty());

        let result = rule.apply(&1.1);
        assert!(!result.is_empty());
    }

    #[test]
    fn test_min_valid() {
        let rule = min(18u8);
        assert!(rule.apply(&18).is_empty());
        assert!(rule.apply(&100).is_empty());
    }

    #[test]
    fn test_min_invalid() {
        let rule = min(18u8);
        let result = rule.apply(&17);
        assert!(!result.is_empty());
        assert_eq!(result.violations[0].code, "below_minimum");
        assert_eq!(result.violations[0].meta.get("min"), Some("18"));
    }

    #[test]
    fn test_max_valid() {
        let rule = max(100u8);
        assert!(rule.apply(&100).is_empty());
        assert!(rule.apply(&50).is_empty());
    }

    #[test]
    fn test_max_invalid() {
        let rule = max(100u8);
        let result = rule.apply(&101);
        assert!(!result.is_empty());
        assert_eq!(result.violations[0].code, "above_maximum");
        assert_eq!(result.violations[0].meta.get("max"), Some("100"));
    }

    #[test]
    fn test_min_and_max_composition() {
        let rule = min(18u8).and(max(120u8));
        assert!(rule.apply(&50).is_empty());

        let result = rule.apply(&17);
        assert_eq!(result.violations.len(), 1);
        assert_eq!(result.violations[0].code, "below_minimum");

        let result = rule.apply(&121);
        assert_eq!(result.violations.len(), 1);
        assert_eq!(result.violations[0].code, "above_maximum");
    }

    #[test]
    fn test_positive_valid() {
        let rule = positive();
        assert!(rule.apply(&1i32).is_empty());
        assert!(rule.apply(&100i32).is_empty());
    }

    #[test]
    fn test_positive_invalid() {
        let rule = positive();

        let result = rule.apply(&0i32);
        assert!(!result.is_empty());
        assert_eq!(result.violations[0].code, "not_positive");

        let result = rule.apply(&-1i32);
        assert!(!result.is_empty());
    }

    #[test]
    fn test_negative_valid() {
        let rule = negative();
        assert!(rule.apply(&-1i32).is_empty());
        assert!(rule.apply(&-100i32).is_empty());
    }

    #[test]
    fn test_negative_invalid() {
        let rule = negative();

        let result = rule.apply(&0i32);
        assert!(!result.is_empty());
        assert_eq!(result.violations[0].code, "not_negative");

        let result = rule.apply(&1i32);
        assert!(!result.is_empty());
    }

    #[test]
    fn test_multiple_of_valid() {
        let rule = multiple_of(5);
        assert!(rule.apply(&10).is_empty());
        assert!(rule.apply(&15).is_empty());
        assert!(rule.apply(&0).is_empty());
        assert!(rule.apply(&-10).is_empty());
    }

    #[test]
    fn test_multiple_of_invalid() {
        let rule = multiple_of(5);

        let result = rule.apply(&7);
        assert!(!result.is_empty());
        assert_eq!(result.violations[0].code, "not_multiple");
        assert_eq!(result.violations[0].meta.get("divisor"), Some("5"));

        let result = rule.apply(&3);
        assert!(!result.is_empty());
    }
}

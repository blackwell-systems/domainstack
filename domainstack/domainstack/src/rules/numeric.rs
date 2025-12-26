use crate::{Rule, RuleContext, ValidationError};

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
///
/// # Generic Type Bounds
/// The generic parameter `T` requires several trait bounds:
/// - `PartialOrd`: Required for comparison operations (`<` and `>`) to check if value is in range
/// - `Copy`: Required for efficient value passing in validation closures (zero-cost, no heap allocation)
/// - `Display`: Required to format boundary values in error messages ("Must be between 18 and 120")
/// - `Send + Sync`: Required for thread-safe rule sharing across async tasks and threads
/// - `'static`: Required for the rule to be stored and used independently of its creation context
pub fn range<T>(min: T, max: T) -> Rule<T>
where
    T: PartialOrd + Copy + std::fmt::Display + Send + Sync + 'static,
{
    Rule::new(move |value: &T, ctx: &RuleContext| {
        if *value < min || *value > max {
            let mut err = ValidationError::single(
                ctx.full_path(),
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
///
/// # Generic Type Bounds
/// See [`range()`] for explanation of generic type bounds.
pub fn min<T>(min: T) -> Rule<T>
where
    T: PartialOrd + Copy + std::fmt::Display + Send + Sync + 'static,
{
    Rule::new(move |value: &T, ctx: &RuleContext| {
        if *value < min {
            let mut err = ValidationError::single(
                ctx.full_path(),
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
///
/// # Generic Type Bounds
/// See [`range()`] for explanation of generic type bounds.
pub fn max<T>(max: T) -> Rule<T>
where
    T: PartialOrd + Copy + std::fmt::Display + Send + Sync + 'static,
{
    Rule::new(move |value: &T, ctx: &RuleContext| {
        if *value > max {
            let mut err = ValidationError::single(
                ctx.full_path(),
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

/// Validates that a numeric value is non-zero.
///
/// Works with any numeric type (signed or unsigned). For unsigned types,
/// this simply checks the value is not zero. For signed types, this allows
/// both positive and negative values.
///
/// # Examples
///
/// ```
/// use domainstack::prelude::*;
///
/// let rule = rules::non_zero();
/// assert!(rule.apply(&1).is_empty());
/// assert!(rule.apply(&-1).is_empty());
/// assert!(!rule.apply(&0).is_empty());
/// ```
///
/// # Error Code
/// - Code: `zero_value`
/// - Message: `"Must be non-zero"`
pub fn non_zero<T>() -> Rule<T>
where
    T: PartialEq + Default + Copy + Send + Sync + 'static,
{
    Rule::new(move |value: &T, ctx: &RuleContext| {
        if *value != T::default() {
            ValidationError::default()
        } else {
            ValidationError::single(ctx.full_path(), "zero_value", "Must be non-zero")
        }
    })
}

/// Validates that a numeric value is positive (greater than zero).
///
/// **Note**: This rule is intended for signed numeric types (i8, i16, i32, i64, i128, f32, f64).
/// While it compiles for unsigned types (u8, u16, etc.), using it with unsigned types
/// is redundant - prefer `non_zero()` instead.
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
    Rule::new(move |value: &T, ctx: &RuleContext| {
        if *value > T::default() {
            ValidationError::default()
        } else {
            ValidationError::single(
                ctx.full_path(),
                "not_positive",
                "Must be positive (greater than zero)",
            )
        }
    })
}

/// Validates that a numeric value is negative (less than zero).
///
/// **Note**: This rule is intended for signed numeric types only (i8, i16, i32, i64, i128, f32, f64).
/// It will compile for unsigned types but will always fail validation (since unsigned
/// types cannot be negative by definition).
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
    Rule::new(move |value: &T, ctx: &RuleContext| {
        if *value < T::default() {
            ValidationError::default()
        } else {
            ValidationError::single(
                ctx.full_path(),
                "not_negative",
                "Must be negative (less than zero)",
            )
        }
    })
}

/// Validates that a floating-point value (f32 or f64) is finite (not NaN or infinity).
///
/// This is crucial for float validation since `NaN` values can slip through
/// comparison-based rules like `range()`, `min()`, and `max()` due to NaN's
/// special comparison semantics (NaN is not less than, greater than, or equal to any value).
///
/// # Examples
///
/// ```
/// use domainstack::prelude::*;
///
/// // Works with f64
/// let rule_f64 = rules::finite();
/// assert!(rule_f64.apply(&1.0f64).is_empty());
/// assert!(rule_f64.apply(&-100.5).is_empty());
/// assert!(rule_f64.apply(&0.0).is_empty());
/// assert!(!rule_f64.apply(&f64::NAN).is_empty());
/// assert!(!rule_f64.apply(&f64::INFINITY).is_empty());
///
/// // Works with f32
/// let rule_f32: domainstack::Rule<f32> = rules::finite();
/// assert!(rule_f32.apply(&1.0f32).is_empty());
/// assert!(!rule_f32.apply(&f32::NAN).is_empty());
/// ```
///
/// # Error Code
/// - Code: `not_finite`
/// - Message: `"Must be a finite number (not NaN or infinity)"`
///
/// # Recommended Usage
///
/// Always combine `finite()` with range/min/max checks for robust float validation:
///
/// ```
/// use domainstack::prelude::*;
///
/// let rule = rules::finite().and(rules::range(0.0, 1.0));
/// assert!(rule.apply(&0.5).is_empty());
/// assert!(!rule.apply(&f64::NAN).is_empty());
/// assert!(!rule.apply(&1.5).is_empty()); // out of range
/// ```
pub fn finite<T>() -> Rule<T>
where
    T: FiniteCheck + Copy + Send + Sync + 'static,
{
    Rule::new(move |value: &T, ctx: &RuleContext| {
        if value.is_finite_value() {
            ValidationError::default()
        } else {
            ValidationError::single(
                ctx.full_path(),
                "not_finite",
                "Must be a finite number (not NaN or infinity)",
            )
        }
    })
}

/// Helper trait for checking if a value is finite
pub trait FiniteCheck {
    fn is_finite_value(&self) -> bool;
}

impl FiniteCheck for f32 {
    fn is_finite_value(&self) -> bool {
        self.is_finite()
    }
}

impl FiniteCheck for f64 {
    fn is_finite_value(&self) -> bool {
        self.is_finite()
    }
}

/// Validates that a floating-point value is within the specified range (inclusive).
///
/// Unlike [`range()`], this function also checks that the value is finite (not NaN or infinity).
/// This prevents NaN values from bypassing range validation due to NaN's special comparison semantics.
///
/// # Examples
///
/// ```
/// use domainstack::prelude::*;
///
/// let rule = rules::float_range(0.0f64, 1.0);
/// assert!(rule.apply(&0.5).is_empty());
/// assert!(!rule.apply(&1.5).is_empty());      // out of range
/// assert!(!rule.apply(&f64::NAN).is_empty()); // NaN rejected
/// assert!(!rule.apply(&f64::INFINITY).is_empty()); // infinity rejected
/// ```
///
/// # Error Codes
/// - Code: `not_finite` if NaN or infinity
/// - Code: `out_of_range` if not within bounds
pub fn float_range<T>(min: T, max: T) -> Rule<T>
where
    T: PartialOrd + Copy + std::fmt::Display + Send + Sync + 'static + FiniteCheck,
{
    finite().and(range(min, max))
}

/// Validates that a floating-point value is at least the minimum.
///
/// Unlike [`min()`], this function also checks that the value is finite (not NaN or infinity).
///
/// # Examples
///
/// ```
/// use domainstack::prelude::*;
///
/// let rule = rules::float_min(0.0f64);
/// assert!(rule.apply(&0.0).is_empty());
/// assert!(rule.apply(&100.0).is_empty());
/// assert!(!rule.apply(&-1.0).is_empty());
/// assert!(!rule.apply(&f64::NAN).is_empty());
/// ```
///
/// # Error Codes
/// - Code: `not_finite` if NaN or infinity
/// - Code: `below_minimum` if less than minimum
pub fn float_min<T>(minimum: T) -> Rule<T>
where
    T: PartialOrd + Copy + std::fmt::Display + Send + Sync + 'static + FiniteCheck,
{
    finite().and(min(minimum))
}

/// Validates that a floating-point value does not exceed the maximum.
///
/// Unlike [`max()`], this function also checks that the value is finite (not NaN or infinity).
///
/// # Examples
///
/// ```
/// use domainstack::prelude::*;
///
/// let rule = rules::float_max(100.0f64);
/// assert!(rule.apply(&50.0).is_empty());
/// assert!(rule.apply(&100.0).is_empty());
/// assert!(!rule.apply(&101.0).is_empty());
/// assert!(!rule.apply(&f64::NAN).is_empty());
/// ```
///
/// # Error Codes
/// - Code: `not_finite` if NaN or infinity
/// - Code: `above_maximum` if greater than maximum
pub fn float_max<T>(maximum: T) -> Rule<T>
where
    T: PartialOrd + Copy + std::fmt::Display + Send + Sync + 'static + FiniteCheck,
{
    finite().and(max(maximum))
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
///
/// # Panics
/// Panics at validation time if divisor is zero. Use [`try_multiple_of`] for a
/// version that validates the divisor at construction time.
pub fn multiple_of<T>(divisor: T) -> Rule<T>
where
    T: std::ops::Rem<Output = T>
        + PartialEq
        + Default
        + Copy
        + std::fmt::Display
        + Send
        + Sync
        + 'static,
{
    // Check for zero divisor at construction time
    assert!(
        divisor != T::default(),
        "multiple_of: divisor cannot be zero"
    );

    Rule::new(move |value: &T, ctx: &RuleContext| {
        if *value % divisor == T::default() {
            ValidationError::default()
        } else {
            let mut err = ValidationError::single(
                ctx.full_path(),
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

/// Validates that a numeric value is a multiple of the specified number (non-panicking version).
///
/// Unlike [`multiple_of`], this function returns `None` if the divisor is zero instead of panicking.
///
/// # Examples
///
/// ```
/// use domainstack::prelude::*;
///
/// // Valid divisor
/// let rule = rules::try_multiple_of(5).unwrap();
/// assert!(rule.apply(&10).is_empty());
///
/// // Zero divisor returns None
/// let result = rules::try_multiple_of(0);
/// assert!(result.is_none());
/// ```
///
/// # Returns
/// - `Some(Rule)` if divisor is non-zero
/// - `None` if divisor is zero
pub fn try_multiple_of<T>(divisor: T) -> Option<Rule<T>>
where
    T: std::ops::Rem<Output = T>
        + PartialEq
        + Default
        + Copy
        + std::fmt::Display
        + Send
        + Sync
        + 'static,
{
    if divisor == T::default() {
        return None;
    }

    Some(Rule::new(move |value: &T, ctx: &RuleContext| {
        if *value % divisor == T::default() {
            ValidationError::default()
        } else {
            let mut err = ValidationError::single(
                ctx.full_path(),
                "not_multiple",
                format!("Must be a multiple of {}", divisor),
            );
            err.violations[0]
                .meta
                .insert("divisor", divisor.to_string());
            err
        }
    }))
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

    #[test]
    fn test_non_zero_valid_signed() {
        let rule = non_zero();
        assert!(rule.apply(&1i32).is_empty());
        assert!(rule.apply(&-1i32).is_empty());
        assert!(rule.apply(&100i32).is_empty());
    }

    #[test]
    fn test_non_zero_valid_unsigned() {
        let rule = non_zero();
        assert!(rule.apply(&1u8).is_empty());
        assert!(rule.apply(&100u8).is_empty());
    }

    #[test]
    fn test_non_zero_invalid() {
        let rule = non_zero();

        let result = rule.apply(&0i32);
        assert!(!result.is_empty());
        assert_eq!(result.violations[0].code, "zero_value");
    }

    #[test]
    fn test_non_zero_invalid_unsigned() {
        let rule = non_zero();

        let result = rule.apply(&0u8);
        assert!(!result.is_empty());
        assert_eq!(result.violations[0].code, "zero_value");
    }

    #[test]
    fn test_finite_f64_valid() {
        let rule = finite();
        assert!(rule.apply(&0.0f64).is_empty());
        assert!(rule.apply(&1.5).is_empty());
        assert!(rule.apply(&-100.5).is_empty());
        assert!(rule.apply(&f64::MIN).is_empty());
        assert!(rule.apply(&f64::MAX).is_empty());
    }

    #[test]
    fn test_finite_f64_invalid() {
        let rule = finite();

        let result = rule.apply(&f64::NAN);
        assert!(!result.is_empty());
        assert_eq!(result.violations[0].code, "not_finite");

        let result = rule.apply(&f64::INFINITY);
        assert!(!result.is_empty());

        let result = rule.apply(&f64::NEG_INFINITY);
        assert!(!result.is_empty());
    }

    #[test]
    fn test_finite_f32_valid() {
        let rule: Rule<f32> = finite();
        assert!(rule.apply(&0.0f32).is_empty());
        assert!(rule.apply(&1.5f32).is_empty());
        assert!(rule.apply(&-100.5f32).is_empty());
    }

    #[test]
    fn test_finite_f32_invalid() {
        let rule: Rule<f32> = finite();

        let result = rule.apply(&f32::NAN);
        assert!(!result.is_empty());
        assert_eq!(result.violations[0].code, "not_finite");

        let result = rule.apply(&f32::INFINITY);
        assert!(!result.is_empty());
    }

    #[test]
    fn test_finite_with_range() {
        // This demonstrates the proper way to validate floats
        let rule = finite().and(range(0.0, 1.0));

        // Valid values
        assert!(rule.apply(&0.5).is_empty());
        assert!(rule.apply(&0.0).is_empty());
        assert!(rule.apply(&1.0).is_empty());

        // NaN should fail finite check
        let result = rule.apply(&f64::NAN);
        assert_eq!(result.violations.len(), 1);
        assert_eq!(result.violations[0].code, "not_finite");

        // Out of range should fail range check
        let result = rule.apply(&1.5);
        assert_eq!(result.violations.len(), 1);
        assert_eq!(result.violations[0].code, "out_of_range");
    }

    #[test]
    fn test_float_range() {
        let rule = float_range(0.0f64, 1.0);

        // Valid values
        assert!(rule.apply(&0.5).is_empty());
        assert!(rule.apply(&0.0).is_empty());
        assert!(rule.apply(&1.0).is_empty());

        // Out of range
        assert!(!rule.apply(&1.5).is_empty());
        assert!(!rule.apply(&-0.1).is_empty());

        // NaN and infinity rejected
        assert!(!rule.apply(&f64::NAN).is_empty());
        assert!(!rule.apply(&f64::INFINITY).is_empty());
        assert!(!rule.apply(&f64::NEG_INFINITY).is_empty());
    }

    #[test]
    fn test_float_min() {
        let rule = float_min(0.0f64);

        assert!(rule.apply(&0.0).is_empty());
        assert!(rule.apply(&100.0).is_empty());
        assert!(!rule.apply(&-1.0).is_empty());
        assert!(!rule.apply(&f64::NAN).is_empty());
        assert!(!rule.apply(&f64::NEG_INFINITY).is_empty());
    }

    #[test]
    fn test_float_max() {
        let rule = float_max(100.0f64);

        assert!(rule.apply(&50.0).is_empty());
        assert!(rule.apply(&100.0).is_empty());
        assert!(!rule.apply(&101.0).is_empty());
        assert!(!rule.apply(&f64::NAN).is_empty());
        assert!(!rule.apply(&f64::INFINITY).is_empty());
    }

    #[test]
    fn test_try_multiple_of_valid() {
        let rule = try_multiple_of(5).unwrap();
        assert!(rule.apply(&10).is_empty());
        assert!(rule.apply(&15).is_empty());
        assert!(!rule.apply(&7).is_empty());
    }

    #[test]
    fn test_try_multiple_of_zero_divisor() {
        // Zero divisor returns None
        let result = try_multiple_of(0);
        assert!(result.is_none());
    }

    #[test]
    #[should_panic(expected = "multiple_of: divisor cannot be zero")]
    fn test_multiple_of_zero_divisor_panics() {
        let _rule = multiple_of(0);
    }

    // Additional float edge cases
    #[test]
    fn test_range_min_equals_max() {
        // Single-value range (exact value required)
        let rule = range(5i32, 5i32);

        assert!(rule.apply(&5).is_empty());
        assert!(!rule.apply(&4).is_empty());
        assert!(!rule.apply(&6).is_empty());
    }

    #[test]
    fn test_range_zero_crossing() {
        let rule = range(-10i32, 10i32);

        assert!(rule.apply(&0).is_empty());
        assert!(rule.apply(&-10).is_empty());
        assert!(rule.apply(&10).is_empty());
        assert!(!rule.apply(&-11).is_empty());
        assert!(!rule.apply(&11).is_empty());
    }

    #[test]
    fn test_finite_min_positive() {
        let rule: Rule<f64> = finite();

        // Smallest positive normal number
        assert!(rule.apply(&f64::MIN_POSITIVE).is_empty());
    }

    #[test]
    fn test_float_range_very_small_decimals() {
        let rule = float_range(0.0f64, 0.001f64);

        assert!(rule.apply(&0.0005).is_empty());
        assert!(rule.apply(&0.0).is_empty());
        assert!(rule.apply(&0.001).is_empty());
        assert!(!rule.apply(&0.002).is_empty());
    }

    #[test]
    fn test_float_min_with_negative() {
        let rule = float_min(-100.5f64);

        assert!(rule.apply(&-100.5).is_empty());
        assert!(rule.apply(&0.0).is_empty());
        assert!(!rule.apply(&-100.6).is_empty());
    }

    #[test]
    fn test_float_max_with_negative() {
        let rule = float_max(-0.1f64);

        assert!(rule.apply(&-0.1).is_empty());
        assert!(rule.apply(&-100.0).is_empty());
        assert!(!rule.apply(&0.0).is_empty());
    }

    #[test]
    fn test_negative_infinity_specific() {
        let rule: Rule<f64> = finite();

        let result = rule.apply(&f64::NEG_INFINITY);
        assert!(!result.is_empty());
        assert_eq!(result.violations[0].code, "not_finite");
    }

    #[test]
    fn test_multiple_of_negative_divisor() {
        // Negative divisor should work
        let rule = multiple_of(-5);

        assert!(rule.apply(&10).is_empty());
        assert!(rule.apply(&-10).is_empty());
        assert!(rule.apply(&0).is_empty());
        assert!(!rule.apply(&7).is_empty());
    }

    #[test]
    fn test_multiple_of_one() {
        // Every integer is multiple of 1
        let rule = multiple_of(1);

        assert!(rule.apply(&0).is_empty());
        assert!(rule.apply(&1).is_empty());
        assert!(rule.apply(&-1).is_empty());
        assert!(rule.apply(&1000).is_empty());
    }

    #[test]
    fn test_range_f64_precision() {
        // Test with values that might have precision issues
        let rule = range(0.0f64, 1.0f64);

        // 0.1 + 0.2 ≈ 0.30000000000000004 (IEEE 754)
        let sum = 0.1 + 0.2;
        assert!(rule.apply(&sum).is_empty());
    }

    #[test]
    fn test_min_max_message_format() {
        let rule = min(10i32);
        let result = rule.apply(&5);
        assert!(result.violations[0].message.contains("10"));

        let rule = max(10i32);
        let result = rule.apply(&15);
        assert!(result.violations[0].message.contains("10"));
    }

    #[test]
    fn test_range_message_format() {
        let rule = range(10i32, 20i32);
        let result = rule.apply(&5);

        assert!(result.violations[0].message.contains("10"));
        assert!(result.violations[0].message.contains("20"));
    }

    #[test]
    fn test_positive_negative_with_float() {
        let rule_pos: Rule<f64> = positive();
        let rule_neg: Rule<f64> = negative();

        assert!(rule_pos.apply(&0.001).is_empty());
        assert!(!rule_pos.apply(&-0.001).is_empty());
        assert!(!rule_pos.apply(&0.0).is_empty());

        assert!(rule_neg.apply(&-0.001).is_empty());
        assert!(!rule_neg.apply(&0.001).is_empty());
        assert!(!rule_neg.apply(&0.0).is_empty());
    }

    #[test]
    fn test_non_zero_with_float() {
        let rule: Rule<f64> = non_zero();

        assert!(rule.apply(&0.001).is_empty());
        assert!(rule.apply(&-0.001).is_empty());

        let result = rule.apply(&0.0);
        assert!(!result.is_empty());
        assert_eq!(result.violations[0].code, "zero_value");
    }

    #[test]
    fn test_float_range_f32() {
        let rule: Rule<f32> = float_range(0.0f32, 1.0f32);

        assert!(rule.apply(&0.5f32).is_empty());
        assert!(!rule.apply(&f32::NAN).is_empty());
        assert!(!rule.apply(&f32::INFINITY).is_empty());
    }
}

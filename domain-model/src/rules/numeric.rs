use crate::{Path, Rule, ValidationError};

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
}

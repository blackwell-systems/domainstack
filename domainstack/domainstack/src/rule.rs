use crate::{Path, ValidationError};
use std::sync::Arc;

pub struct Rule<T: ?Sized> {
    inner: Arc<dyn Fn(&T) -> ValidationError + Send + Sync>,
}

impl<T: ?Sized> Clone for Rule<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl<T: ?Sized + 'static> Rule<T> {
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(&T) -> ValidationError + Send + Sync + 'static,
    {
        Self { inner: Arc::new(f) }
    }

    pub fn apply(&self, value: &T) -> ValidationError {
        (self.inner)(value)
    }

    pub fn and(self, other: Rule<T>) -> Rule<T> {
        Rule::new(move |value| {
            let mut err = self.apply(value);
            err.extend(other.apply(value));
            err
        })
    }

    pub fn or(self, other: Rule<T>) -> Rule<T> {
        Rule::new(move |value| {
            let err1 = self.apply(value);
            if err1.is_empty() {
                return err1;
            }
            let err2 = other.apply(value);
            if err2.is_empty() {
                return err2;
            }
            let mut combined = err1;
            combined.extend(err2);
            combined
        })
    }

    pub fn not(self, code: &'static str, message: &'static str) -> Rule<T> {
        Rule::new(move |value| {
            let err = self.apply(value);
            if err.is_empty() {
                ValidationError::single(Path::root(), code, message)
            } else {
                ValidationError::default()
            }
        })
    }

    pub fn map_path(self, prefix: impl Into<Path> + Clone + Send + Sync + 'static) -> Rule<T> {
        Rule::new(move |value| {
            let err = self.apply(value);
            if err.is_empty() {
                return err;
            }
            let mut prefixed = ValidationError::default();
            prefixed.merge_prefixed(prefix.clone(), err);
            prefixed
        })
    }

    pub fn when<F>(self, predicate: F) -> Rule<T>
    where
        F: Fn() -> bool + Send + Sync + 'static,
    {
        Rule::new(move |value| {
            if predicate() {
                self.apply(value)
            } else {
                ValidationError::default()
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn positive_rule() -> Rule<i32> {
        Rule::new(|value: &i32| {
            if *value >= 0 {
                ValidationError::default()
            } else {
                ValidationError::single(Path::root(), "negative", "Must be positive")
            }
        })
    }

    fn even_rule() -> Rule<i32> {
        Rule::new(|value: &i32| {
            if *value % 2 == 0 {
                ValidationError::default()
            } else {
                ValidationError::single(Path::root(), "odd", "Must be even")
            }
        })
    }

    #[test]
    fn test_rule_new_and_apply() {
        let rule = positive_rule();

        let result = rule.apply(&5);
        assert!(result.is_empty());

        let result = rule.apply(&-5);
        assert!(!result.is_empty());
        assert_eq!(result.violations.len(), 1);
        assert_eq!(result.violations[0].code, "negative");
    }

    #[test]
    fn test_rule_and_both_pass() {
        let rule = positive_rule().and(even_rule());

        let result = rule.apply(&4);
        assert!(result.is_empty());
    }

    #[test]
    fn test_rule_and_first_fails() {
        let rule = positive_rule().and(even_rule());

        let result = rule.apply(&-2);
        assert_eq!(result.violations.len(), 1);
        assert_eq!(result.violations[0].code, "negative");
    }

    #[test]
    fn test_rule_and_second_fails() {
        let rule = positive_rule().and(even_rule());

        let result = rule.apply(&3);
        assert_eq!(result.violations.len(), 1);
        assert_eq!(result.violations[0].code, "odd");
    }

    #[test]
    fn test_rule_and_both_fail() {
        let rule = positive_rule().and(even_rule());

        let result = rule.apply(&-3);
        assert_eq!(result.violations.len(), 2);
    }

    #[test]
    fn test_rule_or_both_pass() {
        let rule = positive_rule().or(even_rule());

        let result = rule.apply(&4);
        assert!(result.is_empty());
    }

    #[test]
    fn test_rule_or_first_passes() {
        let rule = positive_rule().or(even_rule());

        let result = rule.apply(&3);
        assert!(result.is_empty());
    }

    #[test]
    fn test_rule_or_second_passes() {
        let rule = positive_rule().or(even_rule());

        let result = rule.apply(&-2);
        assert!(result.is_empty());
    }

    #[test]
    fn test_rule_or_both_fail() {
        let rule = positive_rule().or(even_rule());

        let result = rule.apply(&-3);
        assert_eq!(result.violations.len(), 2);
    }

    #[test]
    fn test_rule_not() {
        let rule = positive_rule().not("not_positive", "Must not be positive");

        let result = rule.apply(&-5);
        assert!(result.is_empty());

        let result = rule.apply(&5);
        assert_eq!(result.violations.len(), 1);
        assert_eq!(result.violations[0].code, "not_positive");
    }

    #[test]
    fn test_rule_map_path() {
        let rule = positive_rule().map_path("value");

        let result = rule.apply(&-5);
        assert_eq!(result.violations.len(), 1);
        assert_eq!(result.violations[0].path.to_string(), "value");
    }

    #[test]
    fn test_rule_when_true() {
        let rule = positive_rule().when(|| true);

        let result = rule.apply(&-5);
        assert!(!result.is_empty());
    }

    #[test]
    fn test_rule_when_false() {
        let rule = positive_rule().when(|| false);

        let result = rule.apply(&-5);
        assert!(result.is_empty());
    }

    #[test]
    fn test_rule_clone() {
        let rule1 = positive_rule();
        let rule2 = rule1.clone();

        let result1 = rule1.apply(&5);
        let result2 = rule2.apply(&5);

        assert_eq!(result1.is_empty(), result2.is_empty());
    }
}

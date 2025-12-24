use crate::{Path, RuleContext, ValidationError};
use std::sync::Arc;

type RuleFn<T> = Arc<dyn Fn(&T, &RuleContext) -> ValidationError + Send + Sync>;

/// A composable validation rule for values of type `T`.
///
/// Rules are the building blocks of domainstack's validation system. They can be composed
/// using `and()`, `or()`, `not()`, and `when()` to create complex validation logic.
///
/// Rules now receive a `RuleContext` providing field information for better error messages.
///
/// # Examples
///
/// ## Basic Rule
///
/// ```
/// use domainstack::prelude::*;
///
/// let rule = rules::min_len(3);
/// let ctx = RuleContext::root("username");
/// assert!(rule.apply_with_context("alice", &ctx).is_empty());
/// assert!(!rule.apply_with_context("ab", &ctx).is_empty());
/// ```
///
/// ## Composing Rules
///
/// ```
/// use domainstack::prelude::*;
///
/// // Username must be 3-20 characters and alphanumeric
/// let rule = rules::min_len(3)
///     .and(rules::max_len(20))
///     .and(rules::alphanumeric());
///
/// let ctx = RuleContext::root("username");
/// assert!(rule.apply_with_context("alice123", &ctx).is_empty());
/// assert!(!rule.apply_with_context("ab", &ctx).is_empty());  // too short
/// ```
///
/// ## Custom Rules with Context
///
/// ```
/// use domainstack::{Rule, RuleContext, ValidationError, Path};
///
/// fn lowercase_only() -> Rule<str> {
///     Rule::new(|value: &str, ctx: &RuleContext| {
///         if value.chars().all(|c| c.is_lowercase() || !c.is_alphabetic()) {
///             ValidationError::default()
///         } else {
///             ValidationError::single(
///                 ctx.full_path(),
///                 "not_lowercase",
///                 "Must contain only lowercase letters"
///             )
///         }
///     })
/// }
///
/// let rule = lowercase_only();
/// let ctx = RuleContext::root("username");
/// assert!(rule.apply_with_context("hello", &ctx).is_empty());
/// assert!(!rule.apply_with_context("Hello", &ctx).is_empty());
/// ```
pub struct Rule<T: ?Sized> {
    inner: RuleFn<T>,
}

impl<T: ?Sized> Clone for Rule<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl<T: ?Sized + 'static> Rule<T> {
    /// Creates a new validation rule.
    ///
    /// Rules receive both the value to validate and a `RuleContext` providing
    /// field information for better error messages.
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(&T, &RuleContext) -> ValidationError + Send + Sync + 'static,
    {
        Self { inner: Arc::new(f) }
    }

    /// Applies the rule with an anonymous context.
    ///
    /// For better error messages, use `apply_with_context()` instead.
    pub fn apply(&self, value: &T) -> ValidationError {
        self.apply_with_context(value, &RuleContext::anonymous())
    }

    /// Applies the rule with a specific context for field-aware error messages.
    pub fn apply_with_context(&self, value: &T, ctx: &RuleContext) -> ValidationError {
        (self.inner)(value, ctx)
    }

    /// Customize the error code for validation failures.
    ///
    /// # Examples
    ///
    /// ```
    /// use domainstack::prelude::*;
    ///
    /// let rule = rules::min_len(5).code("email_too_short");
    /// let err = rule.apply("hi");
    /// assert_eq!(err.violations[0].code, "email_too_short");
    /// ```
    pub fn code(self, code: &'static str) -> Rule<T> {
        Rule::new(move |value: &T, ctx: &RuleContext| {
            let mut err = self.apply_with_context(value, ctx);
            for violation in &mut err.violations {
                violation.code = code;
            }
            err
        })
    }

    /// Customize the error message for validation failures.
    ///
    /// # Examples
    ///
    /// ```
    /// use domainstack::prelude::*;
    ///
    /// let rule = rules::min_len(5).message("Email too short");
    /// let err = rule.apply("hi");
    /// assert_eq!(err.violations[0].message, "Email too short");
    /// ```
    pub fn message(self, msg: impl Into<String> + Clone + Send + Sync + 'static) -> Rule<T> {
        Rule::new(move |value: &T, ctx: &RuleContext| {
            let mut err = self.apply_with_context(value, ctx);
            let message = msg.clone().into();
            for violation in &mut err.violations {
                violation.message = message.clone();
            }
            err
        })
    }

    /// Add metadata to validation errors.
    ///
    /// # Examples
    ///
    /// ```
    /// use domainstack::prelude::*;
    ///
    /// let rule = rules::min_len(5)
    ///     .meta("hint", "Use at least 5 characters");
    ///
    /// let err = rule.apply("hi");
    /// assert_eq!(err.violations[0].meta.get("hint"), Some("Use at least 5 characters"));
    /// ```
    pub fn meta(
        self,
        key: &'static str,
        value: impl Into<String> + Clone + Send + Sync + 'static,
    ) -> Rule<T> {
        Rule::new(move |val: &T, ctx: &RuleContext| {
            let mut err = self.apply_with_context(val, ctx);
            let v = value.clone().into();
            for violation in &mut err.violations {
                violation.meta.insert(key, v.clone());
            }
            err
        })
    }

    pub fn and(self, other: Rule<T>) -> Rule<T> {
        Rule::new(move |value, ctx| {
            let mut err = self.apply_with_context(value, ctx);
            err.extend(other.apply_with_context(value, ctx));
            err
        })
    }

    pub fn or(self, other: Rule<T>) -> Rule<T> {
        Rule::new(move |value, ctx| {
            let err1 = self.apply_with_context(value, ctx);
            if err1.is_empty() {
                return err1;
            }
            let err2 = other.apply_with_context(value, ctx);
            if err2.is_empty() {
                return err2;
            }
            let mut combined = err1;
            combined.extend(err2);
            combined
        })
    }

    pub fn not(self, code: &'static str, message: &'static str) -> Rule<T> {
        Rule::new(move |value, ctx| {
            let err = self.apply_with_context(value, ctx);
            if err.is_empty() {
                ValidationError::single(ctx.full_path(), code, message)
            } else {
                ValidationError::default()
            }
        })
    }

    pub fn map_path(self, prefix: impl Into<Path> + Clone + Send + Sync + 'static) -> Rule<T> {
        Rule::new(move |value, ctx| {
            let err = self.apply_with_context(value, ctx);
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
        Rule::new(move |value, ctx| {
            if predicate() {
                self.apply_with_context(value, ctx)
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
        Rule::new(|value: &i32, ctx: &RuleContext| {
            if *value >= 0 {
                ValidationError::default()
            } else {
                ValidationError::single(ctx.full_path(), "negative", "Must be positive")
            }
        })
    }

    fn even_rule() -> Rule<i32> {
        Rule::new(|value: &i32, ctx: &RuleContext| {
            if *value % 2 == 0 {
                ValidationError::default()
            } else {
                ValidationError::single(ctx.full_path(), "odd", "Must be even")
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

use crate::Path;
use std::sync::Arc;

/// Context information available to validation rules.
///
/// RuleContext provides rules with information about the field being validated,
/// enabling more helpful, context-aware error messages.
///
/// # Examples
///
/// ```
/// use domainstack::{Rule, RuleContext, ValidationError, Path};
///
/// fn min_len_with_context(min: usize) -> Rule<str> {
///     Rule::new(move |value: &str, ctx: &RuleContext| {
///         if value.len() < min {
///             ValidationError::single(
///                 ctx.parent_path.clone(),
///                 "min_length",
///                 format!(
///                     "Field '{}' must be at least {} characters (got {})",
///                     ctx.field_name.as_ref().map(|s| s.as_ref()).unwrap_or("unknown"),
///                     min,
///                     value.len()
///                 )
///             )
///         } else {
///             ValidationError::default()
///         }
///     })
/// }
/// ```
#[derive(Debug, Clone)]
pub struct RuleContext {
    /// The name of the field being validated, if known.
    ///
    /// This is typically provided by the derive macro or when using the `validate()` helper.
    /// For ad-hoc validation, this may be `None`.
    pub field_name: Option<Arc<str>>,

    /// The path from the root to the parent of the current field.
    ///
    /// For example, when validating `user.email`, the parent_path would be `"user"`.
    /// Rules should typically append their field_name to this path when creating errors.
    pub parent_path: Path,

    /// Optional string representation of the current value for debugging.
    ///
    /// This can be included in error messages to help users understand what value failed validation.
    /// For sensitive fields (passwords, tokens), this should be `None`.
    pub value_debug: Option<String>,
}

impl RuleContext {
    /// Creates a new RuleContext for a root-level field.
    ///
    /// # Examples
    ///
    /// ```
    /// use domainstack::RuleContext;
    ///
    /// let ctx = RuleContext::root("email");
    /// assert_eq!(ctx.field_name, Some("email".into()));
    /// assert_eq!(ctx.parent_path.to_string(), "");
    /// ```
    pub fn root(field_name: impl Into<Arc<str>>) -> Self {
        Self {
            field_name: Some(field_name.into()),
            parent_path: Path::root(),
            value_debug: None,
        }
    }

    /// Creates a new RuleContext without a field name.
    ///
    /// Useful for ad-hoc validation where field information isn't available.
    ///
    /// # Examples
    ///
    /// ```
    /// use domainstack::RuleContext;
    ///
    /// let ctx = RuleContext::anonymous();
    /// assert_eq!(ctx.field_name, None);
    /// ```
    pub fn anonymous() -> Self {
        Self {
            field_name: None,
            parent_path: Path::root(),
            value_debug: None,
        }
    }

    /// Creates a child context for a nested field.
    ///
    /// # Examples
    ///
    /// ```
    /// use domainstack::RuleContext;
    ///
    /// let parent = RuleContext::root("user");
    /// let child = parent.child("email");
    ///
    /// assert_eq!(child.field_name, Some("email".into()));
    /// assert_eq!(child.parent_path.to_string(), "user");
    /// ```
    pub fn child(&self, field_name: impl Into<Arc<str>>) -> Self {
        let parent_path = match &self.field_name {
            Some(name) => self.parent_path.clone().field(name.clone()),
            None => self.parent_path.clone(),
        };

        Self {
            field_name: Some(field_name.into()),
            parent_path,
            value_debug: None,
        }
    }

    /// Sets the debug representation of the value being validated.
    ///
    /// # Examples
    ///
    /// ```
    /// use domainstack::RuleContext;
    ///
    /// let ctx = RuleContext::root("age")
    ///     .with_value_debug("42");
    ///
    /// assert_eq!(ctx.value_debug, Some("42".to_string()));
    /// ```
    pub fn with_value_debug(mut self, value: impl Into<String>) -> Self {
        self.value_debug = Some(value.into());
        self
    }

    /// Gets the full path to the current field.
    ///
    /// Combines parent_path with field_name to produce the complete path.
    ///
    /// # Examples
    ///
    /// ```
    /// use domainstack::{RuleContext, Path};
    ///
    /// let ctx = RuleContext::root("email");
    /// assert_eq!(ctx.full_path(), Path::root().field("email"));
    ///
    /// let parent = RuleContext::root("user");
    /// let child = parent.child("email");
    /// assert_eq!(child.full_path().to_string(), "user.email");
    /// ```
    pub fn full_path(&self) -> Path {
        match &self.field_name {
            Some(name) => self.parent_path.clone().field(name.clone()),
            None => self.parent_path.clone(),
        }
    }
}

impl Default for RuleContext {
    fn default() -> Self {
        Self::anonymous()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_root_context() {
        let ctx = RuleContext::root("username");
        assert_eq!(ctx.field_name, Some("username".into()));
        assert_eq!(ctx.parent_path.to_string(), "");
        assert_eq!(ctx.value_debug, None);
    }

    #[test]
    fn test_anonymous_context() {
        let ctx = RuleContext::anonymous();
        assert_eq!(ctx.field_name, None);
        assert_eq!(ctx.parent_path.to_string(), "");
    }

    #[test]
    fn test_child_context_with_named_parent() {
        let parent = RuleContext::root("user");
        let child = parent.child("email");
        assert_eq!(child.field_name, Some("email".into()));
        assert_eq!(child.parent_path.to_string(), "user");
    }

    #[test]
    fn test_child_context_with_anonymous_parent() {
        let parent = RuleContext::anonymous();
        let child = parent.child("email");
        assert_eq!(child.field_name, Some("email".into()));
        assert_eq!(child.parent_path.to_string(), "");
    }

    #[test]
    fn test_with_value_debug() {
        let ctx = RuleContext::root("age").with_value_debug("42");
        assert_eq!(ctx.value_debug, Some("42".to_string()));
    }

    #[test]
    fn test_full_path_root_field() {
        let ctx = RuleContext::root("email");
        assert_eq!(ctx.full_path().to_string(), "email");
    }

    #[test]
    fn test_full_path_nested_field() {
        let parent = RuleContext::root("user");
        let child = parent.child("email");
        assert_eq!(child.full_path().to_string(), "user.email");
    }

    #[test]
    fn test_full_path_anonymous() {
        let ctx = RuleContext::anonymous();
        assert_eq!(ctx.full_path().to_string(), "");
    }

    #[test]
    fn test_default_context() {
        let ctx = RuleContext::default();
        assert_eq!(ctx.field_name, None);
        assert_eq!(ctx.parent_path.to_string(), "");
    }

    #[test]
    fn test_deeply_nested_context() {
        let root = RuleContext::root("team");
        let member = root.child("members");
        let user = member.child("user");
        let email = user.child("email");
        assert_eq!(email.full_path().to_string(), "team.members.user.email");
    }
}

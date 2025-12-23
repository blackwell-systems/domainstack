use crate::{Meta, Path, Violation};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Default)]
pub struct ValidationError {
    pub violations: Vec<Violation>,
}

impl ValidationError {
    pub fn new() -> Self {
        Self {
            violations: Vec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.violations.is_empty()
    }

    pub fn single(path: impl Into<Path>, code: &'static str, message: impl Into<String>) -> Self {
        let mut err = Self::new();
        err.push(path, code, message);
        err
    }

    pub fn push(&mut self, path: impl Into<Path>, code: &'static str, message: impl Into<String>) {
        self.violations.push(Violation {
            path: path.into(),
            code,
            message: message.into(),
            meta: Meta::default(),
        });
    }

    pub fn extend(&mut self, other: ValidationError) {
        self.violations.extend(other.violations);
    }

    pub fn merge_prefixed(&mut self, prefix: impl Into<Path>, other: ValidationError) {
        let prefix = prefix.into();
        for mut violation in other.violations {
            let mut new_segments = prefix.0.clone();
            new_segments.extend(violation.path.0);
            violation.path = Path(new_segments);
            self.violations.push(violation);
        }
    }

    pub fn field_errors_map(&self) -> BTreeMap<String, Vec<String>> {
        let mut map = BTreeMap::new();
        for violation in &self.violations {
            map.entry(violation.path.to_string())
                .or_insert_with(Vec::new)
                .push(violation.message.clone());
        }
        map
    }

    pub fn prefixed(self, prefix: impl Into<Path>) -> Self {
        let prefix = prefix.into();
        let violations = self
            .violations
            .into_iter()
            .map(|mut v| {
                let mut segments = prefix.0.clone();
                segments.extend(v.path.0);
                v.path = Path(segments);
                v
            })
            .collect();

        Self { violations }
    }
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.violations.is_empty() {
            write!(f, "No validation errors")
        } else if self.violations.len() == 1 {
            write!(f, "Validation error: {}", self.violations[0].message)
        } else {
            write!(f, "Validation failed with {} errors", self.violations.len())
        }
    }
}

impl std::error::Error for ValidationError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let err = ValidationError::new();
        assert!(err.is_empty());
        assert_eq!(err.violations.len(), 0);
    }

    #[test]
    fn test_default() {
        let err = ValidationError::default();
        assert!(err.is_empty());
    }

    #[test]
    fn test_is_empty() {
        let mut err = ValidationError::new();
        assert!(err.is_empty());

        err.push("email", "invalid", "Invalid email");
        assert!(!err.is_empty());
    }

    #[test]
    fn test_single() {
        let err = ValidationError::single("email", "invalid_email", "Invalid email format");

        assert!(!err.is_empty());
        assert_eq!(err.violations.len(), 1);
        assert_eq!(err.violations[0].code, "invalid_email");
        assert_eq!(err.violations[0].message, "Invalid email format");
        assert_eq!(err.violations[0].path.to_string(), "email");
    }

    #[test]
    fn test_push() {
        let mut err = ValidationError::new();
        err.push("email", "invalid_email", "Invalid email");
        err.push("age", "out_of_range", "Age out of range");

        assert_eq!(err.violations.len(), 2);
        assert_eq!(err.violations[0].path.to_string(), "email");
        assert_eq!(err.violations[1].path.to_string(), "age");
    }

    #[test]
    fn test_extend() {
        let mut err1 = ValidationError::new();
        err1.push("email", "invalid", "Invalid email");

        let mut err2 = ValidationError::new();
        err2.push("age", "invalid", "Invalid age");

        err1.extend(err2);
        assert_eq!(err1.violations.len(), 2);
    }

    #[test]
    fn test_merge_prefixed() {
        let mut parent_err = ValidationError::new();

        let mut child_err = ValidationError::new();
        child_err.push("email", "invalid", "Invalid email");

        parent_err.merge_prefixed("guest", child_err);

        assert_eq!(parent_err.violations.len(), 1);
        assert_eq!(parent_err.violations[0].path.to_string(), "guest.email");
    }

    #[test]
    fn test_merge_prefixed_nested() {
        let mut root_err = ValidationError::new();

        let mut nested_err = ValidationError::new();
        nested_err.push(
            Path::root().field("guests").index(0).field("email"),
            "invalid",
            "Invalid email",
        );

        root_err.merge_prefixed("booking", nested_err);

        assert_eq!(root_err.violations.len(), 1);
        assert_eq!(
            root_err.violations[0].path.to_string(),
            "booking.guests[0].email"
        );
    }

    #[test]
    fn test_field_errors_map() {
        let mut err = ValidationError::new();
        err.push("email", "invalid", "Invalid email");
        err.push("email", "too_long", "Email too long");
        err.push("age", "out_of_range", "Age out of range");

        let map = err.field_errors_map();

        assert_eq!(map.len(), 2);
        assert_eq!(map.get("email").unwrap().len(), 2);
        assert_eq!(map.get("age").unwrap().len(), 1);
        assert!(map
            .get("email")
            .unwrap()
            .contains(&"Invalid email".to_string()));
        assert!(map
            .get("email")
            .unwrap()
            .contains(&"Email too long".to_string()));
    }

    #[test]
    fn test_display_empty() {
        let err = ValidationError::new();
        assert_eq!(err.to_string(), "No validation errors");
    }

    #[test]
    fn test_display_single() {
        let err = ValidationError::single("email", "invalid", "Invalid email format");
        assert_eq!(err.to_string(), "Validation error: Invalid email format");
    }

    #[test]
    fn test_display_multiple() {
        let mut err = ValidationError::new();
        err.push("email", "invalid", "Invalid email");
        err.push("age", "invalid", "Invalid age");

        assert_eq!(err.to_string(), "Validation failed with 2 errors");
    }
}

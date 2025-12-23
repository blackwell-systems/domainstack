use crate::{Path, Rule, ValidationError};

pub fn email() -> Rule<str> {
    Rule::new(|value: &str| {
        #[cfg(feature = "email")]
        {
            let re = regex::Regex::new(r"^[^@\s]+@[^@\s]+\.[^@\s]+$").unwrap();

            if re.is_match(value) {
                ValidationError::default()
            } else {
                ValidationError::single(Path::root(), "invalid_email", "Invalid email format")
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
                ValidationError::single(Path::root(), "invalid_email", "Invalid email format")
            }
        }
    })
}

pub fn non_empty() -> Rule<str> {
    Rule::new(|value: &str| {
        if value.is_empty() {
            ValidationError::single(Path::root(), "non_empty", "Must not be empty")
        } else {
            ValidationError::default()
        }
    })
}

pub fn min_len(min: usize) -> Rule<str> {
    Rule::new(move |value: &str| {
        if value.len() < min {
            let mut err = ValidationError::single(
                Path::root(),
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

pub fn max_len(max: usize) -> Rule<str> {
    Rule::new(move |value: &str| {
        if value.len() > max {
            let mut err = ValidationError::single(
                Path::root(),
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

pub fn length(min: usize, max: usize) -> Rule<str> {
    min_len(min).and(max_len(max))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_valid() {
        let rule = email();
        assert!(rule.apply(&"user@example.com").is_empty());
        assert!(rule.apply(&"test.user@domain.co.uk").is_empty());
    }

    #[test]
    fn test_email_invalid() {
        let rule = email();

        let result = rule.apply(&"not-an-email");
        assert!(!result.is_empty());
        assert_eq!(result.violations[0].code, "invalid_email");

        let result = rule.apply(&"missing-domain@");
        assert!(!result.is_empty());

        let result = rule.apply(&"@missing-user.com");
        assert!(!result.is_empty());
    }

    #[test]
    fn test_non_empty_valid() {
        let rule = non_empty();
        assert!(rule.apply(&"hello").is_empty());
        assert!(rule.apply(&" ").is_empty());
    }

    #[test]
    fn test_non_empty_invalid() {
        let rule = non_empty();
        let result = rule.apply(&"");
        assert!(!result.is_empty());
        assert_eq!(result.violations[0].code, "non_empty");
    }

    #[test]
    fn test_min_len_valid() {
        let rule = min_len(5);
        assert!(rule.apply(&"hello").is_empty());
        assert!(rule.apply(&"hello world").is_empty());
    }

    #[test]
    fn test_min_len_invalid() {
        let rule = min_len(5);
        let result = rule.apply(&"hi");
        assert!(!result.is_empty());
        assert_eq!(result.violations[0].code, "min_length");
        assert_eq!(result.violations[0].meta.get("min"), Some("5"));
    }

    #[test]
    fn test_max_len_valid() {
        let rule = max_len(10);
        assert!(rule.apply(&"hello").is_empty());
        assert!(rule.apply(&"").is_empty());
    }

    #[test]
    fn test_max_len_invalid() {
        let rule = max_len(5);
        let result = rule.apply(&"hello world");
        assert!(!result.is_empty());
        assert_eq!(result.violations[0].code, "max_length");
        assert_eq!(result.violations[0].meta.get("max"), Some("5"));
    }

    #[test]
    fn test_length_valid() {
        let rule = length(3, 10);
        assert!(rule.apply(&"hello").is_empty());
        assert!(rule.apply(&"hi!").is_empty());
    }

    #[test]
    fn test_length_too_short() {
        let rule = length(3, 10);
        let result = rule.apply(&"hi");
        assert!(!result.is_empty());
        assert_eq!(result.violations[0].code, "min_length");
    }

    #[test]
    fn test_length_too_long() {
        let rule = length(3, 10);
        let result = rule.apply(&"hello world!");
        assert!(!result.is_empty());
        assert_eq!(result.violations[0].code, "max_length");
    }
}

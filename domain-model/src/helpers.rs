use crate::{Path, Rule, ValidationError};

pub fn validate<T: ?Sized + 'static>(
    path: impl Into<Path>,
    value: &T,
    rule: Rule<T>,
) -> Result<(), ValidationError> {
    let err = rule.apply(value);

    if err.is_empty() {
        Ok(())
    } else {
        let mut prefixed = ValidationError::default();
        prefixed.merge_prefixed(path, err);
        Err(prefixed)
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

    #[test]
    fn test_validate_ok() {
        let result = validate("value", &5, positive_rule());
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_err() {
        let result = validate("value", &-5, positive_rule());
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert_eq!(err.violations.len(), 1);
        assert_eq!(err.violations[0].path.to_string(), "value");
        assert_eq!(err.violations[0].code, "negative");
    }

    #[test]
    fn test_validate_nested_path() {
        let result = validate(
            Path::root().field("guest").field("age"),
            &-5,
            positive_rule(),
        );

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.violations[0].path.to_string(), "guest.age");
    }
}

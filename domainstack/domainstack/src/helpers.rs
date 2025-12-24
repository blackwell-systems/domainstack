use crate::{Path, Rule, RuleContext, ValidationError};

pub fn validate<T: ?Sized + 'static>(
    path: impl Into<Path>,
    value: &T,
    rule: &Rule<T>,
) -> Result<(), ValidationError> {
    let path = path.into();
    // Extract field name from path if it's a simple field
    let field_name = path.0.last().and_then(|seg| match seg {
        crate::PathSegment::Field(name) => Some(name.clone()),
        _ => None,
    });

    let parent_path = if field_name.is_some() && path.0.len() > 1 {
        Path(path.0[..path.0.len() - 1].to_vec())
    } else if field_name.is_some() {
        Path::root()
    } else {
        path.clone()
    };

    let ctx = RuleContext {
        field_name,
        parent_path,
        value_debug: None,
    };

    let err = rule.apply_with_context(value, &ctx);

    if err.is_empty() {
        Ok(())
    } else {
        // Errors already have the correct path from ctx.full_path(), no need to prefix
        Err(err)
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

    #[test]
    fn test_validate_ok() {
        let result = validate("value", &5, &positive_rule());
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_err() {
        let result = validate("value", &-5, &positive_rule());
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
            &positive_rule(),
        );

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.violations[0].path.to_string(), "guest.age");
    }
}

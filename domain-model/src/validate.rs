use crate::ValidationError;

pub trait Validate {
    fn validate(&self) -> Result<(), ValidationError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestStruct {
        value: i32,
    }

    impl Validate for TestStruct {
        fn validate(&self) -> Result<(), ValidationError> {
            if self.value < 0 {
                Err(ValidationError::single(
                    "value",
                    "negative",
                    "Value must be positive",
                ))
            } else {
                Ok(())
            }
        }
    }

    #[test]
    fn test_validate_trait_ok() {
        let test = TestStruct { value: 10 };
        assert!(test.validate().is_ok());
    }

    #[test]
    fn test_validate_trait_err() {
        let test = TestStruct { value: -5 };
        let result = test.validate();
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert_eq!(err.violations.len(), 1);
        assert_eq!(err.violations[0].code, "negative");
    }
}

use domainstack::prelude::*;
use domainstack_derive::Validate;

fn validate_even(value: &u8) -> Result<(), ValidationError> {
    if *value % 2 == 0 {
        Ok(())
    } else {
        Err(ValidationError::single(
            Path::root(),
            "not_even",
            "Must be even",
        ))
    }
}

#[derive(Debug, Validate)]
struct EvenNumber {
    #[validate(range(min = 0, max = 100))]
    #[validate(custom = "validate_even")]
    value: u8,
}

#[test]
fn test_custom_validation_pass() {
    let num = EvenNumber { value: 42 };
    assert!(num.validate().is_ok());
}

#[test]
fn test_custom_validation_fail() {
    let num = EvenNumber { value: 43 };
    
    let result = num.validate();
    assert!(result.is_err());
    
    let err = result.unwrap_err();
    assert_eq!(err.violations.len(), 1);
    assert_eq!(err.violations[0].path.to_string(), "value");
    assert_eq!(err.violations[0].code, "not_even");
}

#[test]
fn test_custom_with_range_fail() {
    let num = EvenNumber { value: 200 };
    
    let result = num.validate();
    assert!(result.is_err());
    
    let err = result.unwrap_err();
    assert_eq!(err.violations.len(), 1);
    assert_eq!(err.violations[0].code, "out_of_range");
}

#[test]
fn test_both_validations_fail() {
    let num = EvenNumber { value: 101 };
    
    let result = num.validate();
    assert!(result.is_err());
    
    let err = result.unwrap_err();
    assert_eq!(err.violations.len(), 2);
    
    assert_eq!(err.violations[0].code, "out_of_range");
    assert_eq!(err.violations[1].code, "not_even");
}

use domain_model::prelude::*;
use domain_model_derive::Validate;

#[derive(Debug, Validate)]
struct User {
    #[validate(length(min = 1, max = 50))]
    name: String,
    
    #[validate(range(min = 18, max = 120))]
    age: u8,
}

#[test]
fn test_valid_user() {
    let user = User {
        name: "John Doe".to_string(),
        age: 25,
    };
    
    assert!(user.validate().is_ok());
}

#[test]
fn test_invalid_name_too_short() {
    let user = User {
        name: "".to_string(),
        age: 25,
    };
    
    let result = user.validate();
    assert!(result.is_err());
    
    let err = result.unwrap_err();
    assert_eq!(err.violations.len(), 1);
    assert_eq!(err.violations[0].path.to_string(), "name");
    assert_eq!(err.violations[0].code, "min_length");
}

#[test]
fn test_invalid_age() {
    let user = User {
        name: "John".to_string(),
        age: 15,
    };
    
    let result = user.validate();
    assert!(result.is_err());
    
    let err = result.unwrap_err();
    assert_eq!(err.violations.len(), 1);
    assert_eq!(err.violations[0].path.to_string(), "age");
    assert_eq!(err.violations[0].code, "out_of_range");
}

#[test]
fn test_multiple_violations() {
    let user = User {
        name: "".to_string(),
        age: 200,
    };
    
    let result = user.validate();
    assert!(result.is_err());
    
    let err = result.unwrap_err();
    assert_eq!(err.violations.len(), 2);
}

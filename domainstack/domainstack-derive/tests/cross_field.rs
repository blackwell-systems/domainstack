use domainstack::prelude::*;
use domainstack_derive::Validate;

#[derive(Debug, Validate)]
#[validate(
    check = "self.password == self.password_confirmation",
    code = "passwords_mismatch",
    message = "Passwords must match"
)]
struct RegisterForm {
    #[validate(length(min = 8))]
    password: String,

    password_confirmation: String,
}

#[test]
fn test_passwords_match() {
    let form = RegisterForm {
        password: "secure_password".to_string(),
        password_confirmation: "secure_password".to_string(),
    };

    assert!(form.validate().is_ok());
}

#[test]
fn test_passwords_mismatch() {
    let form = RegisterForm {
        password: "password123".to_string(),
        password_confirmation: "different".to_string(),
    };

    let result = form.validate();
    assert!(result.is_err());

    let err = result.unwrap_err();
    assert_eq!(err.violations.len(), 1);
    assert_eq!(err.violations[0].code, "passwords_mismatch");
    assert_eq!(err.violations[0].message, "Passwords must match");
}

#[test]
fn test_field_and_cross_field_validation() {
    let form = RegisterForm {
        password: "short".to_string(),
        password_confirmation: "different".to_string(),
    };

    let result = form.validate();
    assert!(result.is_err());

    let err = result.unwrap_err();
    // Should have both min_length and passwords_mismatch violations
    assert_eq!(err.violations.len(), 2);

    let codes: Vec<&str> = err.violations.iter().map(|v| v.code).collect();
    assert!(codes.contains(&"min_length"));
    assert!(codes.contains(&"passwords_mismatch"));
}

#[derive(Debug, Validate)]
#[validate(
    check = "self.end_date > self.start_date",
    code = "invalid_date_range",
    message = "End date must be after start date"
)]
struct DateRange {
    start_date: u32,
    end_date: u32,
}

#[test]
fn test_valid_date_range() {
    let range = DateRange {
        start_date: 20250101,
        end_date: 20250131,
    };

    assert!(range.validate().is_ok());
}

#[test]
fn test_invalid_date_range() {
    let range = DateRange {
        start_date: 20250131,
        end_date: 20250101,
    };

    let result = range.validate();
    assert!(result.is_err());

    let err = result.unwrap_err();
    assert_eq!(err.violations.len(), 1);
    assert_eq!(err.violations[0].code, "invalid_date_range");
}

#[derive(Debug, Validate)]
#[validate(
    check = "self.discount_code.is_empty() || self.discount_percentage == 0",
    code = "discount_conflict",
    message = "Cannot apply both discount code and percentage discount"
)]
struct Order {
    discount_code: String,
    discount_percentage: u8,
}

#[test]
fn test_no_discount() {
    let order = Order {
        discount_code: String::new(),
        discount_percentage: 0,
    };

    assert!(order.validate().is_ok());
}

#[test]
fn test_code_discount_only() {
    let order = Order {
        discount_code: "SUMMER2025".to_string(),
        discount_percentage: 0,
    };

    assert!(order.validate().is_ok());
}

#[test]
fn test_percentage_discount_only() {
    let order = Order {
        discount_code: String::new(),
        discount_percentage: 10,
    };

    assert!(order.validate().is_ok());
}

#[test]
fn test_conflicting_discounts() {
    let order = Order {
        discount_code: "SUMMER2025".to_string(),
        discount_percentage: 10,
    };

    let result = order.validate();
    assert!(result.is_err());

    let err = result.unwrap_err();
    assert_eq!(err.violations.len(), 1);
    assert_eq!(err.violations[0].code, "discount_conflict");
}

#[derive(Debug, Validate)]
#[validate(
    check = "self.age >= 18",
    code = "underage",
    message = "Must be 18 or older",
    when = "self.requires_age_verification"
)]
struct ConditionalValidation {
    age: u8,
    requires_age_verification: bool,
}

#[test]
fn test_conditional_validation_when_required() {
    let data = ConditionalValidation {
        age: 16,
        requires_age_verification: true,
    };

    let result = data.validate();
    assert!(result.is_err());

    let err = result.unwrap_err();
    assert_eq!(err.violations.len(), 1);
    assert_eq!(err.violations[0].code, "underage");
}

#[test]
fn test_conditional_validation_when_not_required() {
    let data = ConditionalValidation {
        age: 16,
        requires_age_verification: false,
    };

    // Should pass because requires_age_verification is false
    assert!(data.validate().is_ok());
}

#[test]
fn test_conditional_validation_passes() {
    let data = ConditionalValidation {
        age: 21,
        requires_age_verification: true,
    };

    assert!(data.validate().is_ok());
}

#[derive(Debug, Validate)]
#[validate(check = "self.a == self.b", message = "A must equal B")]
#[validate(check = "self.b == self.c", message = "B must equal C")]
struct MultipleChecks {
    a: i32,
    b: i32,
    c: i32,
}

#[test]
fn test_multiple_cross_field_checks_all_pass() {
    let data = MultipleChecks { a: 5, b: 5, c: 5 };

    assert!(data.validate().is_ok());
}

#[test]
fn test_multiple_cross_field_checks_one_fails() {
    let data = MultipleChecks { a: 5, b: 5, c: 6 };

    let result = data.validate();
    assert!(result.is_err());

    let err = result.unwrap_err();
    assert_eq!(err.violations.len(), 1);
    assert_eq!(err.violations[0].message, "B must equal C");
}

#[test]
fn test_multiple_cross_field_checks_all_fail() {
    let data = MultipleChecks { a: 1, b: 2, c: 3 };

    let result = data.validate();
    assert!(result.is_err());

    let err = result.unwrap_err();
    assert_eq!(err.violations.len(), 2);
}

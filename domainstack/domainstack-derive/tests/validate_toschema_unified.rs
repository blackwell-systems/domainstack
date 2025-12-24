//! Test that Validate and ToSchema work together with unified rich syntax

use domainstack::prelude::*;
use domainstack_derive::{ToSchema, Validate};
use domainstack_schema::ToSchema as ToSchemaTrait;

// Test basic usage with both macros
#[derive(Validate, ToSchema)]
#[allow(dead_code)]
struct User {
    #[validate(email)]
    #[validate(max_len = 255)]
    #[schema(description = "User's email", example = "alice@example.com")]
    email: String,

    #[validate(range(min = 18, max = 120))]
    #[schema(description = "User's age")]
    age: u8,

    #[validate(alphanumeric)]
    #[validate(min_len = 3)]
    #[validate(max_len = 20)]
    #[schema(description = "Username")]
    username: String,
}

#[test]
fn test_validation_works() {
    // Valid user
    let valid_user = User {
        email: "alice@example.com".to_string(),
        age: 25,
        username: "alice123".to_string(),
    };
    assert!(valid_user.validate().is_ok());

    // Invalid email
    let invalid_email = User {
        email: "not-an-email".to_string(),
        age: 25,
        username: "alice123".to_string(),
    };
    let err = invalid_email.validate().unwrap_err();
    assert!(!err.violations.is_empty());

    // Age too young
    let too_young = User {
        email: "alice@example.com".to_string(),
        age: 10,
        username: "alice123".to_string(),
    };
    let err = too_young.validate().unwrap_err();
    assert!(!err.violations.is_empty());

    // Username too short
    let short_username = User {
        email: "alice@example.com".to_string(),
        age: 25,
        username: "ab".to_string(),
    };
    let err = short_username.validate().unwrap_err();
    assert!(!err.violations.is_empty());

    // Username not alphanumeric
    let bad_username = User {
        email: "alice@example.com".to_string(),
        age: 25,
        username: "alice-123".to_string(),
    };
    let err = bad_username.validate().unwrap_err();
    assert!(!err.violations.is_empty());
}

#[test]
fn test_schema_generation_works() {
    let schema = User::schema();
    let json = serde_json::to_value(&schema).unwrap();

    // Email should have format and maxLength
    assert_eq!(json["properties"]["email"]["format"], "email");
    assert_eq!(
        json["properties"]["email"]["maxLength"].as_f64(),
        Some(255.0)
    );

    // Age should have minimum and maximum
    assert_eq!(json["properties"]["age"]["minimum"].as_f64(), Some(18.0));
    assert_eq!(json["properties"]["age"]["maximum"].as_f64(), Some(120.0));

    // Username should have pattern and length constraints
    assert_eq!(json["properties"]["username"]["pattern"], "^[a-zA-Z0-9]*$");
    assert_eq!(
        json["properties"]["username"]["minLength"].as_f64(),
        Some(3.0)
    );
    assert_eq!(
        json["properties"]["username"]["maxLength"].as_f64(),
        Some(20.0)
    );

    // All fields required
    assert_eq!(
        json["required"],
        serde_json::json!(["email", "age", "username"])
    );
}

// Test with optional fields - simpler version without validation on Option
#[derive(Validate, ToSchema)]
#[allow(dead_code)]
struct Profile {
    #[validate(email)]
    email: String,

    // Optional field - no validation rules
    nickname: Option<String>,
}

#[test]
fn test_optional_fields() {
    // Valid with nickname
    let with_nickname = Profile {
        email: "bob@example.com".to_string(),
        nickname: Some("Bobby".to_string()),
    };
    assert!(with_nickname.validate().is_ok());

    // Valid without nickname
    let without_nickname = Profile {
        email: "bob@example.com".to_string(),
        nickname: None,
    };
    assert!(without_nickname.validate().is_ok());

    // Schema should exclude optional from required
    let schema = Profile::schema();
    let json = serde_json::to_value(&schema).unwrap();
    assert_eq!(json["required"], serde_json::json!(["email"]));
}

// Test string pattern rules
#[derive(Validate, ToSchema)]
#[allow(dead_code)]
struct PatternTest {
    #[validate(ascii)]
    ascii_field: String,

    #[validate(alphanumeric)]
    alnum_field: String,

    #[validate(alpha_only)]
    alpha_field: String,

    #[validate(numeric_string)]
    numeric_field: String,
}

#[test]
fn test_pattern_validations() {
    // Valid patterns
    let valid = PatternTest {
        ascii_field: "hello world".to_string(),
        alnum_field: "abc123".to_string(),
        alpha_field: "abcdef".to_string(),
        numeric_field: "12345".to_string(),
    };
    assert!(valid.validate().is_ok());

    // Invalid alphanumeric (has special chars)
    let invalid_alnum = PatternTest {
        ascii_field: "hello".to_string(),
        alnum_field: "abc-123".to_string(),
        alpha_field: "abcdef".to_string(),
        numeric_field: "12345".to_string(),
    };
    let err = invalid_alnum.validate().unwrap_err();
    assert!(!err.violations.is_empty());

    // Schema should have pattern constraints
    let schema = PatternTest::schema();
    let json = serde_json::to_value(&schema).unwrap();
    assert_eq!(
        json["properties"]["alnum_field"]["pattern"],
        "^[a-zA-Z0-9]*$"
    );
    assert_eq!(json["properties"]["alpha_field"]["pattern"], "^[a-zA-Z]*$");
    assert_eq!(json["properties"]["numeric_field"]["pattern"], "^[0-9]*$");
}

// Test numeric rules
#[derive(Validate, ToSchema)]
#[allow(dead_code)]
struct NumericTest {
    #[validate(positive)]
    score: i32,

    #[validate(range(min = 0, max = 100))]
    percentage: u8,
}

#[test]
fn test_numeric_validations() {
    // Valid
    let valid = NumericTest {
        score: 42,
        percentage: 75,
    };
    assert!(valid.validate().is_ok());

    // Invalid: negative score
    let negative_score = NumericTest {
        score: -10,
        percentage: 75,
    };
    let err = negative_score.validate().unwrap_err();
    assert!(!err.violations.is_empty());

    // Invalid: percentage too high
    let high_pct = NumericTest {
        score: 42,
        percentage: 150,
    };
    let err = high_pct.validate().unwrap_err();
    assert!(!err.violations.is_empty());

    // Schema should have numeric constraints
    let schema = NumericTest::schema();
    let json = serde_json::to_value(&schema).unwrap();
    assert_eq!(
        json["properties"]["percentage"]["minimum"].as_f64(),
        Some(0.0)
    );
    assert_eq!(
        json["properties"]["percentage"]["maximum"].as_f64(),
        Some(100.0)
    );
}

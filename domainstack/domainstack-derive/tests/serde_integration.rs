//! Test ValidateOnDeserialize macro - validates during deserialization

#![cfg(feature = "serde")]

use domainstack::Validate;
use domainstack_derive::ValidateOnDeserialize;

#[test]
fn test_valid_deserialization() {
    #[derive(ValidateOnDeserialize, Debug)]
    struct User {
        #[validate(email)]
        #[validate(max_len = 255)]
        email: String,

        #[validate(range(min = 18, max = 120))]
        age: u8,

        #[validate(alphanumeric)]
        #[validate(min_len = 3)]
        #[validate(max_len = 20)]
        username: String,
    }

    let json = r#"{
        "email": "alice@example.com",
        "age": 25,
        "username": "alice123"
    }"#;

    let user: Result<User, _> = serde_json::from_str(json);
    assert!(user.is_ok(), "Valid data should deserialize successfully");

    let user = user.unwrap();
    assert_eq!(user.email, "alice@example.com");
    assert_eq!(user.age, 25);
    assert_eq!(user.username, "alice123");
}

#[test]
fn test_invalid_email_deserialization() {
    #[derive(ValidateOnDeserialize, Debug)]
    struct User {
        #[validate(email)]
        email: String,
    }

    let json = r#"{ "email": "not-an-email" }"#;

    let result: Result<User, _> = serde_json::from_str(json);
    assert!(result.is_err(), "Invalid email should fail deserialization");

    let err = result.unwrap_err();
    let err_msg = err.to_string();
    assert!(
        err_msg.contains("Validation failed"),
        "Error should mention validation failure, got: {}",
        err_msg
    );
}

#[test]
fn test_invalid_age_deserialization() {
    #[derive(ValidateOnDeserialize, Debug)]
    struct User {
        #[validate(range(min = 18, max = 120))]
        age: u8,
    }

    // Age too young
    let json = r#"{ "age": 10 }"#;
    let result: Result<User, _> = serde_json::from_str(json);
    assert!(result.is_err(), "Age below minimum should fail");

    // Age too old (exceeds u8 max for this test, but within type bounds)
    let json = r#"{ "age": 150 }"#;
    let result: Result<User, _> = serde_json::from_str(json);
    assert!(result.is_err(), "Age above maximum should fail");
}

#[test]
fn test_invalid_username_length() {
    #[derive(ValidateOnDeserialize, Debug)]
    struct User {
        #[validate(min_len = 3)]
        #[validate(max_len = 20)]
        username: String,
    }

    // Too short
    let json = r#"{ "username": "ab" }"#;
    let result: Result<User, _> = serde_json::from_str(json);
    assert!(result.is_err(), "Username too short should fail");

    // Too long
    let json = r#"{ "username": "this_username_is_way_too_long_for_validation" }"#;
    let result: Result<User, _> = serde_json::from_str(json);
    assert!(result.is_err(), "Username too long should fail");
}

#[test]
fn test_alphanumeric_validation() {
    #[derive(ValidateOnDeserialize, Debug)]
    struct User {
        #[validate(alphanumeric)]
        username: String,
    }

    // Valid alphanumeric
    let json = r#"{ "username": "alice123" }"#;
    let result: Result<User, _> = serde_json::from_str(json);
    assert!(result.is_ok(), "Alphanumeric username should be valid");

    // Invalid: contains special characters
    let json = r#"{ "username": "alice-123" }"#;
    let result: Result<User, _> = serde_json::from_str(json);
    assert!(result.is_err(), "Username with special chars should fail");
}

#[test]
fn test_multiple_validation_errors() {
    #[derive(ValidateOnDeserialize, Debug)]
    struct User {
        #[validate(email)]
        email: String,

        #[validate(range(min = 18, max = 120))]
        age: u8,

        #[validate(alphanumeric)]
        #[validate(min_len = 3)]
        username: String,
    }

    let json = r#"{
        "email": "invalid-email",
        "age": 10,
        "username": "a"
    }"#;

    let result: Result<User, _> = serde_json::from_str(json);
    assert!(result.is_err(), "Multiple validation errors should fail");
}

#[test]
fn test_optional_fields() {
    #[derive(ValidateOnDeserialize, Debug)]
    struct User {
        #[validate(email)]
        email: String,

        // Optional field - no validation
        nickname: Option<String>,
    }

    // With nickname
    let json = r#"{ "email": "alice@example.com", "nickname": "Alice" }"#;
    let result: Result<User, _> = serde_json::from_str(json);
    assert!(result.is_ok(), "Valid data with optional field should work");
    assert_eq!(result.unwrap().nickname, Some("Alice".to_string()));

    // Without nickname
    let json = r#"{ "email": "alice@example.com" }"#;
    let result: Result<User, _> = serde_json::from_str(json);
    assert!(
        result.is_ok(),
        "Valid data without optional field should work"
    );
    assert_eq!(result.unwrap().nickname, None);
}

#[test]
fn test_serde_rename_attribute() {
    #[derive(ValidateOnDeserialize, Debug)]
    struct User {
        #[serde(rename = "emailAddress")]
        #[validate(email)]
        email: String,
    }

    // Using renamed field
    let json = r#"{ "emailAddress": "alice@example.com" }"#;
    let result: Result<User, _> = serde_json::from_str(json);
    assert!(result.is_ok(), "Renamed field should deserialize correctly");

    // Using original field name should fail
    let json = r#"{ "email": "alice@example.com" }"#;
    let result: Result<User, _> = serde_json::from_str(json);
    assert!(result.is_err(), "Original field name should not work");
}

#[test]
fn test_serde_default() {
    #[derive(ValidateOnDeserialize, Debug)]
    struct Config {
        #[serde(default = "default_port")]
        #[validate(range(min = 1024, max = 65535))]
        port: u16,
    }

    fn default_port() -> u16 {
        8080
    }

    // Without port field - should use default
    let json = r#"{}"#;
    let result: Result<Config, _> = serde_json::from_str(json);
    assert!(result.is_ok(), "Default value should be used");
    assert_eq!(result.unwrap().port, 8080);

    // With invalid default (this won't happen in practice since default is valid)
    // But with explicit value that's invalid
    let json = r#"{ "port": 80 }"#;
    let result: Result<Config, _> = serde_json::from_str(json);
    assert!(result.is_err(), "Invalid port should fail validation");
}

#[test]
fn test_deserialize_then_validate_separately() {
    #[derive(ValidateOnDeserialize, Debug)]
    struct User {
        #[validate(email)]
        email: String,
    }

    // ValidateOnDeserialize should also implement Validate
    let user = User {
        email: "alice@example.com".to_string(),
    };
    assert!(
        user.validate().is_ok(),
        "Manual validation should also work"
    );

    let invalid_user = User {
        email: "not-an-email".to_string(),
    };
    assert!(
        invalid_user.validate().is_err(),
        "Manual validation should catch errors"
    );
}

#[test]
fn test_numeric_validations() {
    #[derive(ValidateOnDeserialize, Debug)]
    struct Stats {
        #[validate(positive)]
        score: i32,

        #[validate(range(min = 0, max = 100))]
        percentage: u8,
    }

    // Valid
    let json = r#"{ "score": 42, "percentage": 75 }"#;
    let result: Result<Stats, _> = serde_json::from_str(json);
    assert!(result.is_ok());

    // Invalid: negative score
    let json = r#"{ "score": -10, "percentage": 75 }"#;
    let result: Result<Stats, _> = serde_json::from_str(json);
    assert!(result.is_err());

    // Invalid: percentage too high
    let json = r#"{ "score": 42, "percentage": 150 }"#;
    let result: Result<Stats, _> = serde_json::from_str(json);
    assert!(result.is_err());
}

#[test]
fn test_string_pattern_validations() {
    #[derive(ValidateOnDeserialize, Debug)]
    struct Data {
        #[validate(ascii)]
        ascii_field: String,

        #[validate(alphanumeric)]
        alnum_field: String,

        #[validate(alpha_only)]
        alpha_field: String,

        #[validate(numeric_string)]
        numeric_field: String,
    }

    // Valid
    let json = r#"{
        "ascii_field": "hello world",
        "alnum_field": "abc123",
        "alpha_field": "abcdef",
        "numeric_field": "12345"
    }"#;
    let result: Result<Data, _> = serde_json::from_str(json);
    assert!(result.is_ok());

    // Invalid: alphanumeric with special chars
    let json = r#"{
        "ascii_field": "hello",
        "alnum_field": "abc-123",
        "alpha_field": "abcdef",
        "numeric_field": "12345"
    }"#;
    let result: Result<Data, _> = serde_json::from_str(json);
    assert!(result.is_err());
}

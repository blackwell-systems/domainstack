//! # domainstack-envelope
//!
//! Convert domainstack validation errors into structured HTTP error responses.
//!
//! This crate provides the `IntoEnvelopeError` trait to convert `ValidationError` into
//! `error_envelope::Error`, producing structured JSON error responses with field-level details.
//!
//! ## What it provides
//!
//! - **`IntoEnvelopeError`** trait - Convert `ValidationError` to `error_envelope::Error`
//! - **Field-level error mapping** - Preserves error paths like `rooms[0].adults`, `guest.email`
//! - **Structured error format** - Consistent HTTP error response format with field-level details
//! - **Metadata preservation** - Includes validation metadata (min, max, etc.) in responses
//!
//! ## Example
//!
//! ```rust
//! use domainstack::prelude::*;
//! use domainstack_envelope::IntoEnvelopeError;
//!
//! let mut err = domainstack::ValidationError::new();
//! err.push("email", "invalid_email", "Invalid email format");
//! err.push("age", "out_of_range", "Must be between 18 and 120");
//!
//! let envelope = err.into_envelope_error();
//!
//! // Produces structured error response:
//! // {
//! //   "code": "VALIDATION",
//! //   "status": 400,
//! //   "message": "Validation failed with 2 errors",
//! //   "details": {
//! //     "fields": {
//! //       "email": [{"code": "invalid_email", "message": "Invalid email format"}],
//! //       "age": [{"code": "out_of_range", "message": "Must be between 18 and 120"}]
//! //     }
//! //   }
//! // }
//! ```
//!
//! ## Integration with Web Frameworks
//!
//! Use with framework adapters for automatic error response handling:
//!
//! - **`domainstack-axum`** - Axum integration
//! - **`domainstack-actix`** - Actix-web integration
//! - **`domainstack-rocket`** - Rocket integration

use domainstack::{ValidationError, Violation};
use error_envelope::Error;

pub trait IntoEnvelopeError {
    fn into_envelope_error(self) -> Error;
}

impl IntoEnvelopeError for ValidationError {
    fn into_envelope_error(self) -> Error {
        let violation_count = self.violations.len();

        let message = if violation_count == 1 {
            format!("Validation failed: {}", self.violations[0].message)
        } else {
            format!("Validation failed with {} errors", violation_count)
        };

        let details = create_field_details(&self);

        Error::validation(message)
            .with_details(details)
            .with_retryable(false)
    }
}

fn create_field_details(validation_error: &ValidationError) -> serde_json::Value {
    let field_map = validation_error.field_violations_map();

    let mut fields = serde_json::Map::new();

    for (path, violations) in field_map {
        let violations_json: Vec<serde_json::Value> =
            violations.into_iter().map(violation_to_json).collect();

        fields.insert(path, serde_json::Value::Array(violations_json));
    }

    serde_json::json!({
        "fields": fields
    })
}

fn violation_to_json(violation: &Violation) -> serde_json::Value {
    let mut obj = serde_json::Map::new();
    obj.insert(
        "code".to_string(),
        serde_json::Value::String(violation.code.to_string()),
    );
    obj.insert(
        "message".to_string(),
        serde_json::Value::String(violation.message.clone()),
    );

    if !violation.meta.is_empty() {
        let mut meta = serde_json::Map::new();
        for (key, value) in violation.meta.iter() {
            meta.insert(
                key.to_string(),
                serde_json::Value::String(value.to_string()),
            );
        }
        obj.insert("meta".to_string(), serde_json::Value::Object(meta));
    }

    serde_json::Value::Object(obj)
}

#[cfg(test)]
mod tests {
    use super::*;
    use domainstack::{Path, ValidationError};

    #[test]
    fn test_single_violation_conversion() {
        let mut err = ValidationError::new();
        err.push("email", "invalid_email", "Invalid email format");

        let envelope = err.into_envelope_error();

        assert_eq!(envelope.status, 400);
        assert_eq!(envelope.message, "Validation failed: Invalid email format");
        assert!(!envelope.retryable);

        let details = envelope.details.expect("Should have details");
        let fields = details["fields"]
            .as_object()
            .expect("Should have fields object");

        assert!(fields.contains_key("email"));
        let email_violations = fields["email"].as_array().expect("Should be array");
        assert_eq!(email_violations.len(), 1);
        assert_eq!(email_violations[0]["code"], "invalid_email");
        assert_eq!(email_violations[0]["message"], "Invalid email format");
    }

    #[test]
    fn test_multiple_violations_conversion() {
        let mut err = ValidationError::new();
        err.push("name", "min_length", "Must be at least 1 characters");
        err.push("age", "out_of_range", "Must be between 18 and 120");

        let envelope = err.into_envelope_error();

        assert_eq!(envelope.status, 400);
        assert_eq!(envelope.message, "Validation failed with 2 errors");

        let details = envelope.details.expect("Should have details");
        let fields = details["fields"]
            .as_object()
            .expect("Should have fields object");

        assert_eq!(fields.len(), 2);
        assert!(fields.contains_key("name"));
        assert!(fields.contains_key("age"));
    }

    #[test]
    fn test_nested_path_preservation() {
        let mut err = ValidationError::new();
        err.push(
            Path::root().field("guest").field("email"),
            "invalid_email",
            "Invalid email format",
        );

        let envelope = err.into_envelope_error();

        let details = envelope.details.expect("Should have details");
        let fields = details["fields"]
            .as_object()
            .expect("Should have fields object");

        assert!(fields.contains_key("guest.email"));
    }

    #[test]
    fn test_collection_path_with_index() {
        let mut err = ValidationError::new();
        err.push(
            Path::root().field("rooms").index(0).field("adults"),
            "out_of_range",
            "Must be between 1 and 4",
        );
        err.push(
            Path::root().field("rooms").index(1).field("children"),
            "out_of_range",
            "Must be between 0 and 3",
        );

        let envelope = err.into_envelope_error();

        let details = envelope.details.expect("Should have details");
        let fields = details["fields"]
            .as_object()
            .expect("Should have fields object");

        assert!(fields.contains_key("rooms[0].adults"));
        assert!(fields.contains_key("rooms[1].children"));
    }

    #[test]
    fn test_meta_field_inclusion() {
        let mut err = ValidationError::new();
        let mut violation = domainstack::Violation {
            path: Path::from("age"),
            code: "out_of_range",
            message: "Must be between 18 and 120".to_string(),
            meta: domainstack::Meta::new(),
        };
        violation.meta.insert("min", 18);
        violation.meta.insert("max", 120);
        err.violations.push(violation);

        let envelope = err.into_envelope_error();

        let details = envelope.details.expect("Should have details");
        let fields = details["fields"]
            .as_object()
            .expect("Should have fields object");
        let age_violations = fields["age"].as_array().expect("Should be array");

        assert_eq!(age_violations[0]["meta"]["min"], "18");
        assert_eq!(age_violations[0]["meta"]["max"], "120");
    }

    #[test]
    fn test_multiple_violations_same_field() {
        let mut err = ValidationError::new();
        err.push("password", "no_uppercase", "Must contain uppercase letter");
        err.push("password", "no_digit", "Must contain digit");

        let envelope = err.into_envelope_error();

        let details = envelope.details.expect("Should have details");
        let fields = details["fields"]
            .as_object()
            .expect("Should have fields object");

        assert_eq!(fields.len(), 1);
        let password_violations = fields["password"].as_array().expect("Should be array");
        assert_eq!(password_violations.len(), 2);
    }

    #[test]
    fn test_deeply_nested_path() {
        let mut err = ValidationError::new();
        err.push(
            Path::root()
                .field("order")
                .field("items")
                .index(0)
                .field("product")
                .field("variants")
                .index(2)
                .field("sku"),
            "invalid_sku",
            "SKU format is invalid",
        );

        let envelope = err.into_envelope_error();

        let details = envelope.details.expect("Should have details");
        let fields = details["fields"]
            .as_object()
            .expect("Should have fields object");

        assert!(fields.contains_key("order.items[0].product.variants[2].sku"));
    }

    #[test]
    fn test_empty_message_string() {
        let mut err = ValidationError::new();
        err.push("field", "error_code", "");

        let envelope = err.into_envelope_error();

        let details = envelope.details.expect("Should have details");
        let violations = details["fields"]["field"]
            .as_array()
            .expect("Should be array");
        assert_eq!(violations[0]["message"], "");
    }

    #[test]
    fn test_special_characters_in_message() {
        let mut err = ValidationError::new();
        err.push(
            "field",
            "error",
            r#"Message with "quotes", 'apostrophes', and \backslash"#,
        );

        let envelope = err.into_envelope_error();

        let details = envelope.details.expect("Should have details");
        let violations = details["fields"]["field"]
            .as_array()
            .expect("Should be array");
        assert_eq!(
            violations[0]["message"],
            r#"Message with "quotes", 'apostrophes', and \backslash"#
        );
    }

    #[test]
    fn test_meta_with_special_characters() {
        let mut err = ValidationError::new();
        let mut violation = domainstack::Violation {
            path: Path::from("field"),
            code: "error",
            message: "Error".to_string(),
            meta: domainstack::Meta::new(),
        };
        violation.meta.insert("key_with:colon", "value");
        violation.meta.insert("pattern", r"^[\w]+$");
        err.violations.push(violation);

        let envelope = err.into_envelope_error();

        let details = envelope.details.expect("Should have details");
        let violations = details["fields"]["field"]
            .as_array()
            .expect("Should be array");
        let meta = violations[0]["meta"].as_object().expect("Should have meta");
        assert_eq!(meta["key_with:colon"], "value");
        assert_eq!(meta["pattern"], r"^[\w]+$");
    }

    #[test]
    fn test_three_violations_message_format() {
        let mut err = ValidationError::new();
        err.push("a", "err", "Error A");
        err.push("b", "err", "Error B");
        err.push("c", "err", "Error C");

        let envelope = err.into_envelope_error();

        assert_eq!(envelope.message, "Validation failed with 3 errors");
    }

    #[test]
    fn test_empty_meta_not_included() {
        let mut err = ValidationError::new();
        err.push("field", "error_code", "Error message");

        let envelope = err.into_envelope_error();

        let details = envelope.details.expect("Should have details");
        let violations = details["fields"]["field"]
            .as_array()
            .expect("Should be array");
        // Empty meta should not be present in output
        assert!(violations[0].get("meta").is_none());
    }

    #[test]
    fn test_large_number_of_violations() {
        let mut err = ValidationError::new();
        for i in 0..100 {
            err.push(format!("field{}", i), "error", format!("Error {}", i));
        }

        let envelope = err.into_envelope_error();

        assert_eq!(envelope.message, "Validation failed with 100 errors");
        let details = envelope.details.expect("Should have details");
        let fields = details["fields"]
            .as_object()
            .expect("Should have fields object");
        assert_eq!(fields.len(), 100);
    }

    #[test]
    fn test_root_path_violation() {
        let mut err = ValidationError::new();
        let violation = domainstack::Violation {
            path: Path::root(),
            code: "invalid_object",
            message: "Object is invalid".to_string(),
            meta: domainstack::Meta::new(),
        };
        err.violations.push(violation);

        let envelope = err.into_envelope_error();

        let details = envelope.details.expect("Should have details");
        let fields = details["fields"]
            .as_object()
            .expect("Should have fields object");
        // Root path should be represented as empty string
        assert!(fields.contains_key(""));
    }
}

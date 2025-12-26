//! # domainstack-wasm
//!
//! WASM browser validation for domainstack. Run the same validation logic
//! in both browser and server.
//!
//! ## Quick Start
//!
//! ```javascript
//! import { createValidator } from '@domainstack/wasm';
//!
//! const validator = await createValidator();
//! const result = validator.validate('Booking', {
//!   guest_email: 'invalid',
//!   rooms: 15
//! });
//!
//! if (!result.ok) {
//!   result.errors?.forEach(e => console.log(e.path, e.message));
//! }
//! ```
//!
//! ## Architecture
//!
//! This crate provides a thin WASM layer over domainstack validation.
//! Types are registered at compile time, and validation is dispatched
//! by type name at runtime.

use serde::Serialize;
use std::collections::HashMap;
use wasm_bindgen::prelude::*;

// Re-export for use in proc macro generated code
pub use domainstack::{Validate, ValidationError};
pub use serde::de::DeserializeOwned;

/// Initialize panic hook for better error messages in browser console.
/// Call this once at application startup.
#[wasm_bindgen(start)]
pub fn init() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

// ============================================================================
// WASM-Serializable Types
// ============================================================================

/// Result of a validation operation.
///
/// This is the ONLY type returned to JavaScript. No nulls, no exceptions.
#[derive(Debug, Clone, Serialize)]
pub struct ValidationResult {
    /// Whether validation passed
    pub ok: bool,

    /// Validation violations (present when ok=false and validation failed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<Vec<WasmViolation>>,

    /// System error (present when ok=false and a system error occurred)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<SystemError>,
}

impl ValidationResult {
    /// Create a successful result
    pub fn success() -> Self {
        Self {
            ok: true,
            errors: None,
            error: None,
        }
    }

    /// Create a validation failure result
    pub fn validation_failed(violations: Vec<WasmViolation>) -> Self {
        Self {
            ok: false,
            errors: Some(violations),
            error: None,
        }
    }

    /// Create a system error result
    pub fn system_error(code: &'static str, message: String) -> Self {
        Self {
            ok: false,
            errors: None,
            error: Some(SystemError { code, message }),
        }
    }
}

/// A validation violation, serializable for WASM.
#[derive(Debug, Clone, Serialize)]
pub struct WasmViolation {
    /// Field path (e.g., "guest_email", "rooms\[0\].adults")
    pub path: String,

    /// Error code (e.g., "invalid_email", "out_of_range")
    pub code: String,

    /// Human-readable message
    pub message: String,

    /// Additional metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<HashMap<String, String>>,
}

impl From<&domainstack::Violation> for WasmViolation {
    fn from(v: &domainstack::Violation) -> Self {
        let meta = if v.meta.is_empty() {
            None
        } else {
            Some(
                v.meta
                    .iter()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect(),
            )
        };

        Self {
            path: v.path.to_string(),
            code: v.code.to_string(),
            message: v.message.clone(),
            meta,
        }
    }
}

/// System-level error (not a validation failure).
#[derive(Debug, Clone, Serialize)]
pub struct SystemError {
    /// Error code: "unknown_type" | "parse_error" | "internal_error"
    pub code: &'static str,

    /// Human-readable error message
    pub message: String,
}

// ============================================================================
// Dispatch Error
// ============================================================================

/// Internal error type for dispatch operations
pub enum DispatchError {
    /// Type name not found in registry
    UnknownType,

    /// JSON parsing failed
    ParseError(String),

    /// Validation failed (boxed to reduce enum size)
    Validation(Box<ValidationError>),
}

// ============================================================================
// Type Registry
// ============================================================================

/// Function signature for type validators
pub type ValidateFn = fn(&str) -> Result<(), DispatchError>;

/// Registry of type validators.
///
/// In production, this is populated by the `#[wasm_validate]` proc macro.
/// For now, users must manually register types.
pub struct TypeRegistry {
    validators: HashMap<&'static str, ValidateFn>,
}

impl TypeRegistry {
    /// Create an empty registry
    pub fn new() -> Self {
        Self {
            validators: HashMap::new(),
        }
    }

    /// Register a type validator
    pub fn register<T>(&mut self, type_name: &'static str)
    where
        T: Validate + DeserializeOwned + 'static,
    {
        self.validators.insert(type_name, validate_json::<T>);
    }

    /// Validate JSON against a registered type
    pub fn validate(&self, type_name: &str, json: &str) -> Result<(), DispatchError> {
        match self.validators.get(type_name) {
            Some(validate_fn) => validate_fn(json),
            None => Err(DispatchError::UnknownType),
        }
    }

    /// Check if a type is registered
    pub fn has_type(&self, type_name: &str) -> bool {
        self.validators.contains_key(type_name)
    }

    /// List all registered type names
    pub fn type_names(&self) -> Vec<&'static str> {
        self.validators.keys().copied().collect()
    }
}

impl Default for TypeRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Validate JSON for a specific type
fn validate_json<T>(json: &str) -> Result<(), DispatchError>
where
    T: Validate + DeserializeOwned,
{
    let value: T =
        serde_json::from_str(json).map_err(|e| DispatchError::ParseError(e.to_string()))?;
    value
        .validate()
        .map_err(|e| DispatchError::Validation(Box::new(e)))
}

// ============================================================================
// Global Registry
// ============================================================================

use std::cell::RefCell;

thread_local! {
    static REGISTRY: RefCell<TypeRegistry> = RefCell::new(TypeRegistry::new());
}

/// Register a type for WASM validation.
///
/// # Example
///
/// ```ignore
/// use domainstack::prelude::*;
/// use domainstack_wasm::register_type;
///
/// #[derive(Validate, Deserialize)]
/// struct Booking {
///     #[validate(email)]
///     guest_email: String,
/// }
///
/// // Call once at initialization
/// register_type::<Booking>("Booking");
/// ```
pub fn register_type<T>(type_name: &'static str)
where
    T: Validate + DeserializeOwned + 'static,
{
    REGISTRY.with(|r| r.borrow_mut().register::<T>(type_name));
}

/// Check if a type is registered
pub fn is_type_registered(type_name: &str) -> bool {
    REGISTRY.with(|r| r.borrow().has_type(type_name))
}

/// Get list of all registered type names
pub fn registered_types() -> Vec<&'static str> {
    REGISTRY.with(|r| r.borrow().type_names())
}

// ============================================================================
// WASM Entry Points
// ============================================================================

/// Validate JSON data against a registered type.
///
/// # Arguments
///
/// * `type_name` - The name of the type to validate against (e.g., "Booking")
/// * `json` - JSON string to validate
///
/// # Returns
///
/// A `ValidationResult` object (always, never throws):
/// - `{ ok: true }` - Validation passed
/// - `{ ok: false, errors: [...] }` - Validation failed
/// - `{ ok: false, error: { code, message } }` - System error
#[wasm_bindgen]
pub fn validate(type_name: &str, json: &str) -> JsValue {
    let result = REGISTRY.with(|r| r.borrow().validate(type_name, json));

    let validation_result = match result {
        Ok(()) => ValidationResult::success(),
        Err(DispatchError::UnknownType) => {
            ValidationResult::system_error("unknown_type", format!("Unknown type: {}", type_name))
        }
        Err(DispatchError::ParseError(msg)) => ValidationResult::system_error("parse_error", msg),
        Err(DispatchError::Validation(err)) => {
            let violations = err.violations.iter().map(WasmViolation::from).collect();
            ValidationResult::validation_failed(violations)
        }
    };

    // Serialize to JsValue, with fallback for serialization errors
    serde_wasm_bindgen::to_value(&validation_result).unwrap_or_else(|_| {
        let fallback = ValidationResult::system_error(
            "internal_error",
            "Failed to serialize validation result".to_string(),
        );
        serde_wasm_bindgen::to_value(&fallback).unwrap()
    })
}

/// Validate a JavaScript object against a registered type.
///
/// This is a convenience wrapper that accepts a JS object directly
/// instead of a JSON string.
///
/// # Arguments
///
/// * `type_name` - The name of the type to validate against
/// * `value` - JavaScript object to validate
#[wasm_bindgen]
pub fn validate_object(type_name: &str, value: JsValue) -> JsValue {
    // Serialize JS object to JSON string
    let json = match js_sys::JSON::stringify(&value) {
        Ok(s) => s.as_string().unwrap_or_default(),
        Err(_) => {
            let result = ValidationResult::system_error(
                "parse_error",
                "Failed to serialize JavaScript object to JSON".to_string(),
            );
            return serde_wasm_bindgen::to_value(&result).unwrap();
        }
    };

    validate(type_name, &json)
}

/// Get list of registered type names.
///
/// Useful for debugging and introspection.
#[wasm_bindgen]
pub fn get_registered_types() -> JsValue {
    let types = registered_types();
    serde_wasm_bindgen::to_value(&types).unwrap_or(JsValue::NULL)
}

/// Check if a type is registered.
#[wasm_bindgen]
pub fn has_type(type_name: &str) -> bool {
    is_type_registered(type_name)
}

// ============================================================================
// Builder Pattern for TypeScript
// ============================================================================

/// Validator instance for TypeScript ergonomics.
///
/// ```javascript
/// const validator = await createValidator();
/// const result = validator.validate('Booking', { ... });
/// ```
#[wasm_bindgen]
pub struct Validator {
    // Marker field - registry is global via thread_local
    _private: (),
}

#[wasm_bindgen]
impl Validator {
    /// Validate JSON string against a type
    pub fn validate(&self, type_name: &str, json: &str) -> JsValue {
        validate(type_name, json)
    }

    /// Validate JS object against a type
    #[wasm_bindgen(js_name = validateObject)]
    pub fn validate_object(&self, type_name: &str, value: JsValue) -> JsValue {
        validate_object(type_name, value)
    }

    /// Get registered type names
    #[wasm_bindgen(js_name = getTypes)]
    pub fn get_types(&self) -> JsValue {
        get_registered_types()
    }

    /// Check if type is registered
    #[wasm_bindgen(js_name = hasType)]
    pub fn has_type(&self, type_name: &str) -> bool {
        has_type(type_name)
    }
}

/// Create a validator instance.
///
/// This is the main entry point for TypeScript/JavaScript.
///
/// ```javascript
/// import { createValidator } from '@domainstack/wasm';
///
/// const validator = await createValidator();
/// ```
#[wasm_bindgen(js_name = createValidator)]
pub fn create_validator() -> Validator {
    Validator { _private: () }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_result_success() {
        let result = ValidationResult::success();
        assert!(result.ok);
        assert!(result.errors.is_none());
        assert!(result.error.is_none());
    }

    #[test]
    fn test_validation_result_failure() {
        let violations = vec![WasmViolation {
            path: "email".to_string(),
            code: "invalid_email".to_string(),
            message: "Invalid email format".to_string(),
            meta: None,
        }];
        let result = ValidationResult::validation_failed(violations);
        assert!(!result.ok);
        assert!(result.errors.is_some());
        assert_eq!(result.errors.as_ref().unwrap().len(), 1);
        assert!(result.error.is_none());
    }

    #[test]
    fn test_validation_result_system_error() {
        let result =
            ValidationResult::system_error("unknown_type", "Unknown type: Foo".to_string());
        assert!(!result.ok);
        assert!(result.errors.is_none());
        assert!(result.error.is_some());
        assert_eq!(result.error.as_ref().unwrap().code, "unknown_type");
    }

    #[test]
    fn test_registry_unknown_type() {
        let registry = TypeRegistry::new();
        let result = registry.validate("NonExistent", "{}");
        assert!(matches!(result, Err(DispatchError::UnknownType)));
    }

    // Integration tests with actual Validate types
    mod integration {
        use super::*;
        use domainstack::Validate;
        use serde::Deserialize;

        #[derive(Debug, Validate, Deserialize)]
        struct TestBooking {
            #[validate(length(min = 1, max = 100))]
            guest_name: String,

            #[validate(range(min = 1, max = 10))]
            rooms: u8,
        }

        #[test]
        fn test_register_and_validate_success() {
            let mut registry = TypeRegistry::new();
            registry.register::<TestBooking>("TestBooking");

            let json = r#"{"guest_name": "John Doe", "rooms": 2}"#;
            let result = registry.validate("TestBooking", json);
            assert!(result.is_ok());
        }

        #[test]
        fn test_register_and_validate_failure() {
            let mut registry = TypeRegistry::new();
            registry.register::<TestBooking>("TestBooking");

            // rooms = 15 is out of range (max = 10)
            let json = r#"{"guest_name": "John", "rooms": 15}"#;
            let result = registry.validate("TestBooking", json);

            assert!(matches!(result, Err(DispatchError::Validation(_))));
            if let Err(DispatchError::Validation(err)) = result {
                assert!(!err.violations.is_empty());
                assert_eq!(err.violations[0].path.to_string(), "rooms");
            }
        }

        #[test]
        fn test_parse_error() {
            let mut registry = TypeRegistry::new();
            registry.register::<TestBooking>("TestBooking");

            let json = r#"{"guest_name": "John", "rooms": "not a number"}"#;
            let result = registry.validate("TestBooking", json);

            assert!(matches!(result, Err(DispatchError::ParseError(_))));
        }

        #[test]
        fn test_wasm_violation_from_violation() {
            let violation = domainstack::Violation {
                path: domainstack::Path::from("email"),
                code: "invalid_email",
                message: "Invalid email format".to_string(),
                meta: domainstack::Meta::default(),
            };

            let wasm_violation = WasmViolation::from(&violation);
            assert_eq!(wasm_violation.path, "email");
            assert_eq!(wasm_violation.code, "invalid_email");
            assert_eq!(wasm_violation.message, "Invalid email format");
            assert!(wasm_violation.meta.is_none());
        }

        #[test]
        fn test_wasm_violation_with_meta() {
            let mut meta = domainstack::Meta::default();
            meta.insert("min", "1");
            meta.insert("max", "10");

            let violation = domainstack::Violation {
                path: domainstack::Path::from("age"),
                code: "out_of_range",
                message: "Must be between 1 and 10".to_string(),
                meta,
            };

            let wasm_violation = WasmViolation::from(&violation);
            assert!(wasm_violation.meta.is_some());
            let meta = wasm_violation.meta.unwrap();
            assert_eq!(meta.get("min"), Some(&"1".to_string()));
            assert_eq!(meta.get("max"), Some(&"10".to_string()));
        }

        #[test]
        fn test_multiple_violations() {
            let mut registry = TypeRegistry::new();
            registry.register::<TestBooking>("TestBooking");

            // Both fields invalid
            let json = r#"{"guest_name": "", "rooms": 15}"#;
            let result = registry.validate("TestBooking", json);

            assert!(matches!(result, Err(DispatchError::Validation(_))));
            if let Err(DispatchError::Validation(err)) = result {
                assert_eq!(err.violations.len(), 2);
            }
        }

        #[test]
        fn test_empty_json_object() {
            let mut registry = TypeRegistry::new();
            registry.register::<TestBooking>("TestBooking");

            // Missing required fields - should fail parse
            let json = r#"{}"#;
            let result = registry.validate("TestBooking", json);
            assert!(matches!(result, Err(DispatchError::ParseError(_))));
        }

        #[test]
        fn test_invalid_json_syntax() {
            let mut registry = TypeRegistry::new();
            registry.register::<TestBooking>("TestBooking");

            let json = r#"{ invalid json }"#;
            let result = registry.validate("TestBooking", json);
            assert!(matches!(result, Err(DispatchError::ParseError(_))));
        }

        #[test]
        fn test_registry_has_type() {
            let mut registry = TypeRegistry::new();
            assert!(!registry.has_type("TestBooking"));

            registry.register::<TestBooking>("TestBooking");
            assert!(registry.has_type("TestBooking"));
            assert!(!registry.has_type("NonExistent"));
        }

        #[test]
        fn test_registry_type_names() {
            let mut registry = TypeRegistry::new();
            assert!(registry.type_names().is_empty());

            registry.register::<TestBooking>("TestBooking");
            let names = registry.type_names();
            assert_eq!(names.len(), 1);
            assert!(names.contains(&"TestBooking"));
        }

        // Test nested validation
        #[derive(Debug, Validate, Deserialize)]
        struct TestAddress {
            #[validate(length(min = 1, max = 100))]
            street: String,

            #[validate(length(min = 1, max = 50))]
            city: String,
        }

        #[derive(Debug, Validate, Deserialize)]
        struct TestPerson {
            #[validate(length(min = 1, max = 50))]
            name: String,

            #[validate(nested)]
            address: TestAddress,
        }

        #[test]
        fn test_nested_validation_with_path() {
            let mut registry = TypeRegistry::new();
            registry.register::<TestPerson>("TestPerson");

            // Invalid city in nested address
            let json = r#"{"name": "John", "address": {"street": "123 Main", "city": ""}}"#;
            let result = registry.validate("TestPerson", json);

            assert!(matches!(result, Err(DispatchError::Validation(_))));
            if let Err(DispatchError::Validation(err)) = result {
                assert!(!err.violations.is_empty());
                // Path should include nested field
                let path = err.violations[0].path.to_string();
                assert!(path.contains("address") || path.contains("city"));
            }
        }
    }

    // Tests for global registry functions
    mod global_registry {
        use super::*;
        use serde::Deserialize;

        #[derive(Debug, Validate, Deserialize)]
        struct GlobalTestType {
            #[validate(length(min = 1))]
            name: String,
        }

        #[test]
        fn test_global_register_and_check() {
            // Register type globally
            register_type::<GlobalTestType>("GlobalTestType");

            // Check it's registered
            assert!(is_type_registered("GlobalTestType"));
            assert!(!is_type_registered("NotRegistered"));

            // Check it appears in list
            let types = registered_types();
            assert!(types.contains(&"GlobalTestType"));
        }
    }

    // Tests for ValidationResult constructors
    mod validation_result {
        use super::*;

        #[test]
        fn test_success_serialization() {
            let result = ValidationResult::success();
            let json = serde_json::to_string(&result).unwrap();
            assert!(json.contains("\"ok\":true"));
            assert!(!json.contains("errors"));
            assert!(!json.contains("error"));
        }

        #[test]
        fn test_validation_failed_serialization() {
            let violations = vec![
                WasmViolation {
                    path: "field1".to_string(),
                    code: "error1".to_string(),
                    message: "Error 1".to_string(),
                    meta: None,
                },
                WasmViolation {
                    path: "field2".to_string(),
                    code: "error2".to_string(),
                    message: "Error 2".to_string(),
                    meta: None,
                },
            ];
            let result = ValidationResult::validation_failed(violations);
            let json = serde_json::to_string(&result).unwrap();
            assert!(json.contains("\"ok\":false"));
            assert!(json.contains("\"errors\""));
            assert!(json.contains("field1"));
            assert!(json.contains("field2"));
        }

        #[test]
        fn test_system_error_serialization() {
            let result = ValidationResult::system_error("parse_error", "Invalid JSON".to_string());
            let json = serde_json::to_string(&result).unwrap();
            assert!(json.contains("\"ok\":false"));
            assert!(json.contains("\"error\""));
            assert!(json.contains("parse_error"));
            assert!(json.contains("Invalid JSON"));
        }

        #[test]
        fn test_empty_violations_list() {
            let result = ValidationResult::validation_failed(vec![]);
            let json = serde_json::to_string(&result).unwrap();
            assert!(json.contains("\"ok\":false"));
            assert!(json.contains("\"errors\":[]"));
        }

        #[test]
        fn test_violation_with_meta_serialization() {
            let mut meta = HashMap::new();
            meta.insert("min".to_string(), "1".to_string());
            meta.insert("max".to_string(), "10".to_string());

            let violations = vec![WasmViolation {
                path: "count".to_string(),
                code: "out_of_range".to_string(),
                message: "Must be between 1 and 10".to_string(),
                meta: Some(meta),
            }];
            let result = ValidationResult::validation_failed(violations);
            let json = serde_json::to_string(&result).unwrap();
            assert!(json.contains("\"meta\""));
            assert!(json.contains("\"min\":\"1\""));
            assert!(json.contains("\"max\":\"10\""));
        }

        #[test]
        fn test_special_characters_in_message() {
            let result = ValidationResult::system_error(
                "parse_error",
                r#"Expected "}" at line 1, column 5"#.to_string(),
            );
            let json = serde_json::to_string(&result).unwrap();
            assert!(json.contains("parse_error"));
            // Message should be properly escaped - verify valid JSON by parsing as Value
            assert!(serde_json::from_str::<serde_json::Value>(&json).is_ok());
        }
    }

    // Tests for TypeRegistry edge cases
    mod registry_edge_cases {
        use super::*;
        use serde::Deserialize;

        #[derive(Debug, Validate, Deserialize)]
        struct TypeA {
            #[validate(length(min = 1))]
            name: String,
        }

        #[derive(Debug, Validate, Deserialize)]
        struct TypeB {
            #[validate(range(min = 0, max = 100))]
            value: i32,
        }

        #[test]
        fn test_registry_default_impl() {
            let registry = TypeRegistry::default();
            assert!(registry.type_names().is_empty());
        }

        #[test]
        fn test_multiple_types_registration() {
            let mut registry = TypeRegistry::new();
            registry.register::<TypeA>("TypeA");
            registry.register::<TypeB>("TypeB");

            assert!(registry.has_type("TypeA"));
            assert!(registry.has_type("TypeB"));
            assert_eq!(registry.type_names().len(), 2);
        }

        #[test]
        fn test_type_overwriting() {
            let mut registry = TypeRegistry::new();
            registry.register::<TypeA>("SharedName");

            // Validate with TypeA rules (length validation)
            let result = registry.validate("SharedName", r#"{"name": ""}"#);
            assert!(matches!(result, Err(DispatchError::Validation(_))));

            // Overwrite with TypeB
            registry.register::<TypeB>("SharedName");

            // Now validates with TypeB rules (range validation)
            let result = registry.validate("SharedName", r#"{"value": 50}"#);
            assert!(result.is_ok());
        }

        #[test]
        fn test_empty_type_name() {
            let mut registry = TypeRegistry::new();
            registry.register::<TypeA>("");

            assert!(registry.has_type(""));
            let result = registry.validate("", r#"{"name": "test"}"#);
            assert!(result.is_ok());
        }

        #[test]
        fn test_type_name_with_special_characters() {
            let mut registry = TypeRegistry::new();
            registry.register::<TypeA>("Type::With::Colons");
            registry.register::<TypeB>("Type<Generic>");

            assert!(registry.has_type("Type::With::Colons"));
            assert!(registry.has_type("Type<Generic>"));
        }

        #[test]
        fn test_validate_with_whitespace_in_json() {
            let mut registry = TypeRegistry::new();
            registry.register::<TypeA>("TypeA");

            let json = r#"
                {
                    "name": "test"
                }
            "#;
            let result = registry.validate("TypeA", json);
            assert!(result.is_ok());
        }

        #[test]
        fn test_validate_with_extra_fields_in_json() {
            let mut registry = TypeRegistry::new();
            registry.register::<TypeA>("TypeA");

            // Extra fields should be ignored (default serde behavior)
            let json = r#"{"name": "test", "extra": "ignored"}"#;
            let result = registry.validate("TypeA", json);
            assert!(result.is_ok());
        }

        #[test]
        fn test_unicode_in_validation_values() {
            let mut registry = TypeRegistry::new();
            registry.register::<TypeA>("TypeA");

            let json = r#"{"name": "日本語テスト 🎉"}"#;
            let result = registry.validate("TypeA", json);
            assert!(result.is_ok());
        }

        #[test]
        fn test_null_value_in_json() {
            let mut registry = TypeRegistry::new();
            registry.register::<TypeA>("TypeA");

            let json = r#"{"name": null}"#;
            let result = registry.validate("TypeA", json);
            // String field can't be null, should fail parse
            assert!(matches!(result, Err(DispatchError::ParseError(_))));
        }

        #[test]
        fn test_empty_string_validation() {
            let mut registry = TypeRegistry::new();
            registry.register::<TypeA>("TypeA");

            let json = r#"{"name": ""}"#;
            let result = registry.validate("TypeA", json);
            // Empty string fails min length 1 validation
            assert!(matches!(result, Err(DispatchError::Validation(_))));
        }
    }

    // Tests for WasmViolation conversion edge cases
    mod wasm_violation_edge_cases {
        use super::*;

        #[test]
        fn test_violation_with_empty_path() {
            let violation = domainstack::Violation {
                path: domainstack::Path::root(),
                code: "invalid",
                message: "Invalid".to_string(),
                meta: domainstack::Meta::default(),
            };

            let wasm_violation = WasmViolation::from(&violation);
            assert_eq!(wasm_violation.path, "");
        }

        #[test]
        fn test_violation_with_complex_nested_path() {
            let path = domainstack::Path::root()
                .field("orders")
                .index(0)
                .field("items")
                .index(5)
                .field("variant");

            let violation = domainstack::Violation {
                path,
                code: "invalid",
                message: "Invalid".to_string(),
                meta: domainstack::Meta::default(),
            };

            let wasm_violation = WasmViolation::from(&violation);
            assert_eq!(wasm_violation.path, "orders[0].items[5].variant");
        }

        #[test]
        fn test_violation_with_special_chars_in_code() {
            let violation = domainstack::Violation {
                path: domainstack::Path::from("field"),
                code: "error_code_with_underscores",
                message: "Error".to_string(),
                meta: domainstack::Meta::default(),
            };

            let wasm_violation = WasmViolation::from(&violation);
            assert_eq!(wasm_violation.code, "error_code_with_underscores");
        }

        #[test]
        fn test_violation_preserves_long_message() {
            let long_message = "A".repeat(1000);
            let violation = domainstack::Violation {
                path: domainstack::Path::from("field"),
                code: "error",
                message: long_message.clone(),
                meta: domainstack::Meta::default(),
            };

            let wasm_violation = WasmViolation::from(&violation);
            assert_eq!(wasm_violation.message, long_message);
        }

        #[test]
        fn test_violation_meta_numeric_values() {
            let mut meta = domainstack::Meta::default();
            meta.insert("min", 1);
            meta.insert("max", 100);
            meta.insert("actual", 150);

            let violation = domainstack::Violation {
                path: domainstack::Path::from("field"),
                code: "out_of_range",
                message: "Out of range".to_string(),
                meta,
            };

            let wasm_violation = WasmViolation::from(&violation);
            let meta = wasm_violation.meta.unwrap();
            // Numeric values should be converted to strings
            assert_eq!(meta.get("min"), Some(&"1".to_string()));
            assert_eq!(meta.get("max"), Some(&"100".to_string()));
            assert_eq!(meta.get("actual"), Some(&"150".to_string()));
        }
    }

    // Tests for dispatch error handling
    mod dispatch_error_tests {
        use super::*;
        use serde::Deserialize;

        #[derive(Debug, Validate, Deserialize)]
        #[allow(dead_code)]
        struct SimpleType {
            value: i32,
        }

        #[test]
        fn test_dispatch_error_unknown_type_message() {
            let registry = TypeRegistry::new();
            let result = registry.validate("DoesNotExist", "{}");

            match result {
                Err(DispatchError::UnknownType) => {}
                _ => panic!("Expected UnknownType error"),
            }
        }

        #[test]
        fn test_dispatch_error_parse_error_contains_details() {
            let mut registry = TypeRegistry::new();
            registry.register::<SimpleType>("SimpleType");

            let result = registry.validate("SimpleType", r#"{"value": "not_a_number"}"#);

            match result {
                Err(DispatchError::ParseError(msg)) => {
                    assert!(!msg.is_empty());
                }
                _ => panic!("Expected ParseError"),
            }
        }

        #[test]
        fn test_dispatch_error_validation_boxed() {
            let mut registry = TypeRegistry::new();

            #[derive(Debug, Validate, Deserialize)]
            struct AlwaysInvalid {
                #[validate(range(min = 10, max = 5))] // Invalid range, will fail
                value: i32,
            }

            registry.register::<AlwaysInvalid>("AlwaysInvalid");

            // Any value should trigger validation error
            let result = registry.validate("AlwaysInvalid", r#"{"value": 7}"#);

            match result {
                Err(DispatchError::Validation(boxed_err)) => {
                    assert!(!boxed_err.violations.is_empty());
                }
                _ => panic!("Expected Validation error"),
            }
        }
    }
}

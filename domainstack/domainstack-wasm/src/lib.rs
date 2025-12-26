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
    /// Field path (e.g., "guest_email", "rooms[0].adults")
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
            Some(v.meta.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect())
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

    /// Validation failed
    Validation(ValidationError),
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
    let value: T = serde_json::from_str(json).map_err(|e| DispatchError::ParseError(e.to_string()))?;
    value.validate().map_err(DispatchError::Validation)
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
        Err(DispatchError::ParseError(msg)) => {
            ValidationResult::system_error("parse_error", msg)
        }
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
        let result = ValidationResult::system_error("unknown_type", "Unknown type: Foo".to_string());
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
    }
}

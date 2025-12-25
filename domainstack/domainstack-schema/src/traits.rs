//! Traits for schema generation.

use crate::Schema;

/// Types that can generate OpenAPI schemas.
///
/// This trait should be implemented for all domain types that need
/// to appear in OpenAPI specifications.
///
/// # Example
///
/// ```rust
/// use domainstack_schema::{ToSchema, Schema};
///
/// struct User {
///     email: String,
///     age: u8,
/// }
///
/// impl ToSchema for User {
///     fn schema_name() -> &'static str {
///         "User"
///     }
///
///     fn schema() -> Schema {
///         Schema::object()
///             .property("email", Schema::string().format("email"))
///             .property("age", Schema::integer().minimum(0).maximum(150))
///             .required(&["email", "age"])
///     }
/// }
/// ```
pub trait ToSchema {
    /// The name of this schema in the OpenAPI spec.
    fn schema_name() -> &'static str;

    /// Generate the OpenAPI schema for this type.
    fn schema() -> Schema;
}

// Implementations for primitive types
impl ToSchema for String {
    fn schema_name() -> &'static str {
        "string"
    }

    fn schema() -> Schema {
        Schema::string()
    }
}

impl ToSchema for str {
    fn schema_name() -> &'static str {
        "string"
    }

    fn schema() -> Schema {
        Schema::string()
    }
}

impl ToSchema for u8 {
    fn schema_name() -> &'static str {
        "integer"
    }

    fn schema() -> Schema {
        Schema::integer().minimum(0).maximum(255)
    }
}

impl ToSchema for u16 {
    fn schema_name() -> &'static str {
        "integer"
    }

    fn schema() -> Schema {
        Schema::integer().minimum(0).maximum(65535)
    }
}

impl ToSchema for u32 {
    fn schema_name() -> &'static str {
        "integer"
    }

    fn schema() -> Schema {
        Schema::integer().minimum(0)
    }
}

impl ToSchema for i32 {
    fn schema_name() -> &'static str {
        "integer"
    }

    fn schema() -> Schema {
        Schema::integer()
    }
}

impl ToSchema for i64 {
    fn schema_name() -> &'static str {
        "integer"
    }

    fn schema() -> Schema {
        Schema::integer()
    }
}

impl ToSchema for f32 {
    fn schema_name() -> &'static str {
        "number"
    }

    fn schema() -> Schema {
        Schema::number().format("float")
    }
}

impl ToSchema for f64 {
    fn schema_name() -> &'static str {
        "number"
    }

    fn schema() -> Schema {
        Schema::number().format("double")
    }
}

impl ToSchema for bool {
    fn schema_name() -> &'static str {
        "boolean"
    }

    fn schema() -> Schema {
        Schema::boolean()
    }
}

impl<T: ToSchema> ToSchema for Vec<T> {
    fn schema_name() -> &'static str {
        "array"
    }

    fn schema() -> Schema {
        Schema::array(T::schema())
    }
}

impl<T: ToSchema> ToSchema for Option<T> {
    fn schema_name() -> &'static str {
        T::schema_name()
    }

    fn schema() -> Schema {
        T::schema()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_schema() {
        assert_eq!(String::schema_name(), "string");
        let schema = String::schema();
        assert!(matches!(schema.schema_type, Some(crate::SchemaType::String)));
    }

    #[test]
    fn test_str_schema() {
        assert_eq!(<str>::schema_name(), "string");
        let schema = <str>::schema();
        assert!(matches!(schema.schema_type, Some(crate::SchemaType::String)));
    }

    #[test]
    fn test_u8_schema() {
        assert_eq!(u8::schema_name(), "integer");
        let schema = u8::schema();
        assert_eq!(schema.minimum, Some(0.0));
        assert_eq!(schema.maximum, Some(255.0));
    }

    #[test]
    fn test_u16_schema() {
        assert_eq!(u16::schema_name(), "integer");
        let schema = u16::schema();
        assert_eq!(schema.minimum, Some(0.0));
        assert_eq!(schema.maximum, Some(65535.0));
    }

    #[test]
    fn test_u32_schema() {
        assert_eq!(u32::schema_name(), "integer");
        let schema = u32::schema();
        assert_eq!(schema.minimum, Some(0.0));
        assert_eq!(schema.maximum, None);
    }

    #[test]
    fn test_i32_schema() {
        assert_eq!(i32::schema_name(), "integer");
        let schema = i32::schema();
        assert!(matches!(schema.schema_type, Some(crate::SchemaType::Integer)));
    }

    #[test]
    fn test_i64_schema() {
        assert_eq!(i64::schema_name(), "integer");
        let schema = i64::schema();
        assert!(matches!(schema.schema_type, Some(crate::SchemaType::Integer)));
    }

    #[test]
    fn test_f32_schema() {
        assert_eq!(f32::schema_name(), "number");
        let schema = f32::schema();
        assert!(matches!(schema.schema_type, Some(crate::SchemaType::Number)));
        assert_eq!(schema.format, Some("float".to_string()));
    }

    #[test]
    fn test_f64_schema() {
        assert_eq!(f64::schema_name(), "number");
        let schema = f64::schema();
        assert!(matches!(schema.schema_type, Some(crate::SchemaType::Number)));
        assert_eq!(schema.format, Some("double".to_string()));
    }

    #[test]
    fn test_bool_schema() {
        assert_eq!(bool::schema_name(), "boolean");
        let schema = bool::schema();
        assert!(matches!(schema.schema_type, Some(crate::SchemaType::Boolean)));
    }

    #[test]
    fn test_vec_schema() {
        assert_eq!(<Vec<String>>::schema_name(), "array");
        let schema = <Vec<String>>::schema();
        assert!(matches!(schema.schema_type, Some(crate::SchemaType::Array)));
        assert!(schema.items.is_some());
    }

    #[test]
    fn test_option_schema() {
        assert_eq!(<Option<u32>>::schema_name(), "integer");
        let schema = <Option<u32>>::schema();
        assert_eq!(schema.minimum, Some(0.0));
    }
}

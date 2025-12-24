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

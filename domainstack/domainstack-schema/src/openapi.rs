//! OpenAPI specification builder.

use crate::{Schema, ToSchema};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Complete OpenAPI 3.0 specification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenApiSpec {
    pub openapi: String,
    pub info: Info,
    pub components: Components,
}

/// API information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Info {
    pub title: String,
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Components containing reusable schemas.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Components {
    pub schemas: HashMap<String, Schema>,
}

/// Builder for OpenAPI specifications.
pub struct OpenApiBuilder {
    title: String,
    version: String,
    description: Option<String>,
    schemas: HashMap<String, Schema>,
}

impl OpenApiBuilder {
    /// Create a new OpenAPI builder.
    ///
    /// # Example
    ///
    /// ```rust
    /// use domainstack_schema::OpenApiBuilder;
    ///
    /// let spec = OpenApiBuilder::new("My API", "1.0.0")
    ///     .description("A sample API")
    ///     .build();
    /// ```
    pub fn new(title: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            version: version.into(),
            description: None,
            schemas: HashMap::new(),
        }
    }

    /// Set the API description.
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Register a type that implements ToSchema.
    ///
    /// # Example
    ///
    /// ```rust
    /// use domainstack_schema::{OpenApiBuilder, ToSchema, Schema};
    ///
    /// struct User;
    /// impl ToSchema for User {
    ///     fn schema_name() -> &'static str { "User" }
    ///     fn schema() -> Schema { Schema::object() }
    /// }
    ///
    /// let spec = OpenApiBuilder::new("API", "1.0")
    ///     .register::<User>()
    ///     .build();
    /// ```
    pub fn register<T: ToSchema>(mut self) -> Self {
        self.schemas
            .insert(T::schema_name().to_string(), T::schema());
        self
    }

    /// Manually add a schema with a custom name.
    pub fn schema(mut self, name: impl Into<String>, schema: Schema) -> Self {
        self.schemas.insert(name.into(), schema);
        self
    }

    /// Build the final OpenAPI specification.
    pub fn build(self) -> OpenApiSpec {
        OpenApiSpec {
            openapi: "3.0.0".to_string(),
            info: Info {
                title: self.title,
                version: self.version,
                description: self.description,
            },
            components: Components {
                schemas: self.schemas,
            },
        }
    }
}

impl OpenApiSpec {
    /// Create a new builder.
    pub fn builder(title: impl Into<String>, version: impl Into<String>) -> OpenApiBuilder {
        OpenApiBuilder::new(title, version)
    }

    /// Convert to JSON string.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Convert to YAML string (requires serde_yaml - not included by default).
    #[cfg(feature = "yaml")]
    pub fn to_yaml(&self) -> Result<String, serde_yaml::Error> {
        serde_yaml::to_string(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct User;
    impl ToSchema for User {
        fn schema_name() -> &'static str {
            "User"
        }

        fn schema() -> Schema {
            Schema::object()
                .property("email", Schema::string().format("email"))
                .property("age", Schema::integer().minimum(0).maximum(150))
                .required(&["email", "age"])
        }
    }

    struct Product;
    impl ToSchema for Product {
        fn schema_name() -> &'static str {
            "Product"
        }

        fn schema() -> Schema {
            Schema::object()
                .property("name", Schema::string())
                .property("price", Schema::number().minimum(0.0))
                .required(&["name", "price"])
        }
    }

    #[test]
    fn test_openapi_builder() {
        let spec = OpenApiBuilder::new("Test API", "1.0.0")
            .description("A test API")
            .register::<User>()
            .register::<Product>()
            .build();

        assert_eq!(spec.openapi, "3.0.0");
        assert_eq!(spec.info.title, "Test API");
        assert_eq!(spec.info.version, "1.0.0");
        assert_eq!(spec.components.schemas.len(), 2);
        assert!(spec.components.schemas.contains_key("User"));
        assert!(spec.components.schemas.contains_key("Product"));
    }

    #[test]
    fn test_to_json() {
        let spec = OpenApiBuilder::new("API", "1.0").register::<User>().build();

        let json = spec.to_json().unwrap();
        assert!(json.contains("\"openapi\": \"3.0.0\""));
        assert!(json.contains("\"User\""));
    }

    #[test]
    fn test_manual_schema() {
        let custom_schema = Schema::string().format("custom");

        let spec = OpenApiBuilder::new("API", "1.0")
            .schema("CustomType", custom_schema)
            .build();

        assert_eq!(spec.components.schemas.len(), 1);
        assert!(spec.components.schemas.contains_key("CustomType"));
    }
}

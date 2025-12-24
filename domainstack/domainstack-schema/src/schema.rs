//! OpenAPI schema type definitions.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// OpenAPI schema object representing a data type.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Schema {
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub schema_type: Option<SchemaType>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<HashMap<String, Schema>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<Box<Schema>>,

    // String constraints
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_length: Option<usize>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_length: Option<usize>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub pattern: Option<String>,

    // Numeric constraints
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minimum: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub maximum: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub multiple_of: Option<f64>,

    // Array constraints
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_items: Option<usize>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_items: Option<usize>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub unique_items: Option<bool>,

    // Enum constraint
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#enum: Option<Vec<serde_json::Value>>,

    // Reference to another schema
    #[serde(rename = "$ref", skip_serializing_if = "Option::is_none")]
    pub reference: Option<String>,

    // Schema composition (v0.8)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub any_of: Option<Vec<Schema>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub all_of: Option<Vec<Schema>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub one_of: Option<Vec<Schema>>,

    // Metadata (v0.8)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<serde_json::Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub example: Option<serde_json::Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub examples: Option<Vec<serde_json::Value>>,

    // Field modifiers (v0.8)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub read_only: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub write_only: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub deprecated: Option<bool>,

    // Vendor extensions (v0.8) - for non-mappable validations
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<HashMap<String, serde_json::Value>>,
}

/// OpenAPI schema types.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SchemaType {
    String,
    Number,
    Integer,
    Boolean,
    Array,
    Object,
}

impl Schema {
    /// Create a new empty schema.
    pub fn new() -> Self {
        Self {
            schema_type: None,
            format: None,
            description: None,
            properties: None,
            required: None,
            items: None,
            min_length: None,
            max_length: None,
            pattern: None,
            minimum: None,
            maximum: None,
            multiple_of: None,
            min_items: None,
            max_items: None,
            unique_items: None,
            r#enum: None,
            reference: None,
            any_of: None,
            all_of: None,
            one_of: None,
            default: None,
            example: None,
            examples: None,
            read_only: None,
            write_only: None,
            deprecated: None,
            extensions: None,
        }
    }

    /// Create a string schema.
    pub fn string() -> Self {
        Self {
            schema_type: Some(SchemaType::String),
            ..Self::new()
        }
    }

    /// Create an integer schema.
    pub fn integer() -> Self {
        Self {
            schema_type: Some(SchemaType::Integer),
            ..Self::new()
        }
    }

    /// Create a number schema.
    pub fn number() -> Self {
        Self {
            schema_type: Some(SchemaType::Number),
            ..Self::new()
        }
    }

    /// Create a boolean schema.
    pub fn boolean() -> Self {
        Self {
            schema_type: Some(SchemaType::Boolean),
            ..Self::new()
        }
    }

    /// Create an array schema.
    pub fn array(items: Schema) -> Self {
        Self {
            schema_type: Some(SchemaType::Array),
            items: Some(Box::new(items)),
            ..Self::new()
        }
    }

    /// Create an object schema.
    pub fn object() -> Self {
        Self {
            schema_type: Some(SchemaType::Object),
            properties: Some(HashMap::new()),
            ..Self::new()
        }
    }

    /// Create a reference to another schema.
    pub fn reference(name: &str) -> Self {
        Self {
            reference: Some(format!("#/components/schemas/{}", name)),
            ..Self::new()
        }
    }

    /// Set the format (e.g., "email", "date-time").
    pub fn format(mut self, format: impl Into<String>) -> Self {
        self.format = Some(format.into());
        self
    }

    /// Set the description.
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Add a property to an object schema.
    pub fn property(mut self, name: impl Into<String>, schema: Schema) -> Self {
        self.properties
            .get_or_insert_with(HashMap::new)
            .insert(name.into(), schema);
        self
    }

    /// Set required fields.
    pub fn required(mut self, fields: &[&str]) -> Self {
        self.required = Some(fields.iter().map(|s| s.to_string()).collect());
        self
    }

    /// Set minimum length for strings.
    pub fn min_length(mut self, min: usize) -> Self {
        self.min_length = Some(min);
        self
    }

    /// Set maximum length for strings.
    pub fn max_length(mut self, max: usize) -> Self {
        self.max_length = Some(max);
        self
    }

    /// Set regex pattern for strings.
    pub fn pattern(mut self, pattern: impl Into<String>) -> Self {
        self.pattern = Some(pattern.into());
        self
    }

    /// Set minimum value for numbers.
    pub fn minimum(mut self, min: impl Into<f64>) -> Self {
        self.minimum = Some(min.into());
        self
    }

    /// Set maximum value for numbers.
    pub fn maximum(mut self, max: impl Into<f64>) -> Self {
        self.maximum = Some(max.into());
        self
    }

    /// Set multiple_of constraint for numbers.
    pub fn multiple_of(mut self, divisor: impl Into<f64>) -> Self {
        self.multiple_of = Some(divisor.into());
        self
    }

    /// Set minimum items for arrays.
    pub fn min_items(mut self, min: usize) -> Self {
        self.min_items = Some(min);
        self
    }

    /// Set maximum items for arrays.
    pub fn max_items(mut self, max: usize) -> Self {
        self.max_items = Some(max);
        self
    }

    /// Set unique items constraint for arrays.
    pub fn unique_items(mut self, unique: bool) -> Self {
        self.unique_items = Some(unique);
        self
    }

    /// Set enum values.
    pub fn enum_values<T: Serialize>(mut self, values: &[T]) -> Self {
        self.r#enum = Some(
            values
                .iter()
                .map(|v| serde_json::to_value(v).unwrap())
                .collect(),
        );
        self
    }

    // === v0.8 features ===

    /// Create a schema that matches any of the given schemas (union type).
    ///
    /// # Example
    /// ```rust
    /// use domainstack_schema::Schema;
    ///
    /// let schema = Schema::any_of(vec![
    ///     Schema::string(),
    ///     Schema::integer(),
    /// ]);
    /// ```
    pub fn any_of(schemas: Vec<Schema>) -> Self {
        Self {
            any_of: Some(schemas),
            ..Self::new()
        }
    }

    /// Create a schema that matches all of the given schemas (intersection/composition).
    ///
    /// # Example
    /// ```rust
    /// use domainstack_schema::Schema;
    ///
    /// let schema = Schema::all_of(vec![
    ///     Schema::reference("BaseUser"),
    ///     Schema::object().property("admin", Schema::boolean()),
    /// ]);
    /// ```
    pub fn all_of(schemas: Vec<Schema>) -> Self {
        Self {
            all_of: Some(schemas),
            ..Self::new()
        }
    }

    /// Create a schema that matches exactly one of the given schemas (discriminated union).
    ///
    /// # Example
    /// ```rust
    /// use domainstack_schema::Schema;
    ///
    /// let schema = Schema::one_of(vec![
    ///     Schema::object().property("type", Schema::string().enum_values(&["card"])),
    ///     Schema::object().property("type", Schema::string().enum_values(&["cash"])),
    /// ]);
    /// ```
    pub fn one_of(schemas: Vec<Schema>) -> Self {
        Self {
            one_of: Some(schemas),
            ..Self::new()
        }
    }

    /// Set a default value for this schema.
    ///
    /// # Example
    /// ```rust
    /// use domainstack_schema::Schema;
    /// use serde_json::json;
    ///
    /// let schema = Schema::string().default(json!("guest"));
    /// ```
    pub fn default<T: Serialize>(mut self, value: T) -> Self {
        self.default = Some(serde_json::to_value(value).unwrap());
        self
    }

    /// Set an example value for this schema.
    ///
    /// # Example
    /// ```rust
    /// use domainstack_schema::Schema;
    /// use serde_json::json;
    ///
    /// let schema = Schema::string().example(json!("john_doe"));
    /// ```
    pub fn example<T: Serialize>(mut self, value: T) -> Self {
        self.example = Some(serde_json::to_value(value).unwrap());
        self
    }

    /// Set multiple example values for this schema.
    ///
    /// # Example
    /// ```rust
    /// use domainstack_schema::Schema;
    /// use serde_json::json;
    ///
    /// let schema = Schema::string().examples(vec![
    ///     json!("alice"),
    ///     json!("bob"),
    /// ]);
    /// ```
    pub fn examples<T: Serialize>(mut self, values: Vec<T>) -> Self {
        self.examples = Some(
            values
                .into_iter()
                .map(|v| serde_json::to_value(v).unwrap())
                .collect(),
        );
        self
    }

    /// Mark this field as read-only (returned in responses, not accepted in requests).
    ///
    /// # Example
    /// ```rust
    /// use domainstack_schema::Schema;
    ///
    /// let schema = Schema::string().read_only(true);
    /// ```
    pub fn read_only(mut self, read_only: bool) -> Self {
        self.read_only = Some(read_only);
        self
    }

    /// Mark this field as write-only (accepted in requests, not returned in responses).
    ///
    /// # Example
    /// ```rust
    /// use domainstack_schema::Schema;
    ///
    /// let password = Schema::string()
    ///     .format("password")
    ///     .write_only(true);
    /// ```
    pub fn write_only(mut self, write_only: bool) -> Self {
        self.write_only = Some(write_only);
        self
    }

    /// Mark this field as deprecated.
    ///
    /// # Example
    /// ```rust
    /// use domainstack_schema::Schema;
    ///
    /// let schema = Schema::string()
    ///     .deprecated(true)
    ///     .description("Use 'new_field' instead");
    /// ```
    pub fn deprecated(mut self, deprecated: bool) -> Self {
        self.deprecated = Some(deprecated);
        self
    }

    /// Add a vendor extension (for validations that don't map to OpenAPI).
    ///
    /// Extension keys should start with "x-".
    ///
    /// # Example
    /// ```rust
    /// use domainstack_schema::Schema;
    /// use serde_json::json;
    ///
    /// let schema = Schema::object()
    ///     .property("end_date", Schema::string().format("date"))
    ///     .extension("x-domainstack-validations", json!({
    ///         "cross_field": ["end_date > start_date"]
    ///     }));
    /// ```
    pub fn extension<T: Serialize>(mut self, key: impl Into<String>, value: T) -> Self {
        self.extensions
            .get_or_insert_with(HashMap::new)
            .insert(key.into(), serde_json::to_value(value).unwrap());
        self
    }
}

impl Default for Schema {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_schema() {
        let schema = Schema::string()
            .min_length(5)
            .max_length(100)
            .format("email");

        assert!(matches!(schema.schema_type, Some(SchemaType::String)));
        assert_eq!(schema.min_length, Some(5));
        assert_eq!(schema.max_length, Some(100));
        assert_eq!(schema.format, Some("email".to_string()));
    }

    #[test]
    fn test_integer_schema() {
        let schema = Schema::integer().minimum(0).maximum(100);

        assert!(matches!(schema.schema_type, Some(SchemaType::Integer)));
        assert_eq!(schema.minimum, Some(0.0));
        assert_eq!(schema.maximum, Some(100.0));
    }

    #[test]
    fn test_object_schema() {
        let schema = Schema::object()
            .property("name", Schema::string())
            .property("age", Schema::integer().minimum(0))
            .required(&["name", "age"]);

        assert!(matches!(schema.schema_type, Some(SchemaType::Object)));
        assert_eq!(schema.properties.as_ref().unwrap().len(), 2);
        assert_eq!(schema.required.as_ref().unwrap().len(), 2);
    }

    #[test]
    fn test_array_schema() {
        let schema = Schema::array(Schema::string())
            .min_items(1)
            .max_items(10)
            .unique_items(true);

        assert!(matches!(schema.schema_type, Some(SchemaType::Array)));
        assert_eq!(schema.min_items, Some(1));
        assert_eq!(schema.max_items, Some(10));
        assert_eq!(schema.unique_items, Some(true));
    }

    #[test]
    fn test_schema_serialization() {
        let schema = Schema::object()
            .property("email", Schema::string().format("email"))
            .property("age", Schema::integer().minimum(18).maximum(120))
            .required(&["email", "age"]);

        let json = serde_json::to_string_pretty(&schema).unwrap();
        assert!(json.contains("\"type\": \"object\""));
        assert!(json.contains("\"email\""));
        assert!(json.contains("\"age\""));
    }

    // === v0.8 tests ===

    #[test]
    fn test_any_of_composition() {
        let schema = Schema::any_of(vec![Schema::string(), Schema::integer()]);

        assert!(schema.any_of.is_some());
        assert_eq!(schema.any_of.as_ref().unwrap().len(), 2);
    }

    #[test]
    fn test_all_of_composition() {
        let schema = Schema::all_of(vec![
            Schema::reference("BaseUser"),
            Schema::object().property("admin", Schema::boolean()),
        ]);

        assert!(schema.all_of.is_some());
        assert_eq!(schema.all_of.as_ref().unwrap().len(), 2);
    }

    #[test]
    fn test_one_of_composition() {
        let schema = Schema::one_of(vec![
            Schema::object().property("type", Schema::string()),
            Schema::object().property("kind", Schema::string()),
        ]);

        assert!(schema.one_of.is_some());
        assert_eq!(schema.one_of.as_ref().unwrap().len(), 2);
    }

    #[test]
    fn test_default_value() {
        use serde_json::json;

        let schema = Schema::string().default(json!("guest"));

        assert!(schema.default.is_some());
        assert_eq!(schema.default.unwrap(), json!("guest"));
    }

    #[test]
    fn test_example() {
        use serde_json::json;

        let schema = Schema::string().example(json!("john_doe"));

        assert!(schema.example.is_some());
        assert_eq!(schema.example.unwrap(), json!("john_doe"));
    }

    #[test]
    fn test_examples() {
        use serde_json::json;

        let schema = Schema::string().examples(vec![json!("alice"), json!("bob")]);

        assert!(schema.examples.is_some());
        assert_eq!(schema.examples.as_ref().unwrap().len(), 2);
    }

    #[test]
    fn test_read_only() {
        let schema = Schema::string().read_only(true);

        assert_eq!(schema.read_only, Some(true));
    }

    #[test]
    fn test_write_only() {
        let schema = Schema::string().format("password").write_only(true);

        assert_eq!(schema.write_only, Some(true));
        assert_eq!(schema.format, Some("password".to_string()));
    }

    #[test]
    fn test_deprecated() {
        let schema = Schema::string().deprecated(true);

        assert_eq!(schema.deprecated, Some(true));
    }

    #[test]
    fn test_vendor_extension() {
        use serde_json::json;

        let schema = Schema::object().extension(
            "x-domainstack-validations",
            json!({"cross_field": ["end > start"]}),
        );

        assert!(schema.extensions.is_some());
        let extensions = schema.extensions.as_ref().unwrap();
        assert!(extensions.contains_key("x-domainstack-validations"));
    }

    #[test]
    fn test_composition_serialization() {
        let schema = Schema::any_of(vec![Schema::string(), Schema::integer()]);

        let json_value = serde_json::to_value(&schema).unwrap();
        assert!(json_value.get("anyOf").is_some());
    }

    #[test]
    fn test_read_write_only_request_response() {
        // Password: write-only (send in request, never returned)
        let password = Schema::string()
            .format("password")
            .write_only(true)
            .min_length(8);

        // ID: read-only (returned in response, never accepted in request)
        let id = Schema::string().read_only(true);

        let user_schema = Schema::object()
            .property("id", id)
            .property("email", Schema::string().format("email"))
            .property("password", password)
            .required(&["email", "password"]);

        let json = serde_json::to_string_pretty(&user_schema).unwrap();
        assert!(json.contains("\"writeOnly\": true"));
        assert!(json.contains("\"readOnly\": true"));
    }
}

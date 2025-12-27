//! JSON Schema (Draft 2020-12) generation for domainstack validation types.
//!
//! This module provides a trait-based approach to JSON Schema generation,
//! complementing the CLI-based approach for cases where programmatic
//! schema generation is preferred.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// JSON Schema document (Draft 2020-12)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JsonSchema {
    #[serde(rename = "$schema", skip_serializing_if = "Option::is_none")]
    pub schema: Option<String>,

    #[serde(rename = "$id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub schema_type: Option<JsonSchemaType>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<HashMap<String, JsonSchema>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<Box<JsonSchema>>,

    #[serde(
        rename = "additionalProperties",
        skip_serializing_if = "Option::is_none"
    )]
    pub additional_properties: Option<AdditionalProperties>,

    // String constraints
    #[serde(rename = "minLength", skip_serializing_if = "Option::is_none")]
    pub min_length: Option<usize>,

    #[serde(rename = "maxLength", skip_serializing_if = "Option::is_none")]
    pub max_length: Option<usize>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub pattern: Option<String>,

    // Numeric constraints
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minimum: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub maximum: Option<f64>,

    #[serde(rename = "exclusiveMinimum", skip_serializing_if = "Option::is_none")]
    pub exclusive_minimum: Option<f64>,

    #[serde(rename = "exclusiveMaximum", skip_serializing_if = "Option::is_none")]
    pub exclusive_maximum: Option<f64>,

    #[serde(rename = "multipleOf", skip_serializing_if = "Option::is_none")]
    pub multiple_of: Option<f64>,

    // Array constraints
    #[serde(rename = "minItems", skip_serializing_if = "Option::is_none")]
    pub min_items: Option<usize>,

    #[serde(rename = "maxItems", skip_serializing_if = "Option::is_none")]
    pub max_items: Option<usize>,

    #[serde(rename = "uniqueItems", skip_serializing_if = "Option::is_none")]
    pub unique_items: Option<bool>,

    // Enum constraint
    #[serde(rename = "enum", skip_serializing_if = "Option::is_none")]
    pub r#enum: Option<Vec<serde_json::Value>>,

    #[serde(rename = "const", skip_serializing_if = "Option::is_none")]
    pub r#const: Option<serde_json::Value>,

    // Reference
    #[serde(rename = "$ref", skip_serializing_if = "Option::is_none")]
    pub reference: Option<String>,

    // Composition
    #[serde(rename = "anyOf", skip_serializing_if = "Option::is_none")]
    pub any_of: Option<Vec<JsonSchema>>,

    #[serde(rename = "allOf", skip_serializing_if = "Option::is_none")]
    pub all_of: Option<Vec<JsonSchema>>,

    #[serde(rename = "oneOf", skip_serializing_if = "Option::is_none")]
    pub one_of: Option<Vec<JsonSchema>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub not: Option<Box<JsonSchema>>,

    // Metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<serde_json::Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub examples: Option<Vec<serde_json::Value>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub deprecated: Option<bool>,

    #[serde(rename = "readOnly", skip_serializing_if = "Option::is_none")]
    pub read_only: Option<bool>,

    #[serde(rename = "writeOnly", skip_serializing_if = "Option::is_none")]
    pub write_only: Option<bool>,

    // $defs for schema definitions
    #[serde(rename = "$defs", skip_serializing_if = "Option::is_none")]
    pub defs: Option<HashMap<String, JsonSchema>>,
}

/// JSON Schema types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum JsonSchemaType {
    String,
    Number,
    Integer,
    Boolean,
    Array,
    Object,
    Null,
}

/// Additional properties can be a boolean or a schema
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AdditionalProperties {
    Bool(bool),
    Schema(Box<JsonSchema>),
}

impl JsonSchema {
    /// Create a new empty schema
    pub fn new() -> Self {
        Self {
            schema: None,
            id: None,
            title: None,
            description: None,
            schema_type: None,
            format: None,
            properties: None,
            required: None,
            items: None,
            additional_properties: None,
            min_length: None,
            max_length: None,
            pattern: None,
            minimum: None,
            maximum: None,
            exclusive_minimum: None,
            exclusive_maximum: None,
            multiple_of: None,
            min_items: None,
            max_items: None,
            unique_items: None,
            r#enum: None,
            r#const: None,
            reference: None,
            any_of: None,
            all_of: None,
            one_of: None,
            not: None,
            default: None,
            examples: None,
            deprecated: None,
            read_only: None,
            write_only: None,
            defs: None,
        }
    }

    /// Create a string schema
    pub fn string() -> Self {
        Self {
            schema_type: Some(JsonSchemaType::String),
            ..Self::new()
        }
    }

    /// Create an integer schema
    pub fn integer() -> Self {
        Self {
            schema_type: Some(JsonSchemaType::Integer),
            ..Self::new()
        }
    }

    /// Create a number schema
    pub fn number() -> Self {
        Self {
            schema_type: Some(JsonSchemaType::Number),
            ..Self::new()
        }
    }

    /// Create a boolean schema
    pub fn boolean() -> Self {
        Self {
            schema_type: Some(JsonSchemaType::Boolean),
            ..Self::new()
        }
    }

    /// Create an array schema
    pub fn array(items: JsonSchema) -> Self {
        Self {
            schema_type: Some(JsonSchemaType::Array),
            items: Some(Box::new(items)),
            ..Self::new()
        }
    }

    /// Create an object schema
    pub fn object() -> Self {
        Self {
            schema_type: Some(JsonSchemaType::Object),
            properties: Some(HashMap::new()),
            additional_properties: Some(AdditionalProperties::Bool(false)),
            ..Self::new()
        }
    }

    /// Create a null schema
    pub fn null() -> Self {
        Self {
            schema_type: Some(JsonSchemaType::Null),
            ..Self::new()
        }
    }

    /// Create a reference to another schema
    pub fn reference(name: &str) -> Self {
        Self {
            reference: Some(format!("#/$defs/{}", name)),
            ..Self::new()
        }
    }

    /// Set the title
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set the description
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Set the format
    pub fn format(mut self, format: impl Into<String>) -> Self {
        self.format = Some(format.into());
        self
    }

    /// Add a property to an object schema
    pub fn property(mut self, name: impl Into<String>, schema: JsonSchema) -> Self {
        self.properties
            .get_or_insert_with(HashMap::new)
            .insert(name.into(), schema);
        self
    }

    /// Set required fields
    pub fn required(mut self, fields: &[&str]) -> Self {
        self.required = Some(fields.iter().map(|s| s.to_string()).collect());
        self
    }

    /// Set minimum length for strings
    pub fn min_length(mut self, min: usize) -> Self {
        self.min_length = Some(min);
        self
    }

    /// Set maximum length for strings
    pub fn max_length(mut self, max: usize) -> Self {
        self.max_length = Some(max);
        self
    }

    /// Set regex pattern for strings
    pub fn pattern(mut self, pattern: impl Into<String>) -> Self {
        self.pattern = Some(pattern.into());
        self
    }

    /// Set minimum value for numbers
    pub fn minimum(mut self, min: impl Into<f64>) -> Self {
        self.minimum = Some(min.into());
        self
    }

    /// Set maximum value for numbers
    pub fn maximum(mut self, max: impl Into<f64>) -> Self {
        self.maximum = Some(max.into());
        self
    }

    /// Set exclusive minimum for numbers
    pub fn exclusive_minimum(mut self, min: impl Into<f64>) -> Self {
        self.exclusive_minimum = Some(min.into());
        self
    }

    /// Set exclusive maximum for numbers
    pub fn exclusive_maximum(mut self, max: impl Into<f64>) -> Self {
        self.exclusive_maximum = Some(max.into());
        self
    }

    /// Set multipleOf constraint for numbers
    pub fn multiple_of(mut self, divisor: impl Into<f64>) -> Self {
        self.multiple_of = Some(divisor.into());
        self
    }

    /// Set minimum items for arrays
    pub fn min_items(mut self, min: usize) -> Self {
        self.min_items = Some(min);
        self
    }

    /// Set maximum items for arrays
    pub fn max_items(mut self, max: usize) -> Self {
        self.max_items = Some(max);
        self
    }

    /// Set unique items constraint for arrays
    pub fn unique_items(mut self, unique: bool) -> Self {
        self.unique_items = Some(unique);
        self
    }

    /// Set enum values
    pub fn enum_values<T: Serialize>(mut self, values: &[T]) -> Self {
        self.r#enum = Some(
            values
                .iter()
                .map(|v| serde_json::to_value(v).expect("Failed to serialize enum value"))
                .collect(),
        );
        self
    }

    /// Set a const value
    pub fn const_value<T: Serialize>(mut self, value: T) -> Self {
        self.r#const = Some(serde_json::to_value(value).expect("Failed to serialize const value"));
        self
    }

    /// Set a default value
    pub fn default<T: Serialize>(mut self, value: T) -> Self {
        self.default =
            Some(serde_json::to_value(value).expect("Failed to serialize default value"));
        self
    }

    /// Set example values
    pub fn examples<T: Serialize>(mut self, values: Vec<T>) -> Self {
        self.examples = Some(
            values
                .into_iter()
                .map(|v| serde_json::to_value(v).expect("Failed to serialize example value"))
                .collect(),
        );
        self
    }

    /// Mark as deprecated
    pub fn deprecated(mut self, deprecated: bool) -> Self {
        self.deprecated = Some(deprecated);
        self
    }

    /// Mark as read-only
    pub fn read_only(mut self, read_only: bool) -> Self {
        self.read_only = Some(read_only);
        self
    }

    /// Mark as write-only
    pub fn write_only(mut self, write_only: bool) -> Self {
        self.write_only = Some(write_only);
        self
    }

    /// Create an anyOf schema
    pub fn any_of(schemas: Vec<JsonSchema>) -> Self {
        Self {
            any_of: Some(schemas),
            ..Self::new()
        }
    }

    /// Create an allOf schema
    pub fn all_of(schemas: Vec<JsonSchema>) -> Self {
        Self {
            all_of: Some(schemas),
            ..Self::new()
        }
    }

    /// Create a oneOf schema
    pub fn one_of(schemas: Vec<JsonSchema>) -> Self {
        Self {
            one_of: Some(schemas),
            ..Self::new()
        }
    }

    /// Create a negation schema (not)
    pub fn negation(schema: JsonSchema) -> Self {
        Self {
            not: Some(Box::new(schema)),
            ..Self::new()
        }
    }
}

impl Default for JsonSchema {
    fn default() -> Self {
        Self::new()
    }
}

/// Types that can generate JSON Schema (Draft 2020-12).
///
/// This trait provides programmatic JSON Schema generation as an alternative
/// to the CLI-based approach.
///
/// # Example
///
/// ```rust
/// use domainstack_schema::{ToJsonSchema, JsonSchema};
///
/// struct User {
///     email: String,
///     age: u8,
/// }
///
/// impl ToJsonSchema for User {
///     fn schema_name() -> &'static str {
///         "User"
///     }
///
///     fn json_schema() -> JsonSchema {
///         JsonSchema::object()
///             .property("email", JsonSchema::string().format("email"))
///             .property("age", JsonSchema::integer().minimum(0).maximum(150))
///             .required(&["email", "age"])
///     }
/// }
/// ```
pub trait ToJsonSchema {
    /// The name of this schema in the $defs section.
    fn schema_name() -> &'static str;

    /// Generate the JSON Schema for this type.
    fn json_schema() -> JsonSchema;
}

// Implementations for primitive types
impl ToJsonSchema for String {
    fn schema_name() -> &'static str {
        "string"
    }

    fn json_schema() -> JsonSchema {
        JsonSchema::string()
    }
}

impl ToJsonSchema for str {
    fn schema_name() -> &'static str {
        "string"
    }

    fn json_schema() -> JsonSchema {
        JsonSchema::string()
    }
}

impl ToJsonSchema for u8 {
    fn schema_name() -> &'static str {
        "integer"
    }

    fn json_schema() -> JsonSchema {
        JsonSchema::integer().minimum(0).maximum(255)
    }
}

impl ToJsonSchema for u16 {
    fn schema_name() -> &'static str {
        "integer"
    }

    fn json_schema() -> JsonSchema {
        JsonSchema::integer().minimum(0).maximum(65535)
    }
}

impl ToJsonSchema for u32 {
    fn schema_name() -> &'static str {
        "integer"
    }

    fn json_schema() -> JsonSchema {
        JsonSchema::integer().minimum(0)
    }
}

impl ToJsonSchema for i32 {
    fn schema_name() -> &'static str {
        "integer"
    }

    fn json_schema() -> JsonSchema {
        JsonSchema::integer()
    }
}

impl ToJsonSchema for i64 {
    fn schema_name() -> &'static str {
        "integer"
    }

    fn json_schema() -> JsonSchema {
        JsonSchema::integer()
    }
}

impl ToJsonSchema for f32 {
    fn schema_name() -> &'static str {
        "number"
    }

    fn json_schema() -> JsonSchema {
        JsonSchema::number()
    }
}

impl ToJsonSchema for f64 {
    fn schema_name() -> &'static str {
        "number"
    }

    fn json_schema() -> JsonSchema {
        JsonSchema::number()
    }
}

impl ToJsonSchema for bool {
    fn schema_name() -> &'static str {
        "boolean"
    }

    fn json_schema() -> JsonSchema {
        JsonSchema::boolean()
    }
}

impl<T: ToJsonSchema> ToJsonSchema for Vec<T> {
    fn schema_name() -> &'static str {
        "array"
    }

    fn json_schema() -> JsonSchema {
        JsonSchema::array(T::json_schema())
    }
}

impl<T: ToJsonSchema> ToJsonSchema for Option<T> {
    fn schema_name() -> &'static str {
        T::schema_name()
    }

    fn json_schema() -> JsonSchema {
        T::json_schema()
    }
}

/// Builder for creating JSON Schema documents with $defs
pub struct JsonSchemaBuilder {
    id: Option<String>,
    title: Option<String>,
    description: Option<String>,
    defs: HashMap<String, JsonSchema>,
}

impl JsonSchemaBuilder {
    /// Create a new JSON Schema builder
    pub fn new() -> Self {
        Self {
            id: None,
            title: None,
            description: None,
            defs: HashMap::new(),
        }
    }

    /// Set the $id
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Set the title
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set the description
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Register a type that implements ToJsonSchema
    pub fn register<T: ToJsonSchema>(mut self) -> Self {
        self.defs
            .insert(T::schema_name().to_string(), T::json_schema());
        self
    }

    /// Add a schema with a custom name
    pub fn add_schema(mut self, name: impl Into<String>, schema: JsonSchema) -> Self {
        self.defs.insert(name.into(), schema);
        self
    }

    /// Build the final JSON Schema document
    pub fn build(self) -> JsonSchema {
        JsonSchema {
            schema: Some("https://json-schema.org/draft/2020-12/schema".to_string()),
            id: self.id,
            title: self.title,
            description: self.description,
            defs: if self.defs.is_empty() {
                None
            } else {
                Some(self.defs)
            },
            ..JsonSchema::new()
        }
    }

    /// Build and serialize to JSON string
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        let schema = JsonSchema {
            schema: Some("https://json-schema.org/draft/2020-12/schema".to_string()),
            id: self.id.clone(),
            title: self.title.clone(),
            description: self.description.clone(),
            defs: if self.defs.is_empty() {
                None
            } else {
                Some(self.defs.clone())
            },
            ..JsonSchema::new()
        };
        serde_json::to_string_pretty(&schema)
    }
}

impl Default for JsonSchemaBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_schema() {
        let schema = JsonSchema::string().min_length(3).max_length(50);
        assert!(matches!(schema.schema_type, Some(JsonSchemaType::String)));
        assert_eq!(schema.min_length, Some(3));
        assert_eq!(schema.max_length, Some(50));
    }

    #[test]
    fn test_integer_schema() {
        let schema = JsonSchema::integer().minimum(0).maximum(100);
        assert!(matches!(schema.schema_type, Some(JsonSchemaType::Integer)));
        assert_eq!(schema.minimum, Some(0.0));
        assert_eq!(schema.maximum, Some(100.0));
    }

    #[test]
    fn test_object_schema() {
        let schema = JsonSchema::object()
            .property("name", JsonSchema::string())
            .property("age", JsonSchema::integer())
            .required(&["name"]);

        assert!(matches!(schema.schema_type, Some(JsonSchemaType::Object)));
        assert_eq!(schema.properties.as_ref().unwrap().len(), 2);
        assert!(schema
            .required
            .as_ref()
            .unwrap()
            .contains(&"name".to_string()));
    }

    #[test]
    fn test_array_schema() {
        let schema = JsonSchema::array(JsonSchema::string())
            .min_items(1)
            .max_items(10);
        assert!(matches!(schema.schema_type, Some(JsonSchemaType::Array)));
        assert!(schema.items.is_some());
        assert_eq!(schema.min_items, Some(1));
    }

    #[test]
    fn test_reference() {
        let schema = JsonSchema::reference("User");
        assert_eq!(schema.reference, Some("#/$defs/User".to_string()));
    }

    #[test]
    fn test_any_of() {
        let schema = JsonSchema::any_of(vec![JsonSchema::string(), JsonSchema::integer()]);
        assert!(schema.any_of.is_some());
        assert_eq!(schema.any_of.as_ref().unwrap().len(), 2);
    }

    #[test]
    fn test_builder() {
        struct User;
        impl ToJsonSchema for User {
            fn schema_name() -> &'static str {
                "User"
            }

            fn json_schema() -> JsonSchema {
                JsonSchema::object()
                    .property("email", JsonSchema::string().format("email"))
                    .required(&["email"])
            }
        }

        let doc = JsonSchemaBuilder::new()
            .title("My Schema")
            .register::<User>()
            .build();

        assert!(doc.schema.is_some());
        assert_eq!(doc.title, Some("My Schema".to_string()));
        assert!(doc.defs.as_ref().unwrap().contains_key("User"));
    }

    #[test]
    fn test_serialization() {
        let schema = JsonSchema::object()
            .property("email", JsonSchema::string().format("email"))
            .property("age", JsonSchema::integer().minimum(0))
            .required(&["email", "age"]);

        let json = serde_json::to_string(&schema).unwrap();
        assert!(json.contains("\"type\":\"object\""));
        assert!(json.contains("\"email\""));
    }
}

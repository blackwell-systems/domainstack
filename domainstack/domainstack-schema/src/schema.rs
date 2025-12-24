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
}

use crate::parser::{FieldType, ParsedField, ParsedType, ValidationRule};
use anyhow::Result;
use serde_json::{json, Map, Value};

/// OpenAPI version to generate
#[derive(Debug, Clone, Copy, Default)]
pub enum OpenApiVersion {
    #[default]
    V3_0,
    V3_1,
}

/// Generate OpenAPI 3.0/3.1 specification from parsed types
pub fn generate(types: &[ParsedType], version: OpenApiVersion) -> Result<String> {
    let mut schemas = Map::new();

    // Generate schema for each type
    for parsed_type in types {
        let schema = generate_type_schema(parsed_type, version)?;
        schemas.insert(parsed_type.name.clone(), schema);
    }

    // Create the root OpenAPI document
    let openapi_version = match version {
        OpenApiVersion::V3_0 => "3.0.3",
        OpenApiVersion::V3_1 => "3.1.0",
    };

    let root_doc = json!({
        "openapi": openapi_version,
        "info": {
            "title": "Generated API Schema",
            "description": "Auto-generated OpenAPI specification from domainstack validation rules",
            "version": "1.0.0"
        },
        "paths": {},
        "components": {
            "schemas": schemas
        }
    });

    // Pretty print with 2-space indentation
    let output = serde_json::to_string_pretty(&root_doc)?;
    Ok(output)
}

fn generate_type_schema(parsed_type: &ParsedType, version: OpenApiVersion) -> Result<Value> {
    let mut properties = Map::new();
    let mut required = Vec::new();

    for field in &parsed_type.fields {
        let field_schema = generate_field_schema(field, version)?;
        properties.insert(field.name.clone(), field_schema);

        // Non-optional fields are required
        if !matches!(field.ty, FieldType::Option(_)) {
            required.push(Value::String(field.name.clone()));
        }
    }

    let mut schema = json!({
        "type": "object",
        "properties": properties,
        "additionalProperties": false
    });

    if !required.is_empty() {
        schema["required"] = Value::Array(required);
    }

    Ok(schema)
}

fn generate_field_schema(field: &ParsedField, version: OpenApiVersion) -> Result<Value> {
    // Get base schema for the type
    let mut schema = generate_base_type_schema(&field.ty, version);

    // Apply validation rules
    for rule in &field.validation_rules {
        apply_validation_rule(&mut schema, rule, &field.ty);
    }

    Ok(schema)
}

fn generate_base_type_schema(field_type: &FieldType, version: OpenApiVersion) -> Value {
    match field_type {
        FieldType::String => json!({ "type": "string" }),
        FieldType::Bool => json!({ "type": "boolean" }),
        FieldType::U8
        | FieldType::U16
        | FieldType::U32
        | FieldType::I8
        | FieldType::I16
        | FieldType::I32 => {
            json!({ "type": "integer" })
        }
        FieldType::U64 | FieldType::U128 | FieldType::I64 | FieldType::I128 => {
            json!({
                "type": "integer",
                "format": "int64"
            })
        }
        FieldType::F32 => json!({ "type": "number", "format": "float" }),
        FieldType::F64 => json!({ "type": "number", "format": "double" }),
        FieldType::Option(inner) => {
            // For Option<T>, we generate the inner type schema
            // In OpenAPI 3.0, we use nullable: true
            // In OpenAPI 3.1, we can use oneOf with null type
            let inner_schema = generate_base_type_schema(inner, version);
            match version {
                OpenApiVersion::V3_0 => {
                    let mut schema = inner_schema;
                    schema["nullable"] = json!(true);
                    schema
                }
                OpenApiVersion::V3_1 => {
                    json!({
                        "oneOf": [
                            inner_schema,
                            { "type": "null" }
                        ]
                    })
                }
            }
        }
        FieldType::Vec(inner) => {
            json!({
                "type": "array",
                "items": generate_base_type_schema(inner, version)
            })
        }
        FieldType::Custom(name) => {
            // Reference to another type in components/schemas
            json!({ "$ref": format!("#/components/schemas/{}", name) })
        }
    }
}

fn apply_validation_rule(schema: &mut Value, rule: &ValidationRule, _field_type: &FieldType) {
    match rule {
        // String validations
        ValidationRule::Email => {
            schema["format"] = json!("email");
        }
        ValidationRule::Url => {
            schema["format"] = json!("uri");
        }
        ValidationRule::MinLen(min) => {
            schema["minLength"] = json!(min);
        }
        ValidationRule::MaxLen(max) => {
            schema["maxLength"] = json!(max);
        }
        ValidationRule::Length { min, max } => {
            schema["minLength"] = json!(min);
            schema["maxLength"] = json!(max);
        }
        ValidationRule::NonEmpty => {
            schema["minLength"] = json!(1);
        }
        ValidationRule::NonBlank => {
            schema["minLength"] = json!(1);
            schema["pattern"] = json!(r"^\S.*$");
        }
        ValidationRule::Alphanumeric => {
            schema["pattern"] = json!("^[a-zA-Z0-9]*$");
        }
        ValidationRule::AlphaOnly => {
            schema["pattern"] = json!("^[a-zA-Z]*$");
        }
        ValidationRule::NumericString => {
            schema["pattern"] = json!("^[0-9]*$");
        }
        ValidationRule::Ascii => {
            schema["pattern"] = json!(r"^[\x00-\x7F]*$");
        }
        ValidationRule::StartsWith(prefix) => {
            schema["pattern"] = json!(format!("^{}", regex_escape(prefix)));
        }
        ValidationRule::EndsWith(suffix) => {
            schema["pattern"] = json!(format!("{}$", regex_escape(suffix)));
        }
        ValidationRule::Contains(substring) => {
            schema["pattern"] = json!(format!(".*{}.*", regex_escape(substring)));
        }
        ValidationRule::MatchesRegex(pattern) => {
            schema["pattern"] = json!(pattern);
        }
        ValidationRule::NoWhitespace => {
            schema["pattern"] = json!(r"^\S*$");
        }

        // Numeric validations
        ValidationRule::Range { min, max } => {
            if let Ok(min_num) = min.parse::<f64>() {
                schema["minimum"] = json!(min_num);
            }
            if let Ok(max_num) = max.parse::<f64>() {
                schema["maximum"] = json!(max_num);
            }
        }
        ValidationRule::Min(min) => {
            if let Ok(min_num) = min.parse::<f64>() {
                schema["minimum"] = json!(min_num);
            }
        }
        ValidationRule::Max(max) => {
            if let Ok(max_num) = max.parse::<f64>() {
                schema["maximum"] = json!(max_num);
            }
        }
        ValidationRule::Positive => {
            schema["exclusiveMinimum"] = json!(0);
        }
        ValidationRule::Negative => {
            schema["exclusiveMaximum"] = json!(0);
        }
        ValidationRule::NonZero => {
            // Use x-domainstack extension for validations that don't map directly
            let extensions = schema
                .as_object_mut()
                .unwrap()
                .entry("x-domainstack-validations")
                .or_insert_with(|| json!([]));
            if let Some(arr) = extensions.as_array_mut() {
                arr.push(json!("non_zero"));
            }
        }
        ValidationRule::MultipleOf(divisor) => {
            if let Ok(divisor_num) = divisor.parse::<f64>() {
                schema["multipleOf"] = json!(divisor_num);
            }
        }
        ValidationRule::Finite => {
            // All JSON numbers are finite by definition
            if schema.get("description").is_none() {
                schema["description"] = json!("Must be a finite number");
            }
        }

        // Custom rules
        ValidationRule::Custom(name) => {
            // Add as a custom extension
            schema["x-custom-validation"] = json!(name);
        }
    }
}

/// Escape special regex characters
fn regex_escape(s: &str) -> String {
    let special_chars = [
        '\\', '.', '+', '*', '?', '(', ')', '[', ']', '{', '}', '^', '$', '|',
    ];
    let mut result = String::with_capacity(s.len() * 2);
    for c in s.chars() {
        if special_chars.contains(&c) {
            result.push('\\');
        }
        result.push(c);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_basic_type() {
        let types = vec![ParsedType {
            name: "User".to_string(),
            fields: vec![
                ParsedField {
                    name: "email".to_string(),
                    ty: FieldType::String,
                    validation_rules: vec![ValidationRule::Email],
                },
                ParsedField {
                    name: "age".to_string(),
                    ty: FieldType::U8,
                    validation_rules: vec![ValidationRule::Range {
                        min: "18".to_string(),
                        max: "120".to_string(),
                    }],
                },
            ],
        }];

        let output = generate(&types, OpenApiVersion::V3_0).unwrap();
        let parsed: Value = serde_json::from_str(&output).unwrap();

        assert_eq!(parsed["openapi"], "3.0.3");
        assert!(parsed["components"]["schemas"]["User"].is_object());
        assert_eq!(
            parsed["components"]["schemas"]["User"]["properties"]["email"]["format"],
            "email"
        );
    }

    #[test]
    fn test_openapi_31_nullable() {
        let types = vec![ParsedType {
            name: "Profile".to_string(),
            fields: vec![ParsedField {
                name: "bio".to_string(),
                ty: FieldType::Option(Box::new(FieldType::String)),
                validation_rules: vec![],
            }],
        }];

        let output = generate(&types, OpenApiVersion::V3_1).unwrap();
        let parsed: Value = serde_json::from_str(&output).unwrap();

        assert_eq!(parsed["openapi"], "3.1.0");
        // OpenAPI 3.1 uses oneOf with null
        assert!(
            parsed["components"]["schemas"]["Profile"]["properties"]["bio"]["oneOf"].is_array()
        );
    }

    #[test]
    fn test_openapi_30_nullable() {
        let types = vec![ParsedType {
            name: "Profile".to_string(),
            fields: vec![ParsedField {
                name: "bio".to_string(),
                ty: FieldType::Option(Box::new(FieldType::String)),
                validation_rules: vec![],
            }],
        }];

        let output = generate(&types, OpenApiVersion::V3_0).unwrap();
        let parsed: Value = serde_json::from_str(&output).unwrap();

        // OpenAPI 3.0 uses nullable: true
        assert_eq!(
            parsed["components"]["schemas"]["Profile"]["properties"]["bio"]["nullable"],
            true
        );
    }

    #[test]
    fn test_custom_type_reference() {
        let types = vec![ParsedType {
            name: "Order".to_string(),
            fields: vec![ParsedField {
                name: "customer".to_string(),
                ty: FieldType::Custom("Customer".to_string()),
                validation_rules: vec![],
            }],
        }];

        let output = generate(&types, OpenApiVersion::V3_0).unwrap();
        let parsed: Value = serde_json::from_str(&output).unwrap();

        assert_eq!(
            parsed["components"]["schemas"]["Order"]["properties"]["customer"]["$ref"],
            "#/components/schemas/Customer"
        );
    }

    #[test]
    fn test_base_type_string() {
        let schema = generate_base_type_schema(&FieldType::String, OpenApiVersion::V3_0);
        assert_eq!(schema["type"], "string");
    }

    #[test]
    fn test_base_type_boolean() {
        let schema = generate_base_type_schema(&FieldType::Bool, OpenApiVersion::V3_0);
        assert_eq!(schema["type"], "boolean");
    }

    #[test]
    fn test_base_type_integers() {
        let schema = generate_base_type_schema(&FieldType::U8, OpenApiVersion::V3_0);
        assert_eq!(schema["type"], "integer");

        let schema = generate_base_type_schema(&FieldType::I32, OpenApiVersion::V3_0);
        assert_eq!(schema["type"], "integer");
    }

    #[test]
    fn test_base_type_large_integers() {
        let schema = generate_base_type_schema(&FieldType::U64, OpenApiVersion::V3_0);
        assert_eq!(schema["type"], "integer");
        assert_eq!(schema["format"], "int64");

        let schema = generate_base_type_schema(&FieldType::I128, OpenApiVersion::V3_0);
        assert_eq!(schema["type"], "integer");
        assert_eq!(schema["format"], "int64");
    }

    #[test]
    fn test_base_type_floats() {
        let schema = generate_base_type_schema(&FieldType::F32, OpenApiVersion::V3_0);
        assert_eq!(schema["type"], "number");
        assert_eq!(schema["format"], "float");

        let schema = generate_base_type_schema(&FieldType::F64, OpenApiVersion::V3_0);
        assert_eq!(schema["type"], "number");
        assert_eq!(schema["format"], "double");
    }

    #[test]
    fn test_base_type_array() {
        let schema = generate_base_type_schema(
            &FieldType::Vec(Box::new(FieldType::String)),
            OpenApiVersion::V3_0,
        );
        assert_eq!(schema["type"], "array");
        assert_eq!(schema["items"]["type"], "string");
    }

    #[test]
    fn test_nested_array() {
        let nested = FieldType::Vec(Box::new(FieldType::Vec(Box::new(FieldType::I32))));
        let schema = generate_base_type_schema(&nested, OpenApiVersion::V3_0);
        assert_eq!(schema["type"], "array");
        assert_eq!(schema["items"]["type"], "array");
        assert_eq!(schema["items"]["items"]["type"], "integer");
    }

    #[test]
    fn test_email_validation() {
        let mut schema = json!({ "type": "string" });
        apply_validation_rule(&mut schema, &ValidationRule::Email, &FieldType::String);
        assert_eq!(schema["format"], "email");
    }

    #[test]
    fn test_url_validation() {
        let mut schema = json!({ "type": "string" });
        apply_validation_rule(&mut schema, &ValidationRule::Url, &FieldType::String);
        assert_eq!(schema["format"], "uri");
    }

    #[test]
    fn test_min_len_validation() {
        let mut schema = json!({ "type": "string" });
        apply_validation_rule(&mut schema, &ValidationRule::MinLen(5), &FieldType::String);
        assert_eq!(schema["minLength"], 5);
    }

    #[test]
    fn test_max_len_validation() {
        let mut schema = json!({ "type": "string" });
        apply_validation_rule(
            &mut schema,
            &ValidationRule::MaxLen(100),
            &FieldType::String,
        );
        assert_eq!(schema["maxLength"], 100);
    }

    #[test]
    fn test_length_validation() {
        let mut schema = json!({ "type": "string" });
        apply_validation_rule(
            &mut schema,
            &ValidationRule::Length { min: 3, max: 20 },
            &FieldType::String,
        );
        assert_eq!(schema["minLength"], 3);
        assert_eq!(schema["maxLength"], 20);
    }

    #[test]
    fn test_non_empty_validation() {
        let mut schema = json!({ "type": "string" });
        apply_validation_rule(&mut schema, &ValidationRule::NonEmpty, &FieldType::String);
        assert_eq!(schema["minLength"], 1);
    }

    #[test]
    fn test_non_blank_validation() {
        let mut schema = json!({ "type": "string" });
        apply_validation_rule(&mut schema, &ValidationRule::NonBlank, &FieldType::String);
        assert_eq!(schema["minLength"], 1);
        assert!(schema["pattern"].as_str().is_some());
    }

    #[test]
    fn test_alphanumeric_validation() {
        let mut schema = json!({ "type": "string" });
        apply_validation_rule(
            &mut schema,
            &ValidationRule::Alphanumeric,
            &FieldType::String,
        );
        assert_eq!(schema["pattern"], "^[a-zA-Z0-9]*$");
    }

    #[test]
    fn test_alpha_only_validation() {
        let mut schema = json!({ "type": "string" });
        apply_validation_rule(&mut schema, &ValidationRule::AlphaOnly, &FieldType::String);
        assert_eq!(schema["pattern"], "^[a-zA-Z]*$");
    }

    #[test]
    fn test_numeric_string_validation() {
        let mut schema = json!({ "type": "string" });
        apply_validation_rule(
            &mut schema,
            &ValidationRule::NumericString,
            &FieldType::String,
        );
        assert_eq!(schema["pattern"], "^[0-9]*$");
    }

    #[test]
    fn test_ascii_validation() {
        let mut schema = json!({ "type": "string" });
        apply_validation_rule(&mut schema, &ValidationRule::Ascii, &FieldType::String);
        assert!(schema["pattern"].as_str().unwrap().contains("\\x00-\\x7F"));
    }

    #[test]
    fn test_no_whitespace_validation() {
        let mut schema = json!({ "type": "string" });
        apply_validation_rule(
            &mut schema,
            &ValidationRule::NoWhitespace,
            &FieldType::String,
        );
        assert_eq!(schema["pattern"], r"^\S*$");
    }

    #[test]
    fn test_starts_with_validation() {
        let mut schema = json!({ "type": "string" });
        apply_validation_rule(
            &mut schema,
            &ValidationRule::StartsWith("https://".to_string()),
            &FieldType::String,
        );
        assert!(schema["pattern"].as_str().unwrap().starts_with('^'));
    }

    #[test]
    fn test_ends_with_validation() {
        let mut schema = json!({ "type": "string" });
        apply_validation_rule(
            &mut schema,
            &ValidationRule::EndsWith(".com".to_string()),
            &FieldType::String,
        );
        assert!(schema["pattern"].as_str().unwrap().ends_with('$'));
    }

    #[test]
    fn test_contains_validation() {
        let mut schema = json!({ "type": "string" });
        apply_validation_rule(
            &mut schema,
            &ValidationRule::Contains("example".to_string()),
            &FieldType::String,
        );
        assert!(schema["pattern"].as_str().unwrap().contains("example"));
    }

    #[test]
    fn test_matches_regex_validation() {
        let mut schema = json!({ "type": "string" });
        apply_validation_rule(
            &mut schema,
            &ValidationRule::MatchesRegex("^[a-z]+$".to_string()),
            &FieldType::String,
        );
        assert_eq!(schema["pattern"], "^[a-z]+$");
    }

    #[test]
    fn test_range_validation() {
        let mut schema = json!({ "type": "integer" });
        apply_validation_rule(
            &mut schema,
            &ValidationRule::Range {
                min: "18".to_string(),
                max: "120".to_string(),
            },
            &FieldType::U8,
        );
        assert_eq!(schema["minimum"], 18.0);
        assert_eq!(schema["maximum"], 120.0);
    }

    #[test]
    fn test_min_validation() {
        let mut schema = json!({ "type": "integer" });
        apply_validation_rule(
            &mut schema,
            &ValidationRule::Min("0".to_string()),
            &FieldType::I32,
        );
        assert_eq!(schema["minimum"], 0.0);
    }

    #[test]
    fn test_max_validation() {
        let mut schema = json!({ "type": "integer" });
        apply_validation_rule(
            &mut schema,
            &ValidationRule::Max("1000".to_string()),
            &FieldType::I32,
        );
        assert_eq!(schema["maximum"], 1000.0);
    }

    #[test]
    fn test_positive_validation() {
        let mut schema = json!({ "type": "integer" });
        apply_validation_rule(&mut schema, &ValidationRule::Positive, &FieldType::I32);
        assert_eq!(schema["exclusiveMinimum"], 0);
    }

    #[test]
    fn test_negative_validation() {
        let mut schema = json!({ "type": "integer" });
        apply_validation_rule(&mut schema, &ValidationRule::Negative, &FieldType::I32);
        assert_eq!(schema["exclusiveMaximum"], 0);
    }

    #[test]
    fn test_non_zero_validation() {
        let mut schema = json!({ "type": "integer" });
        apply_validation_rule(&mut schema, &ValidationRule::NonZero, &FieldType::I32);
        assert!(schema["x-domainstack-validations"].is_array());
        assert!(schema["x-domainstack-validations"]
            .as_array()
            .unwrap()
            .contains(&json!("non_zero")));
    }

    #[test]
    fn test_multiple_of_validation() {
        let mut schema = json!({ "type": "integer" });
        apply_validation_rule(
            &mut schema,
            &ValidationRule::MultipleOf("5".to_string()),
            &FieldType::I32,
        );
        assert_eq!(schema["multipleOf"], 5.0);
    }

    #[test]
    fn test_finite_validation() {
        let mut schema = json!({ "type": "number" });
        apply_validation_rule(&mut schema, &ValidationRule::Finite, &FieldType::F64);
        assert!(schema["description"]
            .as_str()
            .unwrap()
            .contains("finite"));
    }

    #[test]
    fn test_custom_validation() {
        let mut schema = json!({ "type": "string" });
        apply_validation_rule(
            &mut schema,
            &ValidationRule::Custom("my_validator".to_string()),
            &FieldType::String,
        );
        assert_eq!(schema["x-custom-validation"], "my_validator");
    }

    #[test]
    fn test_regex_escape() {
        assert_eq!(regex_escape("test.com"), r"test\.com");
        assert_eq!(regex_escape("a+b"), r"a\+b");
        assert_eq!(regex_escape("foo*bar"), r"foo\*bar");
        assert_eq!(regex_escape("(group)"), r"\(group\)");
        assert_eq!(regex_escape("[chars]"), r"\[chars\]");
    }

    #[test]
    fn test_generate_multiple_types() {
        let types = vec![
            ParsedType {
                name: "User".to_string(),
                fields: vec![ParsedField {
                    name: "name".to_string(),
                    ty: FieldType::String,
                    validation_rules: vec![],
                }],
            },
            ParsedType {
                name: "Order".to_string(),
                fields: vec![ParsedField {
                    name: "total".to_string(),
                    ty: FieldType::F64,
                    validation_rules: vec![],
                }],
            },
        ];

        let output = generate(&types, OpenApiVersion::V3_0).unwrap();
        let parsed: Value = serde_json::from_str(&output).unwrap();

        assert!(parsed["components"]["schemas"]["User"].is_object());
        assert!(parsed["components"]["schemas"]["Order"].is_object());
    }

    #[test]
    fn test_empty_types() {
        let types: Vec<ParsedType> = vec![];
        let output = generate(&types, OpenApiVersion::V3_0).unwrap();
        let parsed: Value = serde_json::from_str(&output).unwrap();

        assert_eq!(parsed["openapi"], "3.0.3");
        assert!(parsed["components"]["schemas"]
            .as_object()
            .unwrap()
            .is_empty());
    }

    #[test]
    fn test_type_with_required_fields() {
        let parsed_type = ParsedType {
            name: "User".to_string(),
            fields: vec![
                ParsedField {
                    name: "email".to_string(),
                    ty: FieldType::String,
                    validation_rules: vec![],
                },
                ParsedField {
                    name: "age".to_string(),
                    ty: FieldType::U8,
                    validation_rules: vec![],
                },
            ],
        };

        let schema = generate_type_schema(&parsed_type, OpenApiVersion::V3_0).unwrap();
        let required = schema["required"].as_array().unwrap();
        assert_eq!(required.len(), 2);
        assert!(required.contains(&json!("email")));
        assert!(required.contains(&json!("age")));
    }

    #[test]
    fn test_type_with_optional_fields() {
        let parsed_type = ParsedType {
            name: "Profile".to_string(),
            fields: vec![
                ParsedField {
                    name: "name".to_string(),
                    ty: FieldType::String,
                    validation_rules: vec![],
                },
                ParsedField {
                    name: "bio".to_string(),
                    ty: FieldType::Option(Box::new(FieldType::String)),
                    validation_rules: vec![],
                },
            ],
        };

        let schema = generate_type_schema(&parsed_type, OpenApiVersion::V3_0).unwrap();
        let required = schema["required"].as_array().unwrap();
        assert!(required.contains(&json!("name")));
        assert!(!required.contains(&json!("bio")));
    }

    #[test]
    fn test_additional_properties_false() {
        let parsed_type = ParsedType {
            name: "Strict".to_string(),
            fields: vec![],
        };

        let schema = generate_type_schema(&parsed_type, OpenApiVersion::V3_0).unwrap();
        assert_eq!(schema["additionalProperties"], false);
    }

    #[test]
    fn test_openapi_version_default() {
        let version = OpenApiVersion::default();
        assert!(matches!(version, OpenApiVersion::V3_0));
    }

    #[test]
    fn test_openapi_31_version_string() {
        let types = vec![ParsedType {
            name: "Test".to_string(),
            fields: vec![],
        }];

        let output = generate(&types, OpenApiVersion::V3_1).unwrap();
        let parsed: Value = serde_json::from_str(&output).unwrap();

        assert_eq!(parsed["openapi"], "3.1.0");
    }

    #[test]
    fn test_info_section() {
        let types = vec![];
        let output = generate(&types, OpenApiVersion::V3_0).unwrap();
        let parsed: Value = serde_json::from_str(&output).unwrap();

        assert_eq!(parsed["info"]["title"], "Generated API Schema");
        assert_eq!(parsed["info"]["version"], "1.0.0");
        assert!(parsed["info"]["description"].as_str().is_some());
    }

    #[test]
    fn test_paths_empty() {
        let types = vec![];
        let output = generate(&types, OpenApiVersion::V3_0).unwrap();
        let parsed: Value = serde_json::from_str(&output).unwrap();

        assert!(parsed["paths"].as_object().unwrap().is_empty());
    }

    #[test]
    fn test_field_schema_with_validations() {
        let field = ParsedField {
            name: "email".to_string(),
            ty: FieldType::String,
            validation_rules: vec![ValidationRule::Email, ValidationRule::MaxLen(255)],
        };

        let schema = generate_field_schema(&field, OpenApiVersion::V3_0).unwrap();
        assert_eq!(schema["type"], "string");
        assert_eq!(schema["format"], "email");
        assert_eq!(schema["maxLength"], 255);
    }

    #[test]
    fn test_range_with_invalid_min() {
        let mut schema = json!({ "type": "integer" });
        apply_validation_rule(
            &mut schema,
            &ValidationRule::Range {
                min: "invalid".to_string(),
                max: "100".to_string(),
            },
            &FieldType::I32,
        );
        assert!(schema.get("minimum").is_none());
        assert_eq!(schema["maximum"], 100.0);
    }

    #[test]
    fn test_range_with_invalid_max() {
        let mut schema = json!({ "type": "integer" });
        apply_validation_rule(
            &mut schema,
            &ValidationRule::Range {
                min: "0".to_string(),
                max: "invalid".to_string(),
            },
            &FieldType::I32,
        );
        assert_eq!(schema["minimum"], 0.0);
        assert!(schema.get("maximum").is_none());
    }

    #[test]
    fn test_multiple_of_invalid_value() {
        let mut schema = json!({ "type": "integer" });
        apply_validation_rule(
            &mut schema,
            &ValidationRule::MultipleOf("invalid".to_string()),
            &FieldType::I32,
        );
        assert!(schema.get("multipleOf").is_none());
    }

    #[test]
    fn test_option_with_validation_v30() {
        let field = ParsedField {
            name: "website".to_string(),
            ty: FieldType::Option(Box::new(FieldType::String)),
            validation_rules: vec![ValidationRule::Url],
        };

        let schema = generate_field_schema(&field, OpenApiVersion::V3_0).unwrap();
        assert_eq!(schema["nullable"], true);
        assert_eq!(schema["format"], "uri");
    }

    #[test]
    fn test_option_with_validation_v31() {
        let field = ParsedField {
            name: "website".to_string(),
            ty: FieldType::Option(Box::new(FieldType::String)),
            validation_rules: vec![ValidationRule::Url],
        };

        let schema = generate_field_schema(&field, OpenApiVersion::V3_1).unwrap();
        assert!(schema["oneOf"].is_array());
        // Note: validations are applied to the base schema, not the oneOf
    }

    #[test]
    fn test_all_primitive_types() {
        let types = vec![
            FieldType::U8,
            FieldType::U16,
            FieldType::U32,
            FieldType::I8,
            FieldType::I16,
            FieldType::I32,
        ];

        for ty in types {
            let schema = generate_base_type_schema(&ty, OpenApiVersion::V3_0);
            assert_eq!(schema["type"], "integer");
        }
    }
}

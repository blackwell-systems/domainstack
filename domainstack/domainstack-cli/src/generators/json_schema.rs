use crate::parser::{FieldType, ParsedField, ParsedType, ValidationRule};
use anyhow::Result;
use serde_json::{json, Map, Value};

/// Generate JSON Schema (Draft 2020-12) from parsed types
pub fn generate(types: &[ParsedType]) -> Result<String> {
    let mut definitions = Map::new();

    // Generate schema for each type
    for parsed_type in types {
        let schema = generate_type_schema(parsed_type)?;
        definitions.insert(parsed_type.name.clone(), schema);
    }

    // Create the root schema document
    let root_schema = json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "$id": "https://example.com/schemas/generated.json",
        "title": "Generated Schemas",
        "description": "Auto-generated JSON Schema from domainstack validation rules",
        "$defs": definitions
    });

    // Pretty print with 2-space indentation
    let output = serde_json::to_string_pretty(&root_schema)?;
    Ok(output)
}

fn generate_type_schema(parsed_type: &ParsedType) -> Result<Value> {
    let mut properties = Map::new();
    let mut required = Vec::new();

    for field in &parsed_type.fields {
        let field_schema = generate_field_schema(field)?;
        properties.insert(field.name.clone(), field_schema);

        // Non-optional fields are required
        if !matches!(field.ty, FieldType::Option(_)) {
            required.push(Value::String(field.name.clone()));
        }
    }

    let mut schema = json!({
        "type": "object",
        "title": parsed_type.name,
        "properties": properties,
        "additionalProperties": false
    });

    if !required.is_empty() {
        schema["required"] = Value::Array(required);
    }

    Ok(schema)
}

fn generate_field_schema(field: &ParsedField) -> Result<Value> {
    // Get base schema for the type
    let mut schema = generate_base_type_schema(&field.ty);

    // Apply validation rules
    for rule in &field.validation_rules {
        apply_validation_rule(&mut schema, rule, &field.ty);
    }

    Ok(schema)
}

fn generate_base_type_schema(field_type: &FieldType) -> Value {
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
                "description": "Large integer - may exceed JavaScript safe integer range"
            })
        }
        FieldType::F32 | FieldType::F64 => json!({ "type": "number" }),
        FieldType::Option(inner) => {
            // For Option<T>, we generate the inner type schema
            // The field will not be in "required" array
            generate_base_type_schema(inner)
        }
        FieldType::Vec(inner) => {
            json!({
                "type": "array",
                "items": generate_base_type_schema(inner)
            })
        }
        FieldType::Custom(name) => {
            // Reference to another type in $defs
            json!({ "$ref": format!("#/$defs/{}", name) })
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
            // JSON Schema doesn't have a direct "not equal" constraint
            // We use a pattern for this with a description
            schema["not"] = json!({ "const": 0 });
        }
        ValidationRule::MultipleOf(divisor) => {
            if let Ok(divisor_num) = divisor.parse::<f64>() {
                schema["multipleOf"] = json!(divisor_num);
            }
        }
        ValidationRule::Finite => {
            // All JSON numbers are finite by definition
            // Add a description for documentation
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
    fn test_base_type_string() {
        let schema = generate_base_type_schema(&FieldType::String);
        assert_eq!(schema["type"], "string");
    }

    #[test]
    fn test_base_type_integer() {
        let schema = generate_base_type_schema(&FieldType::I32);
        assert_eq!(schema["type"], "integer");
    }

    #[test]
    fn test_base_type_number() {
        let schema = generate_base_type_schema(&FieldType::F64);
        assert_eq!(schema["type"], "number");
    }

    #[test]
    fn test_base_type_boolean() {
        let schema = generate_base_type_schema(&FieldType::Bool);
        assert_eq!(schema["type"], "boolean");
    }

    #[test]
    fn test_base_type_array() {
        let schema = generate_base_type_schema(&FieldType::Vec(Box::new(FieldType::String)));
        assert_eq!(schema["type"], "array");
        assert_eq!(schema["items"]["type"], "string");
    }

    #[test]
    fn test_base_type_custom_ref() {
        let schema = generate_base_type_schema(&FieldType::Custom("Address".to_string()));
        assert_eq!(schema["$ref"], "#/$defs/Address");
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
    fn test_regex_escape() {
        assert_eq!(regex_escape("hello"), "hello");
        assert_eq!(regex_escape("hello.world"), r"hello\.world");
        assert_eq!(regex_escape("a+b*c?"), r"a\+b\*c\?");
    }

    #[test]
    fn test_complete_type_schema() {
        let parsed_type = ParsedType {
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
        };

        let schema = generate_type_schema(&parsed_type).unwrap();

        assert_eq!(schema["type"], "object");
        assert_eq!(schema["title"], "User");
        assert_eq!(schema["properties"]["email"]["format"], "email");
        assert_eq!(schema["properties"]["age"]["minimum"], 18.0);
        assert_eq!(schema["properties"]["age"]["maximum"], 120.0);

        // Both fields should be required (not Optional)
        let required = schema["required"].as_array().unwrap();
        assert!(required.contains(&json!("email")));
        assert!(required.contains(&json!("age")));
    }

    #[test]
    fn test_optional_field_not_required() {
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

        let schema = generate_type_schema(&parsed_type).unwrap();

        let required = schema["required"].as_array().unwrap();
        assert!(required.contains(&json!("name")));
        assert!(!required.contains(&json!("bio"))); // Optional field not required
    }

    #[test]
    fn test_generate_full_document() {
        let types = vec![ParsedType {
            name: "User".to_string(),
            fields: vec![ParsedField {
                name: "email".to_string(),
                ty: FieldType::String,
                validation_rules: vec![ValidationRule::Email],
            }],
        }];

        let output = generate(&types).unwrap();

        // Check it's valid JSON
        let parsed: Value = serde_json::from_str(&output).unwrap();

        // Check schema metadata
        assert_eq!(
            parsed["$schema"],
            "https://json-schema.org/draft/2020-12/schema"
        );
        assert!(parsed["$defs"]["User"].is_object());
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
    fn test_min_validation() {
        let mut schema = json!({ "type": "integer" });
        apply_validation_rule(
            &mut schema,
            &ValidationRule::Min("5".to_string()),
            &FieldType::I32,
        );
        assert_eq!(schema["minimum"], 5.0);
    }

    #[test]
    fn test_max_validation() {
        let mut schema = json!({ "type": "integer" });
        apply_validation_rule(
            &mut schema,
            &ValidationRule::Max("100".to_string()),
            &FieldType::I32,
        );
        assert_eq!(schema["maximum"], 100.0);
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
        assert!(schema["description"].as_str().is_some());
        assert!(schema["description"].as_str().unwrap().contains("finite"));
    }

    #[test]
    fn test_non_zero_validation() {
        let mut schema = json!({ "type": "integer" });
        apply_validation_rule(&mut schema, &ValidationRule::NonZero, &FieldType::I32);
        assert!(schema["not"].is_object());
        assert_eq!(schema["not"]["const"], 0);
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
    fn test_large_integer_types() {
        let schema = generate_base_type_schema(&FieldType::U64);
        assert_eq!(schema["type"], "integer");
        assert!(schema["description"].as_str().is_some());

        let schema = generate_base_type_schema(&FieldType::I128);
        assert_eq!(schema["type"], "integer");
        assert!(schema["description"].as_str().unwrap().contains("Large"));
    }

    #[test]
    fn test_float_types() {
        let schema = generate_base_type_schema(&FieldType::F32);
        assert_eq!(schema["type"], "number");

        let schema = generate_base_type_schema(&FieldType::F64);
        assert_eq!(schema["type"], "number");
    }

    #[test]
    fn test_option_type_schema() {
        let schema = generate_base_type_schema(&FieldType::Option(Box::new(FieldType::String)));
        assert_eq!(schema["type"], "string");
    }

    #[test]
    fn test_nested_array_type() {
        let nested = FieldType::Vec(Box::new(FieldType::Vec(Box::new(FieldType::I32))));
        let schema = generate_base_type_schema(&nested);
        assert_eq!(schema["type"], "array");
        assert_eq!(schema["items"]["type"], "array");
        assert_eq!(schema["items"]["items"]["type"], "integer");
    }

    #[test]
    fn test_regex_escape_special_chars() {
        assert_eq!(regex_escape("test.com"), r"test\.com");
        assert_eq!(regex_escape("a+b"), r"a\+b");
        assert_eq!(regex_escape("foo*bar"), r"foo\*bar");
        assert_eq!(regex_escape("what?"), r"what\?");
        assert_eq!(regex_escape("(group)"), r"\(group\)");
        assert_eq!(regex_escape("[chars]"), r"\[chars\]");
        assert_eq!(regex_escape("{1,2}"), r"\{1,2\}");
        assert_eq!(regex_escape("^start"), r"\^start");
        assert_eq!(regex_escape("end$"), r"end\$");
        assert_eq!(regex_escape("a|b"), r"a\|b");
        assert_eq!(regex_escape("back\\slash"), r"back\\slash");
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
                    name: "id".to_string(),
                    ty: FieldType::U32,
                    validation_rules: vec![],
                }],
            },
        ];

        let output = generate(&types).unwrap();
        let parsed: Value = serde_json::from_str(&output).unwrap();

        assert!(parsed["$defs"]["User"].is_object());
        assert!(parsed["$defs"]["Order"].is_object());
    }

    #[test]
    fn test_empty_types() {
        let types: Vec<ParsedType> = vec![];
        let output = generate(&types).unwrap();
        let parsed: Value = serde_json::from_str(&output).unwrap();

        assert_eq!(
            parsed["$schema"],
            "https://json-schema.org/draft/2020-12/schema"
        );
        assert!(parsed["$defs"].as_object().unwrap().is_empty());
    }

    #[test]
    fn test_type_with_all_required_fields() {
        let parsed_type = ParsedType {
            name: "AllRequired".to_string(),
            fields: vec![
                ParsedField {
                    name: "a".to_string(),
                    ty: FieldType::String,
                    validation_rules: vec![],
                },
                ParsedField {
                    name: "b".to_string(),
                    ty: FieldType::I32,
                    validation_rules: vec![],
                },
            ],
        };

        let schema = generate_type_schema(&parsed_type).unwrap();
        let required = schema["required"].as_array().unwrap();
        assert_eq!(required.len(), 2);
    }

    #[test]
    fn test_type_with_no_required_fields() {
        let parsed_type = ParsedType {
            name: "AllOptional".to_string(),
            fields: vec![
                ParsedField {
                    name: "a".to_string(),
                    ty: FieldType::Option(Box::new(FieldType::String)),
                    validation_rules: vec![],
                },
                ParsedField {
                    name: "b".to_string(),
                    ty: FieldType::Option(Box::new(FieldType::I32)),
                    validation_rules: vec![],
                },
            ],
        };

        let schema = generate_type_schema(&parsed_type).unwrap();
        // When all fields are optional, required should not be present or empty
        assert!(schema.get("required").is_none());
    }

    #[test]
    fn test_field_with_multiple_validations() {
        let field = ParsedField {
            name: "username".to_string(),
            ty: FieldType::String,
            validation_rules: vec![
                ValidationRule::Length { min: 3, max: 20 },
                ValidationRule::Alphanumeric,
            ],
        };

        let schema = generate_field_schema(&field).unwrap();
        assert_eq!(schema["minLength"], 3);
        assert_eq!(schema["maxLength"], 20);
        assert_eq!(schema["pattern"], "^[a-zA-Z0-9]*$");
    }

    #[test]
    fn test_range_with_invalid_values() {
        let mut schema = json!({ "type": "integer" });
        apply_validation_rule(
            &mut schema,
            &ValidationRule::Range {
                min: "invalid".to_string(),
                max: "100".to_string(),
            },
            &FieldType::I32,
        );
        // Invalid min should not be set, but max should
        assert!(schema.get("minimum").is_none());
        assert_eq!(schema["maximum"], 100.0);
    }

    #[test]
    fn test_additional_properties_false() {
        let parsed_type = ParsedType {
            name: "Strict".to_string(),
            fields: vec![],
        };

        let schema = generate_type_schema(&parsed_type).unwrap();
        assert_eq!(schema["additionalProperties"], false);
    }

    #[test]
    fn test_type_title() {
        let parsed_type = ParsedType {
            name: "MyType".to_string(),
            fields: vec![],
        };

        let schema = generate_type_schema(&parsed_type).unwrap();
        assert_eq!(schema["title"], "MyType");
    }
}

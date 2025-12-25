use super::ValidationRule;
use anyhow::Result;
use quote::ToTokens;
use syn::{File, Item, ItemStruct, Type};

/// A parsed Rust type with validation rules
#[derive(Debug, Clone)]
pub struct ParsedType {
    pub name: String,
    pub fields: Vec<ParsedField>,
}

#[derive(Debug, Clone)]
pub struct ParsedField {
    pub name: String,
    pub ty: FieldType,
    pub validation_rules: Vec<ValidationRule>,
}

#[derive(Debug, Clone)]
pub enum FieldType {
    String,
    Bool,
    U8,
    U16,
    U32,
    U64,
    U128,
    I8,
    I16,
    I32,
    I64,
    I128,
    F32,
    F64,
    Option(Box<FieldType>),
    Vec(Box<FieldType>),
    Custom(String),
}

/// Extract all types with #[derive(Validate)] from a syntax tree
pub fn extract_validated_types(file: &File) -> Result<Vec<ParsedType>> {
    let mut types = Vec::new();

    for item in &file.items {
        if let Item::Struct(struct_item) = item {
            if has_validate_derive(struct_item) {
                if let Some(parsed_type) = parse_struct(struct_item)? {
                    types.push(parsed_type);
                }
            }
        }
    }

    Ok(types)
}

/// Check if a struct has #[derive(Validate)]
fn has_validate_derive(struct_item: &ItemStruct) -> bool {
    struct_item.attrs.iter().any(|attr| {
        if let Some(ident) = attr.path().get_ident() {
            if ident == "derive" {
                // Simple check: does the attribute contain "Validate"?
                let tokens = attr.meta.to_token_stream().to_string();
                return tokens.contains("Validate");
            }
        }
        false
    })
}

/// Parse a struct and extract its fields with validation rules
fn parse_struct(struct_item: &ItemStruct) -> Result<Option<ParsedType>> {
    let mut fields = Vec::new();

    match &struct_item.fields {
        syn::Fields::Named(fields_named) => {
            for field in &fields_named.named {
                if let Some(ident) = &field.ident {
                    let field_type = parse_field_type(&field.ty)?;
                    let validation_rules =
                        super::validation::parse_validation_attributes(&field.attrs)?;

                    fields.push(ParsedField {
                        name: ident.to_string(),
                        ty: field_type,
                        validation_rules,
                    });
                }
            }
        }
        _ => {
            // Skip tuple structs and unit structs for now
            return Ok(None);
        }
    }

    Ok(Some(ParsedType {
        name: struct_item.ident.to_string(),
        fields,
    }))
}

/// Parse a Rust type into our FieldType enum
fn parse_field_type(ty: &Type) -> Result<FieldType> {
    match ty {
        Type::Path(type_path) => {
            let segments = &type_path.path.segments;
            if segments.is_empty() {
                return Ok(FieldType::Custom("Unknown".to_string()));
            }

            let last_segment = &segments.last().unwrap();
            let type_name = last_segment.ident.to_string();

            // Handle basic types
            match type_name.as_str() {
                "String" => Ok(FieldType::String),
                "bool" => Ok(FieldType::Bool),
                "u8" => Ok(FieldType::U8),
                "u16" => Ok(FieldType::U16),
                "u32" => Ok(FieldType::U32),
                "u64" => Ok(FieldType::U64),
                "u128" => Ok(FieldType::U128),
                "i8" => Ok(FieldType::I8),
                "i16" => Ok(FieldType::I16),
                "i32" => Ok(FieldType::I32),
                "i64" => Ok(FieldType::I64),
                "i128" => Ok(FieldType::I128),
                "f32" => Ok(FieldType::F32),
                "f64" => Ok(FieldType::F64),
                "Option" => {
                    // Parse Option<T>
                    if let syn::PathArguments::AngleBracketed(args) = &last_segment.arguments {
                        if let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first() {
                            let inner_type = parse_field_type(inner_ty)?;
                            return Ok(FieldType::Option(Box::new(inner_type)));
                        }
                    }
                    Ok(FieldType::Custom("Option".to_string()))
                }
                "Vec" => {
                    // Parse Vec<T>
                    if let syn::PathArguments::AngleBracketed(args) = &last_segment.arguments {
                        if let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first() {
                            let inner_type = parse_field_type(inner_ty)?;
                            return Ok(FieldType::Vec(Box::new(inner_type)));
                        }
                    }
                    Ok(FieldType::Custom("Vec".to_string()))
                }
                _ => Ok(FieldType::Custom(type_name)),
            }
        }
        _ => Ok(FieldType::Custom("Unknown".to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_field_type_primitives() {
        let code = "struct Test { field: String }";
        let file = syn::parse_str::<syn::File>(code).unwrap();
        if let syn::Item::Struct(s) = &file.items[0] {
            if let syn::Fields::Named(fields) = &s.fields {
                let field = fields.named.first().unwrap();
                let field_type = parse_field_type(&field.ty).unwrap();
                assert!(matches!(field_type, FieldType::String));
            }
        }
    }

    #[test]
    fn test_parse_field_type_numbers() {
        let test_cases = vec![
            ("u8", FieldType::U8),
            ("u16", FieldType::U16),
            ("u32", FieldType::U32),
            ("u64", FieldType::U64),
            ("i32", FieldType::I32),
            ("f32", FieldType::F32),
            ("f64", FieldType::F64),
        ];

        for (type_name, expected) in test_cases {
            let code = format!("struct Test {{ field: {} }}", type_name);
            let file = syn::parse_str::<syn::File>(&code).unwrap();
            if let syn::Item::Struct(s) = &file.items[0] {
                if let syn::Fields::Named(fields) = &s.fields {
                    let field = fields.named.first().unwrap();
                    let field_type = parse_field_type(&field.ty).unwrap();
                    assert_eq!(
                        std::mem::discriminant(&field_type),
                        std::mem::discriminant(&expected),
                        "Failed for type: {}",
                        type_name
                    );
                }
            }
        }
    }

    #[test]
    fn test_parse_field_type_option() {
        let code = "struct Test { field: Option<String> }";
        let file = syn::parse_str::<syn::File>(code).unwrap();
        if let syn::Item::Struct(s) = &file.items[0] {
            if let syn::Fields::Named(fields) = &s.fields {
                let field = fields.named.first().unwrap();
                let field_type = parse_field_type(&field.ty).unwrap();
                match field_type {
                    FieldType::Option(inner) => {
                        assert!(matches!(*inner, FieldType::String));
                    }
                    _ => panic!("Expected Option type"),
                }
            }
        }
    }

    #[test]
    fn test_parse_field_type_vec() {
        let code = "struct Test { field: Vec<String> }";
        let file = syn::parse_str::<syn::File>(code).unwrap();
        if let syn::Item::Struct(s) = &file.items[0] {
            if let syn::Fields::Named(fields) = &s.fields {
                let field = fields.named.first().unwrap();
                let field_type = parse_field_type(&field.ty).unwrap();
                match field_type {
                    FieldType::Vec(inner) => {
                        assert!(matches!(*inner, FieldType::String));
                    }
                    _ => panic!("Expected Vec type"),
                }
            }
        }
    }

    #[test]
    fn test_parse_field_type_custom() {
        let code = "struct Test { field: CustomType }";
        let file = syn::parse_str::<syn::File>(code).unwrap();
        if let syn::Item::Struct(s) = &file.items[0] {
            if let syn::Fields::Named(fields) = &s.fields {
                let field = fields.named.first().unwrap();
                let field_type = parse_field_type(&field.ty).unwrap();
                match field_type {
                    FieldType::Custom(name) => {
                        assert_eq!(name, "CustomType");
                    }
                    _ => panic!("Expected Custom type"),
                }
            }
        }
    }

    #[test]
    fn test_has_validate_derive_positive() {
        let code = r#"
            #[derive(Validate)]
            struct User {
                email: String,
            }
        "#;
        let file = syn::parse_str::<syn::File>(code).unwrap();
        if let syn::Item::Struct(s) = &file.items[0] {
            assert!(has_validate_derive(s));
        }
    }

    #[test]
    fn test_has_validate_derive_negative() {
        let code = r#"
            #[derive(Debug, Clone)]
            struct User {
                email: String,
            }
        "#;
        let file = syn::parse_str::<syn::File>(code).unwrap();
        if let syn::Item::Struct(s) = &file.items[0] {
            assert!(!has_validate_derive(s));
        }
    }

    #[test]
    fn test_has_validate_derive_multiple_derives() {
        let code = r#"
            #[derive(Debug, Validate, Clone)]
            struct User {
                email: String,
            }
        "#;
        let file = syn::parse_str::<syn::File>(code).unwrap();
        if let syn::Item::Struct(s) = &file.items[0] {
            assert!(has_validate_derive(s));
        }
    }

    #[test]
    fn test_extract_validated_types_single_type() {
        let code = r#"
            use domainstack::Validate;

            #[derive(Validate)]
            struct User {
                #[validate(email)]
                email: String,
            }
        "#;

        let file = syn::parse_str::<syn::File>(code).unwrap();
        let types = extract_validated_types(&file).unwrap();

        assert_eq!(types.len(), 1);
        assert_eq!(types[0].name, "User");
        assert_eq!(types[0].fields.len(), 1);
        assert_eq!(types[0].fields[0].name, "email");
    }

    #[test]
    fn test_extract_validated_types_multiple_types() {
        let code = r#"
            #[derive(Validate)]
            struct User {
                email: String,
            }

            #[derive(Validate)]
            struct Post {
                title: String,
            }

            #[derive(Debug)]
            struct Comment {
                text: String,
            }
        "#;

        let file = syn::parse_str::<syn::File>(code).unwrap();
        let types = extract_validated_types(&file).unwrap();

        assert_eq!(types.len(), 2);
        assert_eq!(types[0].name, "User");
        assert_eq!(types[1].name, "Post");
    }

    #[test]
    fn test_extract_validated_types_no_validate() {
        let code = r#"
            #[derive(Debug)]
            struct User {
                email: String,
            }
        "#;

        let file = syn::parse_str::<syn::File>(code).unwrap();
        let types = extract_validated_types(&file).unwrap();

        assert_eq!(types.len(), 0);
    }

    #[test]
    fn test_parse_struct_with_multiple_fields() {
        let code = r#"
            #[derive(Validate)]
            struct User {
                #[validate(email)]
                email: String,

                #[validate(range(min = 18, max = 120))]
                age: u8,

                #[validate(url)]
                website: Option<String>,
            }
        "#;

        let file = syn::parse_str::<syn::File>(code).unwrap();
        let types = extract_validated_types(&file).unwrap();

        assert_eq!(types.len(), 1);
        let user_type = &types[0];

        assert_eq!(user_type.fields.len(), 3);
        assert_eq!(user_type.fields[0].name, "email");
        assert_eq!(user_type.fields[1].name, "age");
        assert_eq!(user_type.fields[2].name, "website");

        // Check types
        assert!(matches!(user_type.fields[0].ty, FieldType::String));
        assert!(matches!(user_type.fields[1].ty, FieldType::U8));
        assert!(matches!(user_type.fields[2].ty, FieldType::Option(_)));
    }
}

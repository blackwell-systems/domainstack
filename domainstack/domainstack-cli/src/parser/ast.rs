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
                    let validation_rules = super::validation::parse_validation_attributes(&field.attrs)?;

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

//! Derive implementation for ToJsonSchema trait.
//!
//! This module generates JSON Schema (Draft 2020-12) from Rust structs
//! with `#[validate(...)]` attributes.

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Attribute, Data, DeriveInput, Fields, Lit, Type};

/// Derive implementation for ToJsonSchema
pub fn derive_to_json_schema_impl(input: DeriveInput) -> syn::Result<TokenStream> {
    let name = &input.ident;
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // Only support structs with named fields
    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => {
                return Err(syn::Error::new_spanned(
                    &input,
                    "#[derive(ToJsonSchema)] only supports structs with named fields",
                ))
            }
        },
        _ => {
            return Err(syn::Error::new_spanned(
                &input,
                "#[derive(ToJsonSchema)] only supports structs",
            ))
        }
    };

    // Parse struct-level schema attributes
    let struct_schema_hints = parse_struct_schema_attributes(&input.attrs)?;

    // Parse field validations and schema hints
    let field_schemas: Vec<_> = fields
        .iter()
        .map(|field| {
            let field_name = field.ident.as_ref().unwrap();
            let field_type = &field.ty;
            let validation_rules = parse_validation_attributes(&field.attrs)?;
            let schema_hints = parse_field_schema_attributes(&field.attrs)?;

            Ok(FieldSchema {
                name: field_name.clone(),
                ty: field_type.clone(),
                validation_rules,
                schema_hints,
            })
        })
        .collect::<syn::Result<_>>()?;

    // Generate schema properties
    let properties = generate_properties(&field_schemas)?;

    // Generate required fields array
    let required_fields = generate_required_fields(&field_schemas);

    // Apply struct-level hints
    let schema_name = name.to_string();
    let title = struct_schema_hints
        .title
        .as_ref()
        .map(|t| quote! { .title(#t) })
        .unwrap_or_else(|| quote! { .title(#schema_name) });

    let description = struct_schema_hints
        .description
        .as_ref()
        .map(|desc| quote! { .description(#desc) });

    Ok(quote! {
        impl #impl_generics ::domainstack_schema::ToJsonSchema for #name #ty_generics #where_clause {
            fn schema_name() -> &'static str {
                #schema_name
            }

            fn json_schema() -> ::domainstack_schema::JsonSchema {
                ::domainstack_schema::JsonSchema::object()
                    #title
                    #properties
                    #required_fields
                    #description
            }
        }
    })
}

#[derive(Debug, Clone)]
struct FieldSchema {
    name: syn::Ident,
    ty: Type,
    validation_rules: Vec<ValidationRule>,
    schema_hints: SchemaHints,
}

#[derive(Debug, Clone, Default)]
struct SchemaHints {
    title: Option<String>,
    description: Option<String>,
    example: Option<Lit>,
    deprecated: bool,
    pattern: Option<String>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
enum ValidationRule {
    Email,
    Url,
    MinLen(usize),
    MaxLen(usize),
    Length {
        min: Option<usize>,
        max: Option<usize>,
    },
    MatchesRegex(String),
    Ascii,
    Alphanumeric,
    AlphaOnly,
    NumericString,
    NonEmpty,
    NonBlank,
    Min(TokenStream),
    Max(TokenStream),
    Range {
        min: TokenStream,
        max: TokenStream,
    },
    Positive,
    Negative,
    NonZero,
    MultipleOf(TokenStream),
    OneOf(Vec<String>),
    Equals(String),
    NotEquals(String),
    MinItems(usize),
    MaxItems(usize),
    Unique,
    Nested,
    EachNested,
    Custom(String),
    Contains(String),
    StartsWith(String),
    EndsWith(String),
}

/// Parse #[validate(...)] attributes from a field
fn parse_validation_attributes(attrs: &[Attribute]) -> syn::Result<Vec<ValidationRule>> {
    let mut rules = Vec::new();

    for attr in attrs {
        if !attr.path().is_ident("validate") {
            continue;
        }

        attr.parse_nested_meta(|meta| {
            // Email
            if meta.path.is_ident("email") {
                rules.push(ValidationRule::Email);
                return Ok(());
            }

            // URL
            if meta.path.is_ident("url") {
                rules.push(ValidationRule::Url);
                return Ok(());
            }

            // min_len
            if meta.path.is_ident("min_len") {
                let value: syn::Lit = meta.value()?.parse()?;
                if let syn::Lit::Int(lit_int) = value {
                    let val = lit_int.base10_parse()?;
                    rules.push(ValidationRule::MinLen(val));
                }
                return Ok(());
            }

            // max_len
            if meta.path.is_ident("max_len") {
                let value: syn::Lit = meta.value()?.parse()?;
                if let syn::Lit::Int(lit_int) = value {
                    let val = lit_int.base10_parse()?;
                    rules.push(ValidationRule::MaxLen(val));
                }
                return Ok(());
            }

            // length
            if meta.path.is_ident("length") {
                let mut min = None;
                let mut max = None;

                meta.parse_nested_meta(|nested| {
                    if nested.path.is_ident("min") {
                        let value: syn::Lit = nested.value()?.parse()?;
                        if let syn::Lit::Int(lit_int) = value {
                            min = Some(lit_int.base10_parse()?);
                        }
                    } else if nested.path.is_ident("max") {
                        let value: syn::Lit = nested.value()?.parse()?;
                        if let syn::Lit::Int(lit_int) = value {
                            max = Some(lit_int.base10_parse()?);
                        }
                    }
                    Ok(())
                })?;

                rules.push(ValidationRule::Length { min, max });
                return Ok(());
            }

            // range
            if meta.path.is_ident("range") {
                let mut min = None;
                let mut max = None;

                meta.parse_nested_meta(|nested| {
                    if nested.path.is_ident("min") {
                        let value: syn::Expr = nested.value()?.parse()?;
                        min = Some(quote! { #value });
                    } else if nested.path.is_ident("max") {
                        let value: syn::Expr = nested.value()?.parse()?;
                        max = Some(quote! { #value });
                    }
                    Ok(())
                })?;

                if let (Some(min), Some(max)) = (min, max) {
                    rules.push(ValidationRule::Range { min, max });
                }
                return Ok(());
            }

            // nested
            if meta.path.is_ident("nested") {
                rules.push(ValidationRule::Nested);
                return Ok(());
            }

            // each
            if meta.path.is_ident("each") {
                meta.parse_nested_meta(|nested| {
                    if nested.path.is_ident("nested") {
                        rules.push(ValidationRule::EachNested);
                    }
                    Ok(())
                })?;
                return Ok(());
            }

            // min_items
            if meta.path.is_ident("min_items") {
                let value: syn::Lit = meta.value()?.parse()?;
                if let syn::Lit::Int(lit_int) = value {
                    let val = lit_int.base10_parse()?;
                    rules.push(ValidationRule::MinItems(val));
                }
                return Ok(());
            }

            // max_items
            if meta.path.is_ident("max_items") {
                let value: syn::Lit = meta.value()?.parse()?;
                if let syn::Lit::Int(lit_int) = value {
                    let val = lit_int.base10_parse()?;
                    rules.push(ValidationRule::MaxItems(val));
                }
                return Ok(());
            }

            // unique
            if meta.path.is_ident("unique") {
                rules.push(ValidationRule::Unique);
                return Ok(());
            }

            // Pattern rules
            if meta.path.is_ident("alphanumeric") {
                rules.push(ValidationRule::Alphanumeric);
                return Ok(());
            }

            if meta.path.is_ident("ascii") {
                rules.push(ValidationRule::Ascii);
                return Ok(());
            }

            if meta.path.is_ident("alpha_only") {
                rules.push(ValidationRule::AlphaOnly);
                return Ok(());
            }

            if meta.path.is_ident("numeric_string") {
                rules.push(ValidationRule::NumericString);
                return Ok(());
            }

            if meta.path.is_ident("non_empty") {
                rules.push(ValidationRule::NonEmpty);
                return Ok(());
            }

            if meta.path.is_ident("non_blank") {
                rules.push(ValidationRule::NonBlank);
                return Ok(());
            }

            if meta.path.is_ident("positive") {
                rules.push(ValidationRule::Positive);
                return Ok(());
            }

            if meta.path.is_ident("negative") {
                rules.push(ValidationRule::Negative);
                return Ok(());
            }

            if meta.path.is_ident("non_zero") {
                rules.push(ValidationRule::NonZero);
                return Ok(());
            }

            if meta.path.is_ident("each_nested") {
                rules.push(ValidationRule::EachNested);
                return Ok(());
            }

            // contains
            if meta.path.is_ident("contains") {
                let value: syn::LitStr = meta.value()?.parse()?;
                rules.push(ValidationRule::Contains(value.value()));
                return Ok(());
            }

            // starts_with
            if meta.path.is_ident("starts_with") {
                let value: syn::LitStr = meta.value()?.parse()?;
                rules.push(ValidationRule::StartsWith(value.value()));
                return Ok(());
            }

            // ends_with
            if meta.path.is_ident("ends_with") {
                let value: syn::LitStr = meta.value()?.parse()?;
                rules.push(ValidationRule::EndsWith(value.value()));
                return Ok(());
            }

            // matches_regex / regex
            if meta.path.is_ident("matches_regex") || meta.path.is_ident("regex") {
                let value: syn::LitStr = meta.value()?.parse()?;
                rules.push(ValidationRule::MatchesRegex(value.value()));
                return Ok(());
            }

            // Skip other rules
            Ok(())
        })?;
    }

    Ok(rules)
}

/// Parse #[schema(...)] attributes from a field
fn parse_field_schema_attributes(attrs: &[Attribute]) -> syn::Result<SchemaHints> {
    let mut hints = SchemaHints::default();

    for attr in attrs {
        if !attr.path().is_ident("schema") {
            continue;
        }

        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("title") {
                let value: syn::LitStr = meta.value()?.parse()?;
                hints.title = Some(value.value());
            } else if meta.path.is_ident("description") {
                let value: syn::LitStr = meta.value()?.parse()?;
                hints.description = Some(value.value());
            } else if meta.path.is_ident("example") {
                let value: syn::Lit = meta.value()?.parse()?;
                hints.example = Some(value);
            } else if meta.path.is_ident("deprecated") {
                hints.deprecated = true;
            } else if meta.path.is_ident("pattern") {
                let value: syn::LitStr = meta.value()?.parse()?;
                hints.pattern = Some(value.value());
            }
            Ok(())
        })?;
    }

    Ok(hints)
}

/// Parse #[schema(...)] attributes from the struct
fn parse_struct_schema_attributes(attrs: &[Attribute]) -> syn::Result<SchemaHints> {
    parse_field_schema_attributes(attrs)
}

/// Generate schema properties for all fields
fn generate_properties(fields: &[FieldSchema]) -> syn::Result<TokenStream> {
    let mut properties = TokenStream::new();

    for field in fields {
        let field_name = field.name.to_string();
        let property_schema = generate_field_schema(field)?;

        properties.extend(quote! {
            .property(#field_name, #property_schema)
        });
    }

    Ok(properties)
}

/// Generate schema for a single field
fn generate_field_schema(field: &FieldSchema) -> syn::Result<TokenStream> {
    // Determine base schema from type
    let base_schema = generate_base_schema_from_type(&field.ty, &field.validation_rules)?;

    // Apply validation rule constraints
    let constrained_schema = apply_validation_constraints(base_schema, &field.validation_rules);

    // Apply schema hints
    let final_schema = apply_schema_hints(constrained_schema, &field.schema_hints);

    Ok(final_schema)
}

/// Generate base schema based on Rust type
fn generate_base_schema_from_type(ty: &Type, rules: &[ValidationRule]) -> syn::Result<TokenStream> {
    // Check if it's a nested type
    if rules.iter().any(|r| matches!(r, ValidationRule::Nested)) {
        return Ok(quote! {
            <#ty as ::domainstack_schema::ToJsonSchema>::json_schema()
        });
    }

    // Check if it's an array with nested items
    if rules
        .iter()
        .any(|r| matches!(r, ValidationRule::EachNested))
    {
        let inner_type = extract_vec_inner_type(ty)?;
        return Ok(quote! {
            ::domainstack_schema::JsonSchema::array(<#inner_type as ::domainstack_schema::ToJsonSchema>::json_schema())
        });
    }

    // Handle standard Rust types
    if let Type::Path(type_path) = ty {
        let type_name = type_path
            .path
            .segments
            .last()
            .ok_or_else(|| syn::Error::new_spanned(ty, "Invalid type path"))?
            .ident
            .to_string();

        return Ok(match type_name.as_str() {
            "String" | "str" => quote! { ::domainstack_schema::JsonSchema::string() },
            "u8" | "u16" | "u32" | "u64" | "i8" | "i16" | "i32" | "i64" | "usize" | "isize" => {
                quote! { ::domainstack_schema::JsonSchema::integer() }
            }
            "f32" | "f64" => quote! { ::domainstack_schema::JsonSchema::number() },
            "bool" => quote! { ::domainstack_schema::JsonSchema::boolean() },
            "Vec" => {
                let inner_type = extract_vec_inner_type(ty)?;
                let inner_schema = generate_base_schema_from_type(inner_type, &[])?;
                quote! { ::domainstack_schema::JsonSchema::array(#inner_schema) }
            }
            "Option" => {
                let inner_type = extract_option_inner_type(ty)?;
                return generate_base_schema_from_type(inner_type, rules);
            }
            _ => {
                // Assume it's a type that implements ToJsonSchema
                quote! { <#ty as ::domainstack_schema::ToJsonSchema>::json_schema() }
            }
        });
    }

    Ok(quote! { ::domainstack_schema::JsonSchema::object() })
}

/// Apply validation rule constraints to schema
fn apply_validation_constraints(base: TokenStream, rules: &[ValidationRule]) -> TokenStream {
    let mut schema = base;

    for rule in rules {
        schema = match rule {
            ValidationRule::Email => {
                quote! { #schema.format("email") }
            }
            ValidationRule::Url => {
                quote! { #schema.format("uri") }
            }
            ValidationRule::MinLen(min) => {
                quote! { #schema.min_length(#min) }
            }
            ValidationRule::MaxLen(max) => {
                quote! { #schema.max_length(#max) }
            }
            ValidationRule::Length { min, max } => {
                let mut s = schema;
                if let Some(min) = min {
                    s = quote! { #s.min_length(#min) };
                }
                if let Some(max) = max {
                    s = quote! { #s.max_length(#max) };
                }
                s
            }
            ValidationRule::NonEmpty => {
                quote! { #schema.min_length(1) }
            }
            ValidationRule::NonBlank => {
                // Pattern that requires at least one non-whitespace character
                quote! { #schema.pattern("\\S") }
            }
            ValidationRule::Range { min, max } => {
                quote! { #schema.minimum(#min as f64).maximum(#max as f64) }
            }
            ValidationRule::MinItems(min) => {
                quote! { #schema.min_items(#min) }
            }
            ValidationRule::MaxItems(max) => {
                quote! { #schema.max_items(#max) }
            }
            ValidationRule::Unique => {
                quote! { #schema.unique_items(true) }
            }
            ValidationRule::Ascii => {
                quote! { #schema.pattern("^[\\x00-\\x7F]*$") }
            }
            ValidationRule::Alphanumeric => {
                quote! { #schema.pattern("^[a-zA-Z0-9]*$") }
            }
            ValidationRule::AlphaOnly => {
                quote! { #schema.pattern("^[a-zA-Z]*$") }
            }
            ValidationRule::NumericString => {
                quote! { #schema.pattern("^[0-9]*$") }
            }
            ValidationRule::Positive => {
                quote! { #schema.exclusive_minimum(0.0) }
            }
            ValidationRule::Negative => {
                quote! { #schema.exclusive_maximum(0.0) }
            }
            ValidationRule::MatchesRegex(pattern) => {
                quote! { #schema.pattern(#pattern) }
            }
            ValidationRule::StartsWith(prefix) => {
                let pattern = format!("^{}", regex_escape(prefix));
                quote! { #schema.pattern(#pattern) }
            }
            ValidationRule::EndsWith(suffix) => {
                let pattern = format!("{}$", regex_escape(suffix));
                quote! { #schema.pattern(#pattern) }
            }
            ValidationRule::Contains(substr) => {
                let pattern = format!(".*{}.*", regex_escape(substr));
                quote! { #schema.pattern(#pattern) }
            }
            _ => schema,
        };
    }

    schema
}

/// Escape special regex characters
fn regex_escape(s: &str) -> String {
    let mut result = String::with_capacity(s.len() * 2);
    for c in s.chars() {
        match c {
            '\\' | '.' | '+' | '*' | '?' | '(' | ')' | '[' | ']' | '{' | '}' | '|' | '^' | '$' => {
                result.push('\\');
                result.push(c);
            }
            _ => result.push(c),
        }
    }
    result
}

/// Apply schema hints to schema
fn apply_schema_hints(base: TokenStream, hints: &SchemaHints) -> TokenStream {
    let mut schema = base;

    if let Some(title) = &hints.title {
        schema = quote! { #schema.title(#title) };
    }

    if let Some(desc) = &hints.description {
        schema = quote! { #schema.description(#desc) };
    }

    if let Some(pattern) = &hints.pattern {
        schema = quote! { #schema.pattern(#pattern) };
    }

    // Note: JSON Schema doesn't have direct deprecated or example support in the same way
    // but we can add them as extensions if needed

    schema
}

/// Generate required fields array
fn generate_required_fields(fields: &[FieldSchema]) -> TokenStream {
    let required: Vec<_> = fields
        .iter()
        .filter(|f| !is_option_type(&f.ty))
        .map(|f| f.name.to_string())
        .collect();

    if required.is_empty() {
        return quote! {};
    }

    quote! {
        .required(&[#(#required),*])
    }
}

/// Check if a type is Option<T>
fn is_option_type(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty {
        type_path
            .path
            .segments
            .last()
            .map(|seg| seg.ident == "Option")
            .unwrap_or(false)
    } else {
        false
    }
}

/// Extract inner type from Vec<T>
fn extract_vec_inner_type(ty: &Type) -> syn::Result<&Type> {
    if let Type::Path(type_path) = ty {
        if let Some(seg) = type_path.path.segments.last() {
            if seg.ident == "Vec" {
                if let syn::PathArguments::AngleBracketed(args) = &seg.arguments {
                    if let Some(syn::GenericArgument::Type(inner)) = args.args.first() {
                        return Ok(inner);
                    }
                }
            }
        }
    }
    Err(syn::Error::new_spanned(ty, "Expected Vec<T>"))
}

/// Extract inner type from Option<T>
fn extract_option_inner_type(ty: &Type) -> syn::Result<&Type> {
    if let Type::Path(type_path) = ty {
        if let Some(seg) = type_path.path.segments.last() {
            if seg.ident == "Option" {
                if let syn::PathArguments::AngleBracketed(args) = &seg.arguments {
                    if let Some(syn::GenericArgument::Type(inner)) = args.args.first() {
                        return Ok(inner);
                    }
                }
            }
        }
    }
    Err(syn::Error::new_spanned(ty, "Expected Option<T>"))
}

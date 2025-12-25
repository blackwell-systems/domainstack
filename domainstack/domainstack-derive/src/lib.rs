use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Attribute, Data, DeriveInput, Expr, Field, Fields, Lit, Meta};

mod schema;

#[proc_macro_derive(Validate, attributes(validate))]
pub fn derive_validate(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match generate_validate_impl(&input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro_derive(ToSchema, attributes(schema, validate))]
pub fn derive_to_schema(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match schema::derive_to_schema_impl(input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

#[cfg(feature = "serde")]
#[proc_macro_derive(ValidateOnDeserialize, attributes(validate, serde))]
pub fn derive_validate_on_deserialize(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match generate_validate_on_deserialize_impl(&input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

#[cfg(feature = "serde")]
fn generate_validate_on_deserialize_impl(input: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let name = &input.ident;
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // Only support structs with named fields
    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => {
                return Err(syn::Error::new_spanned(
                    input,
                    "#[derive(ValidateOnDeserialize)] only supports structs with named fields",
                ))
            }
        },
        _ => {
            return Err(syn::Error::new_spanned(
                input,
                "#[derive(ValidateOnDeserialize)] only supports structs",
            ))
        }
    };

    // Parse struct-level validation checks (reuse existing function)
    let struct_validations = parse_struct_attributes(input)?;

    // Parse validation rules for each field (reuse existing function)
    let mut field_validations = Vec::new();
    for field in fields {
        let field_name = field.ident.as_ref().unwrap().clone();
        let field_type = field.ty.clone();
        let rules = parse_field_attributes(field)?;

        if !rules.is_empty() {
            field_validations.push(FieldValidation {
                field_name,
                field_type,
                rules,
            });
        }
    }

    // Generate validation code for each field (reuse existing function)
    let field_validation_code = field_validations.iter().map(generate_field_validation);

    // Generate validation code for struct-level checks (reuse existing function)
    let struct_validation_code = struct_validations.iter().map(generate_struct_validation);

    // Generate intermediate struct name
    let intermediate_name = syn::Ident::new(
        &format!("{}Intermediate", name),
        name.span(),
    );

    // Extract field names and types
    let field_names: Vec<_> = fields
        .iter()
        .map(|f| f.ident.as_ref().unwrap())
        .collect();

    let field_types: Vec<_> = fields
        .iter()
        .map(|f| &f.ty)
        .collect();

    // Forward serde attributes from the original struct to the intermediate struct
    // This ensures rename, rename_all, etc. work correctly
    let struct_serde_attrs: Vec<_> = input.attrs.iter()
        .filter(|attr| attr.path().is_ident("serde"))
        .collect();

    // Forward serde attributes for each field
    let field_serde_attrs: Vec<Vec<_>> = fields.iter()
        .map(|f| f.attrs.iter()
            .filter(|attr| attr.path().is_ident("serde"))
            .collect())
        .collect();

    // Generate the expanded code
    let expanded = quote! {
        // Intermediate struct for deserialization
        #[derive(::serde::Deserialize)]
        #[doc(hidden)]
        #( #struct_serde_attrs )*
        struct #intermediate_name #generics {
            #(
                #( #field_serde_attrs )*
                #field_names: #field_types,
            )*
        }

        // Implement Deserialize for the main struct
        impl<'de> ::serde::Deserialize<'de> for #name #ty_generics #where_clause {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: ::serde::Deserializer<'de>,
            {
                // Phase 1: Deserialize into intermediate struct
                let intermediate = #intermediate_name::deserialize(deserializer)?;

                // Phase 2: Construct the final struct
                let value = #name {
                    #( #field_names: intermediate.#field_names, )*
                };

                // Phase 3: Validate using fully qualified syntax
                <#name #ty_generics as ::domainstack::Validate>::validate(&value)
                    .map_err(|e| ::serde::de::Error::custom(format!("Validation failed: {}", e)))?;

                Ok(value)
            }
        }

        // Generate Validate implementation with actual validation logic
        impl #impl_generics ::domainstack::Validate for #name #ty_generics #where_clause {
            fn validate(&self) -> Result<(), ::domainstack::ValidationError> {
                let mut err = ::domainstack::ValidationError::default();

                // Field-level validations
                #(#field_validation_code)*

                // Struct-level validations (cross-field checks)
                #(#struct_validation_code)*

                if err.is_empty() { Ok(()) } else { Err(err) }
            }
        }
    };

    Ok(expanded)
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
enum ValidationRule {
    // Existing rules
    Length {
        min: Option<usize>,
        max: Option<usize>,
        code: Option<String>,
        message: Option<String>,
    },
    Range {
        min: Option<proc_macro2::TokenStream>,
        max: Option<proc_macro2::TokenStream>,
        code: Option<String>,
        message: Option<String>,
    },
    Nested,
    Each(Box<ValidationRule>),
    Custom(String),

    // New rich syntax rules
    Email,
    Url,
    MinLen(usize),
    MaxLen(usize),
    Alphanumeric,
    Ascii,
    AlphaOnly,
    NumericString,
    NonEmpty,
    NonBlank,
    NoWhitespace,
    Contains(String),
    StartsWith(String),
    EndsWith(String),
    MatchesRegex(String),
    Min(proc_macro2::TokenStream),
    Max(proc_macro2::TokenStream),
    Positive,
    Negative,
    NonZero,
    Finite,
    MultipleOf(proc_macro2::TokenStream),
    Equals(proc_macro2::TokenStream),
    NotEquals(proc_macro2::TokenStream),
    OneOf(Vec<String>),
    MinItems(usize),
    MaxItems(usize),
    Unique,
}

#[derive(Debug, Clone)]
struct StructValidation {
    check: String,
    code: Option<String>,
    message: Option<String>,
    when: Option<String>,
}

#[derive(Debug)]
#[allow(dead_code)]
struct FieldValidation {
    field_name: syn::Ident,
    field_type: syn::Type,
    rules: Vec<ValidationRule>,
}

fn generate_validate_impl(input: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let name = &input.ident;
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // Only support structs with named fields
    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => {
                return Err(syn::Error::new_spanned(
                    input,
                    "#[derive(Validate)] only supports structs with named fields",
                ))
            }
        },
        _ => {
            return Err(syn::Error::new_spanned(
                input,
                "#[derive(Validate)] only supports structs",
            ))
        }
    };

    // Parse struct-level validation checks
    let struct_validations = parse_struct_attributes(input)?;

    // Parse validation rules for each field
    let mut field_validations = Vec::new();
    for field in fields {
        let field_name = field.ident.as_ref().unwrap().clone();
        let field_type = field.ty.clone();
        let rules = parse_field_attributes(field)?;

        if !rules.is_empty() {
            field_validations.push(FieldValidation {
                field_name,
                field_type,
                rules,
            });
        }
    }

    // Generate validation code for each field
    let field_validation_code = field_validations.iter().map(generate_field_validation);

    // Generate validation code for struct-level checks
    let struct_validation_code = struct_validations.iter().map(generate_struct_validation);

    let expanded = quote! {
        impl #impl_generics domainstack::Validate for #name #ty_generics #where_clause {
            fn validate(&self) -> Result<(), domainstack::ValidationError> {
                let mut err = domainstack::ValidationError::default();

                // Field-level validations
                #(#field_validation_code)*

                // Struct-level validations (cross-field checks)
                #(#struct_validation_code)*

                if err.is_empty() { Ok(()) } else { Err(err) }
            }
        }
    };

    Ok(expanded)
}

fn parse_struct_attributes(input: &DeriveInput) -> syn::Result<Vec<StructValidation>> {
    let mut validations = Vec::new();

    for attr in &input.attrs {
        if !attr.path().is_ident("validate") {
            continue;
        }

        let validation = parse_struct_validate_attribute(attr)?;
        validations.push(validation);
    }

    Ok(validations)
}

fn parse_struct_validate_attribute(attr: &Attribute) -> syn::Result<StructValidation> {
    let meta = &attr.meta;

    match meta {
        Meta::List(list) => {
            let nested: syn::punctuated::Punctuated<Meta, syn::Token![,]> =
                list.parse_args_with(syn::punctuated::Punctuated::parse_terminated)?;

            let mut check = None;
            let mut code = None;
            let mut message = None;
            let mut when = None;

            for meta in nested {
                match meta {
                    Meta::NameValue(nv) => {
                        if nv.path.is_ident("check") {
                            check = Some(parse_string_lit(&nv.value)?);
                        } else if nv.path.is_ident("code") {
                            code = Some(parse_string_lit(&nv.value)?);
                        } else if nv.path.is_ident("message") {
                            message = Some(parse_string_lit(&nv.value)?);
                        } else if nv.path.is_ident("when") {
                            when = Some(parse_string_lit(&nv.value)?);
                        }
                    }
                    _ => return Err(syn::Error::new_spanned(meta, "Expected name = value")),
                }
            }

            let check = check.ok_or_else(|| {
                syn::Error::new_spanned(attr, "Struct-level validation requires 'check' parameter")
            })?;

            Ok(StructValidation {
                check,
                code,
                message,
                when,
            })
        }
        _ => Err(syn::Error::new_spanned(
            attr,
            "Struct-level validation requires #[validate(check = \"...\", ...)]",
        )),
    }
}

fn parse_field_attributes(field: &Field) -> syn::Result<Vec<ValidationRule>> {
    let mut rules = Vec::new();

    for attr in &field.attrs {
        if !attr.path().is_ident("validate") {
            continue;
        }

        // Parse all rules from this attribute
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

            // length (legacy syntax)
            if meta.path.is_ident("length") {
                let mut min = None;
                let mut max = None;
                let mut code = None;
                let mut message = None;

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
                    } else if nested.path.is_ident("code") {
                        let value: syn::Lit = nested.value()?.parse()?;
                        if let syn::Lit::Str(lit_str) = value {
                            code = Some(lit_str.value());
                        }
                    } else if nested.path.is_ident("message") {
                        let value: syn::Lit = nested.value()?.parse()?;
                        if let syn::Lit::Str(lit_str) = value {
                            message = Some(lit_str.value());
                        }
                    }
                    Ok(())
                })?;

                rules.push(ValidationRule::Length {
                    min,
                    max,
                    code,
                    message,
                });
                return Ok(());
            }

            // range
            if meta.path.is_ident("range") {
                let mut min = None;
                let mut max = None;
                let mut code = None;
                let mut message = None;

                meta.parse_nested_meta(|nested| {
                    if nested.path.is_ident("min") {
                        let value: syn::Expr = nested.value()?.parse()?;
                        min = Some(quote! { #value });
                    } else if nested.path.is_ident("max") {
                        let value: syn::Expr = nested.value()?.parse()?;
                        max = Some(quote! { #value });
                    } else if nested.path.is_ident("code") {
                        let value: syn::Lit = nested.value()?.parse()?;
                        if let syn::Lit::Str(lit_str) = value {
                            code = Some(lit_str.value());
                        }
                    } else if nested.path.is_ident("message") {
                        let value: syn::Lit = nested.value()?.parse()?;
                        if let syn::Lit::Str(lit_str) = value {
                            message = Some(lit_str.value());
                        }
                    }
                    Ok(())
                })?;

                rules.push(ValidationRule::Range {
                    min,
                    max,
                    code,
                    message,
                });
                return Ok(());
            }

            // nested
            if meta.path.is_ident("nested") {
                rules.push(ValidationRule::Nested);
                return Ok(());
            }

            // each - supports any validation rule
            if meta.path.is_ident("each") {
                meta.parse_nested_meta(|nested| {
                    // Handle nested
                    if nested.path.is_ident("nested") {
                        rules.push(ValidationRule::Each(Box::new(ValidationRule::Nested)));
                        return Ok(());
                    }

                    // Handle length
                    if nested.path.is_ident("length") {
                        let mut min = None;
                        let mut max = None;
                        nested.parse_nested_meta(|inner| {
                            if inner.path.is_ident("min") {
                                let value: syn::Lit = inner.value()?.parse()?;
                                if let syn::Lit::Int(lit_int) = value {
                                    min = Some(lit_int.base10_parse()?);
                                }
                            } else if inner.path.is_ident("max") {
                                let value: syn::Lit = inner.value()?.parse()?;
                                if let syn::Lit::Int(lit_int) = value {
                                    max = Some(lit_int.base10_parse()?);
                                }
                            }
                            Ok(())
                        })?;
                        rules.push(ValidationRule::Each(Box::new(ValidationRule::Length {
                            min,
                            max,
                            code: None,
                            message: None,
                        })));
                        return Ok(());
                    }

                    // Handle range
                    if nested.path.is_ident("range") {
                        let mut min = None;
                        let mut max = None;
                        nested.parse_nested_meta(|inner| {
                            if inner.path.is_ident("min") {
                                let value: syn::Expr = inner.value()?.parse()?;
                                min = Some(quote! { #value });
                            } else if inner.path.is_ident("max") {
                                let value: syn::Expr = inner.value()?.parse()?;
                                max = Some(quote! { #value });
                            }
                            Ok(())
                        })?;
                        rules.push(ValidationRule::Each(Box::new(ValidationRule::Range {
                            min,
                            max,
                            code: None,
                            message: None,
                        })));
                        return Ok(());
                    }

                    // Handle all simple string rules
                    if nested.path.is_ident("email") {
                        rules.push(ValidationRule::Each(Box::new(ValidationRule::Email)));
                        return Ok(());
                    }
                    if nested.path.is_ident("url") {
                        rules.push(ValidationRule::Each(Box::new(ValidationRule::Url)));
                        return Ok(());
                    }
                    if nested.path.is_ident("alphanumeric") {
                        rules.push(ValidationRule::Each(Box::new(ValidationRule::Alphanumeric)));
                        return Ok(());
                    }
                    if nested.path.is_ident("ascii") {
                        rules.push(ValidationRule::Each(Box::new(ValidationRule::Ascii)));
                        return Ok(());
                    }
                    if nested.path.is_ident("alpha_only") {
                        rules.push(ValidationRule::Each(Box::new(ValidationRule::AlphaOnly)));
                        return Ok(());
                    }
                    if nested.path.is_ident("numeric_string") {
                        rules.push(ValidationRule::Each(Box::new(
                            ValidationRule::NumericString,
                        )));
                        return Ok(());
                    }
                    if nested.path.is_ident("non_empty") {
                        rules.push(ValidationRule::Each(Box::new(ValidationRule::NonEmpty)));
                        return Ok(());
                    }
                    if nested.path.is_ident("non_blank") {
                        rules.push(ValidationRule::Each(Box::new(ValidationRule::NonBlank)));
                        return Ok(());
                    }

                    // Handle rules with parameters
                    if nested.path.is_ident("min_len") {
                        let value: syn::Lit = nested.value()?.parse()?;
                        if let syn::Lit::Int(lit_int) = value {
                            let val = lit_int.base10_parse()?;
                            rules.push(ValidationRule::Each(Box::new(ValidationRule::MinLen(val))));
                        }
                        return Ok(());
                    }
                    if nested.path.is_ident("max_len") {
                        let value: syn::Lit = nested.value()?.parse()?;
                        if let syn::Lit::Int(lit_int) = value {
                            let val = lit_int.base10_parse()?;
                            rules.push(ValidationRule::Each(Box::new(ValidationRule::MaxLen(val))));
                        }
                        return Ok(());
                    }
                    if nested.path.is_ident("matches_regex") {
                        let value: syn::Lit = nested.value()?.parse()?;
                        if let syn::Lit::Str(lit_str) = value {
                            rules.push(ValidationRule::Each(Box::new(
                                ValidationRule::MatchesRegex(lit_str.value()),
                            )));
                        }
                        return Ok(());
                    }

                    Ok(())
                })?;
                return Ok(());
            }

            // custom
            if meta.path.is_ident("custom") {
                let value: syn::Lit = meta.value()?.parse()?;
                if let syn::Lit::Str(lit_str) = value {
                    rules.push(ValidationRule::Custom(lit_str.value()));
                }
                return Ok(());
            }

            // String pattern rules
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

            if meta.path.is_ident("no_whitespace") {
                rules.push(ValidationRule::NoWhitespace);
                return Ok(());
            }

            // String content rules with values
            if meta.path.is_ident("contains") {
                let value: syn::Lit = meta.value()?.parse()?;
                if let syn::Lit::Str(lit_str) = value {
                    rules.push(ValidationRule::Contains(lit_str.value()));
                }
                return Ok(());
            }

            if meta.path.is_ident("starts_with") {
                let value: syn::Lit = meta.value()?.parse()?;
                if let syn::Lit::Str(lit_str) = value {
                    rules.push(ValidationRule::StartsWith(lit_str.value()));
                }
                return Ok(());
            }

            if meta.path.is_ident("ends_with") {
                let value: syn::Lit = meta.value()?.parse()?;
                if let syn::Lit::Str(lit_str) = value {
                    rules.push(ValidationRule::EndsWith(lit_str.value()));
                }
                return Ok(());
            }

            if meta.path.is_ident("matches_regex") {
                let value: syn::Lit = meta.value()?.parse()?;
                if let syn::Lit::Str(lit_str) = value {
                    rules.push(ValidationRule::MatchesRegex(lit_str.value()));
                }
                return Ok(());
            }

            // Numeric rules
            if meta.path.is_ident("min") {
                let value: syn::Expr = meta.value()?.parse()?;
                rules.push(ValidationRule::Min(quote! { #value }));
                return Ok(());
            }

            if meta.path.is_ident("max") {
                let value: syn::Expr = meta.value()?.parse()?;
                rules.push(ValidationRule::Max(quote! { #value }));
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

            if meta.path.is_ident("finite") {
                rules.push(ValidationRule::Finite);
                return Ok(());
            }

            if meta.path.is_ident("multiple_of") {
                let value: syn::Expr = meta.value()?.parse()?;
                rules.push(ValidationRule::MultipleOf(quote! { #value }));
                return Ok(());
            }

            // Choice rules
            if meta.path.is_ident("equals") {
                let value: syn::Expr = meta.value()?.parse()?;
                rules.push(ValidationRule::Equals(quote! { #value }));
                return Ok(());
            }

            if meta.path.is_ident("not_equals") {
                let value: syn::Expr = meta.value()?.parse()?;
                rules.push(ValidationRule::NotEquals(quote! { #value }));
                return Ok(());
            }

            // Collection rules
            if meta.path.is_ident("min_items") {
                let value: syn::Lit = meta.value()?.parse()?;
                if let syn::Lit::Int(lit_int) = value {
                    let val = lit_int.base10_parse()?;
                    rules.push(ValidationRule::MinItems(val));
                }
                return Ok(());
            }

            if meta.path.is_ident("max_items") {
                let value: syn::Lit = meta.value()?.parse()?;
                if let syn::Lit::Int(lit_int) = value {
                    let val = lit_int.base10_parse()?;
                    rules.push(ValidationRule::MaxItems(val));
                }
                return Ok(());
            }

            if meta.path.is_ident("unique") {
                rules.push(ValidationRule::Unique);
                return Ok(());
            }

            // Unknown rule - silently ignore for forward compatibility
            Ok(())
        })?;
    }

    Ok(rules)
}

fn parse_string_lit(expr: &Expr) -> syn::Result<String> {
    match expr {
        Expr::Lit(lit_expr) => match &lit_expr.lit {
            Lit::Str(str_lit) => Ok(str_lit.value()),
            _ => Err(syn::Error::new_spanned(expr, "Expected string literal")),
        },
        _ => Err(syn::Error::new_spanned(expr, "Expected string literal")),
    }
}

fn generate_field_validation(fv: &FieldValidation) -> proc_macro2::TokenStream {
    let field_name = &fv.field_name;
    let field_name_str = field_name.to_string();

    let validations: Vec<_> = fv
        .rules
        .iter()
        .map(|rule| match rule {
            // Legacy rules
            ValidationRule::Length { min, max, .. } => {
                generate_length_validation(field_name, &field_name_str, min, max)
            }
            ValidationRule::Range { min, max, .. } => {
                generate_range_validation(field_name, &field_name_str, min, max)
            }
            ValidationRule::Nested => generate_nested_validation(field_name, &field_name_str),
            ValidationRule::Each(inner_rule) => {
                generate_each_validation(field_name, &field_name_str, inner_rule)
            }
            ValidationRule::Custom(fn_path) => {
                generate_custom_validation(field_name, &field_name_str, fn_path)
            }

            // New rich syntax rules - String rules
            ValidationRule::Email => {
                generate_simple_string_rule(field_name, &field_name_str, "email")
            }
            ValidationRule::Url => generate_simple_string_rule(field_name, &field_name_str, "url"),
            ValidationRule::MinLen(min) => generate_min_len(field_name, &field_name_str, *min),
            ValidationRule::MaxLen(max) => generate_max_len(field_name, &field_name_str, *max),
            ValidationRule::Alphanumeric => {
                generate_simple_string_rule(field_name, &field_name_str, "alphanumeric")
            }
            ValidationRule::Ascii => {
                generate_simple_string_rule(field_name, &field_name_str, "ascii")
            }
            ValidationRule::AlphaOnly => {
                generate_simple_string_rule(field_name, &field_name_str, "alpha_only")
            }
            ValidationRule::NumericString => {
                generate_simple_string_rule(field_name, &field_name_str, "numeric_string")
            }
            ValidationRule::NonEmpty => {
                generate_simple_string_rule(field_name, &field_name_str, "non_empty")
            }
            ValidationRule::NonBlank => {
                generate_simple_string_rule(field_name, &field_name_str, "non_blank")
            }
            ValidationRule::NoWhitespace => {
                generate_simple_string_rule(field_name, &field_name_str, "no_whitespace")
            }
            ValidationRule::Contains(substr) => {
                generate_string_param_rule(field_name, &field_name_str, "contains", substr)
            }
            ValidationRule::StartsWith(prefix) => {
                generate_string_param_rule(field_name, &field_name_str, "starts_with", prefix)
            }
            ValidationRule::EndsWith(suffix) => {
                generate_string_param_rule(field_name, &field_name_str, "ends_with", suffix)
            }
            ValidationRule::MatchesRegex(pattern) => {
                generate_matches_regex(field_name, &field_name_str, pattern)
            }

            // Numeric rules
            ValidationRule::Min(min) => generate_min_max(field_name, &field_name_str, "min", min),
            ValidationRule::Max(max) => generate_min_max(field_name, &field_name_str, "max", max),
            ValidationRule::Positive => {
                generate_simple_numeric_rule(field_name, &field_name_str, "positive")
            }
            ValidationRule::Negative => {
                generate_simple_numeric_rule(field_name, &field_name_str, "negative")
            }
            ValidationRule::NonZero => {
                generate_simple_numeric_rule(field_name, &field_name_str, "non_zero")
            }
            ValidationRule::Finite => {
                generate_simple_numeric_rule(field_name, &field_name_str, "finite")
            }
            ValidationRule::MultipleOf(n) => {
                generate_min_max(field_name, &field_name_str, "multiple_of", n)
            }

            // Choice rules
            ValidationRule::Equals(val) => {
                generate_min_max(field_name, &field_name_str, "equals", val)
            }
            ValidationRule::NotEquals(val) => {
                generate_min_max(field_name, &field_name_str, "not_equals", val)
            }
            ValidationRule::OneOf(values) => generate_one_of(field_name, &field_name_str, values),

            // Collection rules
            ValidationRule::MinItems(min) => {
                generate_collection_rule(field_name, &field_name_str, "min_items", *min)
            }
            ValidationRule::MaxItems(max) => {
                generate_collection_rule(field_name, &field_name_str, "max_items", *max)
            }
            ValidationRule::Unique => {
                generate_simple_collection_rule(field_name, &field_name_str, "unique")
            }
        })
        .collect();

    quote! {
        #(#validations)*
    }
}

fn generate_length_validation(
    field_name: &syn::Ident,
    field_name_str: &str,
    min: &Option<usize>,
    max: &Option<usize>,
) -> proc_macro2::TokenStream {
    let rule = match (min, max) {
        (Some(min), Some(max)) => {
            quote! { domainstack::rules::min_len(#min).and(domainstack::rules::max_len(#max)) }
        }
        (Some(min), None) => {
            quote! { domainstack::rules::min_len(#min) }
        }
        (None, Some(max)) => {
            quote! { domainstack::rules::max_len(#max) }
        }
        (None, None) => {
            // No constraints - skip
            return quote! {};
        }
    };

    quote! {
        {
            let rule = #rule;
            if let Err(e) = domainstack::validate(#field_name_str, self.#field_name.as_str(), &rule) {
                err.extend(e);
            }
        }
    }
}

fn generate_range_validation(
    field_name: &syn::Ident,
    field_name_str: &str,
    min: &Option<proc_macro2::TokenStream>,
    max: &Option<proc_macro2::TokenStream>,
) -> proc_macro2::TokenStream {
    match (min, max) {
        (Some(min), Some(max)) => {
            quote! {
                {
                    let rule = domainstack::rules::range(#min, #max);
                    if let Err(e) = domainstack::validate(#field_name_str, &self.#field_name, &rule) {
                        err.extend(e);
                    }
                }
            }
        }
        _ => {
            // Range requires both min and max
            quote! {}
        }
    }
}

fn generate_nested_validation(
    field_name: &syn::Ident,
    field_name_str: &str,
) -> proc_macro2::TokenStream {
    quote! {
        if let Err(e) = self.#field_name.validate() {
            err.merge_prefixed(#field_name_str, e);
        }
    }
}

fn generate_each_validation(
    field_name: &syn::Ident,
    field_name_str: &str,
    inner_rule: &ValidationRule,
) -> proc_macro2::TokenStream {
    match inner_rule {
        ValidationRule::Nested => {
            quote! {
                for (i, item) in self.#field_name.iter().enumerate() {
                    if let Err(e) = item.validate() {
                        let path = domainstack::Path::root().field(#field_name_str).index(i);
                        err.merge_prefixed(path, e);
                    }
                }
            }
        }
        ValidationRule::Length { min, max, .. } => {
            let rule = match (min, max) {
                (Some(min), Some(max)) => {
                    quote! { domainstack::rules::min_len(#min).and(domainstack::rules::max_len(#max)) }
                }
                (Some(min), None) => {
                    quote! { domainstack::rules::min_len(#min) }
                }
                (None, Some(max)) => {
                    quote! { domainstack::rules::max_len(#max) }
                }
                (None, None) => return quote! {},
            };

            quote! {
                {
                    let rule = #rule;
                    for (i, item) in self.#field_name.iter().enumerate() {
                        let path = domainstack::Path::root().field(#field_name_str).index(i);
                        if let Err(e) = domainstack::validate(path, item.as_str(), &rule) {
                            err.extend(e);
                        }
                    }
                }
            }
        }
        ValidationRule::Range { min, max, .. } => match (min, max) {
            (Some(min), Some(max)) => {
                quote! {
                    {
                        let rule = domainstack::rules::range(#min, #max);
                        for (i, item) in self.#field_name.iter().enumerate() {
                            let path = domainstack::Path::root().field(#field_name_str).index(i);
                            if let Err(e) = domainstack::validate(path, item, &rule) {
                                err.extend(e);
                            }
                        }
                    }
                }
            }
            _ => quote! {},
        },

        // New rich syntax rules - String rules (simple)
        ValidationRule::Email => {
            quote! {
                {
                    let rule = domainstack::rules::email();
                    for (i, item) in self.#field_name.iter().enumerate() {
                        let path = domainstack::Path::root().field(#field_name_str).index(i);
                        if let Err(e) = domainstack::validate(path, item.as_str(), &rule) {
                            err.extend(e);
                        }
                    }
                }
            }
        }
        ValidationRule::Url => {
            quote! {
                {
                    let rule = domainstack::rules::url();
                    for (i, item) in self.#field_name.iter().enumerate() {
                        let path = domainstack::Path::root().field(#field_name_str).index(i);
                        if let Err(e) = domainstack::validate(path, item.as_str(), &rule) {
                            err.extend(e);
                        }
                    }
                }
            }
        }
        ValidationRule::Alphanumeric => {
            quote! {
                {
                    let rule = domainstack::rules::alphanumeric();
                    for (i, item) in self.#field_name.iter().enumerate() {
                        let path = domainstack::Path::root().field(#field_name_str).index(i);
                        if let Err(e) = domainstack::validate(path, item.as_str(), &rule) {
                            err.extend(e);
                        }
                    }
                }
            }
        }
        ValidationRule::Ascii => {
            quote! {
                {
                    let rule = domainstack::rules::ascii();
                    for (i, item) in self.#field_name.iter().enumerate() {
                        let path = domainstack::Path::root().field(#field_name_str).index(i);
                        if let Err(e) = domainstack::validate(path, item.as_str(), &rule) {
                            err.extend(e);
                        }
                    }
                }
            }
        }
        ValidationRule::AlphaOnly => {
            quote! {
                {
                    let rule = domainstack::rules::alpha_only();
                    for (i, item) in self.#field_name.iter().enumerate() {
                        let path = domainstack::Path::root().field(#field_name_str).index(i);
                        if let Err(e) = domainstack::validate(path, item.as_str(), &rule) {
                            err.extend(e);
                        }
                    }
                }
            }
        }
        ValidationRule::NumericString => {
            quote! {
                {
                    let rule = domainstack::rules::numeric_string();
                    for (i, item) in self.#field_name.iter().enumerate() {
                        let path = domainstack::Path::root().field(#field_name_str).index(i);
                        if let Err(e) = domainstack::validate(path, item.as_str(), &rule) {
                            err.extend(e);
                        }
                    }
                }
            }
        }
        ValidationRule::NonEmpty => {
            quote! {
                {
                    let rule = domainstack::rules::non_empty();
                    for (i, item) in self.#field_name.iter().enumerate() {
                        let path = domainstack::Path::root().field(#field_name_str).index(i);
                        if let Err(e) = domainstack::validate(path, item.as_str(), &rule) {
                            err.extend(e);
                        }
                    }
                }
            }
        }
        ValidationRule::NonBlank => {
            quote! {
                {
                    let rule = domainstack::rules::non_blank();
                    for (i, item) in self.#field_name.iter().enumerate() {
                        let path = domainstack::Path::root().field(#field_name_str).index(i);
                        if let Err(e) = domainstack::validate(path, item.as_str(), &rule) {
                            err.extend(e);
                        }
                    }
                }
            }
        }

        // String rules with parameters
        ValidationRule::MinLen(min) => {
            quote! {
                {
                    let rule = domainstack::rules::min_len(#min);
                    for (i, item) in self.#field_name.iter().enumerate() {
                        let path = domainstack::Path::root().field(#field_name_str).index(i);
                        if let Err(e) = domainstack::validate(path, item.as_str(), &rule) {
                            err.extend(e);
                        }
                    }
                }
            }
        }
        ValidationRule::MaxLen(max) => {
            quote! {
                {
                    let rule = domainstack::rules::max_len(#max);
                    for (i, item) in self.#field_name.iter().enumerate() {
                        let path = domainstack::Path::root().field(#field_name_str).index(i);
                        if let Err(e) = domainstack::validate(path, item.as_str(), &rule) {
                            err.extend(e);
                        }
                    }
                }
            }
        }
        ValidationRule::MatchesRegex(pattern) => {
            quote! {
                {
                    let rule = domainstack::rules::matches_regex(#pattern);
                    for (i, item) in self.#field_name.iter().enumerate() {
                        let path = domainstack::Path::root().field(#field_name_str).index(i);
                        if let Err(e) = domainstack::validate(path, item.as_str(), &rule) {
                            err.extend(e);
                        }
                    }
                }
            }
        }

        _ => quote! {},
    }
}

fn generate_custom_validation(
    field_name: &syn::Ident,
    field_name_str: &str,
    fn_path: &str,
) -> proc_macro2::TokenStream {
    let fn_path: proc_macro2::TokenStream = fn_path.parse().unwrap();

    quote! {
        if let Err(e) = #fn_path(&self.#field_name) {
            err.extend(e.prefixed(#field_name_str));
        }
    }
}

// Helper functions for new validation rule code generation

fn generate_simple_string_rule(
    field_name: &syn::Ident,
    field_name_str: &str,
    rule_fn: &str,
) -> proc_macro2::TokenStream {
    let rule_fn: proc_macro2::TokenStream = format!("domainstack::rules::{}()", rule_fn)
        .parse()
        .unwrap();
    quote! {
        {
            let rule = #rule_fn;
            if let Err(e) = domainstack::validate(#field_name_str, self.#field_name.as_str(), &rule) {
                err.extend(e);
            }
        }
    }
}

fn generate_min_len(
    field_name: &syn::Ident,
    field_name_str: &str,
    min: usize,
) -> proc_macro2::TokenStream {
    quote! {
        {
            let rule = domainstack::rules::min_len(#min);
            if let Err(e) = domainstack::validate(#field_name_str, self.#field_name.as_str(), &rule) {
                err.extend(e);
            }
        }
    }
}

fn generate_max_len(
    field_name: &syn::Ident,
    field_name_str: &str,
    max: usize,
) -> proc_macro2::TokenStream {
    quote! {
        {
            let rule = domainstack::rules::max_len(#max);
            if let Err(e) = domainstack::validate(#field_name_str, self.#field_name.as_str(), &rule) {
                err.extend(e);
            }
        }
    }
}

fn generate_string_param_rule(
    field_name: &syn::Ident,
    field_name_str: &str,
    rule_fn: &str,
    param: &str,
) -> proc_macro2::TokenStream {
    let rule_fn: proc_macro2::TokenStream =
        format!("domainstack::rules::{}(\"{}\")", rule_fn, param)
            .parse()
            .unwrap();
    quote! {
        {
            let rule = #rule_fn;
            if let Err(e) = domainstack::validate(#field_name_str, self.#field_name.as_str(), &rule) {
                err.extend(e);
            }
        }
    }
}

fn generate_matches_regex(
    field_name: &syn::Ident,
    field_name_str: &str,
    pattern: &str,
) -> proc_macro2::TokenStream {
    quote! {
        {
            let rule = domainstack::rules::matches_regex(#pattern);
            if let Err(e) = domainstack::validate(#field_name_str, self.#field_name.as_str(), &rule) {
                err.extend(e);
            }
        }
    }
}

fn generate_simple_numeric_rule(
    field_name: &syn::Ident,
    field_name_str: &str,
    rule_fn: &str,
) -> proc_macro2::TokenStream {
    let rule_fn: proc_macro2::TokenStream = format!("domainstack::rules::{}()", rule_fn)
        .parse()
        .unwrap();
    quote! {
        {
            let rule = #rule_fn;
            if let Err(e) = domainstack::validate(#field_name_str, &self.#field_name, &rule) {
                err.extend(e);
            }
        }
    }
}

fn generate_min_max(
    field_name: &syn::Ident,
    field_name_str: &str,
    rule_fn: &str,
    val: &proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    let rule_fn: proc_macro2::TokenStream = format!("domainstack::rules::{}({})", rule_fn, val)
        .parse()
        .unwrap();
    quote! {
        {
            let rule = #rule_fn;
            if let Err(e) = domainstack::validate(#field_name_str, &self.#field_name, &rule) {
                err.extend(e);
            }
        }
    }
}

fn generate_one_of(
    field_name: &syn::Ident,
    field_name_str: &str,
    values: &[String],
) -> proc_macro2::TokenStream {
    let values_str = values.iter().map(|v| quote! { #v }).collect::<Vec<_>>();
    quote! {
        {
            let rule = domainstack::rules::one_of(&[#(#values_str),*]);
            if let Err(e) = domainstack::validate(#field_name_str, &self.#field_name, &rule) {
                err.extend(e);
            }
        }
    }
}

fn generate_collection_rule(
    field_name: &syn::Ident,
    field_name_str: &str,
    rule_fn: &str,
    val: usize,
) -> proc_macro2::TokenStream {
    let rule_fn: proc_macro2::TokenStream = format!("domainstack::rules::{}({})", rule_fn, val)
        .parse()
        .unwrap();
    quote! {
        {
            let rule = #rule_fn;
            if let Err(e) = domainstack::validate(#field_name_str, &self.#field_name, &rule) {
                err.extend(e);
            }
        }
    }
}

fn generate_simple_collection_rule(
    field_name: &syn::Ident,
    field_name_str: &str,
    rule_fn: &str,
) -> proc_macro2::TokenStream {
    let rule_fn: proc_macro2::TokenStream = format!("domainstack::rules::{}()", rule_fn)
        .parse()
        .unwrap();
    quote! {
        {
            let rule = #rule_fn;
            if let Err(e) = domainstack::validate(#field_name_str, &self.#field_name, &rule) {
                err.extend(e);
            }
        }
    }
}

fn generate_struct_validation(sv: &StructValidation) -> proc_macro2::TokenStream {
    let check_expr: proc_macro2::TokenStream = sv.check.parse().unwrap();
    let code = sv
        .code
        .as_deref()
        .unwrap_or("cross_field_validation_failed");
    let message = sv
        .message
        .as_deref()
        .unwrap_or("Cross-field validation failed");

    let validation_code = quote! {
        if !(#check_expr) {
            err.violations.push(domainstack::Violation {
                path: domainstack::Path::root(),
                code: #code,
                message: #message.to_string(),
                meta: domainstack::Meta::default(),
            });
        }
    };

    // Wrap in conditional if 'when' is specified
    if let Some(when_expr) = &sv.when {
        let when_tokens: proc_macro2::TokenStream = when_expr.parse().unwrap();
        quote! {
            if #when_tokens {
                #validation_code
            }
        }
    } else {
        validation_code
    }
}

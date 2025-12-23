use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Data, Fields, Field, Attribute, Meta, Lit, Expr};

#[proc_macro_derive(Validate, attributes(validate))]
pub fn derive_validate(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    
    match generate_validate_impl(&input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

#[derive(Debug, Clone)]
enum ValidationRule {
    Length { min: Option<usize>, max: Option<usize>, code: Option<String>, message: Option<String> },
    Range { min: Option<proc_macro2::TokenStream>, max: Option<proc_macro2::TokenStream>, code: Option<String>, message: Option<String> },
    Nested,
    Each(Box<ValidationRule>),
    Custom(String),
}

#[derive(Debug)]
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
        Data::Struct(data) => {
            match &data.fields {
                Fields::Named(fields) => &fields.named,
                _ => return Err(syn::Error::new_spanned(
                    input,
                    "#[derive(Validate)] only supports structs with named fields"
                )),
            }
        },
        _ => return Err(syn::Error::new_spanned(
            input,
            "#[derive(Validate)] only supports structs"
        )),
    };
    
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
    let validation_code = field_validations.iter().map(|fv| {
        generate_field_validation(fv)
    });
    
    let expanded = quote! {
        impl #impl_generics domainstack::Validate for #name #ty_generics #where_clause {
            fn validate(&self) -> Result<(), domainstack::ValidationError> {
                let mut err = domainstack::ValidationError::default();
                
                #(#validation_code)*
                
                if err.is_empty() { Ok(()) } else { Err(err) }
            }
        }
    };
    
    Ok(expanded)
}

fn parse_field_attributes(field: &Field) -> syn::Result<Vec<ValidationRule>> {
    let mut rules = Vec::new();
    
    for attr in &field.attrs {
        if !attr.path().is_ident("validate") {
            continue;
        }
        
        let rule = parse_validate_attribute(attr)?;
        rules.push(rule);
    }
    
    Ok(rules)
}

fn parse_validate_attribute(attr: &Attribute) -> syn::Result<ValidationRule> {
    let meta = &attr.meta;
    
    match meta {
        Meta::Path(_) => {
            // #[validate] with no arguments - error
            Err(syn::Error::new_spanned(meta, "#[validate] requires an argument"))
        },
        Meta::List(list) => {
            // Parse the tokens inside the list
            let nested: syn::punctuated::Punctuated<Meta, syn::Token![,]> = 
                list.parse_args_with(syn::punctuated::Punctuated::parse_terminated)?;
            
            if nested.is_empty() {
                return Err(syn::Error::new_spanned(meta, "#[validate(...)] requires an argument"));
            }
            
            // Get the first meta (e.g., "length", "range", "nested", "each", "custom")
            let first = nested.first().unwrap();
            
            match first {
                Meta::Path(path) if path.is_ident("nested") => {
                    Ok(ValidationRule::Nested)
                },
                Meta::List(list) if list.path.is_ident("length") => {
                    parse_length_rule(list)
                },
                Meta::List(list) if list.path.is_ident("range") => {
                    parse_range_rule(list)
                },
                Meta::List(list) if list.path.is_ident("each") => {
                    parse_each_rule(list)
                },
                Meta::NameValue(nv) if nv.path.is_ident("custom") => {
                    parse_custom_rule(nv)
                },
                _ => Err(syn::Error::new_spanned(
                    first,
                    "Unknown validation rule. Expected: length, range, nested, each, or custom"
                )),
            }
        },
        Meta::NameValue(_) => {
            Err(syn::Error::new_spanned(meta, "Use #[validate(rule)] syntax"))
        },
    }
}

fn parse_length_rule(list: &syn::MetaList) -> syn::Result<ValidationRule> {
    let nested: syn::punctuated::Punctuated<Meta, syn::Token![,]> = 
        list.parse_args_with(syn::punctuated::Punctuated::parse_terminated)?;
    
    let mut min = None;
    let mut max = None;
    let mut code = None;
    let mut message = None;
    
    for meta in nested {
        match meta {
            Meta::NameValue(nv) => {
                if nv.path.is_ident("min") {
                    min = Some(parse_usize_lit(&nv.value)?);
                } else if nv.path.is_ident("max") {
                    max = Some(parse_usize_lit(&nv.value)?);
                } else if nv.path.is_ident("code") {
                    code = Some(parse_string_lit(&nv.value)?);
                } else if nv.path.is_ident("message") {
                    message = Some(parse_string_lit(&nv.value)?);
                }
            },
            _ => return Err(syn::Error::new_spanned(meta, "Expected name = value")),
        }
    }
    
    Ok(ValidationRule::Length { min, max, code, message })
}

fn parse_range_rule(list: &syn::MetaList) -> syn::Result<ValidationRule> {
    let nested: syn::punctuated::Punctuated<Meta, syn::Token![,]> = 
        list.parse_args_with(syn::punctuated::Punctuated::parse_terminated)?;
    
    let mut min = None;
    let mut max = None;
    let mut code = None;
    let mut message = None;
    
    for meta in nested {
        match meta {
            Meta::NameValue(nv) => {
                if nv.path.is_ident("min") {
                    min = Some(expr_to_tokens(&nv.value)?);
                } else if nv.path.is_ident("max") {
                    max = Some(expr_to_tokens(&nv.value)?);
                } else if nv.path.is_ident("code") {
                    code = Some(parse_string_lit(&nv.value)?);
                } else if nv.path.is_ident("message") {
                    message = Some(parse_string_lit(&nv.value)?);
                }
            },
            _ => return Err(syn::Error::new_spanned(meta, "Expected name = value")),
        }
    }
    
    Ok(ValidationRule::Range { min, max, code, message })
}

fn parse_each_rule(list: &syn::MetaList) -> syn::Result<ValidationRule> {
    let nested: syn::punctuated::Punctuated<Meta, syn::Token![,]> = 
        list.parse_args_with(syn::punctuated::Punctuated::parse_terminated)?;
    
    if nested.is_empty() {
        return Err(syn::Error::new_spanned(list, "each requires an inner rule"));
    }
    
    let first = nested.first().unwrap();
    
    let inner_rule = match first {
        Meta::Path(path) if path.is_ident("nested") => ValidationRule::Nested,
        Meta::List(inner_list) if inner_list.path.is_ident("length") => parse_length_rule(inner_list)?,
        Meta::List(inner_list) if inner_list.path.is_ident("range") => parse_range_rule(inner_list)?,
        _ => return Err(syn::Error::new_spanned(first, "each supports: nested, length, or range")),
    };
    
    Ok(ValidationRule::Each(Box::new(inner_rule)))
}

fn parse_custom_rule(nv: &syn::MetaNameValue) -> syn::Result<ValidationRule> {
    let fn_path = parse_string_lit(&nv.value)?;
    Ok(ValidationRule::Custom(fn_path))
}

fn parse_usize_lit(expr: &Expr) -> syn::Result<usize> {
    match expr {
        Expr::Lit(lit_expr) => {
            match &lit_expr.lit {
                Lit::Int(int_lit) => {
                    int_lit.base10_parse()
                },
                _ => Err(syn::Error::new_spanned(expr, "Expected integer literal")),
            }
        },
        _ => Err(syn::Error::new_spanned(expr, "Expected integer literal")),
    }
}

fn parse_string_lit(expr: &Expr) -> syn::Result<String> {
    match expr {
        Expr::Lit(lit_expr) => {
            match &lit_expr.lit {
                Lit::Str(str_lit) => Ok(str_lit.value()),
                _ => Err(syn::Error::new_spanned(expr, "Expected string literal")),
            }
        },
        _ => Err(syn::Error::new_spanned(expr, "Expected string literal")),
    }
}

fn expr_to_tokens(expr: &Expr) -> syn::Result<proc_macro2::TokenStream> {
    Ok(quote! { #expr })
}

fn generate_field_validation(fv: &FieldValidation) -> proc_macro2::TokenStream {
    let field_name = &fv.field_name;
    let field_name_str = field_name.to_string();
    
    let validations: Vec<_> = fv.rules.iter().map(|rule| {
        match rule {
            ValidationRule::Length { min, max, .. } => {
                generate_length_validation(field_name, &field_name_str, min, max)
            },
            ValidationRule::Range { min, max, .. } => {
                generate_range_validation(field_name, &field_name_str, min, max)
            },
            ValidationRule::Nested => {
                generate_nested_validation(field_name, &field_name_str)
            },
            ValidationRule::Each(inner_rule) => {
                generate_each_validation(field_name, &field_name_str, inner_rule)
            },
            ValidationRule::Custom(fn_path) => {
                generate_custom_validation(field_name, &field_name_str, fn_path)
            },
        }
    }).collect();
    
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
        },
        (Some(min), None) => {
            quote! { domainstack::rules::min_len(#min) }
        },
        (None, Some(max)) => {
            quote! { domainstack::rules::max_len(#max) }
        },
        (None, None) => {
            // No constraints - skip
            return quote! {};
        },
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
        },
        _ => {
            // Range requires both min and max
            quote! {}
        },
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
        },
        ValidationRule::Length { min, max, .. } => {
            let rule = match (min, max) {
                (Some(min), Some(max)) => {
                    quote! { domainstack::rules::min_len(#min).and(domainstack::rules::max_len(#max)) }
                },
                (Some(min), None) => {
                    quote! { domainstack::rules::min_len(#min) }
                },
                (None, Some(max)) => {
                    quote! { domainstack::rules::max_len(#max) }
                },
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
        },
        ValidationRule::Range { min, max, .. } => {
            match (min, max) {
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
                },
                _ => quote! {},
            }
        },
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

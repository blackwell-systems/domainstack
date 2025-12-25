use anyhow::Result;
use syn::Attribute;

/// A validation rule extracted from #[validate(...)] attributes
#[derive(Debug, Clone)]
pub enum ValidationRule {
    // String rules
    Email,
    Url,
    MinLen(usize),
    MaxLen(usize),
    Length { min: usize, max: usize },
    NonEmpty,
    NonBlank,
    Alphanumeric,
    AlphaOnly,
    NumericString,
    Ascii,
    StartsWith(String),
    EndsWith(String),
    Contains(String),
    MatchesRegex(String),
    NoWhitespace,

    // Numeric rules
    Range { min: String, max: String },
    Min(String),
    Max(String),
    Positive,
    Negative,
    NonZero,
    MultipleOf(String),
    Finite,

    // Custom/unknown rules
    Custom(String),
}

/// Parse validation rules from field attributes
pub fn parse_validation_attributes(attrs: &[Attribute]) -> Result<Vec<ValidationRule>> {
    let mut rules = Vec::new();

    for attr in attrs {
        if let Some(ident) = attr.path().get_ident() {
            if ident == "validate" {
                // Parse the validation attribute
                attr.parse_nested_meta(|meta| {
                    let path = &meta.path;

                    if let Some(ident) = path.get_ident() {
                        let rule_name = ident.to_string();

                        match rule_name.as_str() {
                            "email" => rules.push(ValidationRule::Email),
                            "url" => rules.push(ValidationRule::Url),
                            "non_empty" => rules.push(ValidationRule::NonEmpty),
                            "non_blank" => rules.push(ValidationRule::NonBlank),
                            "alphanumeric" => rules.push(ValidationRule::Alphanumeric),
                            "alpha_only" => rules.push(ValidationRule::AlphaOnly),
                            "numeric_string" => rules.push(ValidationRule::NumericString),
                            "ascii" => rules.push(ValidationRule::Ascii),
                            "no_whitespace" => rules.push(ValidationRule::NoWhitespace),
                            "positive" => rules.push(ValidationRule::Positive),
                            "negative" => rules.push(ValidationRule::Negative),
                            "non_zero" => rules.push(ValidationRule::NonZero),
                            "finite" => rules.push(ValidationRule::Finite),
                            "min_len" => {
                                if meta.input.peek(syn::Token![=]) {
                                    meta.input.parse::<syn::Token![=]>()?;
                                    let value: syn::LitInt = meta.input.parse()?;
                                    rules.push(ValidationRule::MinLen(value.base10_parse()?));
                                }
                            }
                            "max_len" => {
                                if meta.input.peek(syn::Token![=]) {
                                    meta.input.parse::<syn::Token![=]>()?;
                                    let value: syn::LitInt = meta.input.parse()?;
                                    rules.push(ValidationRule::MaxLen(value.base10_parse()?));
                                }
                            }
                            "length" => {
                                // Parse length(min = X, max = Y)
                                let content;
                                syn::parenthesized!(content in meta.input);

                                let mut min = None;
                                let mut max = None;

                                while !content.is_empty() {
                                    let key: syn::Ident = content.parse()?;
                                    content.parse::<syn::Token![=]>()?;
                                    let value: syn::LitInt = content.parse()?;

                                    match key.to_string().as_str() {
                                        "min" => min = Some(value.base10_parse()?),
                                        "max" => max = Some(value.base10_parse()?),
                                        _ => {}
                                    }

                                    if content.peek(syn::Token![,]) {
                                        content.parse::<syn::Token![,]>()?;
                                    }
                                }

                                if let (Some(min), Some(max)) = (min, max) {
                                    rules.push(ValidationRule::Length { min, max });
                                }
                            }
                            "range" => {
                                // Parse range(min = X, max = Y)
                                let content;
                                syn::parenthesized!(content in meta.input);

                                let mut min = None;
                                let mut max = None;

                                while !content.is_empty() {
                                    let key: syn::Ident = content.parse()?;
                                    content.parse::<syn::Token![=]>()?;
                                    let value: syn::Lit = content.parse()?;

                                    let value_str = match value {
                                        syn::Lit::Int(i) => i.to_string(),
                                        syn::Lit::Float(f) => f.to_string(),
                                        _ => continue,
                                    };

                                    match key.to_string().as_str() {
                                        "min" => min = Some(value_str),
                                        "max" => max = Some(value_str),
                                        _ => {}
                                    }

                                    if content.peek(syn::Token![,]) {
                                        content.parse::<syn::Token![,]>()?;
                                    }
                                }

                                if let (Some(min), Some(max)) = (min, max) {
                                    rules.push(ValidationRule::Range { min, max });
                                }
                            }
                            "min" => {
                                if meta.input.peek(syn::Token![=]) {
                                    meta.input.parse::<syn::Token![=]>()?;
                                    let value: syn::Lit = meta.input.parse()?;
                                    let value_str = match value {
                                        syn::Lit::Int(i) => i.to_string(),
                                        syn::Lit::Float(f) => f.to_string(),
                                        _ => return Ok(()),
                                    };
                                    rules.push(ValidationRule::Min(value_str));
                                }
                            }
                            "max" => {
                                if meta.input.peek(syn::Token![=]) {
                                    meta.input.parse::<syn::Token![=]>()?;
                                    let value: syn::Lit = meta.input.parse()?;
                                    let value_str = match value {
                                        syn::Lit::Int(i) => i.to_string(),
                                        syn::Lit::Float(f) => f.to_string(),
                                        _ => return Ok(()),
                                    };
                                    rules.push(ValidationRule::Max(value_str));
                                }
                            }
                            "multiple_of" => {
                                if meta.input.peek(syn::Token![=]) {
                                    meta.input.parse::<syn::Token![=]>()?;
                                    let value: syn::Lit = meta.input.parse()?;
                                    let value_str = match value {
                                        syn::Lit::Int(i) => i.to_string(),
                                        syn::Lit::Float(f) => f.to_string(),
                                        _ => return Ok(()),
                                    };
                                    rules.push(ValidationRule::MultipleOf(value_str));
                                }
                            }
                            "starts_with" => {
                                if meta.input.peek(syn::Token![=]) {
                                    meta.input.parse::<syn::Token![=]>()?;
                                    let value: syn::LitStr = meta.input.parse()?;
                                    rules.push(ValidationRule::StartsWith(value.value()));
                                }
                            }
                            "ends_with" => {
                                if meta.input.peek(syn::Token![=]) {
                                    meta.input.parse::<syn::Token![=]>()?;
                                    let value: syn::LitStr = meta.input.parse()?;
                                    rules.push(ValidationRule::EndsWith(value.value()));
                                }
                            }
                            "contains" => {
                                if meta.input.peek(syn::Token![=]) {
                                    meta.input.parse::<syn::Token![=]>()?;
                                    let value: syn::LitStr = meta.input.parse()?;
                                    rules.push(ValidationRule::Contains(value.value()));
                                }
                            }
                            "matches_regex" => {
                                if meta.input.peek(syn::Token![=]) {
                                    meta.input.parse::<syn::Token![=]>()?;
                                    let value: syn::LitStr = meta.input.parse()?;
                                    rules.push(ValidationRule::MatchesRegex(value.value()));
                                }
                            }
                            _ => {
                                // Unknown validation rule, store as custom
                                rules.push(ValidationRule::Custom(rule_name));
                            }
                        }
                    }

                    Ok(())
                })?;
            }
        }
    }

    Ok(rules)
}

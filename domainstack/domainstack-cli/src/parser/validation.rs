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

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;

    // Helper function to parse attributes from a field
    fn parse_field_attributes(tokens: proc_macro2::TokenStream) -> Vec<ValidationRule> {
        // Wrap the field in a struct to make it parseable
        let struct_tokens = quote! {
            struct Test {
                #tokens
            }
        };
        let item_struct: syn::ItemStruct =
            syn::parse2(struct_tokens).expect("Failed to parse struct");
        let field = &item_struct.fields.iter().next().expect("No field found");
        parse_validation_attributes(&field.attrs).expect("Failed to parse validation attributes")
    }

    // ===== Simple String Rules =====

    #[test]
    fn test_email_validation() {
        let rules = parse_field_attributes(quote! {
            #[validate(email)]
            email: String
        });
        assert_eq!(rules.len(), 1);
        matches!(rules[0], ValidationRule::Email);
    }

    #[test]
    fn test_url_validation() {
        let rules = parse_field_attributes(quote! {
            #[validate(url)]
            website: String
        });
        assert_eq!(rules.len(), 1);
        matches!(rules[0], ValidationRule::Url);
    }

    #[test]
    fn test_non_empty_validation() {
        let rules = parse_field_attributes(quote! {
            #[validate(non_empty)]
            name: String
        });
        assert_eq!(rules.len(), 1);
        matches!(rules[0], ValidationRule::NonEmpty);
    }

    #[test]
    fn test_non_blank_validation() {
        let rules = parse_field_attributes(quote! {
            #[validate(non_blank)]
            description: String
        });
        assert_eq!(rules.len(), 1);
        matches!(rules[0], ValidationRule::NonBlank);
    }

    #[test]
    fn test_alphanumeric_validation() {
        let rules = parse_field_attributes(quote! {
            #[validate(alphanumeric)]
            username: String
        });
        assert_eq!(rules.len(), 1);
        matches!(rules[0], ValidationRule::Alphanumeric);
    }

    #[test]
    fn test_alpha_only_validation() {
        let rules = parse_field_attributes(quote! {
            #[validate(alpha_only)]
            name: String
        });
        assert_eq!(rules.len(), 1);
        matches!(rules[0], ValidationRule::AlphaOnly);
    }

    #[test]
    fn test_numeric_string_validation() {
        let rules = parse_field_attributes(quote! {
            #[validate(numeric_string)]
            code: String
        });
        assert_eq!(rules.len(), 1);
        matches!(rules[0], ValidationRule::NumericString);
    }

    #[test]
    fn test_ascii_validation() {
        let rules = parse_field_attributes(quote! {
            #[validate(ascii)]
            text: String
        });
        assert_eq!(rules.len(), 1);
        matches!(rules[0], ValidationRule::Ascii);
    }

    #[test]
    fn test_no_whitespace_validation() {
        let rules = parse_field_attributes(quote! {
            #[validate(no_whitespace)]
            token: String
        });
        assert_eq!(rules.len(), 1);
        matches!(rules[0], ValidationRule::NoWhitespace);
    }

    // ===== Parameterized String Rules =====

    #[test]
    fn test_min_len_validation() {
        let rules = parse_field_attributes(quote! {
            #[validate(min_len = 3)]
            username: String
        });
        assert_eq!(rules.len(), 1);
        if let ValidationRule::MinLen(val) = &rules[0] {
            assert_eq!(*val, 3);
        } else {
            panic!("Expected MinLen rule");
        }
    }

    #[test]
    fn test_max_len_validation() {
        let rules = parse_field_attributes(quote! {
            #[validate(max_len = 255)]
            email: String
        });
        assert_eq!(rules.len(), 1);
        if let ValidationRule::MaxLen(val) = &rules[0] {
            assert_eq!(*val, 255);
        } else {
            panic!("Expected MaxLen rule");
        }
    }

    #[test]
    fn test_length_validation_with_min_and_max() {
        let rules = parse_field_attributes(quote! {
            #[validate(length(min = 3, max = 50))]
            username: String
        });
        assert_eq!(rules.len(), 1);
        if let ValidationRule::Length { min, max } = &rules[0] {
            assert_eq!(*min, 3);
            assert_eq!(*max, 50);
        } else {
            panic!("Expected Length rule");
        }
    }

    #[test]
    fn test_length_validation_reversed_order() {
        let rules = parse_field_attributes(quote! {
            #[validate(length(max = 100, min = 10))]
            description: String
        });
        assert_eq!(rules.len(), 1);
        if let ValidationRule::Length { min, max } = &rules[0] {
            assert_eq!(*min, 10);
            assert_eq!(*max, 100);
        } else {
            panic!("Expected Length rule");
        }
    }

    #[test]
    fn test_starts_with_validation() {
        let rules = parse_field_attributes(quote! {
            #[validate(starts_with = "https://")]
            url: String
        });
        assert_eq!(rules.len(), 1);
        if let ValidationRule::StartsWith(val) = &rules[0] {
            assert_eq!(val, "https://");
        } else {
            panic!("Expected StartsWith rule");
        }
    }

    #[test]
    fn test_ends_with_validation() {
        let rules = parse_field_attributes(quote! {
            #[validate(ends_with = ".com")]
            domain: String
        });
        assert_eq!(rules.len(), 1);
        if let ValidationRule::EndsWith(val) = &rules[0] {
            assert_eq!(val, ".com");
        } else {
            panic!("Expected EndsWith rule");
        }
    }

    #[test]
    fn test_contains_validation() {
        let rules = parse_field_attributes(quote! {
            #[validate(contains = "@")]
            email: String
        });
        assert_eq!(rules.len(), 1);
        if let ValidationRule::Contains(val) = &rules[0] {
            assert_eq!(val, "@");
        } else {
            panic!("Expected Contains rule");
        }
    }

    #[test]
    fn test_matches_regex_validation() {
        let rules = parse_field_attributes(quote! {
            #[validate(matches_regex = r"^[A-Z][a-z]+$")]
            name: String
        });
        assert_eq!(rules.len(), 1);
        if let ValidationRule::MatchesRegex(val) = &rules[0] {
            assert_eq!(val, r"^[A-Z][a-z]+$");
        } else {
            panic!("Expected MatchesRegex rule");
        }
    }

    // ===== Simple Numeric Rules =====

    #[test]
    fn test_positive_validation() {
        let rules = parse_field_attributes(quote! {
            #[validate(positive)]
            amount: i32
        });
        assert_eq!(rules.len(), 1);
        matches!(rules[0], ValidationRule::Positive);
    }

    #[test]
    fn test_negative_validation() {
        let rules = parse_field_attributes(quote! {
            #[validate(negative)]
            debt: i32
        });
        assert_eq!(rules.len(), 1);
        matches!(rules[0], ValidationRule::Negative);
    }

    #[test]
    fn test_non_zero_validation() {
        let rules = parse_field_attributes(quote! {
            #[validate(non_zero)]
            divisor: i32
        });
        assert_eq!(rules.len(), 1);
        matches!(rules[0], ValidationRule::NonZero);
    }

    #[test]
    fn test_finite_validation() {
        let rules = parse_field_attributes(quote! {
            #[validate(finite)]
            value: f64
        });
        assert_eq!(rules.len(), 1);
        matches!(rules[0], ValidationRule::Finite);
    }

    // ===== Parameterized Numeric Rules =====

    #[test]
    fn test_min_integer_validation() {
        let rules = parse_field_attributes(quote! {
            #[validate(min = 18)]
            age: u8
        });
        assert_eq!(rules.len(), 1);
        if let ValidationRule::Min(val) = &rules[0] {
            assert_eq!(val, "18");
        } else {
            panic!("Expected Min rule");
        }
    }

    #[test]
    fn test_max_integer_validation() {
        let rules = parse_field_attributes(quote! {
            #[validate(max = 120)]
            age: u8
        });
        assert_eq!(rules.len(), 1);
        if let ValidationRule::Max(val) = &rules[0] {
            assert_eq!(val, "120");
        } else {
            panic!("Expected Max rule");
        }
    }

    #[test]
    fn test_min_float_validation() {
        let rules = parse_field_attributes(quote! {
            #[validate(min = 0.0)]
            price: f64
        });
        assert_eq!(rules.len(), 1);
        if let ValidationRule::Min(val) = &rules[0] {
            assert_eq!(val, "0.0");
        } else {
            panic!("Expected Min rule");
        }
    }

    #[test]
    fn test_max_float_validation() {
        let rules = parse_field_attributes(quote! {
            #[validate(max = 999.99)]
            price: f64
        });
        assert_eq!(rules.len(), 1);
        if let ValidationRule::Max(val) = &rules[0] {
            assert_eq!(val, "999.99");
        } else {
            panic!("Expected Max rule");
        }
    }

    #[test]
    fn test_range_integer_validation() {
        let rules = parse_field_attributes(quote! {
            #[validate(range(min = 18, max = 120))]
            age: u8
        });
        assert_eq!(rules.len(), 1);
        if let ValidationRule::Range { min, max } = &rules[0] {
            assert_eq!(min, "18");
            assert_eq!(max, "120");
        } else {
            panic!("Expected Range rule");
        }
    }

    #[test]
    fn test_range_float_validation() {
        let rules = parse_field_attributes(quote! {
            #[validate(range(min = 0.0, max = 100.0))]
            percentage: f64
        });
        assert_eq!(rules.len(), 1);
        if let ValidationRule::Range { min, max } = &rules[0] {
            assert_eq!(min, "0.0");
            assert_eq!(max, "100.0");
        } else {
            panic!("Expected Range rule");
        }
    }

    #[test]
    fn test_range_reversed_order() {
        let rules = parse_field_attributes(quote! {
            #[validate(range(max = 100, min = 0))]
            score: i32
        });
        assert_eq!(rules.len(), 1);
        if let ValidationRule::Range { min, max } = &rules[0] {
            assert_eq!(min, "0");
            assert_eq!(max, "100");
        } else {
            panic!("Expected Range rule");
        }
    }

    #[test]
    fn test_multiple_of_integer_validation() {
        let rules = parse_field_attributes(quote! {
            #[validate(multiple_of = 5)]
            quantity: i32
        });
        assert_eq!(rules.len(), 1);
        if let ValidationRule::MultipleOf(val) = &rules[0] {
            assert_eq!(val, "5");
        } else {
            panic!("Expected MultipleOf rule");
        }
    }

    #[test]
    fn test_multiple_of_float_validation() {
        let rules = parse_field_attributes(quote! {
            #[validate(multiple_of = 0.25)]
            price: f64
        });
        assert_eq!(rules.len(), 1);
        if let ValidationRule::MultipleOf(val) = &rules[0] {
            assert_eq!(val, "0.25");
        } else {
            panic!("Expected MultipleOf rule");
        }
    }

    // ===== Multiple Validation Rules =====

    #[test]
    fn test_multiple_rules_on_same_field() {
        let rules = parse_field_attributes(quote! {
            #[validate(email)]
            #[validate(max_len = 255)]
            #[validate(non_empty)]
            email: String
        });
        assert_eq!(rules.len(), 3);
        matches!(rules[0], ValidationRule::Email);
        matches!(rules[1], ValidationRule::MaxLen(_));
        matches!(rules[2], ValidationRule::NonEmpty);
    }

    #[test]
    fn test_multiple_rules_in_single_attribute() {
        let rules = parse_field_attributes(quote! {
            #[validate(alphanumeric, min_len = 3, max_len = 50)]
            username: String
        });
        assert_eq!(rules.len(), 3);
        matches!(rules[0], ValidationRule::Alphanumeric);
        matches!(rules[1], ValidationRule::MinLen(3));
        matches!(rules[2], ValidationRule::MaxLen(50));
    }

    #[test]
    fn test_complex_field_with_many_rules() {
        let rules = parse_field_attributes(quote! {
            #[validate(non_empty)]
            #[validate(min_len = 8)]
            #[validate(max_len = 128)]
            #[validate(matches_regex = r"[A-Z]")]
            #[validate(matches_regex = r"[0-9]")]
            password: String
        });
        assert_eq!(rules.len(), 5);
        matches!(rules[0], ValidationRule::NonEmpty);
        matches!(rules[1], ValidationRule::MinLen(8));
        matches!(rules[2], ValidationRule::MaxLen(128));
        matches!(rules[3], ValidationRule::MatchesRegex(_));
        matches!(rules[4], ValidationRule::MatchesRegex(_));
    }

    // ===== Custom/Unknown Rules =====

    #[test]
    fn test_custom_unknown_rule() {
        let rules = parse_field_attributes(quote! {
            #[validate(custom_rule)]
            field: String
        });
        assert_eq!(rules.len(), 1);
        if let ValidationRule::Custom(name) = &rules[0] {
            assert_eq!(name, "custom_rule");
        } else {
            panic!("Expected Custom rule");
        }
    }

    #[test]
    fn test_multiple_custom_rules() {
        let rules = parse_field_attributes(quote! {
            #[validate(credit_card)]
            #[validate(luhn_check)]
            card_number: String
        });
        assert_eq!(rules.len(), 2);
        if let ValidationRule::Custom(name1) = &rules[0] {
            assert_eq!(name1, "credit_card");
        }
        if let ValidationRule::Custom(name2) = &rules[1] {
            assert_eq!(name2, "luhn_check");
        }
    }

    #[test]
    fn test_mixed_known_and_custom_rules() {
        let rules = parse_field_attributes(quote! {
            #[validate(email)]
            #[validate(disposable_email_check)]
            email: String
        });
        assert_eq!(rules.len(), 2);
        matches!(rules[0], ValidationRule::Email);
        if let ValidationRule::Custom(name) = &rules[1] {
            assert_eq!(name, "disposable_email_check");
        }
    }

    // ===== Edge Cases =====

    #[test]
    fn test_no_validation_attributes() {
        let rules = parse_field_attributes(quote! {
            name: String
        });
        assert_eq!(rules.len(), 0);
    }

    #[test]
    fn test_non_validate_attributes_ignored() {
        let rules = parse_field_attributes(quote! {
            #[serde(rename = "userName")]
            #[validate(email)]
            #[doc = "User email address"]
            email: String
        });
        assert_eq!(rules.len(), 1);
        matches!(rules[0], ValidationRule::Email);
    }

    #[test]
    fn test_large_numeric_values() {
        let rules = parse_field_attributes(quote! {
            #[validate(max = 9999999999)]
            id: i64
        });
        assert_eq!(rules.len(), 1);
        if let ValidationRule::Max(val) = &rules[0] {
            assert_eq!(val, "9999999999");
        } else {
            panic!("Expected Max rule");
        }
    }

    #[test]
    fn test_negative_numeric_values() {
        let rules = parse_field_attributes(quote! {
            #[validate(min = -100)]
            temperature: i32
        });
        assert_eq!(rules.len(), 1);
        if let ValidationRule::Min(val) = &rules[0] {
            assert_eq!(val, "-100");
        } else {
            panic!("Expected Min rule");
        }
    }

    #[test]
    fn test_zero_values() {
        let rules = parse_field_attributes(quote! {
            #[validate(min_len = 0)]
            optional_text: String
        });
        assert_eq!(rules.len(), 1);
        if let ValidationRule::MinLen(val) = &rules[0] {
            assert_eq!(*val, 0);
        } else {
            panic!("Expected MinLen rule");
        }
    }

    #[test]
    fn test_empty_string_values() {
        let rules = parse_field_attributes(quote! {
            #[validate(starts_with = "")]
            text: String
        });
        assert_eq!(rules.len(), 1);
        if let ValidationRule::StartsWith(val) = &rules[0] {
            assert_eq!(val, "");
        } else {
            panic!("Expected StartsWith rule");
        }
    }

    #[test]
    fn test_special_characters_in_strings() {
        let rules = parse_field_attributes(quote! {
            #[validate(contains = "!@#$%")]
            special: String
        });
        assert_eq!(rules.len(), 1);
        if let ValidationRule::Contains(val) = &rules[0] {
            assert_eq!(val, "!@#$%");
        } else {
            panic!("Expected Contains rule");
        }
    }

    #[test]
    fn test_unicode_in_string_values() {
        let rules = parse_field_attributes(quote! {
            #[validate(contains = "你好")]
            chinese: String
        });
        assert_eq!(rules.len(), 1);
        if let ValidationRule::Contains(val) = &rules[0] {
            assert_eq!(val, "你好");
        } else {
            panic!("Expected Contains rule");
        }
    }

    #[test]
    fn test_regex_with_escape_sequences() {
        let rules = parse_field_attributes(quote! {
            #[validate(matches_regex = r"\d{3}-\d{2}-\d{4}")]
            ssn: String
        });
        assert_eq!(rules.len(), 1);
        if let ValidationRule::MatchesRegex(val) = &rules[0] {
            assert_eq!(val, r"\d{3}-\d{2}-\d{4}");
        } else {
            panic!("Expected MatchesRegex rule");
        }
    }

    #[test]
    fn test_very_long_max_len() {
        let rules = parse_field_attributes(quote! {
            #[validate(max_len = 1048576)]
            large_text: String
        });
        assert_eq!(rules.len(), 1);
        if let ValidationRule::MaxLen(val) = &rules[0] {
            assert_eq!(*val, 1048576);
        } else {
            panic!("Expected MaxLen rule");
        }
    }

    #[test]
    fn test_scientific_notation_floats() {
        let rules = parse_field_attributes(quote! {
            #[validate(max = 1e10)]
            big_number: f64
        });
        assert_eq!(rules.len(), 1);
        if let ValidationRule::Max(val) = &rules[0] {
            assert_eq!(val, "1e10");
        } else {
            panic!("Expected Max rule");
        }
    }

    // ===== Real-World Scenarios =====

    #[test]
    fn test_email_field_realistic() {
        let rules = parse_field_attributes(quote! {
            #[validate(email)]
            #[validate(max_len = 255)]
            #[validate(non_empty)]
            email: String
        });
        assert_eq!(rules.len(), 3);
    }

    #[test]
    fn test_username_field_realistic() {
        let rules = parse_field_attributes(quote! {
            #[validate(length(min = 3, max = 50))]
            #[validate(alphanumeric)]
            #[validate(non_empty)]
            username: String
        });
        assert_eq!(rules.len(), 3);
    }

    #[test]
    fn test_age_field_realistic() {
        let rules = parse_field_attributes(quote! {
            #[validate(range(min = 18, max = 120))]
            #[validate(positive)]
            age: u8
        });
        assert_eq!(rules.len(), 2);
    }

    #[test]
    fn test_password_field_realistic() {
        let rules = parse_field_attributes(quote! {
            #[validate(length(min = 8, max = 128))]
            #[validate(matches_regex = r"[A-Z]")]
            #[validate(matches_regex = r"[a-z]")]
            #[validate(matches_regex = r"[0-9]")]
            #[validate(matches_regex = r"[!@#$%^&*]")]
            password: String
        });
        assert_eq!(rules.len(), 5);
    }

    #[test]
    fn test_url_field_realistic() {
        let rules = parse_field_attributes(quote! {
            #[validate(url)]
            #[validate(starts_with = "https://")]
            #[validate(max_len = 2048)]
            website: String
        });
        assert_eq!(rules.len(), 3);
    }

    #[test]
    fn test_price_field_realistic() {
        let rules = parse_field_attributes(quote! {
            #[validate(positive)]
            #[validate(finite)]
            #[validate(range(min = 0.01, max = 999999.99))]
            price: f64
        });
        assert_eq!(rules.len(), 3);
    }

    #[test]
    fn test_quantity_field_realistic() {
        let rules = parse_field_attributes(quote! {
            #[validate(positive)]
            #[validate(multiple_of = 1)]
            #[validate(range(min = 1, max = 1000))]
            quantity: i32
        });
        assert_eq!(rules.len(), 3);
    }

    #[test]
    fn test_phone_number_realistic() {
        let rules = parse_field_attributes(quote! {
            #[validate(matches_regex = r"^\+?[1-9]\d{1,14}$")]
            #[validate(non_empty)]
            phone: String
        });
        assert_eq!(rules.len(), 2);
    }

    #[test]
    fn test_postal_code_realistic() {
        let rules = parse_field_attributes(quote! {
            #[validate(matches_regex = r"^\d{5}(-\d{4})?$")]
            #[validate(non_empty)]
            postal_code: String
        });
        assert_eq!(rules.len(), 2);
    }

    #[test]
    fn test_slug_field_realistic() {
        let rules = parse_field_attributes(quote! {
            #[validate(matches_regex = r"^[a-z0-9-]+$")]
            #[validate(length(min = 3, max = 100))]
            #[validate(no_whitespace)]
            slug: String
        });
        assert_eq!(rules.len(), 3);
    }
}

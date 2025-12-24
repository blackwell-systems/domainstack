# Schema Derivation Implementation Plan

**Technical design for auto-deriving OpenAPI schemas from validation rules**

This document outlines the implementation approach for the `#[derive(ToSchema)]` macro that automatically generates OpenAPI 3.0 schemas from validation attributes.

## Overview

### Goal
Allow users to write:
```rust
#[derive(Validate, ToSchema)]
struct User {
    #[validate(email)]
    #[validate(max_len = 255)]
    email: String,
}
```

And automatically generate a `ToSchema` implementation that includes the validation constraints (format: email, maxLength: 255).

### Architecture

```
┌─────────────────────────────────────┐
│   User Code                         │
│   #[derive(Validate, ToSchema)]     │
│   struct User { ... }                │
└──────────────┬──────────────────────┘
               │
               ▼
┌─────────────────────────────────────┐
│   domainstack-derive                │
│   - parse_validate_attrs()          │
│   - map_to_schema_constraints()     │
│   - generate_to_schema_impl()       │
└──────────────┬──────────────────────┘
               │
               ▼
┌─────────────────────────────────────┐
│   Generated Code                    │
│   impl ToSchema for User {          │
│     fn schema() -> Schema {         │
│       Schema::object()              │
│         .property("email",          │
│           Schema::string()          │
│             .format("email")        │
│             .max_length(255))       │
│     }                               │
│   }                                 │
└─────────────────────────────────────┘
```

## Implementation Phases

### Phase 1: Core Infrastructure (Foundation)

**Goal:** Set up the derive macro skeleton and attribute parsing.

**Tasks:**
1. Add `ToSchema` derive macro to `domainstack-derive/src/lib.rs`
2. Create `domainstack-derive/src/schema.rs` module
3. Implement attribute parsing for `#[validate(...)]`
4. Implement attribute parsing for `#[schema(...)]`

**Code Structure:**
```rust
// domainstack-derive/src/lib.rs
#[proc_macro_derive(ToSchema, attributes(schema, validate))]
pub fn derive_to_schema(input: TokenStream) -> TokenStream {
    schema::derive_to_schema_impl(input)
}

// domainstack-derive/src/schema.rs
pub fn derive_to_schema_impl(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_to_schema(&ast)
}

fn impl_to_schema(ast: &DeriveInput) -> TokenStream {
    // Parse struct fields
    // Extract validation attributes
    // Extract schema attributes
    // Generate ToSchema impl
}
```

**Attribute Parsing:**
```rust
struct FieldSchema {
    field_name: Ident,
    field_type: Type,
    validate_attrs: Vec<ValidationRule>,
    schema_hints: SchemaHints,
}

struct ValidationRule {
    kind: ValidationRuleKind,  // email, min_len, max_len, range, etc.
    params: HashMap<String, Lit>,  // min = 3, max = 20, etc.
}

enum ValidationRuleKind {
    Email,
    Url,
    MinLen,
    MaxLen,
    Length,
    Range,
    MinItems,
    MaxItems,
    Unique,
    NonEmptyItems,
    // ... all 37 rules (includes 5 date/time rules when chrono feature enabled)
}

struct SchemaHints {
    description: Option<String>,
    example: Option<Lit>,
    deprecated: bool,
    read_only: bool,
    write_only: bool,
    pattern: Option<String>,
    // ... other OpenAPI metadata
}
```

### Phase 2: Rule → Schema Mapping

**Goal:** Implement mappings from validation rules to OpenAPI constraints.

**Core Mapping Function:**
```rust
fn map_validation_to_schema_constraints(
    field_type: &Type,
    rules: &[ValidationRule],
) -> SchemaConstraints {
    let mut constraints = SchemaConstraints::new();

    for rule in rules {
        match rule.kind {
            ValidationRuleKind::Email => {
                constraints.format = Some("email");
            }
            ValidationRuleKind::MinLen => {
                let min = extract_param(rule, "min");
                constraints.min_length = Some(min);
            }
            ValidationRuleKind::Range => {
                let min = extract_param(rule, "min");
                let max = extract_param(rule, "max");
                constraints.minimum = Some(min);
                constraints.maximum = Some(max);
            }
            // ... all other rules
        }
    }

    constraints
}

struct SchemaConstraints {
    // String constraints
    format: Option<&'static str>,
    min_length: Option<usize>,
    max_length: Option<usize>,
    pattern: Option<String>,

    // Numeric constraints
    minimum: Option<i64>,
    maximum: Option<i64>,
    exclusive_minimum: bool,
    exclusive_maximum: bool,
    multiple_of: Option<i64>,

    // Collection constraints
    min_items: Option<usize>,
    max_items: Option<usize>,
    unique_items: bool,

    // Choice constraints
    enum_values: Option<Vec<String>>,
    const_value: Option<String>,
}
```

**Mapping Table Implementation:**
```rust
fn map_string_rules(rule: &ValidationRule) -> Option<SchemaConstraint> {
    match rule.kind {
        ValidationRuleKind::Email => Some(SchemaConstraint::Format("email")),
        ValidationRuleKind::Url => Some(SchemaConstraint::Format("uri")),
        ValidationRuleKind::MinLen => {
            let min = extract_usize_param(rule, "min")?;
            Some(SchemaConstraint::MinLength(min))
        }
        ValidationRuleKind::MaxLen => {
            let max = extract_usize_param(rule, "max")?;
            Some(SchemaConstraint::MaxLength(max))
        }
        ValidationRuleKind::MatchesRegex => {
            let pattern = extract_string_param(rule, "pattern")?;
            Some(SchemaConstraint::Pattern(pattern))
        }
        ValidationRuleKind::Ascii => {
            Some(SchemaConstraint::Pattern("^[\\x00-\\x7F]*$"))
        }
        ValidationRuleKind::Alphanumeric => {
            Some(SchemaConstraint::Pattern("^[a-zA-Z0-9]*$"))
        }
        // ... all other string rules
        _ => None,
    }
}

fn map_numeric_rules(rule: &ValidationRule) -> Option<SchemaConstraint> {
    match rule.kind {
        ValidationRuleKind::Min => {
            let min = extract_number_param(rule, "min")?;
            Some(SchemaConstraint::Minimum(min, false))
        }
        ValidationRuleKind::Max => {
            let max = extract_number_param(rule, "max")?;
            Some(SchemaConstraint::Maximum(max, false))
        }
        ValidationRuleKind::Range => {
            let min = extract_number_param(rule, "min")?;
            let max = extract_number_param(rule, "max")?;
            Some(SchemaConstraint::Range(min, max))
        }
        ValidationRuleKind::Positive => {
            Some(SchemaConstraint::Minimum(0, true)) // exclusive
        }
        ValidationRuleKind::Negative => {
            Some(SchemaConstraint::Maximum(0, true)) // exclusive
        }
        ValidationRuleKind::MultipleOf => {
            let divisor = extract_number_param(rule, "divisor")?;
            Some(SchemaConstraint::MultipleOf(divisor))
        }
        // ... all other numeric rules
        _ => None,
    }
}
```

### Phase 3: Code Generation

**Goal:** Generate the actual `ToSchema` implementation.

**Code Generator:**
```rust
fn generate_to_schema_impl(
    struct_name: &Ident,
    fields: &[FieldSchema],
    struct_schema_hints: &SchemaHints,
) -> TokenStream {
    let schema_name = struct_name.to_string();
    let properties = generate_properties(fields);
    let required = generate_required_fields(fields);

    quote! {
        impl ::domainstack_schema::ToSchema for #struct_name {
            fn schema_name() -> &'static str {
                #schema_name
            }

            fn schema() -> ::domainstack_schema::Schema {
                ::domainstack_schema::Schema::object()
                    #properties
                    #required
            }
        }
    }
}

fn generate_properties(fields: &[FieldSchema]) -> TokenStream {
    let mut tokens = TokenStream::new();

    for field in fields {
        let field_name = field.field_name.to_string();
        let property_schema = generate_property_schema(field);

        tokens.extend(quote! {
            .property(#field_name, #property_schema)
        });
    }

    tokens
}

fn generate_property_schema(field: &FieldSchema) -> TokenStream {
    let base_schema = match &field.field_type {
        Type::Path(path) => {
            let type_name = path.path.segments.last().unwrap().ident.to_string();
            match type_name.as_str() {
                "String" | "str" => quote! { ::domainstack_schema::Schema::string() },
                "u8" | "u16" | "u32" | "u64" | "i8" | "i16" | "i32" | "i64" => {
                    quote! { ::domainstack_schema::Schema::integer() }
                }
                "f32" | "f64" => quote! { ::domainstack_schema::Schema::number() },
                "bool" => quote! { ::domainstack_schema::Schema::boolean() },
                "Vec" => {
                    // Handle collections
                    generate_array_schema(field)
                }
                "Option" => {
                    // Handle optional fields
                    generate_optional_schema(field)
                }
                _ => {
                    // Assume it's a nested type that implements ToSchema
                    quote! { <#type_name as ::domainstack_schema::ToSchema>::schema() }
                }
            }
        }
        _ => quote! { ::domainstack_schema::Schema::object() },
    };

    // Apply validation constraints
    apply_constraints(base_schema, &field.validate_attrs, &field.schema_hints)
}

fn apply_constraints(
    base: TokenStream,
    rules: &[ValidationRule],
    hints: &SchemaHints,
) -> TokenStream {
    let mut schema = base;

    // Apply validation rule constraints
    for rule in rules {
        schema = match rule.kind {
            ValidationRuleKind::Email => {
                quote! { #schema.format("email") }
            }
            ValidationRuleKind::MinLen => {
                let min = extract_param(rule, "min");
                quote! { #schema.min_length(#min) }
            }
            ValidationRuleKind::MaxLen => {
                let max = extract_param(rule, "max");
                quote! { #schema.max_length(#max) }
            }
            ValidationRuleKind::Range => {
                let min = extract_param(rule, "min");
                let max = extract_param(rule, "max");
                quote! { #schema.minimum(#min).maximum(#max) }
            }
            // ... all other rules
            _ => schema,
        };
    }

    // Apply schema hints (descriptions, examples, etc.)
    if let Some(desc) = &hints.description {
        schema = quote! { #schema.description(#desc) };
    }
    if let Some(example) = &hints.example {
        schema = quote! { #schema.example(#example) };
    }

    schema
}
```

### Phase 4: Nested Types & Collections

**Goal:** Handle nested validation and array types.

**Nested Type Detection:**
```rust
fn is_nested_type(field: &FieldSchema) -> bool {
    field.validate_attrs.iter().any(|attr| {
        matches!(attr.kind, ValidationRuleKind::Nested)
    })
}

fn generate_nested_schema(field: &FieldSchema) -> TokenStream {
    let type_name = extract_type_name(&field.field_type);
    quote! {
        <#type_name as ::domainstack_schema::ToSchema>::schema()
    }
}
```

**Collection Handling:**
```rust
fn generate_array_schema(field: &FieldSchema) -> TokenStream {
    let item_type = extract_vec_inner_type(&field.field_type);
    let has_nested = field.validate_attrs.iter().any(|attr| {
        matches!(attr.kind, ValidationRuleKind::EachNested)
    });

    let items_schema = if has_nested {
        quote! { <#item_type as ::domainstack_schema::ToSchema>::schema() }
    } else {
        generate_primitive_schema(&item_type)
    };

    let mut array_schema = quote! {
        ::domainstack_schema::Schema::array()
            .items(#items_schema)
    };

    // Apply array-specific validation rules
    for rule in &field.validate_attrs {
        array_schema = match rule.kind {
            ValidationRuleKind::MinItems => {
                let min = extract_param(rule, "min");
                quote! { #array_schema.min_items(#min) }
            }
            ValidationRuleKind::MaxItems => {
                let max = extract_param(rule, "max");
                quote! { #array_schema.max_items(#max) }
            }
            ValidationRuleKind::Unique => {
                quote! { #array_schema.unique_items(true) }
            }
            _ => array_schema,
        };
    }

    array_schema
}
```

**Optional Field Handling:**
```rust
fn is_optional(field_type: &Type) -> bool {
    match field_type {
        Type::Path(path) => {
            path.path.segments.last()
                .map(|seg| seg.ident == "Option")
                .unwrap_or(false)
        }
        _ => false,
    }
}

fn generate_required_fields(fields: &[FieldSchema]) -> TokenStream {
    let required_fields: Vec<_> = fields
        .iter()
        .filter(|f| !is_optional(&f.field_type))
        .map(|f| f.field_name.to_string())
        .collect();

    if required_fields.is_empty() {
        return quote! {};
    }

    quote! {
        .required(&[#(#required_fields),*])
    }
}
```

### Phase 5: Schema Hints Support

**Goal:** Parse and apply `#[schema(...)]` attributes.

**Attribute Parsing:**
```rust
fn parse_schema_hints(attrs: &[Attribute]) -> SchemaHints {
    let mut hints = SchemaHints::default();

    for attr in attrs {
        if !attr.path.is_ident("schema") {
            continue;
        }

        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("description") {
                let value: LitStr = meta.value()?.parse()?;
                hints.description = Some(value.value());
            } else if meta.path.is_ident("example") {
                let value: Lit = meta.value()?.parse()?;
                hints.example = Some(value);
            } else if meta.path.is_ident("pattern") {
                let value: LitStr = meta.value()?.parse()?;
                hints.pattern = Some(value.value());
            } else if meta.path.is_ident("deprecated") {
                hints.deprecated = true;
            } else if meta.path.is_ident("read_only") {
                hints.read_only = true;
            } else if meta.path.is_ident("write_only") {
                hints.write_only = true;
            }
            // ... other hints
            Ok(())
        }).ok();
    }

    hints
}
```

**Applying Hints:**
```rust
fn apply_schema_hints(schema: TokenStream, hints: &SchemaHints) -> TokenStream {
    let mut result = schema;

    if let Some(desc) = &hints.description {
        result = quote! { #result.description(#desc) };
    }

    if let Some(example) = &hints.example {
        result = quote! { #result.example(::serde_json::json!(#example)) };
    }

    if let Some(pattern) = &hints.pattern {
        result = quote! { #result.pattern(#pattern) };
    }

    if hints.deprecated {
        result = quote! { #result.deprecated(true) };
    }

    if hints.read_only {
        result = quote! { #result.read_only(true) };
    }

    if hints.write_only {
        result = quote! { #result.write_only(true) };
    }

    result
}
```

## Testing Strategy

### Unit Tests

Test each mapping function individually:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_rule_mapping() {
        let rule = ValidationRule {
            kind: ValidationRuleKind::Email,
            params: HashMap::new(),
        };

        let constraint = map_string_rules(&rule).unwrap();
        assert_eq!(constraint, SchemaConstraint::Format("email"));
    }

    #[test]
    fn test_range_rule_mapping() {
        let mut params = HashMap::new();
        params.insert("min".to_string(), Lit::Int(18));
        params.insert("max".to_string(), Lit::Int(120));

        let rule = ValidationRule {
            kind: ValidationRuleKind::Range,
            params,
        };

        let constraint = map_numeric_rules(&rule).unwrap();
        assert_eq!(constraint, SchemaConstraint::Range(18, 120));
    }
}
```

### Integration Tests

Test complete struct derivation:

```rust
#[test]
fn test_simple_struct_derivation() {
    let input = quote! {
        #[derive(Validate, ToSchema)]
        struct User {
            #[validate(email)]
            #[validate(max_len = 255)]
            email: String,

            #[validate(range(min = 18, max = 120))]
            age: u8,
        }
    };

    let output = derive_to_schema_impl(input.into());

    // Assert that generated code compiles
    // Assert that schema has correct constraints
}
```

### Compilation Tests

Use `trybuild` to test that generated code compiles:

```rust
#[test]
fn test_derive_compiles() {
    let t = trybuild::TestCases::new();
    t.pass("tests/schema/*.rs");
}
```

## Dependencies

### New Crate Dependencies

```toml
# domainstack-derive/Cargo.toml
[dependencies]
syn = { version = "2.0", features = ["full", "extra-traits"] }
quote = "1.0"
proc-macro2 = "1.0"

# For parsing nested attributes
darling = "0.20"  # Simplifies attribute parsing
```

### Feature Flags

```toml
# domainstack/Cargo.toml
[features]
schema-derive = ["domainstack-derive/schema", "domainstack-schema"]
```

## Error Handling

### Compile-Time Errors

Provide helpful error messages:

```rust
fn validate_schema_derivation(ast: &DeriveInput) -> Result<(), Error> {
    // Check that struct has named fields
    let fields = match &ast.data {
        Data::Struct(DataStruct { fields: Fields::Named(fields), .. }) => fields,
        _ => {
            return Err(Error::new_spanned(
                ast,
                "ToSchema can only be derived for structs with named fields"
            ));
        }
    };

    // Check for conflicting attributes
    for field in &fields.named {
        let validate_attrs = extract_validate_attrs(&field.attrs);
        if has_conflicting_rules(&validate_attrs) {
            return Err(Error::new_spanned(
                field,
                "Conflicting validation rules detected. Use .or() to combine alternatives."
            ));
        }
    }

    Ok(())
}
```

### Runtime Warnings

For unsupported rules, emit warnings:

```rust
fn map_rule_to_constraint(rule: &ValidationRule) -> Option<SchemaConstraint> {
    match rule.kind {
        ValidationRuleKind::Custom => {
            // Can't auto-generate schema for custom validators
            eprintln!(
                "warning: Custom validator '{}' has no automatic schema mapping. \
                 Use #[schema(...)] hints to document constraints.",
                rule.function_name
            );
            None
        }
        ValidationRuleKind::Finite => {
            // Finite has no OpenAPI equivalent
            eprintln!(
                "warning: Rule 'finite()' has no OpenAPI schema equivalent. \
                 Consider adding #[schema(description = \"Must be finite\")] for documentation."
            );
            None
        }
        _ => Some(/* ... mapping ... */),
    }
}
```

## Migration Path

### Backward Compatibility

- Existing manual `ToSchema` impls continue to work
- Auto-derivation is opt-in via `#[derive(ToSchema)]`
- No breaking changes to existing APIs

### Incremental Adoption

Users can migrate gradually:

1. Keep existing manual impls for complex types
2. Use auto-derivation for new simple types
3. Migrate one type at a time during refactoring

### Version Strategy

- Release as `v1.1.0` (minor version bump)
- Mark as "experimental" initially
- Stabilize after community feedback

## Performance Considerations

### Compile-Time Performance

- Proc macros add compile time
- Mitigate by:
  - Lazy evaluation where possible
  - Caching parsed attributes
  - Minimal allocations in macro code

### Runtime Performance

- Zero runtime cost (all code generated at compile time)
- No dynamic dispatch
- Same performance as hand-written `ToSchema` impls

## Next Steps

1. **Phase 1:** Set up derive macro skeleton (1-2 days)
2. **Phase 2:** Implement rule mapping (2-3 days)
3. **Phase 3:** Code generation (2-3 days)
4. **Phase 4:** Nested types & collections (2-3 days)
5. **Phase 5:** Schema hints (1-2 days)
6. **Testing:** Comprehensive test suite (2-3 days)
7. **Documentation:** Examples and guides (1-2 days)

**Total Estimated Time:** 2-3 weeks for complete implementation

## Open Questions

1. **Conflicting rules:** How to handle `#[validate(email, matches_regex = "...")]`?
   - **Decision:** Apply both, let OpenAPI spec handle multiple constraints
   - **Alternative:** Emit warning if conflicts detected

2. **Custom validators:** Should we try to infer patterns from validator source?
   - **Decision:** No, too complex. Require manual `#[schema(...)]` hints
   - **Rationale:** Keeps macro simple and predictable

3. **Cross-field validation:** How to represent in OpenAPI?
   - **Decision:** Add to struct-level description, no automatic constraint
   - **Rationale:** OpenAPI 3.0 has limited cross-field support

4. **Conditional validation (`.when()`):** How to handle?
   - **Decision:** Document in schema description, no automatic constraint
   - **Rationale:** Conditional logic doesn't map cleanly to OpenAPI

## Conclusion

This implementation provides a practical, incremental path to eliminating DRY violations between validation rules and OpenAPI schemas. By following the phased approach, we can deliver value early (basic types first) while building toward full feature parity with manual implementations.

The key insight is that **most** validation rules map cleanly to OpenAPI constraints, and for edge cases (custom validators, conditional logic), we provide escape hatches via `#[schema(...)]` hints.

This feature will make domainstack's "single source of truth" promise even stronger and significantly improve developer experience.

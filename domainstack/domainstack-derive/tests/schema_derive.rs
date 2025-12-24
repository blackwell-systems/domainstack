// Suppress dead_code warnings for test data structures
#![allow(dead_code)]

use domainstack_derive::ToSchema;
use domainstack_schema::{OpenApiBuilder, ToSchema as ToSchemaTrait};
use serde_json;

// Test basic ToSchema derivation
#[derive(ToSchema)]
struct SimpleUser {
    #[validate(email)]
    #[validate(max_len = 255)]
    #[schema(description = "User's email address", example = "user@example.com")]
    email: String,

    #[validate(range(min = 18, max = 120))]
    #[schema(description = "User's age")]
    age: u8,
}

#[test]
fn test_simple_schema_derivation() {
    let schema = SimpleUser::schema();

    // Schema should be an object
    let json = serde_json::to_value(&schema).unwrap();
    eprintln!(
        "SimpleUser schema: {}",
        serde_json::to_string_pretty(&json).unwrap()
    );
    assert_eq!(json["type"], "object");

    // Should have email and age properties
    assert!(json["properties"]["email"].is_object());
    assert!(json["properties"]["age"].is_object());

    // Email should have format and maxLength
    assert_eq!(json["properties"]["email"]["type"], "string");
    assert_eq!(json["properties"]["email"]["format"], "email");
    assert_eq!(
        json["properties"]["email"]["maxLength"].as_f64(),
        Some(255.0)
    );

    // Age should have minimum and maximum
    assert_eq!(json["properties"]["age"]["type"], "integer");
    assert_eq!(json["properties"]["age"]["minimum"].as_f64(), Some(18.0));
    assert_eq!(json["properties"]["age"]["maximum"].as_f64(), Some(120.0));

    // Should have required fields
    assert_eq!(json["required"], serde_json::json!(["email", "age"]));
}

// Test optional fields
#[derive(ToSchema)]
struct UserWithOptional {
    #[validate(email)]
    email: String,

    #[validate(min_len = 1)]
    #[validate(max_len = 100)]
    nickname: Option<String>,
}

#[test]
fn test_optional_fields() {
    let schema = UserWithOptional::schema();
    let json = serde_json::to_value(&schema).unwrap();

    // Only email should be required (nickname is Option<T>)
    assert_eq!(json["required"], serde_json::json!(["email"]));

    // Nickname property should still exist
    assert!(json["properties"]["nickname"].is_object());
    assert_eq!(
        json["properties"]["nickname"]["minLength"].as_f64(),
        Some(1.0)
    );
    assert_eq!(
        json["properties"]["nickname"]["maxLength"].as_f64(),
        Some(100.0)
    );
}

// Test nested types
#[derive(ToSchema)]
struct Address {
    #[validate(min_len = 1)]
    #[validate(max_len = 100)]
    street: String,

    #[validate(min_len = 1)]
    #[validate(max_len = 50)]
    city: String,
}

#[derive(ToSchema)]
struct UserWithAddress {
    #[validate(email)]
    email: String,

    #[validate(nested)]
    address: Address,
}

#[test]
fn test_nested_types() {
    let schema = UserWithAddress::schema();
    let json = serde_json::to_value(&schema).unwrap();

    // Address should reference the Address schema
    assert!(json["properties"]["address"].is_object());
    // Note: The actual $ref check would need the full schema context
}

// Test collections
#[derive(ToSchema)]
struct Team {
    #[validate(min_len = 1)]
    #[validate(max_len = 50)]
    name: String,

    #[validate(each_nested)]
    #[validate(min_items = 1)]
    #[validate(max_items = 10)]
    members: Vec<SimpleUser>,
}

#[test]
fn test_collections() {
    let schema = Team::schema();
    let json = serde_json::to_value(&schema).unwrap();

    // Members should be an array
    assert_eq!(json["properties"]["members"]["type"], "array");
    assert_eq!(
        json["properties"]["members"]["minItems"].as_f64(),
        Some(1.0)
    );
    assert_eq!(
        json["properties"]["members"]["maxItems"].as_f64(),
        Some(10.0)
    );
}

// Test schema hints
#[derive(ToSchema)]
#[schema(description = "Product in the catalog")]
struct Product {
    #[validate(min_len = 1)]
    #[validate(max_len = 100)]
    #[schema(description = "Product name", example = "Acme Widget")]
    name: String,

    #[validate(range(min = 0, max = 1000000))]
    #[schema(description = "Price in cents", example = 1999)]
    price: i32,
}

#[test]
fn test_schema_hints() {
    let schema = Product::schema();
    let json = serde_json::to_value(&schema).unwrap();

    // Name should have description and example
    assert_eq!(json["properties"]["name"]["description"], "Product name");
    assert_eq!(json["properties"]["name"]["example"], "Acme Widget");

    // Price should have description and example
    assert_eq!(json["properties"]["price"]["description"], "Price in cents");
    assert_eq!(json["properties"]["price"]["example"].as_i64(), Some(1999));
}

// Test OpenAPI builder integration
#[test]
fn test_openapi_builder_integration() {
    let spec = OpenApiBuilder::new("Test API", "1.0.0")
        .description("API with auto-derived schemas")
        .register::<SimpleUser>()
        .register::<Product>()
        .build();

    let json_string = spec.to_json().unwrap();
    let json: serde_json::Value = serde_json::from_str(&json_string).unwrap();

    // Should have SimpleUser schema
    assert!(json["components"]["schemas"]["SimpleUser"].is_object());

    // Should have Product schema
    assert!(json["components"]["schemas"]["Product"].is_object());
}

// Test string pattern rules
#[derive(ToSchema)]
struct PatternTest {
    #[validate(alphanumeric)]
    #[schema(description = "Alphanumeric only")]
    code: String,

    #[validate(ascii)]
    #[schema(description = "ASCII characters only")]
    text: String,
}

#[test]
fn test_pattern_rules() {
    let schema = PatternTest::schema();
    let json = serde_json::to_value(&schema).unwrap();
    eprintln!(
        "PatternTest schema: {}",
        serde_json::to_string_pretty(&json).unwrap()
    );

    // Alphanumeric should have pattern
    assert_eq!(json["properties"]["code"]["pattern"], "^[a-zA-Z0-9]*$");

    // ASCII should have pattern
    assert_eq!(json["properties"]["text"]["pattern"], "^[\\x00-\\x7F]*$");
}

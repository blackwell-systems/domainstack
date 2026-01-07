use domainstack_derive::ToJsonSchema;
use domainstack_schema::{JsonSchemaBuilder, ToJsonSchema as ToJsonSchemaTrait};

// Test basic ToJsonSchema derivation
#[derive(ToJsonSchema)]
#[allow(dead_code)]
struct SimpleUser {
    #[validate(email)]
    #[validate(max_len = 255)]
    #[schema(description = "User's email address")]
    email: String,

    #[validate(range(min = 18, max = 120))]
    #[schema(description = "User's age")]
    age: u8,
}

#[test]
fn test_simple_json_schema_derivation() {
    let schema = SimpleUser::json_schema();

    // Schema should be an object
    let json = serde_json::to_value(&schema).unwrap();
    eprintln!(
        "SimpleUser JSON schema: {}",
        serde_json::to_string_pretty(&json).unwrap()
    );
    assert_eq!(json["type"], "object");

    // Should have email and age properties
    assert!(json["properties"]["email"].is_object());
    assert!(json["properties"]["age"].is_object());

    // Email should have format and maxLength
    assert_eq!(json["properties"]["email"]["type"], "string");
    assert_eq!(json["properties"]["email"]["format"], "email");
    assert_eq!(json["properties"]["email"]["maxLength"].as_u64(), Some(255));

    // Age should have minimum and maximum
    assert_eq!(json["properties"]["age"]["type"], "integer");
    assert_eq!(json["properties"]["age"]["minimum"].as_f64(), Some(18.0));
    assert_eq!(json["properties"]["age"]["maximum"].as_f64(), Some(120.0));

    // Should have required fields
    let required = json["required"].as_array().unwrap();
    assert!(required.contains(&serde_json::json!("email")));
    assert!(required.contains(&serde_json::json!("age")));
}

#[test]
fn test_schema_name() {
    assert_eq!(SimpleUser::schema_name(), "SimpleUser");
}

// Test optional fields
#[derive(ToJsonSchema)]
#[allow(dead_code)]
struct UserWithOptional {
    #[validate(email)]
    email: String,

    #[validate(min_len = 1)]
    #[validate(max_len = 100)]
    nickname: Option<String>,
}

#[test]
fn test_optional_fields() {
    let schema = UserWithOptional::json_schema();
    let json = serde_json::to_value(&schema).unwrap();

    // Only email should be required (nickname is Option<T>)
    let required = json["required"].as_array().unwrap();
    assert!(required.contains(&serde_json::json!("email")));
    assert!(!required.contains(&serde_json::json!("nickname")));

    // Nickname property should still exist
    assert!(json["properties"]["nickname"].is_object());
    assert_eq!(
        json["properties"]["nickname"]["minLength"].as_u64(),
        Some(1)
    );
    assert_eq!(
        json["properties"]["nickname"]["maxLength"].as_u64(),
        Some(100)
    );
}

// Test nested types
#[derive(ToJsonSchema)]
#[allow(dead_code)]
struct Address {
    #[validate(min_len = 1)]
    #[validate(max_len = 100)]
    street: String,

    #[validate(min_len = 1)]
    #[validate(max_len = 50)]
    city: String,
}

#[derive(ToJsonSchema)]
#[allow(dead_code)]
struct UserWithAddress {
    #[validate(email)]
    email: String,

    #[validate(nested)]
    address: Address,
}

#[test]
fn test_nested_types() {
    let schema = UserWithAddress::json_schema();
    let json = serde_json::to_value(&schema).unwrap();
    eprintln!(
        "UserWithAddress JSON schema: {}",
        serde_json::to_string_pretty(&json).unwrap()
    );

    // Address should be an object with properties
    assert!(json["properties"]["address"].is_object());
    assert!(json["properties"]["address"]["properties"]["street"].is_object());
}

// Test collections
#[derive(ToJsonSchema)]
#[allow(dead_code)]
struct Team {
    #[validate(min_len = 1)]
    #[validate(max_len = 50)]
    name: String,

    #[validate(each(nested))]
    #[validate(min_items = 1)]
    #[validate(max_items = 10)]
    members: Vec<SimpleUser>,
}

#[test]
fn test_collections() {
    let schema = Team::json_schema();
    let json = serde_json::to_value(&schema).unwrap();
    eprintln!(
        "Team JSON schema: {}",
        serde_json::to_string_pretty(&json).unwrap()
    );

    // Members should be an array
    assert_eq!(json["properties"]["members"]["type"], "array");
    assert_eq!(json["properties"]["members"]["minItems"].as_u64(), Some(1));
    assert_eq!(json["properties"]["members"]["maxItems"].as_u64(), Some(10));
}

// Test schema hints
#[derive(ToJsonSchema)]
#[allow(dead_code)]
#[schema(description = "Product in the catalog")]
struct Product {
    #[validate(min_len = 1)]
    #[validate(max_len = 100)]
    #[schema(description = "Product name")]
    name: String,

    #[validate(range(min = 0, max = 1000000))]
    #[schema(description = "Price in cents")]
    price: i32,
}

#[test]
fn test_schema_hints() {
    let schema = Product::json_schema();
    let json = serde_json::to_value(&schema).unwrap();
    eprintln!(
        "Product JSON schema: {}",
        serde_json::to_string_pretty(&json).unwrap()
    );

    // Name should have description
    assert_eq!(json["properties"]["name"]["description"], "Product name");

    // Price should have description
    assert_eq!(json["properties"]["price"]["description"], "Price in cents");

    // Struct-level description
    assert_eq!(json["description"], "Product in the catalog");
}

// Test JSON Schema builder integration
#[test]
fn test_json_schema_builder_integration() {
    let doc = JsonSchemaBuilder::new()
        .title("Test Schema")
        .description("Schema with auto-derived types")
        .register::<SimpleUser>()
        .register::<Product>()
        .build();

    let json_string = serde_json::to_string_pretty(&doc).unwrap();
    let json: serde_json::Value = serde_json::from_str(&json_string).unwrap();

    eprintln!("Full JSON Schema doc: {}", json_string);

    // Should have the schema meta fields
    assert!(json["$schema"].as_str().unwrap().contains("2020-12"));

    // Should have title
    assert_eq!(json["title"], "Test Schema");

    // Should have definitions
    assert!(json["$defs"]["SimpleUser"].is_object());
    assert!(json["$defs"]["Product"].is_object());
}

// Test string pattern rules
#[derive(ToJsonSchema)]
#[allow(dead_code)]
struct PatternTest {
    #[validate(alphanumeric)]
    #[schema(description = "Alphanumeric only")]
    code: String,

    #[validate(ascii)]
    #[schema(description = "ASCII characters only")]
    text: String,

    #[validate(non_empty)]
    name: String,

    #[validate(non_blank)]
    description: String,
}

#[test]
fn test_pattern_rules() {
    let schema = PatternTest::json_schema();
    let json = serde_json::to_value(&schema).unwrap();
    eprintln!(
        "PatternTest JSON schema: {}",
        serde_json::to_string_pretty(&json).unwrap()
    );

    // Alphanumeric should have pattern
    assert_eq!(json["properties"]["code"]["pattern"], "^[a-zA-Z0-9]*$");

    // ASCII should have pattern
    assert_eq!(json["properties"]["text"]["pattern"], "^[\\x00-\\x7F]*$");

    // Non-empty should have minLength
    assert_eq!(json["properties"]["name"]["minLength"].as_u64(), Some(1));

    // Non-blank should have pattern
    assert!(json["properties"]["description"]["pattern"]
        .as_str()
        .unwrap()
        .contains("\\S"));
}

// Test numeric rules
#[derive(ToJsonSchema)]
#[allow(dead_code)]
struct NumericRulesTest {
    #[validate(positive)]
    positive_number: i32,

    #[validate(negative)]
    negative_number: i32,

    #[validate(range(min = 0, max = 100))]
    percentage: u8,
}

#[test]
fn test_numeric_rules() {
    let schema = NumericRulesTest::json_schema();
    let json = serde_json::to_value(&schema).unwrap();
    eprintln!(
        "NumericRulesTest JSON schema: {}",
        serde_json::to_string_pretty(&json).unwrap()
    );

    // Positive should have exclusiveMinimum of 0
    assert_eq!(
        json["properties"]["positive_number"]["exclusiveMinimum"].as_f64(),
        Some(0.0)
    );

    // Negative should have exclusiveMaximum of 0
    assert_eq!(
        json["properties"]["negative_number"]["exclusiveMaximum"].as_f64(),
        Some(0.0)
    );

    // Range should have minimum and maximum
    assert_eq!(
        json["properties"]["percentage"]["minimum"].as_f64(),
        Some(0.0)
    );
    assert_eq!(
        json["properties"]["percentage"]["maximum"].as_f64(),
        Some(100.0)
    );
}

// Test array rules
#[derive(ToJsonSchema)]
#[allow(dead_code)]
struct ArrayRulesTest {
    #[validate(min_items = 1)]
    #[validate(max_items = 5)]
    tags: Vec<String>,

    #[validate(unique)]
    unique_ids: Vec<i32>,
}

#[test]
fn test_array_rules() {
    let schema = ArrayRulesTest::json_schema();
    let json = serde_json::to_value(&schema).unwrap();
    eprintln!(
        "ArrayRulesTest JSON schema: {}",
        serde_json::to_string_pretty(&json).unwrap()
    );

    // tags should have minItems and maxItems
    assert_eq!(json["properties"]["tags"]["minItems"].as_u64(), Some(1));
    assert_eq!(json["properties"]["tags"]["maxItems"].as_u64(), Some(5));

    // unique_ids should have uniqueItems
    assert_eq!(json["properties"]["unique_ids"]["uniqueItems"], true);
}

// Test format rules
#[derive(ToJsonSchema)]
#[allow(dead_code)]
struct FormatRulesTest {
    #[validate(email)]
    email_field: String,

    #[validate(url)]
    url_field: String,
}

#[test]
fn test_format_rules() {
    let schema = FormatRulesTest::json_schema();
    let json = serde_json::to_value(&schema).unwrap();

    // email should have format: email
    assert_eq!(json["properties"]["email_field"]["format"], "email");

    // url should have format: uri
    assert_eq!(json["properties"]["url_field"]["format"], "uri");
}

// Test length rules
#[derive(ToJsonSchema)]
#[allow(dead_code)]
struct LengthRulesTest {
    #[validate(length(min = 3, max = 50))]
    username: String,

    #[validate(min_len = 8)]
    password: String,

    #[validate(max_len = 1000)]
    bio: String,
}

#[test]
fn test_length_rules() {
    let schema = LengthRulesTest::json_schema();
    let json = serde_json::to_value(&schema).unwrap();

    // username should have minLength and maxLength
    assert_eq!(
        json["properties"]["username"]["minLength"].as_u64(),
        Some(3)
    );
    assert_eq!(
        json["properties"]["username"]["maxLength"].as_u64(),
        Some(50)
    );

    // password should have minLength
    assert_eq!(
        json["properties"]["password"]["minLength"].as_u64(),
        Some(8)
    );

    // bio should have maxLength
    assert_eq!(json["properties"]["bio"]["maxLength"].as_u64(), Some(1000));
}

// Test regex pattern
#[derive(ToJsonSchema)]
#[allow(dead_code)]
struct RegexTest {
    #[validate(matches_regex = "^[A-Z]{2}\\d{4}$")]
    code: String,
}

#[test]
fn test_regex_pattern() {
    let schema = RegexTest::json_schema();
    let json = serde_json::to_value(&schema).unwrap();

    assert_eq!(json["properties"]["code"]["pattern"], "^[A-Z]{2}\\d{4}$");
}

// Test contains/starts_with/ends_with
#[derive(ToJsonSchema)]
#[allow(dead_code)]
struct StringContentTest {
    #[validate(contains = "@")]
    has_at: String,

    #[validate(starts_with = "https://")]
    secure_url: String,

    #[validate(ends_with = ".com")]
    domain: String,
}

#[test]
fn test_string_content_rules() {
    let schema = StringContentTest::json_schema();
    let json = serde_json::to_value(&schema).unwrap();

    // contains should generate a pattern
    assert!(json["properties"]["has_at"]["pattern"]
        .as_str()
        .unwrap()
        .contains("@"));

    // starts_with should generate a pattern starting with ^
    let starts_pattern = json["properties"]["secure_url"]["pattern"]
        .as_str()
        .unwrap();
    assert!(starts_pattern.starts_with("^"));
    assert!(starts_pattern.contains("https://"));

    // ends_with should generate a pattern ending with $
    let ends_pattern = json["properties"]["domain"]["pattern"].as_str().unwrap();
    assert!(ends_pattern.ends_with("$"));
    assert!(ends_pattern.contains(".com"));
}

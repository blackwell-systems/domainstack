//! Unified validation and schema generation example
//!
//! This example demonstrates writing validation rules ONCE and getting BOTH:
//! 1. Runtime validation with #[derive(Validate)]
//! 2. OpenAPI schemas with #[derive(ToSchema)]
//!
//! Both macros support the SAME rich validation syntax—no duplication needed.
//!
//! Run with:
//! ```sh
//! cargo run --example unified_validation_schema
//! ```

use domainstack::prelude::*;
use domainstack::Validate;
use domainstack_derive::ToSchema;
use domainstack_schema::OpenApiBuilder;

/// User registration with unified validation and schema generation
///
/// Notice: The SAME #[validate(...)] attributes work for BOTH macros!
#[derive(Debug, Validate, ToSchema)]
#[schema(description = "User registration request")]
struct UserRegistration {
    /// Email field - validated at runtime AND in OpenAPI schema
    #[validate(email)]
    #[validate(max_len = 255)]
    #[schema(description = "Valid email address", example = "user@example.com")]
    email: String,

    /// Age field - range validation applied to both runtime and schema
    #[validate(range(min = 18, max = 120))]
    #[schema(description = "User age (must be 18+)")]
    age: u8,

    /// Username - alphanumeric validation works in both contexts
    #[validate(alphanumeric)]
    #[validate(min_len = 3)]
    #[validate(max_len = 20)]
    #[schema(description = "Unique username", example = "alice123")]
    username: String,

    /// Optional field - automatically excluded from OpenAPI required array
    #[schema(description = "Optional display name")]
    display_name: Option<String>,
}

/// Address with nested validation
#[derive(Debug, Validate, ToSchema)]
#[schema(description = "Physical address")]
struct Address {
    #[validate(min_len = 1)]
    #[validate(max_len = 200)]
    #[schema(description = "Street address", example = "123 Main St")]
    street: String,

    #[validate(min_len = 1)]
    #[validate(max_len = 100)]
    #[schema(description = "City", example = "San Francisco")]
    city: String,

    #[validate(alphanumeric)]
    #[validate(min_len = 5)]
    #[validate(max_len = 10)]
    #[schema(description = "Postal code", example = "94102")]
    postal_code: String,
}

/// User profile with nested types
#[derive(Debug, Validate, ToSchema)]
#[schema(description = "Complete user profile")]
struct UserProfile {
    #[validate(email)]
    #[validate(max_len = 255)]
    email: String,

    /// Nested validation - validates the Address AND generates $ref in schema
    #[validate(nested)]
    #[schema(description = "Primary address")]
    address: Address,

    /// Optional nested - excluded from required array
    #[schema(description = "Billing address (optional)")]
    billing_address: Option<Address>,
}

/// Tag for blog posts
#[derive(Debug, Validate, ToSchema)]
#[schema(description = "Content tag")]
struct Tag {
    #[validate(alphanumeric)]
    #[validate(min_len = 1)]
    #[validate(max_len = 50)]
    #[schema(description = "Tag name", example = "rust")]
    name: String,
}

/// Blog post with collection validation
#[derive(Debug, Validate, ToSchema)]
#[schema(description = "Blog post with tags")]
struct BlogPost {
    #[validate(min_len = 1)]
    #[validate(max_len = 200)]
    #[schema(description = "Post title", example = "Getting Started with Rust")]
    title: String,

    #[validate(min_len = 1)]
    #[validate(max_len = 10000)]
    #[schema(description = "Post content")]
    content: String,

    /// Collection of nested types - validates each tag
    #[validate(each_nested)]
    #[schema(description = "Content tags")]
    tags: Vec<Tag>,
}

fn main() {
    println!("=== Unified Validation & Schema Generation ===\n");

    // ============================================
    // 1. RUNTIME VALIDATION WORKS
    // ============================================

    println!("1. Testing Runtime Validation:\n");

    let valid_user = UserRegistration {
        email: "alice@example.com".to_string(),
        age: 25,
        username: "alice123".to_string(),
        display_name: Some("Alice Smith".to_string()),
    };

    match valid_user.validate() {
        Ok(_) => println!("   ✓ Valid user passed validation"),
        Err(e) => println!("   ✗ Validation failed: {:?}", e),
    }

    let invalid_user = UserRegistration {
        email: "not-an-email".to_string(), // Invalid!
        age: 15,                           // Too young!
        username: "ab".to_string(),        // Too short!
        display_name: None,
    };

    match invalid_user.validate() {
        Ok(_) => println!("   ✗ Invalid user should have failed!"),
        Err(e) => {
            println!("   ✓ Invalid user rejected with {} errors:", e.violations.len());
            for v in &e.violations {
                println!("     - {}: {}", v.path, v.message);
            }
        }
    }

    // ============================================
    // 2. SCHEMA GENERATION WORKS
    // ============================================

    println!("\n2. Generating OpenAPI Schemas:\n");

    let spec = OpenApiBuilder::new("User Management API", "1.0.0")
        .description("API with unified validation and schema generation")
        .register::<UserRegistration>()
        .register::<Address>()
        .register::<UserProfile>()
        .register::<Tag>()
        .register::<BlogPost>()
        .build();

    match spec.to_json() {
        Ok(json) => {
            // Pretty print a section of the spec
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&json) {
                if let Some(schemas) = parsed["components"]["schemas"].as_object() {
                    if let Some(user_schema) = schemas.get("UserRegistration") {
                        println!("   Generated schema for UserRegistration:");
                        println!("{}", serde_json::to_string_pretty(user_schema).unwrap());
                    }
                }
            }
        }
        Err(e) => println!("   ✗ Schema generation failed: {}", e),
    }

    // ============================================
    // 3. VALIDATION RULES → SCHEMA MAPPING
    // ============================================

    println!("\n3. Validation Rule → OpenAPI Schema Mapping:\n");
    println!("   #[validate(email)]");
    println!("   → format: 'email'\n");

    println!("   #[validate(max_len = 255)]");
    println!("   → maxLength: 255\n");

    println!("   #[validate(range(min = 18, max = 120))]");
    println!("   → minimum: 18, maximum: 120\n");

    println!("   #[validate(alphanumeric)]");
    println!("   → pattern: '^[a-zA-Z0-9]*$'\n");

    println!("   #[validate(min_len = 3)], #[validate(max_len = 20)]");
    println!("   → minLength: 3, maxLength: 20\n");

    println!("   display_name: Option<String>");
    println!("   → excluded from required array\n");

    println!("   #[validate(nested)]");
    println!("   → $ref: '#/components/schemas/Address'\n");

    println!("   #[validate(min_items = 1)], #[validate(max_items = 10)]");
    println!("   → minItems: 1, maxItems: 10\n");

    // ============================================
    // 4. KEY BENEFITS
    // ============================================

    println!("\n4. Benefits of Unified Approach:\n");
    println!("   ✓ Write validation rules ONCE");
    println!("   ✓ Get runtime validation automatically");
    println!("   ✓ Get OpenAPI schemas automatically");
    println!("   ✓ No duplication between code and docs");
    println!("   ✓ Single source of truth");
    println!("   ✓ Changes propagate automatically");
    println!("   ✓ Same rich syntax for both macros");

    println!("\n==============================================");
    println!("Example complete! Both validation and schema generation work from the same attributes.");
}

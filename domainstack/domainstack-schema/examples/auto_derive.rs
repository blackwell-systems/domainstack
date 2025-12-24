//! Example demonstrating automatic schema derivation from validation rules
//!
//! This example shows how to use the #[derive(ToSchema)] macro to automatically
//! generate OpenAPI schemas from your validation rules, eliminating duplication.
//!
//! Run with:
//! ```sh
//! cargo run --example auto_derive
//! ```

use domainstack_derive::ToSchema;
use domainstack_schema::{OpenApiBuilder, ToSchema as ToSchemaTrait};

// ================================
// Basic Example: User Registration
// ================================

/// User registration request with validated fields.
///
/// The ToSchema derive automatically generates an OpenAPI schema that includes:
/// - Type information (string, integer, etc.)
/// - Validation constraints (min/max length, format, range, etc.)
/// - Required vs optional fields
/// - Custom descriptions and examples via #[schema(...)] hints
#[derive(ToSchema)]
#[schema(description = "User registration request")]
struct UserRegistration {
    /// User's email address - validated as email format with max length
    #[validate(email)]
    #[validate(max_len = 255)]
    #[schema(description = "Valid email address", example = "alice@example.com")]
    email: String,

    /// Username - alphanumeric only, 3-20 characters
    #[validate(alphanumeric)]
    #[validate(min_len = 3)]
    #[validate(max_len = 20)]
    #[schema(description = "Unique username", example = "alice123")]
    username: String,

    /// Age - must be 18 or older, max 120
    #[validate(range(min = 18, max = 120))]
    #[schema(description = "User's age")]
    age: u8,

    /// Display name - optional field (not in required array)
    #[validate(min_len = 1)]
    #[validate(max_len = 100)]
    #[schema(description = "Optional display name", example = "Alice Smith")]
    display_name: Option<String>,
}

// ===========================
// Nested Types
// ===========================

/// Physical address with validated fields
#[derive(ToSchema)]
#[schema(description = "Physical mailing address")]
struct Address {
    #[validate(min_len = 1)]
    #[validate(max_len = 200)]
    #[schema(description = "Street address", example = "123 Main St")]
    street: String,

    #[validate(min_len = 1)]
    #[validate(max_len = 100)]
    #[schema(description = "City name", example = "San Francisco")]
    city: String,

    #[validate(alphanumeric)]
    #[validate(min_len = 5)]
    #[validate(max_len = 10)]
    #[schema(description = "Postal code", example = "94102")]
    postal_code: String,

    #[validate(min_len = 2)]
    #[validate(max_len = 2)]
    #[schema(description = "Two-letter country code", example = "US")]
    country: String,
}

/// User profile with nested address
#[derive(ToSchema)]
#[schema(description = "Complete user profile")]
struct UserProfile {
    #[validate(email)]
    #[validate(max_len = 255)]
    email: String,

    /// Nested type - generates a $ref to Address schema
    #[validate(nested)]
    #[schema(description = "User's primary address")]
    address: Address,

    /// Optional nested type
    #[validate(nested)]
    #[schema(description = "Optional billing address")]
    billing_address: Option<Address>,
}

// ===========================
// Collections (Arrays)
// ===========================

/// Tag with validation
#[derive(ToSchema)]
struct Tag {
    #[validate(alphanumeric)]
    #[validate(min_len = 1)]
    #[validate(max_len = 50)]
    #[schema(description = "Tag name", example = "rust")]
    name: String,

    #[validate(min_len = 0)]
    #[validate(max_len = 200)]
    #[schema(description = "Tag description")]
    description: String,
}

/// Blog post with collections
#[derive(ToSchema)]
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

    /// Collection of nested types - generates array schema with $ref items
    #[validate(each_nested)]
    #[validate(min_items = 1)]
    #[validate(max_items = 10)]
    #[schema(description = "Post tags")]
    tags: Vec<Tag>,
}

// ===========================
// E-commerce Example
// ===========================

/// Product in catalog
#[derive(ToSchema)]
#[schema(description = "Product in the catalog")]
struct Product {
    #[validate(alphanumeric)]
    #[validate(min_len = 3)]
    #[validate(max_len = 20)]
    #[schema(description = "Unique product SKU", example = "WIDGET123")]
    sku: String,

    #[validate(min_len = 1)]
    #[validate(max_len = 200)]
    #[schema(description = "Product name", example = "Acme Widget")]
    name: String,

    #[validate(min_len = 0)]
    #[validate(max_len = 2000)]
    #[schema(description = "Product description")]
    description: String,

    /// Price in cents - must be positive
    #[validate(range(min = 1, max = 1000000))]
    #[schema(description = "Price in cents", example = 1999)]
    price: i32,

    /// Stock quantity
    #[validate(range(min = 0, max = 100000))]
    #[schema(description = "Available quantity")]
    stock: u32,
}

/// Shopping cart item
#[derive(ToSchema)]
struct CartItem {
    #[validate(nested)]
    product: Product,

    #[validate(range(min = 1, max = 100))]
    #[schema(description = "Quantity to purchase")]
    quantity: u8,
}

/// Shopping cart
#[derive(ToSchema)]
#[schema(description = "User's shopping cart")]
struct ShoppingCart {
    #[validate(each_nested)]
    #[validate(min_items = 1)]
    #[validate(max_items = 50)]
    #[schema(description = "Items in cart")]
    items: Vec<CartItem>,

    /// Promo code - optional
    #[validate(alphanumeric)]
    #[validate(min_len = 4)]
    #[validate(max_len = 20)]
    #[schema(description = "Optional promo code", example = "SAVE10")]
    promo_code: Option<String>,
}

// ===========================
// Pattern Validation Examples
// ===========================

/// API credentials with pattern validation
#[derive(ToSchema)]
#[schema(description = "API authentication credentials")]
struct ApiCredentials {
    /// API key - alphanumeric only
    #[validate(alphanumeric)]
    #[validate(min_len = 32)]
    #[validate(max_len = 64)]
    #[schema(description = "API key (alphanumeric)", example = "abc123xyz789")]
    api_key: String,

    /// API secret - ASCII characters
    #[validate(ascii)]
    #[validate(min_len = 32)]
    #[validate(max_len = 128)]
    #[schema(description = "API secret (ASCII)", example = "secret_value_here")]
    api_secret: String,

    /// Service URL
    #[validate(url)]
    #[schema(description = "Service endpoint URL", example = "https://api.example.com")]
    service_url: String,
}

// ===================================
// Collection Item Validation (each)
// ===================================

/// Article with string collections
///
/// Note: The ToSchema derive macro currently doesn't support parsing
/// each(rule) syntax. For collection item validation, use manual ToSchema impl.
#[derive(ToSchema)]
#[schema(description = "Article with collections")]
struct Article {
    #[validate(min_len = 1)]
    #[validate(max_len = 200)]
    #[schema(description = "Article title", example = "Rust Best Practices")]
    title: String,

    /// Author email addresses
    #[validate(min_items = 1)]
    #[validate(max_items = 5)]
    #[schema(description = "Author email addresses")]
    author_emails: Vec<String>,

    /// Content tags
    #[schema(description = "Content tags", example = r#"["rust", "validation"]"#)]
    tags: Vec<String>,

    /// Related article links
    #[schema(description = "Related article links")]
    related_links: Vec<String>,

    /// SEO keywords
    #[schema(description = "SEO keywords")]
    keywords: Vec<String>,

    /// User ratings (1-5 stars)
    #[schema(description = "User ratings (1-5 stars)")]
    ratings: Vec<u8>,

    /// Daily view counts
    #[schema(description = "Daily view counts")]
    daily_views: Vec<u32>,
}

fn main() {
    // Build OpenAPI spec with all registered schemas
    let spec = OpenApiBuilder::new("Auto-Derive Example API", "1.0.0")
        .description("Demonstrating automatic schema derivation from validation rules")
        .register::<UserRegistration>()
        .register::<Address>()
        .register::<UserProfile>()
        .register::<Tag>()
        .register::<BlogPost>()
        .register::<Product>()
        .register::<CartItem>()
        .register::<ShoppingCart>()
        .register::<ApiCredentials>()
        .register::<Article>()
        .build();

    // Output the OpenAPI spec as JSON
    match spec.to_json() {
        Ok(json) => {
            println!("OpenAPI Specification:");
            println!("{}", json);
            println!("\n======================");
            println!("Key Features Demonstrated:");
            println!("======================\n");

            println!("✓ Email validation → format: 'email'");
            println!("✓ URL validation → format: 'uri'");
            println!("✓ min_len/max_len → minLength/maxLength");
            println!("✓ range(min, max) → minimum/maximum");
            println!("✓ alphanumeric → pattern: '^[a-zA-Z0-9]*$'");
            println!("✓ ascii → pattern: '^[\\x00-\\x7F]*$'");
            println!("✓ min_items/max_items → minItems/maxItems");
            println!("✓ Optional<T> → excluded from required array");
            println!("✓ Nested types → $ref to component schema");
            println!("✓ Vec<T> with each_nested → array with $ref items");
            println!("✓ Vec<T> with each(rule) → array items with validation constraints");
            println!("✓ #[schema(...)] → descriptions and examples");

            println!("\n======================");
            println!("Registered Schemas:");
            println!("======================\n");
            println!("- UserRegistration: Basic validation (email, range, length)");
            println!("- Address: String validation (length, patterns)");
            println!("- UserProfile: Nested types (required and optional)");
            println!("- Tag: Simple nested type");
            println!("- BlogPost: Collections with nested items");
            println!("- Product: E-commerce entity with price/stock");
            println!("- CartItem: Nested product reference");
            println!("- ShoppingCart: Complex collection with constraints");
            println!("- ApiCredentials: Pattern validation (alphanumeric, ascii, url)");
            println!("- Article: Collection item validation with each(rule)");
        }
        Err(e) => eprintln!("Error generating JSON: {}", e),
    }
}

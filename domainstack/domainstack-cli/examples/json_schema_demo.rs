//! JSON Schema Generation Demo
//!
//! This file demonstrates all the types that can be used with `domainstack json-schema`:
//! - Named structs with various validation rules
//! - Tuple structs (newtype patterns)
//! - Enums with unit, tuple, and struct variants
//! - Nested types and arrays
//!
//! Generate JSON Schema with:
//! ```bash
//! domainstack json-schema --input examples --output schema.json --verbose
//! ```

use domainstack_derive::Validate;

// =============================================================================
// TUPLE STRUCTS (Newtype Patterns)
// =============================================================================

/// Email address newtype - validates format
#[derive(Debug, Validate)]
pub struct Email(#[validate(email)] pub String);

/// Age newtype - validates range
#[derive(Debug, Validate)]
pub struct Age(#[validate(range(min = 0, max = 150))] pub u8);

/// Username newtype - validates alphanumeric and length
#[derive(Debug, Validate)]
pub struct Username(
    #[validate(alphanumeric)]
    #[validate(length(min = 3, max = 30))]
    pub String,
);

/// Price newtype - validates positive value
#[derive(Debug, Validate)]
pub struct Price(#[validate(positive)] pub f64);

/// URL newtype - validates URL format
#[derive(Debug, Validate)]
pub struct Url(#[validate(url)] pub String);

/// Phone number newtype - validates format
#[derive(Debug, Validate)]
pub struct PhoneNumber(
    #[validate(matches_regex = r"^\+?[1-9]\d{1,14}$")]
    pub String,
);

// =============================================================================
// ENUMS
// =============================================================================

/// Payment method enum with different variant types
#[derive(Debug, Validate)]
pub enum PaymentMethod {
    /// Cash payment - no additional validation
    Cash,

    /// Credit card with validated fields
    CreditCard {
        #[validate(length(min = 13, max = 19))]
        #[validate(numeric_string)]
        card_number: String,

        #[validate(range(min = 1, max = 12))]
        exp_month: u8,

        #[validate(range(min = 2024, max = 2040))]
        exp_year: u16,

        #[validate(length(min = 3, max = 4))]
        #[validate(numeric_string)]
        cvv: String,
    },

    /// PayPal payment with email validation
    PayPal(#[validate(email)] String),

    /// Bank transfer with account details
    BankTransfer {
        #[validate(alphanumeric)]
        #[validate(length(min = 5, max = 34))]
        account_number: String,

        #[validate(length(min = 6, max = 11))]
        routing_number: String,
    },

    /// Cryptocurrency payment
    Crypto {
        #[validate(length(min = 26, max = 62))]
        wallet_address: String,

        currency: CryptoCurrency,
    },
}

/// Cryptocurrency type
#[derive(Debug, Validate)]
pub enum CryptoCurrency {
    Bitcoin,
    Ethereum,
    Solana,
    Other(#[validate(length(min = 2, max = 10))] String),
}

/// Contact preference enum
#[derive(Debug, Validate)]
pub enum ContactPreference {
    /// Email contact with validated address
    Email(#[validate(email)] String),

    /// Phone contact with validated number
    Phone(#[validate(matches_regex = r"^\+?[0-9]{10,15}$")] String),

    /// SMS contact with validated number
    Sms(#[validate(matches_regex = r"^\+?[0-9]{10,15}$")] String),

    /// No contact preference
    None,
}

/// Order status enum (unit variants only)
#[derive(Debug, Validate)]
pub enum OrderStatus {
    Pending,
    Processing,
    Shipped,
    Delivered,
    Cancelled,
    Refunded,
}

// =============================================================================
// NAMED STRUCTS
// =============================================================================

/// User profile with various validations
#[derive(Debug, Validate)]
pub struct UserProfile {
    #[validate(email)]
    #[validate(max_len = 255)]
    pub email: String,

    #[validate(length(min = 3, max = 50))]
    #[validate(alphanumeric)]
    pub username: String,

    #[validate(length(min = 1, max = 100))]
    pub display_name: String,

    #[validate(range(min = 13, max = 120))]
    pub age: u8,

    #[validate(url)]
    pub avatar_url: Option<String>,

    #[validate(max_len = 500)]
    pub bio: Option<String>,

    pub contact_preference: ContactPreference,
}

/// Address with full validation
#[derive(Debug, Validate)]
pub struct Address {
    #[validate(length(min = 1, max = 100))]
    pub street_line_1: String,

    #[validate(max_len = 100)]
    pub street_line_2: Option<String>,

    #[validate(length(min = 2, max = 50))]
    pub city: String,

    #[validate(length(min = 2, max = 50))]
    pub state: String,

    #[validate(matches_regex = r"^[0-9]{5}(-[0-9]{4})?$")]
    pub postal_code: String,

    #[validate(length(min = 2, max = 2))]
    pub country_code: String,
}

/// Product in the catalog
#[derive(Debug, Validate)]
pub struct Product {
    #[validate(length(min = 1, max = 200))]
    pub name: String,

    #[validate(max_len = 2000)]
    pub description: Option<String>,

    #[validate(positive)]
    pub price: Price,

    #[validate(range(min = 0, max = 1000000))]
    pub stock_quantity: u32,

    pub tags: Vec<String>,

    pub category: ProductCategory,
}

/// Product category enum
#[derive(Debug, Validate)]
pub enum ProductCategory {
    Electronics,
    Clothing,
    Books,
    HomeAndGarden,
    Sports,
    Toys,
    Food,
    Other(#[validate(length(min = 2, max = 50))] String),
}

/// Shopping cart item
#[derive(Debug, Validate)]
pub struct CartItem {
    #[validate(length(min = 1, max = 50))]
    pub product_id: String,

    #[validate(range(min = 1, max = 100))]
    pub quantity: u32,

    pub unit_price: Price,
}

/// Complete order
#[derive(Debug, Validate)]
pub struct Order {
    #[validate(length(min = 1, max = 50))]
    pub order_id: String,

    #[validate(nested)]
    pub customer: UserProfile,

    #[validate(nested)]
    pub shipping_address: Address,

    #[validate(nested)]
    pub billing_address: Option<Address>,

    pub items: Vec<CartItem>,

    pub payment_method: PaymentMethod,

    pub status: OrderStatus,

    #[validate(positive)]
    pub total_amount: Price,

    #[validate(max_len = 500)]
    pub notes: Option<String>,
}

/// API request for creating a user
#[derive(Debug, Validate)]
pub struct CreateUserRequest {
    pub email: Email,
    pub username: Username,

    #[validate(length(min = 8, max = 128))]
    pub password: String,

    pub age: Age,

    #[validate(url)]
    pub website: Option<String>,

    pub phone: Option<PhoneNumber>,
}

/// API response for user creation
#[derive(Debug, Validate)]
pub struct CreateUserResponse {
    #[validate(length(min = 1, max = 50))]
    pub user_id: String,

    pub email: Email,
    pub username: Username,

    #[validate(url)]
    pub profile_url: String,
}

// =============================================================================
// MAIN - For testing
// =============================================================================

fn main() {
    println!("JSON Schema Demo Types");
    println!("======================");
    println!();
    println!("This file contains example types for JSON Schema generation.");
    println!();
    println!("To generate JSON Schema, run:");
    println!("  domainstack json-schema --input examples --output schema.json --verbose");
    println!();
    println!("Types included:");
    println!("  - Tuple structs: Email, Age, Username, Price, Url, PhoneNumber");
    println!("  - Enums: PaymentMethod, ContactPreference, OrderStatus, ProductCategory");
    println!("  - Named structs: UserProfile, Address, Product, CartItem, Order");
    println!("  - API types: CreateUserRequest, CreateUserResponse");
}

//! # Cross-Field Validation (v0.5.0)
//!
//! This example demonstrates cross-field validation, which allows you to validate
//! relationships between multiple fields on the same struct.
//!
//! ## Features Demonstrated
//!
//! 1. **Password Confirmation**: Ensuring two fields match (e.g., password and password_confirmation)
//! 2. **Date Range Validation**: Ensuring end_date is after start_date
//! 3. **Conditional Validation**: Using `when` to apply checks conditionally
//! 4. **Multiple Cross-Field Checks**: Applying multiple struct-level validations
//!
//! ## Usage
//!
//! ```bash
//! cargo run --example v5_cross_field_validation
//! ```

use domainstack::prelude::*;
use domainstack_derive::Validate;

/// Registration form with password confirmation
#[derive(Debug, Validate)]
#[validate(
    check = "self.password == self.password_confirmation",
    code = "passwords_mismatch",
    message = "Passwords must match"
)]
struct RegisterForm {
    #[validate(length(min = 3, max = 50))]
    username: String,

    #[validate(custom = "crate::validate_email")]
    email: String,

    #[validate(length(min = 8, max = 100))]
    password: String,

    password_confirmation: String,
}

#[allow(clippy::result_large_err)]
fn validate_email(email: &str) -> Result<(), ValidationError> {
    let rule = rules::email();
    validate("email", email, &rule)
}

/// Date range with cross-field validation
#[derive(Debug, Validate)]
#[validate(
    check = "self.end_date > self.start_date",
    code = "invalid_date_range",
    message = "End date must be after start date"
)]
struct Reservation {
    #[validate(length(min = 1))]
    guest_name: String,

    start_date: String,
    end_date: String,

    #[validate(range(min = 1, max = 10))]
    guests: u8,
}

/// Order with conditional discount validation
#[derive(Debug, Validate)]
#[validate(
    check = "self.discount_code.is_empty() || self.discount_percentage == 0",
    code = "discount_conflict",
    message = "Cannot apply both discount code and percentage discount"
)]
#[validate(
    check = "self.total >= self.minimum_order_amount",
    code = "below_minimum",
    message = "Order total is below minimum",
    when = "self.requires_minimum"
)]
struct Order {
    #[validate(range(min = 0.0, max = 1000000.0))]
    total: f64,

    discount_code: String,

    #[validate(range(min = 0, max = 100))]
    discount_percentage: u8,

    requires_minimum: bool,
    minimum_order_amount: f64,
}

/// Account update form with conditional validation
#[derive(Debug, Validate)]
#[allow(clippy::duplicated_attributes)]
#[validate(
    check = "self.new_password == self.confirm_password",
    code = "password_mismatch",
    message = "New password and confirmation must match",
    when = "!self.new_password.is_empty()"
)]
#[validate(
    check = "self.new_password.len() >= 8",
    code = "password_too_short",
    message = "New password must be at least 8 characters",
    when = "!self.new_password.is_empty()"
)]
struct AccountUpdate {
    #[validate(length(min = 3, max = 50))]
    username: String,

    /// New password (optional - only validated if provided)
    new_password: String,

    /// Confirmation for new password
    confirm_password: String,
}

fn main() {
    println!("=== Cross-Field Validation Examples ===\n");

    // Example 1: Password confirmation - VALID
    println!("1. Password Confirmation (Valid):");
    let form = RegisterForm {
        username: "john_doe".to_string(),
        email: "john@example.com".to_string(),
        password: "secure_password_123".to_string(),
        password_confirmation: "secure_password_123".to_string(),
    };
    match form.validate() {
        Ok(_) => println!("   [ok] Registration form is valid\n"),
        Err(e) => println!("   [error] Validation failed:\n{}\n", e),
    }

    // Example 2: Password confirmation - MISMATCH
    println!("2. Password Confirmation (Mismatch):");
    let form = RegisterForm {
        username: "jane_doe".to_string(),
        email: "jane@example.com".to_string(),
        password: "password123".to_string(),
        password_confirmation: "different_password".to_string(),
    };
    match form.validate() {
        Ok(_) => println!("   [ok] Registration form is valid\n"),
        Err(e) => println!("   [error] Validation failed:\n{}\n", e),
    }

    // Example 3: Date range validation - VALID
    println!("3. Date Range Validation (Valid):");
    let reservation = Reservation {
        guest_name: "Alice".to_string(),
        start_date: "2025-01-15".to_string(),
        end_date: "2025-01-20".to_string(),
        guests: 2,
    };
    match reservation.validate() {
        Ok(_) => println!("   [ok] Reservation is valid\n"),
        Err(e) => println!("   [error] Validation failed:\n{}\n", e),
    }

    // Example 4: Date range validation - INVALID
    println!("4. Date Range Validation (Invalid - End before Start):");
    let reservation = Reservation {
        guest_name: "Bob".to_string(),
        start_date: "2025-01-20".to_string(),
        end_date: "2025-01-15".to_string(),
        guests: 3,
    };
    match reservation.validate() {
        Ok(_) => println!("   [ok] Reservation is valid\n"),
        Err(e) => println!("   [error] Validation failed:\n{}\n", e),
    }

    // Example 5: Discount validation - VALID (code only)
    println!("5. Discount Validation (Valid - Code Only):");
    let order = Order {
        total: 150.0,
        discount_code: "SUMMER2025".to_string(),
        discount_percentage: 0,
        requires_minimum: false,
        minimum_order_amount: 0.0,
    };
    match order.validate() {
        Ok(_) => println!("   [ok] Order is valid\n"),
        Err(e) => println!("   [error] Validation failed:\n{}\n", e),
    }

    // Example 6: Discount validation - INVALID (both code and percentage)
    println!("6. Discount Validation (Invalid - Both Code and Percentage):");
    let order = Order {
        total: 150.0,
        discount_code: "SUMMER2025".to_string(),
        discount_percentage: 10,
        requires_minimum: false,
        minimum_order_amount: 0.0,
    };
    match order.validate() {
        Ok(_) => println!("   [ok] Order is valid\n"),
        Err(e) => println!("   [error] Validation failed:\n{}\n", e),
    }

    // Example 7: Conditional validation - APPLIES (below minimum when required)
    println!("7. Conditional Validation (Applies - Below Minimum):");
    let order = Order {
        total: 25.0,
        discount_code: String::new(),
        discount_percentage: 0,
        requires_minimum: true,
        minimum_order_amount: 50.0,
    };
    match order.validate() {
        Ok(_) => println!("   [ok] Order is valid\n"),
        Err(e) => println!("   [error] Validation failed:\n{}\n", e),
    }

    // Example 8: Conditional validation - SKIPPED (minimum not required)
    println!("8. Conditional Validation (Skipped - Minimum Not Required):");
    let order = Order {
        total: 25.0,
        discount_code: String::new(),
        discount_percentage: 0,
        requires_minimum: false,
        minimum_order_amount: 50.0,
    };
    match order.validate() {
        Ok(_) => println!("   [ok] Order is valid (minimum check skipped)\n"),
        Err(e) => println!("   [error] Validation failed:\n{}\n", e),
    }

    // Example 9: Optional password update - NO PASSWORD (valid)
    println!("9. Optional Password Update (No Password - Valid):");
    let update = AccountUpdate {
        username: "alice".to_string(),
        new_password: String::new(),
        confirm_password: String::new(),
    };
    match update.validate() {
        Ok(_) => println!("   [ok] Update is valid (password check skipped)\n"),
        Err(e) => println!("   [error] Validation failed:\n{}\n", e),
    }

    // Example 10: Optional password update - MISMATCH
    println!("10. Optional Password Update (Mismatch):");
    let update = AccountUpdate {
        username: "alice".to_string(),
        new_password: "new_secure_password".to_string(),
        confirm_password: "different_password".to_string(),
    };
    match update.validate() {
        Ok(_) => println!("   [ok] Update is valid\n"),
        Err(e) => println!("   [error] Validation failed:\n{}\n", e),
    }

    println!("=== Key Takeaways ===");
    println!(
        "1. Use #[validate(check = \"...\", code = \"...\", message = \"...\")] at struct level"
    );
    println!("2. Access fields via self.field_name in check expressions");
    println!("3. Add 'when' parameter for conditional cross-field validation");
    println!("4. Combine with field-level validation for comprehensive checks");
    println!("5. Multiple struct-level validations are evaluated in order");
}

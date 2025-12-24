//! Example demonstrating phantom types for compile-time validation guarantees.
//!
//! This example shows:
//! - Type-state pattern with validation markers
//! - Compile-time safety for validated data
//! - Zero-cost abstractions with PhantomData
//! - Builder pattern integration
//! - Simulated database operations requiring validated data
//!
//! Run with: cargo run --example phantom_types --features regex

use domainstack::typestate::{Unvalidated, Validated};
use domainstack::{rules, validate, ValidationError};
use std::marker::PhantomData;

/// Email address with validation state tracking.
///
/// State can be either `Unvalidated` (default) or `Validated`.
/// Only validated emails can be used in operations like sending emails or saving to database.
#[derive(Debug, Clone)]
pub struct Email<State = Unvalidated> {
    value: String,
    _state: PhantomData<State>,
}

impl Email<Unvalidated> {
    /// Create a new unvalidated email.
    ///
    /// This does not perform validation - the email may be invalid.
    pub fn new(value: String) -> Self {
        Self {
            value,
            _state: PhantomData,
        }
    }

    /// Validate the email and transition to validated state.
    ///
    /// Returns `Email<Validated>` on success, or `ValidationError` on failure.
    #[allow(clippy::result_large_err)]
    pub fn validate(self) -> Result<Email<Validated>, ValidationError> {
        validate("email", self.value.as_str(), &rules::email())?;
        Ok(Email {
            value: self.value,
            _state: PhantomData,
        })
    }
}

// Methods available for both validated and unvalidated emails
impl<State> Email<State> {
    pub fn as_str(&self) -> &str {
        &self.value
    }

    pub fn len(&self) -> usize {
        self.value.len()
    }

    pub fn is_empty(&self) -> bool {
        self.value.is_empty()
    }
}

/// User registration form with validation state tracking.
///
/// Demonstrates multi-field validation with phantom types.
#[derive(Debug)]
pub struct UserRegistration<State = Unvalidated> {
    username: String,
    email: String,
    password: String,
    age: u8,
    _state: PhantomData<State>,
}

impl UserRegistration<Unvalidated> {
    /// Create a builder for constructing a user registration.
    pub fn builder() -> UserRegistrationBuilder {
        UserRegistrationBuilder::default()
    }

    /// Validate all fields and transition to validated state.
    ///
    /// Accumulates all validation errors across all fields.
    #[allow(clippy::result_large_err)]
    pub fn validate(self) -> Result<UserRegistration<Validated>, ValidationError> {
        let mut errors = ValidationError::default();

        // Validate username: 3-20 characters
        if let Err(e) = validate(
            "username",
            self.username.as_str(),
            &rules::min_len(3).and(rules::max_len(20)),
        ) {
            errors.extend(e);
        }

        // Validate email format
        if let Err(e) = validate("email", self.email.as_str(), &rules::email()) {
            errors.extend(e);
        }

        // Validate password: minimum 8 characters
        if let Err(e) = validate("password", self.password.as_str(), &rules::min_len(8)) {
            errors.extend(e);
        }

        // Validate age: 13-120
        if let Err(e) = validate("age", &self.age, &rules::range(13, 120)) {
            errors.extend(e);
        }

        if errors.is_empty() {
            Ok(UserRegistration {
                username: self.username,
                email: self.email,
                password: self.password,
                age: self.age,
                _state: PhantomData,
            })
        } else {
            Err(errors)
        }
    }
}

// Methods available for both validated and unvalidated users
impl<State> UserRegistration<State> {
    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn email(&self) -> &str {
        &self.email
    }

    pub fn age(&self) -> u8 {
        self.age
    }
}

/// Builder for constructing UserRegistration.
#[derive(Default)]
pub struct UserRegistrationBuilder {
    username: Option<String>,
    email: Option<String>,
    password: Option<String>,
    age: Option<u8>,
}

impl UserRegistrationBuilder {
    pub fn username(mut self, username: impl Into<String>) -> Self {
        self.username = Some(username.into());
        self
    }

    pub fn email(mut self, email: impl Into<String>) -> Self {
        self.email = Some(email.into());
        self
    }

    pub fn password(mut self, password: impl Into<String>) -> Self {
        self.password = Some(password.into());
        self
    }

    pub fn age(mut self, age: u8) -> Self {
        self.age = Some(age);
        self
    }

    /// Build an unvalidated UserRegistration.
    ///
    /// The caller must call `.validate()` before using in validated contexts.
    pub fn build(self) -> UserRegistration<Unvalidated> {
        UserRegistration {
            username: self.username.unwrap_or_default(),
            email: self.email.unwrap_or_default(),
            password: self.password.unwrap_or_default(),
            age: self.age.unwrap_or(0),
            _state: PhantomData,
        }
    }
}

// ========================================
// Simulated Application Layer
// ========================================

/// Simulated database operation that ONLY accepts validated users.
///
/// This function signature ensures at compile-time that only validated
/// users can be saved to the database.
fn save_to_database(user: &UserRegistration<Validated>) -> Result<i64, String> {
    println!("💾 Saving to database:");
    println!("   Username: {}", user.username());
    println!("   Email: {}", user.email());
    println!("   Age: {}", user.age());

    // Simulate database insert
    let user_id = 12345; // Simulated ID
    println!("   ✓ User saved with ID: {}\n", user_id);

    Ok(user_id)
}

/// Send welcome email to a validated email address.
///
/// The type signature ensures only validated emails can be sent.
fn send_welcome_email(email: &Email<Validated>) {
    println!("📧 Sending welcome email to: {}", email.as_str());
    println!("   ✓ Email sent successfully\n");
}

/// Complete user registration workflow.
///
/// Only accepts validated users, ensuring all validation passed.
fn complete_registration(user: UserRegistration<Validated>) -> Result<(), String> {
    println!("🎉 Completing registration for: {}", user.username());

    // Save to database
    let user_id = save_to_database(&user)?;

    // Send welcome email (using validated email)
    let email = Email::new(user.email().to_string())
        .validate()
        .map_err(|e| format!("Email validation failed: {}", e))?;

    send_welcome_email(&email);

    println!("✓ Registration complete! User ID: {}\n", user_id);
    Ok(())
}

fn main() {
    println!("=== Phantom Types Validation Example ===\n");

    // ========================================
    // Example 1: Simple Email Validation
    // ========================================
    println!("Example 1: Simple Email Validation");
    println!("-----------------------------------");

    let email = Email::new("alice@example.com".to_string());
    println!("Created unvalidated email: {}", email.as_str());

    // This would NOT compile - send_welcome_email requires Email<Validated>:
    // send_welcome_email(&email);  // ❌ Compile error!

    match email.validate() {
        Ok(validated_email) => {
            println!("✓ Email validated successfully");
            send_welcome_email(&validated_email); // ✅ Compiles!
        }
        Err(e) => println!("✗ Email validation failed: {}\n", e),
    }

    // ========================================
    // Example 2: Invalid Email
    // ========================================
    println!("Example 2: Invalid Email Validation");
    println!("------------------------------------");

    let invalid_email = Email::new("not-an-email".to_string());
    println!("Created unvalidated email: {}", invalid_email.as_str());

    match invalid_email.validate() {
        Ok(_) => println!("✓ Email validated (unexpected)\n"),
        Err(e) => {
            println!("✗ Email validation failed as expected:");
            for v in &e.violations {
                println!("   [{}] {}: {}", v.path, v.code, v.message);
            }
            println!();
        }
    }

    // ========================================
    // Example 3: Builder Pattern with Validation
    // ========================================
    println!("Example 3: Builder Pattern with Validation");
    println!("-------------------------------------------");

    let user = UserRegistration::builder()
        .username("alice")
        .email("alice@example.com")
        .password("secure_password_123")
        .age(25)
        .build();

    println!("Built unvalidated user: {}", user.username());

    // This would NOT compile - complete_registration requires UserRegistration<Validated>:
    // complete_registration(user);  // ❌ Compile error!

    match user.validate() {
        Ok(validated_user) => {
            println!("✓ User validated successfully");
            complete_registration(validated_user).unwrap();
        }
        Err(e) => {
            println!("✗ User validation failed:");
            for v in &e.violations {
                println!("   [{}] {}: {}", v.path, v.code, v.message);
            }
            println!();
        }
    }

    // ========================================
    // Example 4: Multiple Validation Errors
    // ========================================
    println!("Example 4: Multiple Validation Errors");
    println!("--------------------------------------");

    let invalid_user = UserRegistration::builder()
        .username("ab") // Too short (min 3)
        .email("not-an-email") // Invalid format
        .password("short") // Too short (min 8)
        .age(200) // Out of range (max 120)
        .build();

    println!("Built invalid user: {}", invalid_user.username());

    match invalid_user.validate() {
        Ok(_) => println!("✓ User validated (unexpected)\n"),
        Err(e) => {
            println!(
                "✗ User validation failed with {} errors:",
                e.violations.len()
            );
            for v in &e.violations {
                println!("   [{}] {}: {}", v.path, v.code, v.message);
            }
            println!();
        }
    }

    // ========================================
    // Example 5: Partial Validation Errors
    // ========================================
    println!("Example 5: Partial Validation Errors");
    println!("-------------------------------------");

    let partial_invalid = UserRegistration::builder()
        .username("bob")
        .email("bob@example.com")
        .password("short") // Only this field is invalid
        .age(30)
        .build();

    match partial_invalid.validate() {
        Ok(_) => println!("✓ User validated (unexpected)\n"),
        Err(e) => {
            println!("✗ User validation failed:");
            for v in &e.violations {
                println!("   [{}] {}: {}", v.path, v.code, v.message);
            }
            println!();
        }
    }

    // ========================================
    // Example 6: Zero-Cost Demonstration
    // ========================================
    println!("Example 6: Zero-Cost Abstraction");
    println!("---------------------------------");

    println!(
        "Size of Email<Unvalidated>: {} bytes",
        std::mem::size_of::<Email<Unvalidated>>()
    );
    println!(
        "Size of Email<Validated>:   {} bytes",
        std::mem::size_of::<Email<Validated>>()
    );
    println!(
        "Size of String:             {} bytes",
        std::mem::size_of::<String>()
    );
    println!("✓ PhantomData adds ZERO bytes of overhead!\n");

    println!("=== Example Completed ===");
}

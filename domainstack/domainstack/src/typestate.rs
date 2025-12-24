//! Type-state validation markers for compile-time validation guarantees.
//!
//! This module provides marker types that enable tracking validation state at the type level,
//! ensuring validated data cannot be confused with unvalidated data at compile time.
//!
//! # Overview
//!
//! Phantom types allow you to encode validation state in the type system with **zero runtime cost**.
//! The compiler enforces that only validated data can be used where validated data is expected.
//!
//! # Basic Usage
//!
//! ```rust
//! use domainstack::{ValidationError, validate, rules, typestate::{Validated, Unvalidated}};
//! use std::marker::PhantomData;
//!
//! // Domain type with validation state
//! pub struct Email<State = Unvalidated> {
//!     value: String,
//!     _state: PhantomData<State>,
//! }
//!
//! // Only unvalidated emails can be created
//! impl Email<Unvalidated> {
//!     pub fn new(value: String) -> Self {
//!         Self {
//!             value,
//!             _state: PhantomData,
//!         }
//!     }
//!
//!     // Transform to validated state
//!     pub fn validate(self) -> Result<Email<Validated>, ValidationError> {
//!         validate("email", self.value.as_str(), &rules::email())?;
//!         Ok(Email {
//!             value: self.value,
//!             _state: PhantomData,
//!         })
//!     }
//! }
//!
//! // Methods available for any state
//! impl<State> Email<State> {
//!     pub fn as_str(&self) -> &str {
//!         &self.value
//!     }
//! }
//!
//! // Only validated emails can be used in certain contexts
//! fn send_email(email: Email<Validated>) {
//!     println!("Sending to: {}", email.as_str());
//! }
//!
//! // Example usage
//! let email = Email::new("user@example.com".to_string());
//! // send_email(email); // ❌ Compile error: expected Email<Validated>, found Email<Unvalidated>
//!
//! let validated = email.validate().unwrap();
//! send_email(validated); // ✅ Compiles!
//! ```
//!
//! # Benefits
//!
//! 1. **Compile-time safety** - Cannot accidentally use unvalidated data
//! 2. **Zero runtime cost** - PhantomData has zero size, optimizes away completely
//! 3. **Self-documenting** - Function signatures make validation requirements explicit
//! 4. **Progressive adoption** - Can be added to existing types without breaking changes
//!
//! # Advanced Patterns
//!
//! ## Builder Pattern with Validation
//!
//! ```rust
//! use domainstack::{ValidationError, validate, rules, typestate::{Validated, Unvalidated}};
//! use std::marker::PhantomData;
//!
//! pub struct User<State = Unvalidated> {
//!     username: String,
//!     email: String,
//!     age: u8,
//!     _state: PhantomData<State>,
//! }
//!
//! impl User<Unvalidated> {
//!     pub fn builder() -> UserBuilder {
//!         UserBuilder::default()
//!     }
//!
//!     pub fn validate(self) -> Result<User<Validated>, ValidationError> {
//!         let mut errors = ValidationError::default();
//!
//!         if let Err(e) = validate("username", self.username.as_str(), &rules::min_len(3)) {
//!             errors.extend(e);
//!         }
//!         if let Err(e) = validate("email", self.email.as_str(), &rules::email()) {
//!             errors.extend(e);
//!         }
//!         if let Err(e) = validate("age", &self.age, &rules::range(0, 120)) {
//!             errors.extend(e);
//!         }
//!
//!         if errors.is_empty() {
//!             Ok(User {
//!                 username: self.username,
//!                 email: self.email,
//!                 age: self.age,
//!                 _state: PhantomData,
//!             })
//!         } else {
//!             Err(errors)
//!         }
//!     }
//! }
//!
//! #[derive(Default)]
//! pub struct UserBuilder {
//!     username: Option<String>,
//!     email: Option<String>,
//!     age: Option<u8>,
//! }
//!
//! impl UserBuilder {
//!     pub fn username(mut self, username: String) -> Self {
//!         self.username = Some(username);
//!         self
//!     }
//!
//!     pub fn email(mut self, email: String) -> Self {
//!         self.email = Some(email);
//!         self
//!     }
//!
//!     pub fn age(mut self, age: u8) -> Self {
//!         self.age = Some(age);
//!         self
//!     }
//!
//!     pub fn build(self) -> User<Unvalidated> {
//!         User {
//!             username: self.username.unwrap_or_default(),
//!             email: self.email.unwrap_or_default(),
//!             age: self.age.unwrap_or(0),
//!             _state: PhantomData,
//!         }
//!     }
//! }
//! ```
//!
//! ## Database Operations
//!
//! ```rust,ignore
//! use domainstack::typestate::{Validated, Unvalidated};
//! use std::marker::PhantomData;
//!
//! pub struct User<State = Unvalidated> {
//!     username: String,
//!     email: String,
//!     _state: PhantomData<State>,
//! }
//!
//! // Only accept validated users for database operations
//! async fn save_to_database(user: User<Validated>) -> Result<i64, DatabaseError> {
//!     // Compiler guarantees user is validated!
//!     sqlx::query("INSERT INTO users (username, email) VALUES (?, ?)")
//!         .bind(&user.username)
//!         .bind(&user.email)
//!         .execute(&pool)
//!         .await
//! }
//!
//! // Only accept validated users for business logic
//! fn send_welcome_email(user: &User<Validated>) {
//!     // Compiler guarantees user is validated!
//!     email_service.send(&user.email, "Welcome!");
//! }
//! ```
//!
//! # Design Principles
//!
//! 1. **Opt-in** - Add to types that benefit from compile-time validation tracking
//! 2. **Zero-cost** - PhantomData compiles to zero bytes
//! 3. **Ergonomic** - Default to `Unvalidated` state for ease of construction
//! 4. **Explicit** - Validation boundaries are clear in function signatures
//!
//! # When to Use
//!
//! Use phantom types when:
//! - Data flows through multiple functions/layers
//! - Validation is expensive or has side effects (DB queries, API calls)
//! - You want compile-time guarantees that validation occurred
//! - Multiple code paths could accidentally bypass validation
//!
//! Don't use phantom types when:
//! - Type is only used in one place immediately after validation
//! - Validation is trivial and cheap to repeat
//! - The type already uses "valid-by-construction" pattern (validation in constructor)
//!
//! # Performance
//!
//! Phantom types have **zero runtime cost**:
//! - `PhantomData<T>` has size 0 bytes
//! - No memory overhead
//! - No CPU overhead
//! - Optimized away completely by the compiler
//!
//! ```rust
//! use std::marker::PhantomData;
//! use domainstack::typestate::{Validated, Unvalidated};
//!
//! struct Data<State> {
//!     value: String,
//!     _state: PhantomData<State>,
//! }
//!
//! // Both are the same size in memory!
//! assert_eq!(
//!     std::mem::size_of::<Data<Unvalidated>>(),
//!     std::mem::size_of::<Data<Validated>>()
//! );
//! assert_eq!(
//!     std::mem::size_of::<Data<Unvalidated>>(),
//!     std::mem::size_of::<String>()  // Just the string, no overhead!
//! );
//! ```

/// Marker type indicating that data has **not** been validated.
///
/// This is typically the default state for types with validation state tracking.
///
/// # Example
///
/// ```rust
/// use domainstack::typestate::Unvalidated;
/// use std::marker::PhantomData;
///
/// pub struct Email<State = Unvalidated> {
///     value: String,
///     _state: PhantomData<State>,
/// }
///
/// impl Email<Unvalidated> {
///     pub fn new(value: String) -> Self {
///         Self {
///             value,
///             _state: PhantomData,
///         }
///     }
/// }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Unvalidated;

/// Marker type indicating that data **has been** validated.
///
/// Types in this state have passed all validation rules and are guaranteed
/// to be valid at the point of state transition.
///
/// # Example
///
/// ```rust
/// use domainstack::{ValidationError, validate, rules, typestate::{Validated, Unvalidated}};
/// use std::marker::PhantomData;
///
/// pub struct Email<State = Unvalidated> {
///     value: String,
///     _state: PhantomData<State>,
/// }
///
/// impl Email<Unvalidated> {
///     pub fn validate(self) -> Result<Email<Validated>, ValidationError> {
///         validate("email", self.value.as_str(), &rules::email())?;
///         Ok(Email {
///             value: self.value,
///             _state: PhantomData,
///         })
///     }
/// }
///
/// // Only accept validated emails
/// fn send_email(email: Email<Validated>) {
///     println!("Sending to: {}", email.value);
/// }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Validated;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{rules, validate, ValidationError};
    use std::marker::PhantomData;

    // Example domain type using phantom types
    #[derive(Debug)]
    struct Email<State = Unvalidated> {
        value: String,
        _state: PhantomData<State>,
    }

    impl Email<Unvalidated> {
        fn new(value: String) -> Self {
            Self {
                value,
                _state: PhantomData,
            }
        }

        fn validate(self) -> Result<Email<Validated>, ValidationError> {
            validate("email", self.value.as_str(), &rules::email())?;
            Ok(Email {
                value: self.value,
                _state: PhantomData,
            })
        }
    }

    impl<State> Email<State> {
        fn as_str(&self) -> &str {
            &self.value
        }
    }

    // Function that only accepts validated emails
    fn send_email(email: &Email<Validated>) -> String {
        format!("Sending to: {}", email.as_str())
    }

    #[test]
    fn test_unvalidated_marker_properties() {
        // Marker types are zero-sized
        assert_eq!(std::mem::size_of::<Unvalidated>(), 0);

        // They implement common traits
        let u1 = Unvalidated;
        let u2 = Unvalidated;
        assert_eq!(u1, u2);
        assert_eq!(format!("{:?}", u1), "Unvalidated");
    }

    #[test]
    fn test_validated_marker_properties() {
        // Marker types are zero-sized
        assert_eq!(std::mem::size_of::<Validated>(), 0);

        // They implement common traits
        let v1 = Validated;
        let v2 = Validated;
        assert_eq!(v1, v2);
        assert_eq!(format!("{:?}", v1), "Validated");
    }

    #[test]
    fn test_phantom_types_zero_cost() {
        // Both Email<Unvalidated> and Email<Validated> have the same size
        assert_eq!(
            std::mem::size_of::<Email<Unvalidated>>(),
            std::mem::size_of::<Email<Validated>>()
        );

        // They're the same size as just the String field
        assert_eq!(
            std::mem::size_of::<Email<Unvalidated>>(),
            std::mem::size_of::<String>()
        );
    }

    #[test]
    fn test_validation_state_transition() {
        // Create unvalidated email
        let email = Email::new("user@example.com".to_string());

        // Cannot send unvalidated email (would not compile):
        // send_email(&email);  // ❌ Type error

        // Validate to transition state
        let validated = email.validate().expect("Should be valid");

        // Can send validated email
        let result = send_email(&validated);
        assert_eq!(result, "Sending to: user@example.com");
    }

    #[test]
    fn test_validation_failure_preserves_unvalidated_state() {
        // Create invalid email
        let email = Email::new("not-an-email".to_string());

        // Validation fails
        let result = email.validate();
        assert!(result.is_err());

        // Original email was moved, so we can't use it again
        // This is correct - failed validation consumes the value
    }

    #[test]
    fn test_methods_work_across_states() {
        let unvalidated = Email::new("user@example.com".to_string());
        assert_eq!(unvalidated.as_str(), "user@example.com");

        let validated = unvalidated.validate().unwrap();
        assert_eq!(validated.as_str(), "user@example.com");
    }

    // Example with struct having multiple fields
    #[derive(Debug)]
    struct User<State = Unvalidated> {
        username: String,
        email: String,
        age: u8,
        _state: PhantomData<State>,
    }

    impl User<Unvalidated> {
        fn new(username: String, email: String, age: u8) -> Self {
            Self {
                username,
                email,
                age,
                _state: PhantomData,
            }
        }

        fn validate(self) -> Result<User<Validated>, ValidationError> {
            let mut errors = ValidationError::default();

            if let Err(e) = validate("username", self.username.as_str(), &rules::min_len(3)) {
                errors.extend(e);
            }
            if let Err(e) = validate("email", self.email.as_str(), &rules::email()) {
                errors.extend(e);
            }
            if let Err(e) = validate("age", &self.age, &rules::range(0, 120)) {
                errors.extend(e);
            }

            if errors.is_empty() {
                Ok(User {
                    username: self.username,
                    email: self.email,
                    age: self.age,
                    _state: PhantomData,
                })
            } else {
                Err(errors)
            }
        }
    }

    impl<State> User<State> {
        fn username(&self) -> &str {
            &self.username
        }
    }

    fn register_user(user: User<Validated>) -> String {
        format!("Registered: {}", user.username())
    }

    #[test]
    fn test_multi_field_validation() {
        let user = User::new("alice".to_string(), "alice@example.com".to_string(), 25);
        let validated = user.validate().expect("Should be valid");
        let result = register_user(validated);
        assert_eq!(result, "Registered: alice");
    }

    #[test]
    fn test_multi_field_validation_failure() {
        // Invalid: username too short, invalid email, age out of range
        let user = User::new("ab".to_string(), "not-email".to_string(), 150);
        let result = user.validate();

        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.violations.len(), 3);
    }

    #[test]
    fn test_partial_validation_failure() {
        // Only username is invalid
        let user = User::new("ab".to_string(), "alice@example.com".to_string(), 25);
        let result = user.validate();

        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.violations.len(), 1);
        assert!(errors.violations[0].path.to_string().contains("username"));
    }
}

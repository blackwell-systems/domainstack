//! Example demonstrating async validation with database uniqueness checks.
//!
//! This example shows how to:
//! - Create async validation rules
//! - Pass database connections through ValidationContext
//! - Check uniqueness constraints asynchronously
//! - Implement AsyncValidate trait manually
//!
//! Run with: cargo run --example async_validation --features async

use async_trait::async_trait;
use domainstack::{AsyncRule, AsyncValidate, RuleContext, ValidationContext, ValidationError};
use std::sync::Arc;

// Mock database for demonstration
#[derive(Clone)]
struct Database {
    registered_emails: Vec<String>,
}

impl Database {
    fn new() -> Self {
        Self {
            registered_emails: vec![
                "admin@example.com".to_string(),
                "user@example.com".to_string(),
            ],
        }
    }

    async fn email_exists(&self, email: &str) -> bool {
        // Simulate async database query
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        self.registered_emails.contains(&email.to_string())
    }

    async fn username_exists(&self, username: &str) -> bool {
        // Simulate async database query
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        username == "admin" || username == "root"
    }
}

// Example: User registration with async validation
struct UserRegistration {
    username: String,
    email: String,
    password: String,
}

#[async_trait]
impl AsyncValidate for UserRegistration {
    async fn validate_async(&self, ctx: &ValidationContext) -> Result<(), ValidationError> {
        let db = ctx
            .get_resource::<Database>("db")
            .expect("Database not in context");

        let mut errors = ValidationError::default();

        // Check email uniqueness
        if db.email_exists(&self.email).await {
            errors.push(
                domainstack::Path::from("email"),
                "email_taken",
                format!("Email '{}' is already registered", self.email),
            );
        }

        // Check username uniqueness
        if db.username_exists(&self.username).await {
            errors.push(
                domainstack::Path::from("username"),
                "username_taken",
                format!("Username '{}' is already taken", self.username),
            );
        }

        // Check password strength (sync check)
        if self.password.len() < 8 {
            errors.push(
                domainstack::Path::from("password"),
                "password_too_short",
                "Password must be at least 8 characters",
            );
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

// Example: Using AsyncRule for field-level validation
fn email_unique() -> AsyncRule<str> {
    AsyncRule::new(|email: &str, ctx: &RuleContext, vctx: &ValidationContext| {
        let db = vctx
            .get_resource::<Database>("db")
            .expect("Database not in context");
        let email = email.to_string();
        let path = ctx.full_path();

        async move {
            if db.email_exists(&email).await {
                ValidationError::single(path, "email_taken", "Email is already registered")
            } else {
                ValidationError::default()
            }
        }
    })
}

#[tokio::main]
async fn main() {
    println!("=== Async Validation Example ===\n");

    // Setup database and context
    let db = Arc::new(Database::new());
    let ctx = ValidationContext::new().with_resource("db", db);

    // Example 1: Successful registration
    println!("Example 1: Successful registration");
    let user1 = UserRegistration {
        username: "newuser".to_string(),
        email: "newuser@example.com".to_string(),
        password: "securepassword123".to_string(),
    };

    match user1.validate_async(&ctx).await {
        Ok(()) => println!("[ok] User registration validated successfully\n"),
        Err(e) => println!("[error] Validation failed:\n{}\n", e),
    }

    // Example 2: Email already taken
    println!("Example 2: Email already taken");
    let user2 = UserRegistration {
        username: "newuser2".to_string(),
        email: "admin@example.com".to_string(), // Already exists
        password: "password123".to_string(),
    };

    match user2.validate_async(&ctx).await {
        Ok(()) => println!("[ok] User registration validated successfully\n"),
        Err(e) => println!("[error] Validation failed:\n{}\n", e),
    }

    // Example 3: Username taken
    println!("Example 3: Username already taken");
    let user3 = UserRegistration {
        username: "admin".to_string(), // Already exists
        email: "new@example.com".to_string(),
        password: "password123".to_string(),
    };

    match user3.validate_async(&ctx).await {
        Ok(()) => println!("[ok] User registration validated successfully\n"),
        Err(e) => println!("[error] Validation failed:\n{}\n", e),
    }

    // Example 4: Multiple errors
    println!("Example 4: Multiple validation errors");
    let user4 = UserRegistration {
        username: "admin".to_string(),         // Already exists
        email: "user@example.com".to_string(), // Already exists
        password: "short".to_string(),         // Too short
    };

    match user4.validate_async(&ctx).await {
        Ok(()) => println!("[ok] User registration validated successfully\n"),
        Err(e) => println!("[error] Validation failed:\n{}\n", e),
    }

    // Example 5: Using AsyncRule directly
    println!("Example 5: Using AsyncRule for email validation");
    let rule = email_unique();
    let rule_ctx = RuleContext::root("email");

    let result = rule.apply("admin@example.com", &rule_ctx, &ctx).await;
    if result.is_empty() {
        println!("[ok] Email is available\n");
    } else {
        println!("[error] Email validation failed:\n{}\n", result);
    }

    let result = rule.apply("available@example.com", &rule_ctx, &ctx).await;
    if result.is_empty() {
        println!("[ok] Email is available\n");
    } else {
        println!("[error] Email validation failed:\n{}\n", result);
    }

    println!("=== All examples completed ===");
}

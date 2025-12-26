//! Example demonstrating async validation with SQLite database.
//!
//! This example shows:
//! - Real async database operations with SQLite
//! - Checking uniqueness constraints (email, username)
//! - Using sqlx for async database queries
//! - Implementing AsyncValidate with real I/O
//!
//! Run with: cargo run --example async_sqlite --features async

use async_trait::async_trait;
use domainstack::{AsyncValidate, Path, ValidationContext, ValidationError};
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use std::sync::Arc;

/// User registration form with async database validation
#[derive(Debug)]
struct UserRegistration {
    username: String,
    email: String,
    password: String,
}

#[async_trait]
impl AsyncValidate for UserRegistration {
    async fn validate_async(&self, ctx: &ValidationContext) -> Result<(), ValidationError> {
        let pool = ctx
            .get_resource::<SqlitePool>("db")
            .expect("Database pool not in context");

        let mut errors = ValidationError::default();

        // Check username length (sync validation)
        if self.username.len() < 3 {
            errors.push(
                Path::from("username"),
                "username_too_short",
                "Username must be at least 3 characters",
            );
        }

        // Check email format (basic sync check)
        if !self.email.contains('@') {
            errors.push(Path::from("email"), "invalid_email", "Email must contain @");
        }

        // Check password strength (sync validation)
        if self.password.len() < 8 {
            errors.push(
                Path::from("password"),
                "password_too_short",
                "Password must be at least 8 characters",
            );
        }

        // Async database checks - only if basic validation passed
        if errors.is_empty() {
            // Check if username already exists (async I/O)
            let username_exists: bool =
                sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE username = ?)")
                    .bind(&self.username)
                    .fetch_one(pool.as_ref())
                    .await
                    .unwrap_or(false);

            if username_exists {
                errors.push(
                    Path::from("username"),
                    "username_taken",
                    format!("Username '{}' is already taken", self.username),
                );
            }

            // Check if email already exists (async I/O)
            let email_exists: bool =
                sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE email = ?)")
                    .bind(&self.email)
                    .fetch_one(pool.as_ref())
                    .await
                    .unwrap_or(false);

            if email_exists {
                errors.push(
                    Path::from("email"),
                    "email_taken",
                    format!("Email '{}' is already registered", self.email),
                );
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

/// Initialize database with schema and sample data
async fn setup_database(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    // Create users table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            username TEXT NOT NULL UNIQUE,
            email TEXT NOT NULL UNIQUE,
            password_hash TEXT NOT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )
        "#,
    )
    .execute(pool)
    .await?;

    // Insert sample data
    sqlx::query("INSERT OR IGNORE INTO users (username, email, password_hash) VALUES (?, ?, ?)")
        .bind("admin")
        .bind("admin@example.com")
        .bind("$2b$12$hashed_password_here")
        .execute(pool)
        .await?;

    sqlx::query("INSERT OR IGNORE INTO users (username, email, password_hash) VALUES (?, ?, ?)")
        .bind("johndoe")
        .bind("john@example.com")
        .bind("$2b$12$another_hashed_password")
        .execute(pool)
        .await?;

    Ok(())
}

/// Insert a validated user into the database
async fn register_user(pool: &SqlitePool, user: &UserRegistration) -> Result<i64, sqlx::Error> {
    let result = sqlx::query("INSERT INTO users (username, email, password_hash) VALUES (?, ?, ?)")
        .bind(&user.username)
        .bind(&user.email)
        .bind(&user.password) // In production, hash the password!
        .execute(pool)
        .await?;

    Ok(result.last_insert_rowid())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Async SQLite Validation Example ===\n");

    // Create in-memory SQLite database
    println!("📦 Setting up SQLite database...");
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect("sqlite::memory:")
        .await?;

    // Initialize schema and data
    setup_database(&pool).await?;
    println!("[ok] Database initialized with sample users\n");

    // Create validation context with database pool
    let ctx = ValidationContext::new().with_resource("db", Arc::new(pool.clone()));

    // Example 1: Successful registration
    println!("Example 1: Valid new user registration");
    let user1 = UserRegistration {
        username: "newuser".to_string(),
        email: "newuser@example.com".to_string(),
        password: "securepass123".to_string(),
    };

    match user1.validate_async(&ctx).await {
        Ok(()) => {
            let user_id = register_user(&pool, &user1).await?;
            println!("[ok] User registered successfully (ID: {})", user_id);
            println!("  Username: {}", user1.username);
            println!("  Email: {}\n", user1.email);
        }
        Err(e) => println!("[error] Validation failed:\n{}\n", e),
    }

    // Example 2: Username already taken
    println!("Example 2: Username already exists in database");
    let user2 = UserRegistration {
        username: "admin".to_string(), // Already exists
        email: "new@example.com".to_string(),
        password: "password123".to_string(),
    };

    match user2.validate_async(&ctx).await {
        Ok(()) => println!("[ok] User registered successfully\n"),
        Err(e) => println!("[error] Validation failed:\n{}\n", e),
    }

    // Example 3: Email already taken
    println!("Example 3: Email already exists in database");
    let user3 = UserRegistration {
        username: "newuser2".to_string(),
        email: "admin@example.com".to_string(), // Already exists
        password: "password123".to_string(),
    };

    match user3.validate_async(&ctx).await {
        Ok(()) => println!("[ok] User registered successfully\n"),
        Err(e) => println!("[error] Validation failed:\n{}\n", e),
    }

    // Example 4: Multiple validation errors
    println!("Example 4: Multiple validation errors (sync + async)");
    let user4 = UserRegistration {
        username: "ab".to_string(),         // Too short (sync)
        email: "invalid-email".to_string(), // No @ symbol (sync)
        password: "short".to_string(),      // Too short (sync)
    };

    match user4.validate_async(&ctx).await {
        Ok(()) => println!("[ok] User registered successfully\n"),
        Err(e) => println!("[error] Validation failed:\n{}\n", e),
    }

    // Example 5: Both username and email taken
    println!("Example 5: Both username and email already taken");
    let user5 = UserRegistration {
        username: "admin".to_string(),         // Exists in DB
        email: "john@example.com".to_string(), // Exists in DB
        password: "password123".to_string(),
    };

    match user5.validate_async(&ctx).await {
        Ok(()) => println!("[ok] User registered successfully\n"),
        Err(e) => println!("[error] Validation failed:\n{}\n", e),
    }

    // Show all users in database
    println!("=== Users in Database ===");
    let users: Vec<(String, String)> =
        sqlx::query_as("SELECT username, email FROM users ORDER BY created_at")
            .fetch_all(&pool)
            .await?;

    for (i, (username, email)) in users.iter().enumerate() {
        println!("{}. {} <{}>", i + 1, username, email);
    }

    println!("\n=== Example completed ===");
    Ok(())
}

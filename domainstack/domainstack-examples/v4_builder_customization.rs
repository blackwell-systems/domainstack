use domainstack::prelude::*;

fn main() {
    println!("=== Builder-Style Rule Customization Demo ===\n");

    // Example 1: Custom error messages
    println!("1. Custom Error Messages:");
    let rule = rules::min_len(5).message("Email too short");
    let err = rule.apply("hi");
    if err.is_empty() {
        println!("  ✓ Valid");
    } else {
        for v in &err.violations {
            println!("  ✗ {}: {}", v.path, v.message);
        }
    }

    // Example 2: Custom error codes
    println!("\n2. Custom Error Codes:");
    let rule = rules::min_len(5).code("email_too_short");
    let err = rule.apply("hi");
    if err.is_empty() {
        println!("  ✓ Valid");
    } else {
        for v in &err.violations {
            println!("  ✗ Code: {}, Message: {}", v.code, v.message);
        }
    }

    // Example 3: Adding metadata
    println!("\n3. Adding Metadata:");
    let rule = rules::min_len(5)
        .meta("hint", "Use format: user@domain.com")
        .meta("field_type", "email");
    let err = rule.apply("hi");
    if err.is_empty() {
        println!("  ✓ Valid");
    } else {
        for v in &err.violations {
            println!("  ✗ {}", v.message);
            println!("     Hint: {}", v.meta.get("hint").map_or("N/A", |s| s));
            println!(
                "     Field Type: {}",
                v.meta.get("field_type").map_or("N/A", |s| s)
            );
        }
    }

    // Example 4: Chaining everything together
    println!("\n4. Chaining Code + Message + Meta:");
    let rule = rules::email()
        .code("invalid_email_format")
        .message("Please provide a valid email address")
        .meta("hint", "Format should be: name@domain.com")
        .meta("required", "true");

    let err = rule.apply("not-an-email");
    if err.is_empty() {
        println!("  ✓ Valid");
    } else {
        for v in &err.violations {
            println!("  ✗ Code: {}", v.code);
            println!("     Message: {}", v.message);
            println!("     Hint: {}", v.meta.get("hint").map_or("N/A", |s| s));
            println!(
                "     Required: {}",
                v.meta.get("required").map_or("N/A", |s| s)
            );
        }
    }

    // Example 5: Combining with rule composition
    println!("\n5. Builder + Composition:");
    let rule = rules::min_len(5)
        .message("Too short")
        .and(rules::max_len(255).message("Too long"))
        .and(rules::email().message("Invalid format"));

    let err = rule.apply("hi");
    if err.is_empty() {
        println!("  ✓ Valid");
    } else {
        println!("  ✗ {} errors:", err.violations.len());
        for v in &err.violations {
            println!("     - {}", v.message);
        }
    }

    // Example 6: Real-world use case
    println!("\n6. Real-World Example (User Registration):");

    #[derive(Debug)]
    struct UserDto {
        username: String,
        email: String,
        password: String,
    }

    let dto = UserDto {
        username: "ab".to_string(),
        email: "invalid".to_string(),
        password: "123".to_string(),
    };

    let mut errors = ValidationError::new();

    // Username validation with custom messages
    let username_rule = rules::alphanumeric()
        .message("Username can only contain letters and numbers")
        .and(
            rules::min_len(3)
                .message("Username must be at least 3 characters")
                .meta("hint", "Try a longer username"),
        )
        .and(rules::max_len(20).message("Username must be at most 20 characters"));

    if let Err(e) = validate("username", dto.username.as_str(), &username_rule) {
        errors.extend(e);
    }

    // Email validation with custom messages
    let email_rule = rules::email()
        .message("Please provide a valid email address")
        .meta("example", "user@example.com");

    if let Err(e) = validate("email", dto.email.as_str(), &email_rule) {
        errors.extend(e);
    }

    // Password validation with custom messages
    let password_rule = rules::min_len(8)
        .message("Password must be at least 8 characters for security")
        .meta("hint", "Use a mix of letters, numbers, and symbols");

    if let Err(e) = validate("password", dto.password.as_str(), &password_rule) {
        errors.extend(e);
    }

    if !errors.is_empty() {
        println!(
            "  ✗ Validation failed with {} errors:",
            errors.violations.len()
        );
        for v in &errors.violations {
            println!("\n     Field: {}", v.path);
            println!("     Error: {}", v.message);
            if let Some(hint) = v.meta.get("hint") {
                println!("     💡 Hint: {}", hint);
            }
            if let Some(example) = v.meta.get("example") {
                println!("     📋 Example: {}", example);
            }
        }
    } else {
        println!("  ✓ All valid");
    }

    println!("\n=== Demo Complete ===");
}

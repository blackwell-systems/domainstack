use domainstack::Validate;
use domainstack_envelope::IntoEnvelopeError;

#[derive(Debug, Validate)]
struct User {
    #[validate(length(min = 1, max = 50))]
    name: String,

    #[validate(range(min = 18, max = 120))]
    age: u8,
}

fn main() {
    println!("=== error-envelope Integration: Basic Example ===\n");

    println!("1. Valid user:");
    let user = User {
        name: "Alice Johnson".to_string(),
        age: 30,
    };

    match user.validate() {
        Ok(_) => println!("   ✓ User is valid\n"),
        Err(e) => {
            let envelope = e.into_envelope_error();
            println!("   ✗ Error envelope:");
            println!("     Status: {}", envelope.status);
            println!("     Message: {}", envelope.message);
            println!("     Retryable: {}\n", envelope.retryable);
        }
    }

    println!("2. Single validation error (name too long):");
    let user = User {
        name: "A".repeat(60),
        age: 30,
    };

    match user.validate() {
        Ok(_) => println!("   ✓ User is valid\n"),
        Err(e) => {
            let envelope = e.into_envelope_error();
            println!("   ✗ Error envelope:");
            println!("     Status: {}", envelope.status);
            println!("     Message: {}", envelope.message);
            println!("     Retryable: {}", envelope.retryable);

            if let Some(details) = &envelope.details {
                println!("\n     Details (JSON):");
                println!("{}", serde_json::to_string_pretty(details).unwrap());
            }
            println!();
        }
    }

    println!("3. Multiple validation errors (empty name + invalid age):");
    let user = User {
        name: String::new(),
        age: 200,
    };

    match user.validate() {
        Ok(_) => println!("   ✓ User is valid\n"),
        Err(e) => {
            let envelope = e.into_envelope_error();
            println!("   ✗ Error envelope:");
            println!("     Status: {}", envelope.status);
            println!("     Message: {}", envelope.message);
            println!("     Retryable: {}", envelope.retryable);

            if let Some(details) = &envelope.details {
                println!("\n     Details (JSON):");
                println!("{}", serde_json::to_string_pretty(details).unwrap());
            }
            println!();
        }
    }

    println!("4. Usage in HTTP handler pattern:");
    println!("   ```rust");
    println!("   async fn create_user(Json(user): Json<User>) -> Result<Json<User>, Error> {{");
    println!("       user.validate()");
    println!("           .map_err(|e| e.into_envelope_error())?;");
    println!("       // ... save user ...");
    println!("       Ok(Json(user))");
    println!("   }}");
    println!("   ```");
    println!("\n   Client receives consistent 400 error with field-level details!");
}

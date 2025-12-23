use domain_model::prelude::*;
use domain_model_derive::Validate;

#[derive(Debug, Validate)]
struct User {
    #[validate(length(min = 1, max = 50))]
    name: String,
    
    #[validate(range(min = 18, max = 120))]
    age: u8,
}

fn main() {
    println!("=== Basic Length and Range Validation ===\n");
    
    println!("1. Valid user:");
    let user = User {
        name: "Alice Johnson".to_string(),
        age: 30,
    };
    match user.validate() {
        Ok(_) => println!("   ✓ User is valid: {:?}\n", user),
        Err(e) => println!("   ✗ Validation errors:\n{}\n", e),
    }
    
    println!("2. Invalid: name too long (60 chars):");
    let user = User {
        name: "A".repeat(60),
        age: 30,
    };
    match user.validate() {
        Ok(_) => println!("   ✓ User is valid\n"),
        Err(e) => println!("   ✗ Validation errors:\n{}\n", e),
    }
    
    println!("3. Invalid: name empty:");
    let user = User {
        name: String::new(),
        age: 30,
    };
    match user.validate() {
        Ok(_) => println!("   ✓ User is valid\n"),
        Err(e) => println!("   ✗ Validation errors:\n{}\n", e),
    }
    
    println!("4. Invalid: age below minimum (17):");
    let user = User {
        name: "Bob Smith".to_string(),
        age: 17,
    };
    match user.validate() {
        Ok(_) => println!("   ✓ User is valid\n"),
        Err(e) => println!("   ✗ Validation errors:\n{}\n", e),
    }
    
    println!("5. Invalid: age above maximum (150):");
    let user = User {
        name: "Charlie Brown".to_string(),
        age: 150,
    };
    match user.validate() {
        Ok(_) => println!("   ✓ User is valid\n"),
        Err(e) => println!("   ✗ Validation errors:\n{}\n", e),
    }
    
    println!("6. Multiple errors (empty name + invalid age):");
    let user = User {
        name: String::new(),
        age: 200,
    };
    match user.validate() {
        Ok(_) => println!("   ✓ User is valid\n"),
        Err(e) => println!("   ✗ Validation errors:\n{}", e),
    }
}

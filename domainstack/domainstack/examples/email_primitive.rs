use domainstack::prelude::*;

#[derive(Debug, Clone)]
pub struct Email(String);

impl Email {
    pub fn new(raw: impl Into<String>) -> Result<Self, ValidationError> {
        let raw = raw.into();
        let rule = rules::email().and(rules::max_len(255));
        validate("email", raw.as_str(), &rule)?;
        Ok(Self(raw))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Validate for Email {
    fn validate(&self) -> Result<(), ValidationError> {
        let rule = rules::email().and(rules::max_len(255));
        validate("email", self.0.as_str(), &rule)
    }
}

fn main() {
    println!("=== Email Primitive Example ===\n");

    println!("1. Valid email:");
    match Email::new("user@example.com") {
        Ok(email) => println!("   Valid: {}", email.as_str()),
        Err(e) => println!("   Error: {}", e),
    }

    println!("\n2. Invalid email (missing @):");
    match Email::new("not-an-email") {
        Ok(_) => println!("   Unexpected success"),
        Err(e) => {
            println!("   Validation failed:");
            for v in &e.violations {
                println!("   [{} {}] {}", v.path, v.code, v.message);
            }
        }
    }

    println!("\n3. Too long email:");
    match Email::new("a".repeat(300) + "@example.com") {
        Ok(_) => println!("   Unexpected success"),
        Err(e) => {
            println!("   Multiple violations:");
            for v in &e.violations {
                println!("   [{} {}] {}", v.path, v.code, v.message);
            }
        }
    }
}

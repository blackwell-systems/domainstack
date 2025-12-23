use domain_model::prelude::*;
use domain_model_derive::Validate;

fn validate_even(value: &u8) -> Result<(), ValidationError> {
    if *value % 2 == 0 {
        Ok(())
    } else {
        Err(ValidationError::single(
            Path::root(),
            "not_even",
            "Must be even",
        ))
    }
}

fn validate_positive_balance(value: &i32) -> Result<(), ValidationError> {
    if *value >= 0 {
        Ok(())
    } else {
        Err(ValidationError::single(
            Path::root(),
            "negative_balance",
            "Balance cannot be negative",
        ))
    }
}

fn validate_strong_password(value: &String) -> Result<(), ValidationError> {
    let mut errors = ValidationError::new();
    
    if !value.chars().any(|c| c.is_uppercase()) {
        errors.push(Path::root(), "no_uppercase", "Must contain uppercase letter");
    }
    if !value.chars().any(|c| c.is_lowercase()) {
        errors.push(Path::root(), "no_lowercase", "Must contain lowercase letter");
    }
    if !value.chars().any(|c| c.is_numeric()) {
        errors.push(Path::root(), "no_digit", "Must contain digit");
    }
    
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

#[derive(Debug, Validate)]
struct EvenNumber {
    #[validate(range(min = 0, max = 100))]
    #[validate(custom = "validate_even")]
    value: u8,
}

#[derive(Debug, Validate)]
struct BankAccount {
    #[validate(length(min = 1, max = 50))]
    account_holder: String,
    
    #[validate(custom = "validate_positive_balance")]
    balance: i32,
}

#[derive(Debug, Validate)]
struct UserAccount {
    #[validate(length(min = 3, max = 20))]
    username: String,
    
    #[validate(length(min = 8, max = 128))]
    #[validate(custom = "validate_strong_password")]
    password: String,
}

fn main() {
    println!("=== Custom Validation Functions ===\n");
    
    println!("1. Valid even number:");
    let num = EvenNumber { value: 42 };
    match num.validate() {
        Ok(_) => println!("   ✓ Number is valid: {:?}\n", num),
        Err(e) => println!("   ✗ Validation errors:\n{}\n", e),
    }
    
    println!("2. Invalid: odd number (custom validation fails):");
    let num = EvenNumber { value: 43 };
    match num.validate() {
        Ok(_) => println!("   ✓ Number is valid\n"),
        Err(e) => {
            println!("   ✗ Validation errors:\n{}", e);
            println!("   Note: Custom function 'validate_even' checked parity\n");
        }
    }
    
    println!("3. Invalid: both range and custom fail:");
    let num = EvenNumber { value: 101 };
    match num.validate() {
        Ok(_) => println!("   ✓ Number is valid\n"),
        Err(e) => {
            println!("   ✗ Validation errors:\n{}", e);
            println!("   Note: Both range (>100) and custom (odd) violations reported\n");
        }
    }
    
    println!("4. Valid bank account:");
    let account = BankAccount {
        account_holder: "Alice Johnson".to_string(),
        balance: 1500,
    };
    match account.validate() {
        Ok(_) => println!("   ✓ Account is valid: {:?}\n", account),
        Err(e) => println!("   ✗ Validation errors:\n{}\n", e),
    }
    
    println!("5. Invalid: negative balance (custom validation):");
    let account = BankAccount {
        account_holder: "Bob Smith".to_string(),
        balance: -500,
    };
    match account.validate() {
        Ok(_) => println!("   ✓ Account is valid\n"),
        Err(e) => {
            println!("   ✗ Validation errors:\n{}", e);
            println!("   Note: Custom validation caught negative balance\n");
        }
    }
    
    println!("6. Valid user with strong password:");
    let user = UserAccount {
        username: "alice99".to_string(),
        password: "SecurePass123".to_string(),
    };
    match user.validate() {
        Ok(_) => println!("   ✓ User is valid: {:?}\n", user),
        Err(e) => println!("   ✗ Validation errors:\n{}\n", e),
    }
    
    println!("7. Invalid: weak password (multiple custom violations):");
    let user = UserAccount {
        username: "bob42".to_string(),
        password: "weakpass".to_string(),
    };
    match user.validate() {
        Ok(_) => println!("   ✓ User is valid\n"),
        Err(e) => {
            println!("   ✗ Validation errors:\n{}", e);
            println!("   Note: Custom function can return multiple violations:");
            println!("     - no_uppercase");
            println!("     - no_digit");
        }
    }
}

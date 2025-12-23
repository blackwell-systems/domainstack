use domainstack::prelude::*;
use domainstack::Validate;

#[derive(Debug, Clone, Validate)]
struct Email {
    #[validate(length(min = 1, max = 255))]
    value: String,
}

#[derive(Debug, Validate)]
struct Guest {
    #[validate(length(min = 1, max = 50))]
    name: String,
    
    #[validate(nested)]
    email: Email,
}

#[derive(Debug, Validate)]
struct Booking {
    #[validate(nested)]
    guest: Guest,
    
    #[validate(range(min = 1, max = 10))]
    guests_count: u8,
}

fn main() {
    println!("=== Nested Validation with Path Prefixing ===\n");
    
    println!("1. Valid booking:");
    let booking = Booking {
        guest: Guest {
            name: "John Doe".to_string(),
            email: Email {
                value: "john@example.com".to_string(),
            },
        },
        guests_count: 2,
    };
    match booking.validate() {
        Ok(_) => println!("   ✓ Booking is valid: {:?}\n", booking),
        Err(e) => println!("   ✗ Validation errors:\n{}\n", e),
    }
    
    println!("2. Invalid: email value is empty (nested error):");
    let booking = Booking {
        guest: Guest {
            name: "Jane Smith".to_string(),
            email: Email {
                value: String::new(),
            },
        },
        guests_count: 2,
    };
    match booking.validate() {
        Ok(_) => println!("   ✓ Booking is valid\n"),
        Err(e) => {
            println!("   ✗ Validation errors:\n{}", e);
            println!("   Note: Error path is 'guest.email.value' - nested structure is preserved!\n");
        }
    }
    
    println!("3. Multiple nested errors:");
    let booking = Booking {
        guest: Guest {
            name: String::new(),
            email: Email {
                value: "a".repeat(300),
            },
        },
        guests_count: 15,
    };
    match booking.validate() {
        Ok(_) => println!("   ✓ Booking is valid\n"),
        Err(e) => {
            println!("   ✗ Validation errors:\n{}", e);
            println!("   Note: All three error paths are preserved:");
            println!("     - guest.name");
            println!("     - guest.email.value");
            println!("     - guests_count");
        }
    }
}

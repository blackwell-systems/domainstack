use domain_model::prelude::*;
use domain_model_derive::Validate;

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

#[test]
fn test_valid_nested() {
    let booking = Booking {
        guest: Guest {
            name: "John Doe".to_string(),
            email: Email {
                value: "john@example.com".to_string(),
            },
        },
        guests_count: 2,
    };
    
    assert!(booking.validate().is_ok());
}

#[test]
fn test_nested_email_error() {
    let booking = Booking {
        guest: Guest {
            name: "John".to_string(),
            email: Email {
                value: "".to_string(),
            },
        },
        guests_count: 2,
    };
    
    let result = booking.validate();
    assert!(result.is_err());
    
    let err = result.unwrap_err();
    assert_eq!(err.violations.len(), 1);
    assert_eq!(err.violations[0].path.to_string(), "guest.email.value");
    assert_eq!(err.violations[0].code, "min_length");
}

#[test]
fn test_multiple_nested_errors() {
    let booking = Booking {
        guest: Guest {
            name: "".to_string(),
            email: Email {
                value: "a".repeat(300),
            },
        },
        guests_count: 15,
    };
    
    let result = booking.validate();
    assert!(result.is_err());
    
    let err = result.unwrap_err();
    assert_eq!(err.violations.len(), 3);
    
    let paths: Vec<String> = err.violations.iter()
        .map(|v| v.path.to_string())
        .collect();
    
    assert!(paths.contains(&"guest.name".to_string()));
    assert!(paths.contains(&"guest.email.value".to_string()));
    assert!(paths.contains(&"guests_count".to_string()));
}

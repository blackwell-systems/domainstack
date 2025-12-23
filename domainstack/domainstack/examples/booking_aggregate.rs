use domainstack::prelude::*;

#[derive(Debug, Clone)]
pub struct Email(String);

impl Email {
    pub fn new(raw: String) -> Result<Self, ValidationError> {
        let rule = rules::email();
        validate("email", raw.as_str(), &rule)?;
        Ok(Self(raw))
    }
}

impl Validate for Email {
    fn validate(&self) -> Result<(), ValidationError> {
        let rule = rules::email();
        validate("email", self.0.as_str(), &rule)
    }
}

#[derive(Debug)]
pub struct Guest {
    pub name: String,
    pub email: Email,
}

impl Guest {
    pub fn new(name: String, email: Email) -> Result<Self, ValidationError> {
        let guest = Self { name, email };
        guest.validate()?;
        Ok(guest)
    }
}

impl Validate for Guest {
    fn validate(&self) -> Result<(), ValidationError> {
        let mut err = ValidationError::default();

        let rule = rules::min_len(1).and(rules::max_len(50));
        if let Err(e) = validate(
            "name",
            self.name.as_str(),
            &rule,
        ) {
            err.extend(e);
        }

        if let Err(e) = self.email.validate() {
            err.merge_prefixed("email", e);
        }

        if err.is_empty() {
            Ok(())
        } else {
            Err(err)
        }
    }
}

#[derive(Debug)]
pub struct BookingRequest {
    pub guest: Guest,
    pub guests_count: u8,
}

impl BookingRequest {
    pub fn new(guest: Guest, guests_count: u8) -> Result<Self, ValidationError> {
        let booking = Self {
            guest,
            guests_count,
        };
        booking.validate()?;
        Ok(booking)
    }
}

impl Validate for BookingRequest {
    fn validate(&self) -> Result<(), ValidationError> {
        let mut err = ValidationError::default();

        let rule = rules::range(1, 10);
        if let Err(e) = validate("guests_count", &self.guests_count, &rule) {
            err.extend(e);
        }

        if let Err(e) = self.guest.validate() {
            err.merge_prefixed("guest", e);
        }

        if err.is_empty() {
            Ok(())
        } else {
            Err(err)
        }
    }
}

fn main() {
    println!("=== Booking Aggregate Example ===\n");

    println!("1. Valid booking:");
    let email = Email::new("john@example.com".to_string()).unwrap();
    let guest = Guest::new("John Doe".to_string(), email).unwrap();
    let booking = BookingRequest::new(guest, 2).unwrap();

    println!("   Valid booking created: {:?}", booking);

    println!("\n2. Invalid booking (empty name, too many guests):");
    if let Ok(email) = Email::new("jane@example.com".to_string()) {
        match Guest::new("".to_string(), email) {
            Ok(_) => println!("   Unexpected success for empty name"),
            Err(e) => {
                println!("   Guest validation failed:");
                for v in &e.violations {
                    println!("     [{} {}] {}", v.path, v.code, v.message);
                }
            }
        }
    }

    println!("\n3. Invalid booking (bad email):");
    match Email::new("not-an-email".to_string()) {
        Ok(_) => println!("   Unexpected success"),
        Err(e) => {
            println!("   Email validation failed:");
            for v in &e.violations {
                println!("     [{} {}] {}", v.path, v.code, v.message);
            }
        }
    }

    println!("\n4. Multiple nested errors:");
    let bad_email = Email::new("test@example.com".to_string()).unwrap();
    let bad_guest = Guest {
        name: "".to_string(),
        email: bad_email,
    };

    match BookingRequest::new(bad_guest, 15) {
        Ok(_) => println!("   Unexpected success"),
        Err(e) => {
            println!(
                "   Booking validation failed with {} errors:",
                e.violations.len()
            );
            for v in &e.violations {
                println!("     [{} {}] {}", v.path, v.code, v.message);
            }

            println!("\n   Field errors map:");
            let map = e.field_errors_map();
            for (field, messages) in map {
                println!("     {}: {:?}", field, messages);
            }
        }
    }
}

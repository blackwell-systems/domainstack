use domainstack::Validate;
use domainstack_envelope::IntoEnvelopeError;

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
struct Room {
    #[validate(range(min = 1, max = 4))]
    adults: u8,

    #[validate(range(min = 0, max = 3))]
    children: u8,
}

#[derive(Debug, Validate)]
struct HotelBooking {
    #[validate(nested)]
    guest: Guest,

    #[validate(each(nested))]
    rooms: Vec<Room>,

    #[validate(range(min = 1, max = 30))]
    nights: u8,
}

fn main() {
    println!("=== error-envelope Integration: Nested & Collection Paths ===\n");

    println!("1. Valid booking:");
    let booking = HotelBooking {
        guest: Guest {
            name: "John Doe".to_string(),
            email: Email {
                value: "john@example.com".to_string(),
            },
        },
        rooms: vec![
            Room {
                adults: 2,
                children: 1,
            },
            Room {
                adults: 1,
                children: 0,
            },
        ],
        nights: 3,
    };

    match booking.validate() {
        Ok(_) => println!("   ✓ Booking is valid\n"),
        Err(e) => {
            let envelope = e.into_envelope_error();
            println!("   ✗ Error: {}\n", envelope.message);
        }
    }

    println!("2. Nested path error (guest.email.value):");
    let booking = HotelBooking {
        guest: Guest {
            name: "Jane Smith".to_string(),
            email: Email {
                value: String::new(), // Empty email
            },
        },
        rooms: vec![Room {
            adults: 2,
            children: 0,
        }],
        nights: 3,
    };

    match booking.validate() {
        Ok(_) => println!("   ✓ Booking is valid\n"),
        Err(e) => {
            let envelope = e.into_envelope_error();
            println!("   ✗ Error envelope:");
            println!("     Status: {}", envelope.status);
            println!("     Message: {}", envelope.message);

            if let Some(details) = &envelope.details {
                println!("\n     Details (JSON):");
                println!("{}", serde_json::to_string_pretty(details).unwrap());
                println!("\n     Note: Path is 'guest.email.value' - full nesting preserved!");
            }
            println!();
        }
    }

    println!("3. Collection path errors with indices:");
    let booking = HotelBooking {
        guest: Guest {
            name: "Bob Johnson".to_string(),
            email: Email {
                value: "bob@example.com".to_string(),
            },
        },
        rooms: vec![
            Room {
                adults: 2,
                children: 1,
            },
            Room {
                adults: 5,
                children: 0,
            }, // Too many adults
            Room {
                adults: 1,
                children: 4,
            }, // Too many children
        ],
        nights: 3,
    };

    match booking.validate() {
        Ok(_) => println!("   ✓ Booking is valid\n"),
        Err(e) => {
            let envelope = e.into_envelope_error();
            println!("   ✗ Error envelope:");
            println!("     Status: {}", envelope.status);
            println!("     Message: {}", envelope.message);

            if let Some(details) = &envelope.details {
                println!("\n     Details (JSON):");
                println!("{}", serde_json::to_string_pretty(details).unwrap());
                println!("\n     Note: Paths include array indices:");
                println!("       - rooms[1].adults");
                println!("       - rooms[2].children");
            }
            println!();
        }
    }

    println!("4. Multiple errors across nested and collection fields:");
    let booking = HotelBooking {
        guest: Guest {
            name: String::new(), // Empty name
            email: Email {
                value: "a".repeat(300), // Too long
            },
        },
        rooms: vec![
            Room {
                adults: 0,
                children: 0,
            }, // No guests
            Room {
                adults: 5,
                children: 4,
            }, // Both out of range
        ],
        nights: 50, // Too many nights
    };

    match booking.validate() {
        Ok(_) => println!("   ✓ Booking is valid\n"),
        Err(e) => {
            let envelope = e.into_envelope_error();
            println!("   ✗ Error envelope:");
            println!("     Status: {}", envelope.status);
            println!("     Message: {}", envelope.message);

            if let Some(details) = &envelope.details {
                println!("\n     Details (JSON):");
                println!("{}", serde_json::to_string_pretty(details).unwrap());

                println!("\n     Field paths in this error:");
                if let Some(fields) = details["fields"].as_object() {
                    for path in fields.keys() {
                        println!("       - {}", path);
                    }
                }
            }
            println!(
                "\n     All paths preserved with correct structure for client-side rendering!"
            );
        }
    }
}

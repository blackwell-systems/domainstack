use domainstack::Validate;

#[derive(Debug, Validate)]
struct Room {
    #[validate(range(min = 1, max = 4))]
    adults: u8,

    #[validate(range(min = 0, max = 3))]
    children: u8,
}

#[derive(Debug, Validate)]
struct HotelBooking {
    #[validate(length(min = 1))]
    guest_name: String,

    #[validate(each(nested))]
    rooms: Vec<Room>,
}

#[derive(Debug, Validate)]
struct TagList {
    #[validate(each(length(min = 3, max = 10)))]
    tags: Vec<String>,
}

fn main() {
    println!("=== Collection Validation with each() ===\n");

    println!("1. Valid hotel booking with multiple rooms:");
    let booking = HotelBooking {
        guest_name: "Alice Cooper".to_string(),
        rooms: vec![
            Room {
                adults: 2,
                children: 1,
            },
            Room {
                adults: 1,
                children: 0,
            },
            Room {
                adults: 3,
                children: 2,
            },
        ],
    };
    match booking.validate() {
        Ok(_) => println!("   ✓ Booking is valid: {:?}\n", booking),
        Err(e) => println!("   ✗ Validation errors:\n{}\n", e),
    }

    println!("2. Invalid: rooms with out-of-range values:");
    let booking = HotelBooking {
        guest_name: "Bob Johnson".to_string(),
        rooms: vec![
            Room {
                adults: 2,
                children: 1,
            },
            Room {
                adults: 5,
                children: 0,
            },
            Room {
                adults: 1,
                children: 4,
            },
        ],
    };
    match booking.validate() {
        Ok(_) => println!("   ✓ Booking is valid\n"),
        Err(e) => {
            println!("   ✗ Validation errors:\n{}", e);
            println!("   Note: Errors include array indices in paths:");
            println!("     - rooms[1].adults (5 exceeds max of 4)");
            println!("     - rooms[2].children (4 exceeds max of 3)\n");
        }
    }

    println!("3. Valid tag list with primitive validation:");
    let tags = TagList {
        tags: vec![
            "rust".to_string(),
            "validation".to_string(),
            "domain".to_string(),
        ],
    };
    match tags.validate() {
        Ok(_) => println!("   ✓ Tags are valid: {:?}\n", tags),
        Err(e) => println!("   ✗ Validation errors:\n{}\n", e),
    }

    println!("4. Invalid: tags with length violations:");
    let tags = TagList {
        tags: vec![
            "valid".to_string(),
            "x".to_string(),
            "toolongstring".to_string(),
        ],
    };
    match tags.validate() {
        Ok(_) => println!("   ✓ Tags are valid\n"),
        Err(e) => {
            println!("   ✗ Validation errors:\n{}", e);
            println!("   Note: each() works with primitive rules too:");
            println!("     - tags[1] (too short: 1 char < 3 min)");
            println!("     - tags[2] (too long: 13 chars > 10 max)");
        }
    }
}

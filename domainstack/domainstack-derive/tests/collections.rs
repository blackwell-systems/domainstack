use domainstack::prelude::*;
use domainstack_derive::Validate;

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

#[test]
fn test_valid_collection() {
    let booking = HotelBooking {
        guest_name: "John Doe".to_string(),
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
    };

    assert!(booking.validate().is_ok());
}

#[test]
fn test_collection_with_invalid_items() {
    let booking = HotelBooking {
        guest_name: "John Doe".to_string(),
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

    let result = booking.validate();
    assert!(result.is_err());

    let err = result.unwrap_err();
    assert_eq!(err.violations.len(), 2);

    assert_eq!(err.violations[0].path.to_string(), "rooms[1].adults");
    assert_eq!(err.violations[0].code, "out_of_range");

    assert_eq!(err.violations[1].path.to_string(), "rooms[2].children");
    assert_eq!(err.violations[1].code, "out_of_range");
}

// Note: The Validate derive macro supports each(nested) for nested validation,
// but does not support each(rule) for primitive types.
// For collection-level validation, use min_items/max_items/unique.
// For validating primitive items, implement custom validation or use manual validation.

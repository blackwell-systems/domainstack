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

#[test]
fn test_each_with_primitive_rules() {
    #[derive(Debug, Validate)]
    struct TagList {
        #[validate(each(length(min = 1, max = 20)))]
        tags: Vec<String>,
    }

    let tags = TagList {
        tags: vec![
            "rust".to_string(),
            "validation".to_string(),
            "domain".to_string(),
        ],
    };

    assert!(tags.validate().is_ok());
}

#[test]
fn test_each_length_violation() {
    #[derive(Debug, Validate)]
    struct TagList {
        #[validate(each(length(min = 3, max = 10)))]
        tags: Vec<String>,
    }

    let tags = TagList {
        tags: vec![
            "valid".to_string(),
            "x".to_string(),
            "toolongstring".to_string(),
        ],
    };

    let result = tags.validate();
    assert!(result.is_err());

    let err = result.unwrap_err();
    assert_eq!(err.violations.len(), 2);

    assert_eq!(err.violations[0].path.to_string(), "tags[1]");
    assert_eq!(err.violations[0].code, "min_length");

    assert_eq!(err.violations[1].path.to_string(), "tags[2]");
    assert_eq!(err.violations[1].code, "max_length");
}

#[test]
fn test_each_with_rich_syntax() {
    #[derive(Debug, Validate)]
    struct UrlList {
        #[validate(each(url))]
        urls: Vec<String>,
    }

    let urls = UrlList {
        urls: vec![
            "https://example.com".to_string(),
            "https://rust-lang.org".to_string(),
        ],
    };

    assert!(urls.validate().is_ok());
}

#[test]
fn test_each_email() {
    #[derive(Debug, Validate)]
    struct EmailList {
        #[validate(each(email))]
        emails: Vec<String>,
    }

    let emails = EmailList {
        emails: vec![
            "alice@example.com".to_string(),
            "bob@example.com".to_string(),
        ],
    };

    assert!(emails.validate().is_ok());

    let invalid = EmailList {
        emails: vec!["alice@example.com".to_string(), "not-an-email".to_string()],
    };

    let result = invalid.validate();
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.violations.len(), 1);
    assert_eq!(err.violations[0].path.to_string(), "emails[1]");
}

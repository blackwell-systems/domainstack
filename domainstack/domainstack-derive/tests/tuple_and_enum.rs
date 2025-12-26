//! Tests for tuple struct (newtype) and enum validation derive support

use domainstack::Validate;

// =============================================================================
// Tuple Struct (Newtype) Tests
// =============================================================================

/// Simple newtype with email validation
#[derive(Debug, Validate)]
struct Email(#[validate(email)] String);

#[test]
fn test_newtype_email_valid() {
    let email = Email("user@example.com".to_string());
    assert!(email.validate().is_ok());
}

#[test]
fn test_newtype_email_invalid() {
    let email = Email("not-an-email".to_string());
    let result = email.validate();
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.violations.len(), 1);
    assert_eq!(err.violations[0].path.to_string(), "0");
    assert_eq!(err.violations[0].code, "invalid_email");
}

/// Newtype with range validation
#[derive(Debug, Validate)]
struct Age(#[validate(range(min = 0, max = 150))] u8);

#[test]
fn test_newtype_range_valid() {
    let age = Age(25);
    assert!(age.validate().is_ok());
}

#[test]
fn test_newtype_range_invalid() {
    let age = Age(200);
    let result = age.validate();
    assert!(result.is_err());
}

/// Newtype with length validation
#[derive(Debug, Validate)]
struct Username(#[validate(length(min = 3, max = 20))] String);

#[test]
fn test_newtype_length_valid() {
    let username = Username("alice".to_string());
    assert!(username.validate().is_ok());
}

#[test]
fn test_newtype_length_too_short() {
    let username = Username("ab".to_string());
    let result = username.validate();
    assert!(result.is_err());
}

#[test]
fn test_newtype_length_too_long() {
    let username = Username("a".repeat(25));
    let result = username.validate();
    assert!(result.is_err());
}

/// Tuple struct with multiple fields
#[derive(Debug, Validate)]
struct Coordinate(
    #[validate(range(min = -180, max = 180))] i16,
    #[validate(range(min = -90, max = 90))] i16,
);

#[test]
fn test_multi_field_tuple_valid() {
    let coord = Coordinate(45, 30);
    assert!(coord.validate().is_ok());
}

#[test]
fn test_multi_field_tuple_first_invalid() {
    let coord = Coordinate(200, 30);
    let result = coord.validate();
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.violations.len(), 1);
    assert_eq!(err.violations[0].path.to_string(), "0");
}

#[test]
fn test_multi_field_tuple_second_invalid() {
    let coord = Coordinate(45, 100);
    let result = coord.validate();
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.violations.len(), 1);
    assert_eq!(err.violations[0].path.to_string(), "1");
}

#[test]
fn test_multi_field_tuple_both_invalid() {
    let coord = Coordinate(200, 100);
    let result = coord.validate();
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.violations.len(), 2);
}

/// Unit struct (no fields)
#[derive(Debug, Validate)]
struct UnitStruct;

#[test]
fn test_unit_struct_always_valid() {
    let unit = UnitStruct;
    assert!(unit.validate().is_ok());
}

// =============================================================================
// Enum Tests
// =============================================================================

/// Enum with unit variants only
#[derive(Debug, Validate)]
enum Status {
    Active,
    Inactive,
    Pending,
}

#[test]
fn test_enum_unit_variants_valid() {
    assert!(Status::Active.validate().is_ok());
    assert!(Status::Inactive.validate().is_ok());
    assert!(Status::Pending.validate().is_ok());
}

/// Enum with struct variants
#[derive(Debug, Validate)]
enum PaymentMethod {
    Card {
        #[validate(length(min = 13, max = 19))]
        number: String,
        #[validate(range(min = 1, max = 12))]
        exp_month: u8,
        #[validate(range(min = 2024, max = 2050))]
        exp_year: u16,
    },
    BankTransfer {
        #[validate(alphanumeric)]
        account_number: String,
        #[validate(length(min = 6, max = 11))]
        routing_number: String,
    },
    Cash,
}

#[test]
fn test_enum_struct_variant_valid() {
    let card = PaymentMethod::Card {
        number: "4111111111111111".to_string(),
        exp_month: 12,
        exp_year: 2025,
    };
    assert!(card.validate().is_ok());
}

#[test]
fn test_enum_struct_variant_invalid_number() {
    let card = PaymentMethod::Card {
        number: "123".to_string(), // Too short
        exp_month: 12,
        exp_year: 2025,
    };
    let result = card.validate();
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.violations[0].path.to_string(), "number");
}

#[test]
fn test_enum_struct_variant_invalid_month() {
    let card = PaymentMethod::Card {
        number: "4111111111111111".to_string(),
        exp_month: 13, // Invalid month
        exp_year: 2025,
    };
    let result = card.validate();
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.violations[0].path.to_string(), "exp_month");
}

#[test]
fn test_enum_struct_variant_multiple_errors() {
    let card = PaymentMethod::Card {
        number: "123".to_string(), // Too short
        exp_month: 13,             // Invalid
        exp_year: 2020,            // Too old
    };
    let result = card.validate();
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.violations.len(), 3);
}

#[test]
fn test_enum_bank_transfer_valid() {
    let transfer = PaymentMethod::BankTransfer {
        account_number: "12345678".to_string(),
        routing_number: "021000021".to_string(),
    };
    assert!(transfer.validate().is_ok());
}

#[test]
fn test_enum_bank_transfer_invalid() {
    let transfer = PaymentMethod::BankTransfer {
        account_number: "invalid!@#".to_string(), // Not alphanumeric
        routing_number: "12345".to_string(),      // Too short
    };
    let result = transfer.validate();
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.violations.len(), 2);
}

#[test]
fn test_enum_unit_variant_always_valid() {
    let cash = PaymentMethod::Cash;
    assert!(cash.validate().is_ok());
}

/// Enum with tuple variants
#[derive(Debug, Validate)]
enum Identifier {
    Email(#[validate(email)] String),
    Phone(#[validate(length(min = 10, max = 15))] String),
    UserId(#[validate(range(min = 1, max = 999999))] u32),
    Anonymous,
}

#[test]
fn test_enum_tuple_variant_email_valid() {
    let id = Identifier::Email("user@example.com".to_string());
    assert!(id.validate().is_ok());
}

#[test]
fn test_enum_tuple_variant_email_invalid() {
    let id = Identifier::Email("not-an-email".to_string());
    let result = id.validate();
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.violations[0].path.to_string(), "0");
}

#[test]
fn test_enum_tuple_variant_phone_valid() {
    let id = Identifier::Phone("1234567890".to_string());
    assert!(id.validate().is_ok());
}

#[test]
fn test_enum_tuple_variant_phone_invalid() {
    let id = Identifier::Phone("123".to_string()); // Too short
    let result = id.validate();
    assert!(result.is_err());
}

#[test]
fn test_enum_tuple_variant_userid_valid() {
    let id = Identifier::UserId(12345);
    assert!(id.validate().is_ok());
}

#[test]
fn test_enum_tuple_variant_userid_invalid() {
    let id = Identifier::UserId(0); // Below minimum
    let result = id.validate();
    assert!(result.is_err());
}

#[test]
fn test_enum_tuple_variant_anonymous_valid() {
    let id = Identifier::Anonymous;
    assert!(id.validate().is_ok());
}

/// Mixed enum with all variant types
#[derive(Debug, Validate)]
enum ContactInfo {
    // Unit variant
    None,
    // Tuple variant
    Email(#[validate(email)] String),
    // Struct variant
    Address {
        #[validate(min_len = 1)]
        street: String,
        #[validate(length(min = 2, max = 50))]
        city: String,
        #[validate(length(min = 2, max = 10))]
        postal_code: String,
    },
}

#[test]
fn test_mixed_enum_none() {
    assert!(ContactInfo::None.validate().is_ok());
}

#[test]
fn test_mixed_enum_email_valid() {
    let contact = ContactInfo::Email("test@example.com".to_string());
    assert!(contact.validate().is_ok());
}

#[test]
fn test_mixed_enum_email_invalid() {
    let contact = ContactInfo::Email("invalid".to_string());
    assert!(contact.validate().is_err());
}

#[test]
fn test_mixed_enum_address_valid() {
    let contact = ContactInfo::Address {
        street: "123 Main St".to_string(),
        city: "New York".to_string(),
        postal_code: "10001".to_string(),
    };
    assert!(contact.validate().is_ok());
}

#[test]
fn test_mixed_enum_address_invalid() {
    let contact = ContactInfo::Address {
        street: "".to_string(),                 // Empty, fails min_len = 1
        city: "X".to_string(),                  // Too short
        postal_code: "12345678901".to_string(), // Too long
    };
    let result = contact.validate();
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.violations.len(), 3);
}

// =============================================================================
// Edge Cases
// =============================================================================

/// Newtype with alphanumeric validation
#[derive(Debug, Validate)]
struct AlphanumericId(#[validate(alphanumeric)] String);

#[test]
fn test_newtype_alphanumeric_valid() {
    let id = AlphanumericId("abc123".to_string());
    assert!(id.validate().is_ok());
}

#[test]
fn test_newtype_alphanumeric_invalid() {
    let id = AlphanumericId("abc-123".to_string());
    assert!(id.validate().is_err());
}

/// Newtype with URL validation
#[derive(Debug, Validate)]
struct WebUrl(#[validate(url)] String);

#[test]
fn test_newtype_url_valid() {
    let url = WebUrl("https://example.com".to_string());
    assert!(url.validate().is_ok());
}

#[test]
fn test_newtype_url_invalid() {
    let url = WebUrl("not a url".to_string());
    assert!(url.validate().is_err());
}

/// Newtype with min_len validation
#[derive(Debug, Validate)]
struct NonEmptyString(#[validate(min_len = 1)] String);

#[test]
fn test_newtype_min_len_valid() {
    let s = NonEmptyString("hello".to_string());
    assert!(s.validate().is_ok());
}

#[test]
fn test_newtype_min_len_invalid() {
    let s = NonEmptyString("".to_string());
    assert!(s.validate().is_err());
}

/// Newtype with numeric validation
#[derive(Debug, Validate)]
struct PositiveNumber(#[validate(positive)] i32);

#[test]
fn test_newtype_positive_valid() {
    let n = PositiveNumber(42);
    assert!(n.validate().is_ok());
}

#[test]
fn test_newtype_positive_invalid() {
    let n = PositiveNumber(-5);
    assert!(n.validate().is_err());
}

/// Newtype with non_zero validation
#[derive(Debug, Validate)]
struct NonZeroValue(#[validate(non_zero)] i32);

#[test]
fn test_newtype_non_zero_valid() {
    assert!(NonZeroValue(1).validate().is_ok());
    assert!(NonZeroValue(-1).validate().is_ok());
}

#[test]
fn test_newtype_non_zero_invalid() {
    assert!(NonZeroValue(0).validate().is_err());
}

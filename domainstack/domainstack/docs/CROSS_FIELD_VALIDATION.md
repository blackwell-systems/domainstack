# Cross-Field Validation

**Complete guide to validating relationships between fields: date ranges, password confirmation, conditional requirements, and complex business rules.**

## Table of Contents

- [Overview](#overview)
- [Derive Macro Approach](#derive-macro-approach)
- [Manual Implementation](#manual-implementation)
- [Common Patterns](#common-patterns)
- [Conditional Cross-Field Validation](#conditional-cross-field-validation)
- [Complex Business Rules](#complex-business-rules)
- [Error Handling](#error-handling)
- [Best Practices](#best-practices)

## Overview

Cross-field validation enforces relationships between multiple fields in a struct. Unlike single-field rules (email format, string length), cross-field validation answers questions like:

- Is `check_out` after `check_in`?
- Does `password` match `password_confirmation`?
- Is `discount` only applied when `total > 100`?
- Is `shipping_address` required when `requires_shipping` is true?

## Derive Macro Approach

Use struct-level `#[validate(...)]` attributes for declarative cross-field validation.

### Basic Syntax

```rust
use domainstack::prelude::*;
use chrono::{DateTime, Utc};

#[derive(Validate)]
#[validate(
    check = "self.end_date > self.start_date",
    code = "invalid_date_range",
    message = "End date must be after start date"
)]
struct DateRange {
    start_date: DateTime<Utc>,
    end_date: DateTime<Utc>,
}
```

**Parameters:**

| Parameter | Required | Description |
|-----------|----------|-------------|
| `check` | Yes | Rust expression that evaluates to `bool`. Use `self.` to reference fields. |
| `code` | Yes | Machine-readable error code (used by frontends, logging, i18n) |
| `message` | Yes | Human-readable error message |
| `when` | No | Condition that must be true for validation to run |

### Date Range Validation

The most common cross-field validation pattern:

```rust
use domainstack::prelude::*;
use chrono::{DateTime, Utc, Duration};

#[derive(Validate)]
#[validate(
    check = "self.end_date > self.start_date",
    code = "invalid_date_range",
    message = "End date must be after start date"
)]
struct EventBooking {
    #[validate(future)]
    start_date: DateTime<Utc>,

    #[validate(future)]
    end_date: DateTime<Utc>,

    #[validate(length(min = 1, max = 100))]
    event_name: String,
}

// Usage
let booking = EventBooking {
    start_date: Utc::now() + Duration::days(1),
    end_date: Utc::now() + Duration::days(30),
    event_name: "Conference".to_string(),
};
booking.validate()?;  // [ok] Valid

let invalid = EventBooking {
    start_date: Utc::now() + Duration::days(30),
    end_date: Utc::now() + Duration::days(1),  // Before start!
    event_name: "Conference".to_string(),
};
invalid.validate()?;  // [error] Error: invalid_date_range
```

### Password Confirmation

```rust
#[derive(Validate)]
#[validate(
    check = "self.password == self.password_confirmation",
    code = "password_mismatch",
    message = "Passwords do not match"
)]
struct PasswordChange {
    #[validate(length(min = 8, max = 128))]
    password: String,

    password_confirmation: String,
}

// Usage
let change = PasswordChange {
    password: "SecureP@ss123".to_string(),
    password_confirmation: "SecureP@ss123".to_string(),
};
change.validate()?;  // [ok] Valid

let mismatch = PasswordChange {
    password: "SecureP@ss123".to_string(),
    password_confirmation: "DifferentPass".to_string(),
};
mismatch.validate()?;  // [error] Error: password_mismatch
```

### Multiple Cross-Field Rules

Stack multiple struct-level validations:

```rust
#[derive(Validate)]
#[validate(
    check = "self.end_date > self.start_date",
    code = "invalid_date_range",
    message = "End date must be after start date"
)]
#[validate(
    check = "self.total >= self.minimum_order",
    code = "below_minimum",
    message = "Order total is below minimum"
)]
#[validate(
    check = "self.discount <= self.total * 0.5",
    code = "discount_too_high",
    message = "Discount cannot exceed 50% of total"
)]
struct Order {
    start_date: DateTime<Utc>,
    end_date: DateTime<Utc>,
    total: f64,
    minimum_order: f64,
    discount: f64,
}
```

**Execution order:**
1. All field-level validations run first
2. All struct-level `#[validate(check = "...")]` rules run after
3. All violations accumulate (fail-slow)

## Manual Implementation

For complex business logic that doesn't fit in a single expression, implement `Validate` manually.

### When to Use Manual Implementation

- Complex calculations (e.g., duration checks)
- Multiple related checks with shared logic
- Dynamic error messages based on calculated values
- Early returns for performance
- Conditional validation based on complex state

### Basic Manual Cross-Field Validation

```rust
use domainstack::prelude::*;
use chrono::{DateTime, Utc, Duration};

pub struct EventBooking {
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub event_name: String,
}

impl Validate for EventBooking {
    fn validate(&self) -> Result<(), ValidationError> {
        let mut err = ValidationError::new();

        // Field-level validation
        if let Err(e) = validate("event_name", self.event_name.as_str(),
                                 &rules::length(1, 100)) {
            err.extend(e);
        }

        // Cross-field: end must be after start
        if self.end_date <= self.start_date {
            err.push(
                "end_date",
                "invalid_date_range",
                "End date must be after start date"
            );
        }

        // Cross-field: minimum duration (1 day)
        let duration = self.end_date.signed_duration_since(self.start_date);
        if duration.num_days() < 1 {
            err.push(
                "end_date",
                "duration_too_short",
                "Event must be at least 1 day long"
            );
        }

        // Cross-field: maximum duration (30 days)
        if duration.num_days() > 30 {
            err.push(
                "end_date",
                "duration_too_long",
                format!("Event cannot exceed 30 days (got {} days)", duration.num_days())
            );
        }

        if err.is_empty() { Ok(()) } else { Err(err) }
    }
}
```

### Combining Derive and Manual

Use derive for simple rules, manual for complex ones:

```rust
// Use derive for field-level rules
#[derive(Validate)]
pub struct Order {
    #[validate(positive)]
    pub total: f64,

    #[validate(range(min = 0.0, max = 100.0))]
    pub discount_percent: f64,

    #[validate(min_items = 1)]
    pub items: Vec<OrderItem>,
}

// Override with manual for cross-field
impl Order {
    pub fn validate_business_rules(&self) -> Result<(), ValidationError> {
        let mut err = ValidationError::new();

        // Run derive validation first
        if let Err(e) = self.validate() {
            err.extend(e);
        }

        // Add cross-field business rules
        let discount_amount = self.total * (self.discount_percent / 100.0);
        if discount_amount > 50.0 {
            err.push(
                "discount_percent",
                "discount_exceeds_limit",
                format!("Discount amount ${:.2} exceeds maximum of $50", discount_amount)
            );
        }

        if err.is_empty() { Ok(()) } else { Err(err) }
    }
}
```

## Common Patterns

### Date Range with Minimum/Maximum Duration

```rust
#[derive(Validate)]
#[validate(
    check = "self.check_out > self.check_in",
    code = "invalid_date_range",
    message = "Check-out must be after check-in"
)]
struct HotelBooking {
    check_in: NaiveDate,
    check_out: NaiveDate,
}

impl HotelBooking {
    pub fn validate_with_duration_limits(&self) -> Result<(), ValidationError> {
        let mut err = ValidationError::new();

        // Run derive validation
        if let Err(e) = self.validate() {
            err.extend(e);
        }

        // Duration checks
        let nights = (self.check_out - self.check_in).num_days();

        if nights < 1 {
            err.push("check_out", "minimum_stay", "Minimum stay is 1 night");
        }

        if nights > 30 {
            err.push("check_out", "maximum_stay", "Maximum stay is 30 nights");
        }

        if err.is_empty() { Ok(()) } else { Err(err) }
    }
}
```

### Numeric Comparison

```rust
#[derive(Validate)]
#[validate(
    check = "self.max_value >= self.min_value",
    code = "invalid_range",
    message = "Maximum must be greater than or equal to minimum"
)]
struct PriceRange {
    #[validate(positive)]
    min_value: f64,

    #[validate(positive)]
    max_value: f64,
}
```

### String Matching

```rust
#[derive(Validate)]
#[validate(
    check = "self.email == self.email_confirmation",
    code = "email_mismatch",
    message = "Email addresses do not match"
)]
struct AccountCreation {
    #[validate(email)]
    email: String,

    email_confirmation: String,
}
```

### Collection Size Dependencies

```rust
#[derive(Validate)]
#[validate(
    check = "self.rooms.len() <= self.max_rooms",
    code = "too_many_rooms",
    message = "Number of rooms exceeds limit"
)]
struct Reservation {
    #[validate(each(nested))]
    rooms: Vec<Room>,

    #[validate(range(min = 1, max = 10))]
    max_rooms: usize,
}
```

## Conditional Cross-Field Validation

Use the `when` parameter to conditionally run cross-field validation.

### Basic Conditional Validation

```rust
#[derive(Validate)]
#[validate(
    check = "self.total >= self.minimum_order",
    code = "below_minimum",
    message = "Order total is below minimum",
    when = "self.requires_minimum"
)]
struct FlexibleOrder {
    total: f64,
    minimum_order: f64,
    requires_minimum: bool,
}

// Usage
let order = FlexibleOrder {
    total: 50.0,
    minimum_order: 100.0,
    requires_minimum: false,  // Condition is false, validation skipped
};
order.validate()?;  // [ok] Valid - condition not met

let required = FlexibleOrder {
    total: 50.0,
    minimum_order: 100.0,
    requires_minimum: true,  // Condition is true, validation runs
};
required.validate()?;  // [error] Error: below_minimum
```

### Multiple Conditional Rules

```rust
#[derive(Validate)]
#[validate(
    check = "self.shipping_address.is_some()",
    code = "shipping_required",
    message = "Shipping address is required for physical products",
    when = "self.is_physical_product"
)]
#[validate(
    check = "self.download_url.is_some()",
    code = "download_required",
    message = "Download URL is required for digital products",
    when = "!self.is_physical_product"
)]
struct ProductOrder {
    is_physical_product: bool,
    shipping_address: Option<Address>,
    download_url: Option<String>,
}
```

### Enum-Based Conditions

```rust
#[derive(Validate)]
#[validate(
    check = "self.tracking_number.is_some()",
    code = "tracking_required",
    message = "Tracking number required for shipped orders",
    when = "matches!(self.status, OrderStatus::Shipped | OrderStatus::Delivered)"
)]
struct TrackedOrder {
    status: OrderStatus,
    tracking_number: Option<String>,
}

enum OrderStatus {
    Pending,
    Processing,
    Shipped,
    Delivered,
}
```

## Complex Business Rules

Some validation requires calculations or multi-step logic beyond what the derive macro supports.

### Discount Validation with Calculations

```rust
impl Validate for DiscountedOrder {
    fn validate(&self) -> Result<(), ValidationError> {
        let mut err = ValidationError::new();

        // Calculate effective discount
        let discount_amount = self.total * (self.discount_percent / 100.0);
        let final_total = self.total - discount_amount;

        // Rule 1: Discount cannot exceed 50% of total
        if self.discount_percent > 50.0 {
            err.push(
                "discount_percent",
                "discount_too_high",
                "Discount cannot exceed 50%"
            );
        }

        // Rule 2: Final total must be at least $10
        if final_total < 10.0 {
            err.push(
                "discount_percent",
                "final_total_too_low",
                format!("Final total ${:.2} is below minimum of $10", final_total)
            );
        }

        // Rule 3: Premium discounts require minimum order
        if self.is_premium_discount && self.total < 100.0 {
            err.push(
                "discount_percent",
                "premium_minimum_order",
                "Premium discounts require orders of $100 or more"
            );
        }

        if err.is_empty() { Ok(()) } else { Err(err) }
    }
}
```

### Multi-Field Inventory Validation

```rust
impl Validate for WarehouseTransfer {
    fn validate(&self) -> Result<(), ValidationError> {
        let mut err = ValidationError::new();

        // Can't transfer to same warehouse
        if self.source_warehouse == self.destination_warehouse {
            err.push(
                "destination_warehouse",
                "same_warehouse",
                "Cannot transfer to the same warehouse"
            );
        }

        // Quantity must be positive
        if self.quantity <= 0 {
            err.push("quantity", "invalid_quantity", "Quantity must be positive");
        }

        // Transfer date must be in the future
        if self.scheduled_date <= Utc::now() {
            err.push(
                "scheduled_date",
                "past_date",
                "Transfer must be scheduled for a future date"
            );
        }

        // Express transfers have quantity limits
        if self.is_express && self.quantity > 100 {
            err.push(
                "quantity",
                "express_limit",
                "Express transfers limited to 100 units"
            );
        }

        if err.is_empty() { Ok(()) } else { Err(err) }
    }
}
```

## Error Handling

### Error Path Strategy

For cross-field errors, choose the most relevant field for the error path:

```rust
// Date range: attach error to end_date (the field being compared)
err.push("end_date", "invalid_date_range", "End must be after start");

// Password match: attach to confirmation field
err.push("password_confirmation", "mismatch", "Passwords do not match");

// Calculation error: attach to the input field
err.push("discount_percent", "final_total_too_low", "Discount too high");
```

### Dynamic Error Messages

Include calculated values in error messages:

```rust
let duration = (self.end_date - self.start_date).num_days();
err.push(
    "end_date",
    "duration_too_long",
    format!("Duration of {} days exceeds maximum of 30 days", duration)
);
```

### Multiple Related Errors

Group related errors logically:

```rust
// Check minimum/maximum range
if self.min_value > self.max_value {
    // Attach to both fields for clear UI feedback
    err.push("min_value", "range_inverted", "Minimum exceeds maximum");
    err.push("max_value", "range_inverted", "Maximum is below minimum");
}
```

## Best Practices

### 1. Use Derive for Simple Comparisons

```rust
// GOOD: Simple comparison in derive
#[validate(
    check = "self.end > self.start",
    code = "invalid_range",
    message = "End must be after start"
)]

// [x] BAD: Complex logic in derive (hard to read/debug)
#[validate(
    check = "self.items.iter().map(|i| i.price).sum::<f64>() == self.total",
    ...
)]
```

### 2. Use Manual for Calculations

```rust
// GOOD: Complex logic in manual implementation
impl Validate for Order {
    fn validate(&self) -> Result<(), ValidationError> {
        let calculated_total: f64 = self.items.iter().map(|i| i.price).sum();
        if (self.total - calculated_total).abs() > 0.01 {
            // ...
        }
    }
}
```

### 3. Include Context in Error Messages

```rust
// GOOD: Specific, actionable message
err.push(
    "end_date",
    "duration_too_short",
    format!("Stay of {} nights is below minimum of 2 nights", nights)
);

// [x] BAD: Generic message
err.push("end_date", "invalid", "Invalid date");
```

### 4. Choose Meaningful Error Codes

```rust
// GOOD: Specific, machine-readable codes
err.push("password_confirmation", "password_mismatch", ...);
err.push("end_date", "before_start_date", ...);
err.push("discount", "exceeds_maximum_percent", ...);

// [x] BAD: Generic codes
err.push("password_confirmation", "invalid", ...);
err.push("end_date", "error", ...);
```

### 5. Run Field Validation Before Cross-Field

```rust
impl Validate for Order {
    fn validate(&self) -> Result<(), ValidationError> {
        let mut err = ValidationError::new();

        // 1. Field-level validation first
        if let Err(e) = validate("total", &self.total, &rules::positive()) {
            err.extend(e);
        }
        if let Err(e) = validate("discount", &self.discount, &rules::range(0.0, 100.0)) {
            err.extend(e);
        }

        // 2. Cross-field validation after (uses validated fields)
        if self.discount > self.total * 0.5 {
            err.push("discount", "too_high", "Discount exceeds 50% of total");
        }

        if err.is_empty() { Ok(()) } else { Err(err) }
    }
}
```

### 6. Test Edge Cases

```rust
#[test]
fn test_date_range_edge_cases() {
    // Same date (boundary)
    let same = DateRange {
        start: date(2025, 1, 1),
        end: date(2025, 1, 1),
    };
    assert!(same.validate().is_err());

    // One day apart (minimum valid)
    let one_day = DateRange {
        start: date(2025, 1, 1),
        end: date(2025, 1, 2),
    };
    assert!(one_day.validate().is_ok());

    // Reversed (invalid)
    let reversed = DateRange {
        start: date(2025, 1, 2),
        end: date(2025, 1, 1),
    };
    assert!(reversed.validate().is_err());
}
```

## See Also

- [Derive Macro](DERIVE_MACRO.md) - Field-level validation with `#[derive(Validate)]`
- [Manual Validation](MANUAL_VALIDATION.md) - Implementing `Validate` trait
- [Conditional Validation](CONDITIONAL_VALIDATION.md) - Runtime-determined validation
- [Error Handling](ERROR_HANDLING.md) - Working with `ValidationError`
- [Rules Reference](RULES.md) - Built-in validation rules

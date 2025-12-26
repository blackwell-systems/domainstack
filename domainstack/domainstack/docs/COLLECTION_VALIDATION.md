# Collection Validation

**Complete guide to validating arrays, vectors, and collections: item validation with `each()`, size constraints, uniqueness, and nested types.**

## Table of Contents

- [Overview](#overview)
- [Collection-Level Rules](#collection-level-rules)
- [Item Validation with each()](#item-validation-with-each)
- [Nested Collection Validation](#nested-collection-validation)
- [Combining Collection and Item Rules](#combining-collection-and-item-rules)
- [Manual Collection Validation](#manual-collection-validation)
- [Error Paths for Collections](#error-paths-for-collections)
- [Common Patterns](#common-patterns)
- [Best Practices](#best-practices)

## Overview

domainstack provides two levels of collection validation:

1. **Collection-level** - Size constraints, uniqueness (`min_items`, `max_items`, `unique`)
2. **Item-level** - Validate each item in the collection (`each(rule)`)

```rust
#[derive(Validate)]
struct BlogPost {
    // Collection-level: 1-10 tags required, all unique
    #[validate(min_items = 1)]
    #[validate(max_items = 10)]
    #[validate(unique)]
    // Item-level: each tag must be 1-50 chars, alphanumeric
    #[validate(each(length(min = 1, max = 50)))]
    #[validate(each(alphanumeric))]
    tags: Vec<String>,
}
```

## Collection-Level Rules

Rules that validate the collection itself (not individual items).

### min_items

Minimum number of items required:

```rust
#[derive(Validate)]
struct Order {
    #[validate(min_items = 1)]  // At least 1 item
    items: Vec<OrderItem>,
}

// Error: "Must have at least 1 items"
// Code: "too_few_items"
// Meta: {"min": "1", "actual": "0"}
```

### max_items

Maximum number of items allowed:

```rust
#[derive(Validate)]
struct Playlist {
    #[validate(max_items = 100)]  // No more than 100 songs
    songs: Vec<Song>,
}

// Error: "Must have at most 100 items"
// Code: "too_many_items"
// Meta: {"max": "100", "actual": "150"}
```

### unique

All items must be unique (no duplicates):

```rust
#[derive(Validate)]
struct TagList {
    #[validate(unique)]
    tags: Vec<String>,
}

let list = TagList {
    tags: vec!["rust".into(), "validation".into(), "rust".into()], // Duplicate!
};
list.validate();  // Error: "All items must be unique"
// Code: "duplicate_items"
// Meta: {"duplicates": "1"}
```

### non_empty_items

All string items must be non-empty:

```rust
#[derive(Validate)]
struct Keywords {
    #[validate(non_empty_items)]
    values: Vec<String>,
}

let keywords = Keywords {
    values: vec!["rust".into(), "".into(), "code".into()],  // Empty string!
};
keywords.validate();  // Error at values[1]: "Must not be empty"
// Code: "empty_item"
// Meta: {"empty_count": "1", "indices": "[1]"}
```

## Item Validation with each()

The `each(rule)` attribute applies a validation rule to every item in a collection.

### String Rules with each()

```rust
#[derive(Validate)]
struct Newsletter {
    // Each email must be valid format
    #[validate(each(email))]
    subscriber_emails: Vec<String>,

    // Each URL must be valid
    #[validate(each(url))]
    related_links: Vec<String>,

    // Each tag: 1-30 chars, alphanumeric
    #[validate(each(length(min = 1, max = 30)))]
    #[validate(each(alphanumeric))]
    tags: Vec<String>,

    // Each keyword must start with #
    #[validate(each(starts_with = "#"))]
    hashtags: Vec<String>,
}
```

**Supported string rules:**
- `email`, `url`
- `min_len`, `max_len`, `length`
- `alphanumeric`, `alpha_only`, `ascii`
- `numeric_string`, `non_empty`, `non_blank`, `no_whitespace`
- `contains`, `starts_with`, `ends_with`
- `matches_regex`

### Numeric Rules with each()

```rust
#[derive(Validate)]
struct Survey {
    // Each rating: 1-5
    #[validate(each(range(min = 1, max = 5)))]
    ratings: Vec<u8>,

    // Each score must be positive
    #[validate(each(positive))]
    scores: Vec<i32>,

    // Each amount must be a multiple of 5
    #[validate(each(multiple_of = 5))]
    donation_amounts: Vec<u32>,
}
```

**Supported numeric rules:**
- `range`, `min`, `max`
- `positive`, `negative`, `non_zero`
- `finite`, `multiple_of`
- `equals`, `not_equals`

### Nested Types with each()

```rust
#[derive(Validate)]
struct Comment {
    #[validate(length(min = 1, max = 1000))]
    text: String,

    #[validate(email)]
    author_email: String,
}

#[derive(Validate)]
struct BlogPost {
    // Each comment is validated using Comment's Validate impl
    #[validate(each(nested))]
    comments: Vec<Comment>,
}

// Error paths include indices:
// "comments[0].text" - "Must be at least 1 characters"
// "comments[2].author_email" - "Invalid email format"
```

### Choice Rules with each()

```rust
#[derive(Validate)]
struct Configuration {
    // Each status must be one of the allowed values
    #[validate(each(one_of = ["active", "pending", "archived"]))]
    statuses: Vec<String>,
}
```

## Nested Collection Validation

### Vec of Validated Structs

```rust
#[derive(Validate)]
struct Guest {
    #[validate(length(min = 1, max = 100))]
    name: String,

    #[validate(email)]
    email: String,
}

#[derive(Validate)]
struct Room {
    #[validate(range(min = 1, max = 4))]
    adults: u8,

    #[validate(range(min = 0, max = 3))]
    children: u8,

    #[validate(each(nested))]
    guests: Vec<Guest>,
}

#[derive(Validate)]
struct Booking {
    // Rooms with nested guests
    #[validate(min_items = 1)]
    #[validate(max_items = 5)]
    #[validate(each(nested))]
    rooms: Vec<Room>,
}

// Deep error paths:
// "rooms[0].adults" - "Must be between 1 and 4"
// "rooms[1].guests[0].email" - "Invalid email format"
// "rooms[2].guests[1].name" - "Must be at least 1 characters"
```

### Optional Items in Collections

```rust
#[derive(Validate)]
struct Survey {
    // Optional comments - validate only if Some
    #[validate(each(nested))]
    optional_responses: Vec<Option<Response>>,
}
```

## Combining Collection and Item Rules

Apply both collection-level and item-level validation:

```rust
#[derive(Validate)]
struct Article {
    // Collection: 1-20 tags, unique
    // Items: 1-50 chars, no whitespace
    #[validate(min_items = 1)]
    #[validate(max_items = 20)]
    #[validate(unique)]
    #[validate(non_empty_items)]
    #[validate(each(length(min = 1, max = 50)))]
    #[validate(each(no_whitespace))]
    tags: Vec<String>,
}
```

**Execution order:**
1. Collection-level rules run first (`min_items`, `max_items`, `unique`)
2. Item-level rules run after (`each(...)`)
3. All errors accumulate (fail-slow)

### Common Combinations

```rust
// Required, unique tags with format validation
#[validate(min_items = 1)]
#[validate(unique)]
#[validate(each(alphanumeric))]
#[validate(each(length(min = 2, max = 30)))]
tags: Vec<String>,

// Limited, validated emails
#[validate(max_items = 10)]
#[validate(each(email))]
recipients: Vec<String>,

// Nested items with size limits
#[validate(min_items = 1)]
#[validate(max_items = 5)]
#[validate(each(nested))]
rooms: Vec<Room>,
```

## Manual Collection Validation

For complex collection validation, implement `Validate` manually.

### Basic Manual Validation

```rust
use domainstack::prelude::*;

pub struct Team {
    pub members: Vec<Member>,
}

impl Validate for Team {
    fn validate(&self) -> Result<(), ValidationError> {
        let mut err = ValidationError::new();

        // Collection-level: at least one member
        if self.members.is_empty() {
            err.push("members", "min_items", "Team must have at least one member");
        }

        // Collection-level: no more than 10 members
        if self.members.len() > 10 {
            err.push("members", "max_items", "Team cannot have more than 10 members");
        }

        // Item-level: validate each member
        for (i, member) in self.members.iter().enumerate() {
            if let Err(e) = member.validate() {
                let path = Path::root().field("members").index(i);
                err.merge_prefixed(path, e);
            }
        }

        if err.is_empty() { Ok(()) } else { Err(err) }
    }
}
```

### Custom Uniqueness Checks

```rust
use std::collections::HashSet;

impl Validate for UserList {
    fn validate(&self) -> Result<(), ValidationError> {
        let mut err = ValidationError::new();

        // Check for duplicate emails
        let mut seen_emails = HashSet::new();
        for (i, user) in self.users.iter().enumerate() {
            if !seen_emails.insert(&user.email) {
                err.push(
                    Path::root().field("users").index(i).field("email"),
                    "duplicate_email",
                    format!("Email '{}' is already used by another user", user.email)
                );
            }
        }

        if err.is_empty() { Ok(()) } else { Err(err) }
    }
}
```

### Cross-Item Validation

```rust
impl Validate for Schedule {
    fn validate(&self) -> Result<(), ValidationError> {
        let mut err = ValidationError::new();

        // Validate each event
        for (i, event) in self.events.iter().enumerate() {
            if let Err(e) = event.validate() {
                err.merge_prefixed(Path::root().field("events").index(i), e);
            }
        }

        // Check for overlapping events
        for i in 0..self.events.len() {
            for j in (i + 1)..self.events.len() {
                if events_overlap(&self.events[i], &self.events[j]) {
                    err.push(
                        Path::root().field("events").index(j),
                        "overlapping_event",
                        format!("Overlaps with event at index {}", i)
                    );
                }
            }
        }

        if err.is_empty() { Ok(()) } else { Err(err) }
    }
}
```

## Error Paths for Collections

Collection errors include array indices for precise field targeting.

### Path Examples

```rust
// Collection-level error
"items" -> "Must have at least 1 items"

// Item at index 0
"items[0]" -> "Invalid email format"

// Nested field at index 2
"rooms[2].adults" -> "Must be between 1 and 4"

// Deeply nested
"orders[0].items[3].product.name" -> "Must not be empty"
```

### Using Path API

```rust
use domainstack::Path;

// Build paths programmatically
let path = Path::root()
    .field("rooms")
    .index(0)
    .field("guests")
    .index(1)
    .field("email");
// -> "rooms[0].guests[1].email"

// Use in manual validation
for (i, item) in self.items.iter().enumerate() {
    let base_path = Path::root().field("items").index(i);

    if item.quantity <= 0 {
        err.push(
            base_path.clone().field("quantity"),
            "invalid_quantity",
            "Quantity must be positive"
        );
    }
}
```

## Common Patterns

### Email List Validation

```rust
#[derive(Validate)]
struct Newsletter {
    #[validate(min_items = 1)]
    #[validate(max_items = 1000)]
    #[validate(unique)]
    #[validate(each(email))]
    #[validate(each(max_len = 255))]
    recipients: Vec<String>,
}
```

### Tag System

```rust
#[derive(Validate)]
struct TaggedContent {
    #[validate(min_items = 1, message = "At least one tag required")]
    #[validate(max_items = 10, message = "Maximum 10 tags allowed")]
    #[validate(unique)]
    #[validate(non_empty_items)]
    #[validate(each(length(min = 2, max = 30)))]
    #[validate(each(alphanumeric))]
    tags: Vec<String>,
}
```

### Shopping Cart

```rust
#[derive(Validate)]
struct CartItem {
    #[validate(length(min = 1, max = 100))]
    product_id: String,

    #[validate(range(min = 1, max = 99))]
    quantity: u32,
}

#[derive(Validate)]
struct ShoppingCart {
    #[validate(min_items = 1)]
    #[validate(max_items = 50)]
    #[validate(each(nested))]
    items: Vec<CartItem>,
}
```

### Multi-Room Booking

```rust
#[derive(Validate)]
struct Room {
    #[validate(range(min = 1, max = 4))]
    adults: u8,

    #[validate(range(min = 0, max = 3))]
    children: u8,
}

#[derive(Validate)]
#[validate(
    check = "self.check_out > self.check_in",
    code = "invalid_dates",
    message = "Check-out must be after check-in"
)]
struct BookingRequest {
    check_in: NaiveDate,
    check_out: NaiveDate,

    #[validate(min_items = 1)]
    #[validate(max_items = 5)]
    #[validate(each(nested))]
    rooms: Vec<Room>,
}
```

### File Upload with Metadata

```rust
#[derive(Validate)]
struct FileMetadata {
    #[validate(length(min = 1, max = 255))]
    filename: String,

    #[validate(one_of = ["image/jpeg", "image/png", "application/pdf"])]
    content_type: String,

    #[validate(range(min = 1, max = 10485760))]  // 10MB max
    size_bytes: u64,
}

#[derive(Validate)]
struct UploadBatch {
    #[validate(min_items = 1)]
    #[validate(max_items = 20)]
    #[validate(each(nested))]
    files: Vec<FileMetadata>,
}
```

## Best Practices

### 1. Validate Collection Before Items

```rust
// ✅ GOOD: Collection rules first in attribute list
#[validate(min_items = 1)]  // Fail fast if empty
#[validate(max_items = 10)]
#[validate(each(email))]    // Then validate items
emails: Vec<String>,
```

### 2. Use Specific Error Paths

```rust
// ✅ GOOD: Error points to exact item
err.push(
    Path::root().field("items").index(i).field("quantity"),
    "invalid_quantity",
    "Quantity must be positive"
);

// ❌ BAD: Generic path loses context
err.push("items", "invalid", "Invalid item");
```

### 3. Include Context in Messages

```rust
// ✅ GOOD: Helpful message with limits
#[validate(min_items = 1, message = "At least one recipient required")]
#[validate(max_items = 100, message = "Maximum 100 recipients allowed")]

// ❌ BAD: Generic message
#[validate(min_items = 1)]  // "Must have at least 1 items"
```

### 4. Combine Related Rules

```rust
// ✅ GOOD: All tag rules together
#[validate(min_items = 1)]
#[validate(max_items = 10)]
#[validate(unique)]
#[validate(each(length(min = 1, max = 30)))]
tags: Vec<String>,
```

### 5. Test Edge Cases

```rust
#[test]
fn test_collection_validation() {
    // Empty collection
    let empty = Cart { items: vec![] };
    assert!(empty.validate().is_err());

    // Exactly at limit
    let at_limit = Cart { items: vec![item(); 50] };
    assert!(at_limit.validate().is_ok());

    // Over limit
    let over = Cart { items: vec![item(); 51] };
    assert!(over.validate().is_err());

    // With invalid item
    let invalid_item = Cart { items: vec![invalid_item()] };
    let err = invalid_item.validate().unwrap_err();
    assert!(err.violations[0].path.to_string().contains("[0]"));
}
```

## See Also

- [Derive Macro](DERIVE_MACRO.md) - Complete `#[derive(Validate)]` guide
- [Rules Reference](RULES.md) - All 37 built-in rules
- [Error Handling](ERROR_HANDLING.md) - Working with `ValidationError`
- [Manual Validation](MANUAL_VALIDATION.md) - Custom validation logic

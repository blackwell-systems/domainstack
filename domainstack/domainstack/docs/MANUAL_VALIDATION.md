# Manual Validation

**When and how to implement the `Validate` trait manually for custom validation logic.**

## Table of Contents

- [When to Use Manual Validation](#when-to-use-manual-validation)
- [Implementing Validate Trait](#implementing-validate-trait)
- [Validating Collections](#validating-collections)
- [Merging Nested Errors](#merging-nested-errors)
- [Manual vs Derive](#manual-vs-derive)
- [Advanced Patterns](#advanced-patterns)

## When to Use Manual Validation

Use manual `Validate` implementation when you need:

### 1. **Newtype Wrappers**

Tuple structs that wrap primitives:

```rust
pub struct Email(String);
pub struct Username(String);
pub struct Age(u8);
```

These can't use `#[derive(Validate)]` because they're tuple structs.

### 2. **Complex Business Logic**

Validation that goes beyond declarative rules:

```rust
impl Validate for Order {
    fn validate(&self) -> Result<(), ValidationError> {
        let mut err = ValidationError::new();

        // Complex logic: discount only applies if total > $100
        if self.discount > 0.0 && self.total < 100.0 {
            err.push("discount", "invalid_discount",
                "Discount only available for orders over $100");
        }

        // Multi-field calculation
        let calculated_total = self.items.iter().sum::<f64>();
        if (self.total - calculated_total).abs() > 0.01 {
            err.push("total", "total_mismatch", "Total doesn't match items");
        }

        if err.is_empty() { Ok(()) } else { Err(err) }
    }
}
```

### 3. **Custom Error Messages**

Fine-grained control over error codes and messages:

```rust
if age < 18 {
    err.push("age", "underage",
        format!("You must be 18 or older. You are {} years old.", age));
}
```

### 4. **Dynamic Validation**

Runtime-determined validation rules:

```rust
let min_length = if is_admin { 6 } else { 8 };
let rule = rules::min_len(min_length);
```

## Implementing Validate Trait

The `Validate` trait has a single method:

```rust
pub trait Validate {
    fn validate(&self) -> Result<(), ValidationError>;
}
```

### Basic Implementation

```rust
use domainstack::prelude::*;

pub struct User {
    pub name: String,
    pub email: Email,
    pub age: u8,
}

impl Validate for User {
    fn validate(&self) -> Result<(), ValidationError> {
        let mut err = ValidationError::new();

        // Validate name
        let name_rule = rules::min_len(1).and(rules::max_len(50));
        if let Err(e) = validate("name", self.name.as_str(), &name_rule) {
            err.extend(e);
        }

        // Validate nested email
        if let Err(e) = self.email.validate() {
            err.merge_prefixed("email", e);
        }

        // Validate age
        let age_rule = rules::range(18, 120);
        if let Err(e) = validate("age", &self.age, &age_rule) {
            err.extend(e);
        }

        if err.is_empty() {
            Ok(())
        } else {
            Err(err)
        }
    }
}
```

### Step-by-Step Pattern

```rust
impl Validate for Type {
    fn validate(&self) -> Result<(), ValidationError> {
        // 1. Create error accumulator
        let mut err = ValidationError::new();

        // 2. Validate each field
        if let Err(e) = validate("field", &self.field, &rule) {
            err.extend(e);
        }

        // 3. Validate nested types
        if let Err(e) = self.nested.validate() {
            err.merge_prefixed("nested", e);
        }

        // 4. Custom business logic
        if some_condition {
            err.push("field", "code", "message");
        }

        // 5. Return result
        if err.is_empty() { Ok(()) } else { Err(err) }
    }
}
```

## Validating Collections

### Array Indices in Error Paths

When validating collections, include array indices in error paths for precise error tracking:

```rust
impl Validate for Team {
    fn validate(&self) -> Result<(), ValidationError> {
        let mut err = ValidationError::new();

        // Validate each member with array index
        for (i, member) in self.members.iter().enumerate() {
            if let Err(e) = member.validate() {
                let path = Path::root().field("members").index(i);
                err.merge_prefixed(path, e);
                // Error paths: "members[0].name", "members[1].email", etc.
            }
        }

        if err.is_empty() { Ok(()) } else { Err(err) }
    }
}
```

### Collection-Level Rules

Combine item validation with collection-level rules:

```rust
impl Validate for BlogPost {
    fn validate(&self) -> Result<(), ValidationError> {
        let mut err = ValidationError::new();

        // Collection-level rules
        if self.tags.is_empty() {
            err.push("tags", "min_items", "Must have at least 1 tag");
        }

        if self.tags.len() > 10 {
            err.push("tags", "max_items", "Cannot have more than 10 tags");
        }

        // Validate each tag
        for (i, tag) in self.tags.iter().enumerate() {
            if tag.is_empty() {
                err.push(
                    Path::root().field("tags").index(i),
                    "empty_string",
                    "Tag cannot be empty"
                );
            }
        }

        if err.is_empty() { Ok(()) } else { Err(err) }
    }
}
```

### Nested Collections

Handle deeply nested structures:

```rust
impl Validate for Booking {
    fn validate(&self) -> Result<(), ValidationError> {
        let mut err = ValidationError::new();

        // Validate nested collection
        for (i, room) in self.rooms.iter().enumerate() {
            if let Err(e) = room.validate() {
                let path = Path::root().field("rooms").index(i);
                err.merge_prefixed(path, e);
                // Produces paths like: "rooms[0].adults", "rooms[1].bed_type"
            }

            // Validate room's nested guests
            for (j, guest) in room.guests.iter().enumerate() {
                if let Err(e) = guest.validate() {
                    let path = Path::root()
                        .field("rooms")
                        .index(i)
                        .field("guests")
                        .index(j);
                    err.merge_prefixed(path, e);
                    // Produces: "rooms[0].guests[1].email"
                }
            }
        }

        if err.is_empty() { Ok(()) } else { Err(err) }
    }
}
```

## Merging Nested Errors

### Why Merge with Prefix?

When validating nested types, you need to prefix their error paths to maintain the full field path:

```rust
// Email validates itself with path "value"
impl Validate for Email {
    fn validate(&self) -> Result<(), ValidationError> {
        validate("value", self.0.as_str(), &rules::email())
        // Error path: "value"
    }
}

// User needs to prefix "email" when merging
impl Validate for User {
    fn validate(&self) -> Result<(), ValidationError> {
        let mut err = ValidationError::new();

        if let Err(e) = self.email.validate() {
            err.merge_prefixed("email", e);
            // Final error path: "email.value"
        }

        if err.is_empty() { Ok(()) } else { Err(err) }
    }
}
```

### extend() vs merge_prefixed()

**`extend()`** - Adds violations without changing paths:

```rust
// Use for same-level validation
if let Err(e) = validate("name", &self.name, &name_rule) {
    err.extend(e);  // Path stays "name"
}
```

**`merge_prefixed()`** - Adds violations with path prefix:

```rust
// Use for nested validation
if let Err(e) = self.guest.validate() {
    err.merge_prefixed("guest", e);  // "email" becomes "guest.email"
}
```

### Path Building with Path API

For complex paths, use the Path API:

```rust
// String path
err.merge_prefixed("guest.contact", nested_err);

// Path API (type-safe)
let path = Path::root()
    .field("guest")
    .field("contact");
err.merge_prefixed(path, nested_err);

// With array indices
let path = Path::root()
    .field("rooms")
    .index(0)
    .field("guest");
err.merge_prefixed(path, nested_err);
```

## Manual vs Derive

### When to Use Each

| Use Manual | Use Derive |
|------------|------------|
| Newtype wrappers | Structs with named fields |
| Complex business logic | Declarative validation rules |
| Custom error messages | Standard error messages |
| Dynamic validation rules | Static validation rules |
| Conditional field validation | Cross-field validation |
| Early returns | Fail-slow accumulation |

### Example: Same Validation, Both Ways

**Manual implementation:**

```rust
impl Validate for User {
    fn validate(&self) -> Result<(), ValidationError> {
        let mut err = ValidationError::new();

        if let Err(e) = validate("name", self.name.as_str(), &rules::min_len(1).and(rules::max_len(50))) {
            err.extend(e);
        }

        if let Err(e) = validate("age", &self.age, &rules::range(18, 120)) {
            err.extend(e);
        }

        if err.is_empty() { Ok(()) } else { Err(err) }
    }
}
```

**Derive implementation:**

```rust
#[derive(Validate)]
struct User {
    #[validate(length(min = 1, max = 50))]
    name: String,

    #[validate(range(min = 18, max = 120))]
    age: u8,
}
```

**When derive is better:**
- Less boilerplate (3 lines vs 15 lines)
- Declarative and readable
- Compile-time code generation
- Automatic path tracking

**When manual is better:**
- Custom error messages
- Complex business logic
- Newtype wrappers
- Runtime-determined rules

## Advanced Patterns

### Early Returns

For performance-critical paths, return early:

```rust
impl Validate for ExpensiveType {
    fn validate(&self) -> Result<(), ValidationError> {
        // Check cheap validation first
        if self.critical_field.is_empty() {
            return Err(ValidationError::single(
                "critical_field",
                "empty",
                "Cannot be empty"
            ));
        }

        // Only do expensive validation if needed
        self.expensive_nested_validation()
    }
}
```

### Combining Manual and Derive

Use both in the same codebase:

```rust
// Newtype - manual
impl Validate for Email {
    fn validate(&self) -> Result<(), ValidationError> {
        validate("value", self.0.as_str(), &rules::email())
    }
}

// Complex struct - derive
#[derive(Validate)]
struct User {
    #[validate(nested)]
    email: Email,

    #[validate(range(min = 18, max = 120))]
    age: u8,
}
```

### Custom Validation Functions

Reusable validation logic:

```rust
fn validate_password_strength(password: &str) -> Result<(), ValidationError> {
    let mut err = ValidationError::new();

    if !password.chars().any(|c| c.is_uppercase()) {
        err.push("", "no_uppercase", "Must contain uppercase letter");
    }

    if !password.chars().any(|c| c.is_lowercase()) {
        err.push("", "no_lowercase", "Must contain lowercase letter");
    }

    if !password.chars().any(|c| c.is_numeric()) {
        err.push("", "no_digit", "Must contain digit");
    }

    if err.is_empty() { Ok(()) } else { Err(err) }
}

impl Validate for PasswordChange {
    fn validate(&self) -> Result<(), ValidationError> {
        let mut err = ValidationError::new();

        if let Err(e) = validate_password_strength(&self.password) {
            err.merge_prefixed("password", e);
        }

        if err.is_empty() { Ok(()) } else { Err(err) }
    }
}
```

### Builder Pattern Integration

Validate during build:

```rust
pub struct UserBuilder {
    name: Option<String>,
    email: Option<Email>,
    age: Option<u8>,
}

impl UserBuilder {
    pub fn build(self) -> Result<User, ValidationError> {
        let user = User {
            name: self.name.ok_or_else(|| {
                ValidationError::single("name", "required", "Name is required")
            })?,
            email: self.email.ok_or_else(|| {
                ValidationError::single("email", "required", "Email is required")
            })?,
            age: self.age.ok_or_else(|| {
                ValidationError::single("age", "required", "Age is required")
            })?,
        };

        user.validate()?;  // Validate after construction
        Ok(user)
    }
}
```

## See Also

- [Core Concepts](CORE_CONCEPTS.md) - Valid-by-construction types and smart constructors
- [Derive Macro](DERIVE_MACRO.md) - Declarative validation with `#[derive(Validate)]`
- [Error Handling](ERROR_HANDLING.md) - Working with ValidationError
- [Rules Reference](RULES.md) - Complete list of 37 built-in rules

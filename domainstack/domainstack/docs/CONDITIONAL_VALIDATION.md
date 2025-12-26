# Conditional Validation

**Runtime-determined validation: rules based on field values, configuration, external context, and dynamic conditions.**

## Table of Contents

- [Overview](#overview)
- [The when() Combinator](#the-when-combinator)
- [Manual Conditional Validation](#manual-conditional-validation)
- [Context-Based Validation](#context-based-validation)
- [Configuration-Driven Validation](#configuration-driven-validation)
- [Multi-Branch Validation](#multi-branch-validation)
- [Optional Field Validation](#optional-field-validation)
- [Best Practices](#best-practices)

## Overview

Conditional validation applies different rules based on runtime conditions:

- **Field values** - Validate shipping address only if `requires_shipping` is true
- **User roles** - Enterprise users have different validation rules
- **Configuration** - Password length from config, not hardcoded
- **Payment type** - Credit card vs bank transfer have different fields

```rust
// Only validate shipping if product is physical
#[validate(
    check = "self.shipping_address.is_some()",
    message = "Shipping address required",
    when = "self.is_physical_product"
)]
```

## The when() Combinator

Rules can be conditionally applied using `.when()`:

```rust
use domainstack::prelude::*;

// URL validation only runs if string is not empty
let optional_url_rule = rules::url()
    .when(|s: &String| !s.is_empty());

validate("website", &user.website, &optional_url_rule)?;
```

### Basic Usage

```rust
// Only validate if condition is true
let rule = rules::min_len(10).when(|s: &String| !s.is_empty());

// Validate - empty strings skip the rule
assert!(rule.apply(&"".to_string()).is_empty());       // [ok] Skipped
assert!(rule.apply(&"hello world".to_string()).is_empty()); // [ok] Valid
assert!(!rule.apply(&"short".to_string()).is_empty()); // [error] Invalid (5 < 10)
```

### Closure Conditions

```rust
// Closure with external state
let is_premium = true;

let max_length_rule = rules::max_len(if is_premium { 10000 } else { 1000 });
// Note: This isn't quite .when() - it's conditional rule construction

// True .when() with closure
let premium_only_rule = rules::max_len(10000)
    .when(move |_: &String| is_premium);
```

### Combining with Other Rules

```rust
// Multiple conditions
let flexible_rule = rules::email()
    .and(rules::max_len(255))
    .when(|s: &String| !s.is_empty());

// Or with conditional branches
let rule = if is_admin {
    rules::length(6, 50)
} else {
    rules::length(8, 128)
};
```

## Manual Conditional Validation

For complex conditional logic, implement `Validate` manually.

### Basic Conditional Validation

```rust
use domainstack::prelude::*;

pub struct Order {
    pub requires_shipping: bool,
    pub shipping_address: Option<Address>,
    pub items: Vec<Item>,
}

impl Validate for Order {
    fn validate(&self) -> Result<(), ValidationError> {
        let mut err = ValidationError::new();

        // Always validate items
        if self.items.is_empty() {
            err.push("items", "min_items", "Order must have at least one item");
        }

        // Conditionally validate shipping address
        if self.requires_shipping {
            match &self.shipping_address {
                Some(addr) => {
                    // Validate the address
                    if let Err(e) = addr.validate() {
                        err.merge_prefixed("shipping_address", e);
                    }
                }
                None => {
                    err.push(
                        "shipping_address",
                        "required",
                        "Shipping address required for physical products"
                    );
                }
            }
        }

        if err.is_empty() { Ok(()) } else { Err(err) }
    }
}
```

### Field-Dependent Validation

```rust
pub struct PaymentDetails {
    pub payment_type: PaymentType,
    pub card_number: Option<String>,
    pub card_cvv: Option<String>,
    pub bank_account: Option<String>,
    pub bank_routing: Option<String>,
}

pub enum PaymentType {
    CreditCard,
    BankTransfer,
    PayPal,
}

impl Validate for PaymentDetails {
    fn validate(&self) -> Result<(), ValidationError> {
        let mut err = ValidationError::new();

        match self.payment_type {
            PaymentType::CreditCard => {
                // Validate credit card fields
                if self.card_number.is_none() {
                    err.push("card_number", "required", "Card number is required");
                } else if let Some(ref num) = self.card_number {
                    if let Err(e) = validate("card_number", num.as_str(),
                                             &rules::numeric_string().and(rules::length(13, 19))) {
                        err.extend(e);
                    }
                }

                if self.card_cvv.is_none() {
                    err.push("card_cvv", "required", "CVV is required");
                } else if let Some(ref cvv) = self.card_cvv {
                    if let Err(e) = validate("card_cvv", cvv.as_str(),
                                             &rules::matches_regex(r"^\d{3,4}$")) {
                        err.extend(e);
                    }
                }
            }

            PaymentType::BankTransfer => {
                // Validate bank transfer fields
                if self.bank_account.is_none() {
                    err.push("bank_account", "required", "Bank account is required");
                }
                if self.bank_routing.is_none() {
                    err.push("bank_routing", "required", "Routing number is required");
                }
            }

            PaymentType::PayPal => {
                // No additional fields needed for PayPal
            }
        }

        if err.is_empty() { Ok(()) } else { Err(err) }
    }
}
```

## Context-Based Validation

Validate against external state like existing records, user permissions, or configuration.

### ValidationContext Pattern

```rust
use std::collections::HashSet;
use domainstack::prelude::*;

pub struct ValidationContext {
    pub existing_emails: HashSet<String>,
    pub existing_usernames: HashSet<String>,
    pub user_role: UserRole,
    pub config: AppConfig,
}

pub struct User {
    pub email: String,
    pub username: String,
    pub age: u8,
}

impl User {
    pub fn validate_with_context(
        &self,
        ctx: &ValidationContext
    ) -> Result<(), ValidationError> {
        let mut err = ValidationError::new();

        // Basic field validation
        if let Err(e) = validate("email", &self.email, &rules::email()) {
            err.extend(e);
        }

        if let Err(e) = validate("username", &self.username, &rules::min_len(3)) {
            err.extend(e);
        }

        // Context-dependent: role-based age limit
        let min_age = match ctx.user_role {
            UserRole::Enterprise => 21,
            UserRole::Standard => 18,
            UserRole::Minor => 13,
        };

        if let Err(e) = validate("age", &self.age, &rules::range(min_age, 120)) {
            err.extend(e);
        }

        // Context-dependent: uniqueness checks
        if ctx.existing_emails.contains(&self.email) {
            err.push("email", "email_taken", "Email already registered");
        }

        if ctx.existing_usernames.contains(&self.username) {
            err.push("username", "username_taken", "Username already taken");
        }

        if err.is_empty() { Ok(()) } else { Err(err) }
    }
}

// Usage
let ctx = ValidationContext {
    existing_emails: ["alice@example.com".into()].into_iter().collect(),
    existing_usernames: ["alice".into()].into_iter().collect(),
    user_role: UserRole::Standard,
    config: AppConfig::default(),
};

let user = User {
    email: "bob@example.com".to_string(),
    username: "bob".to_string(),
    age: 25,
};

user.validate_with_context(&ctx)?;
```

### Update vs Create Context

```rust
pub enum OperationType {
    Create,
    Update { current_id: i64 },
}

pub struct UpdateContext {
    pub operation: OperationType,
    pub existing_emails: HashMap<String, i64>,  // email -> user_id
}

impl User {
    pub fn validate_with_update_context(
        &self,
        ctx: &UpdateContext
    ) -> Result<(), ValidationError> {
        let mut err = ValidationError::new();

        // Check email uniqueness with update awareness
        if let Some(&owner_id) = ctx.existing_emails.get(&self.email) {
            let is_own_email = match ctx.operation {
                OperationType::Update { current_id } => owner_id == current_id,
                OperationType::Create => false,
            };

            if !is_own_email {
                err.push("email", "email_taken", "Email already in use");
            }
        }

        if err.is_empty() { Ok(()) } else { Err(err) }
    }
}
```

## Configuration-Driven Validation

Adjust validation rules based on application configuration.

### Config-Based Rules

```rust
pub struct PasswordConfig {
    pub min_length: usize,
    pub max_length: usize,
    pub require_uppercase: bool,
    pub require_lowercase: bool,
    pub require_digit: bool,
    pub require_special: bool,
}

pub struct PasswordChange {
    pub new_password: String,
}

impl PasswordChange {
    pub fn validate_with_config(
        &self,
        config: &PasswordConfig
    ) -> Result<(), ValidationError> {
        let mut err = ValidationError::new();

        // Length from config
        let length_rule = rules::length(config.min_length, config.max_length);
        if let Err(e) = validate("new_password", &self.new_password, &length_rule) {
            err.extend(e);
        }

        let password = &self.new_password;

        // Conditional character requirements
        if config.require_uppercase && !password.chars().any(|c| c.is_uppercase()) {
            err.push("new_password", "no_uppercase", "Must contain uppercase letter");
        }

        if config.require_lowercase && !password.chars().any(|c| c.is_lowercase()) {
            err.push("new_password", "no_lowercase", "Must contain lowercase letter");
        }

        if config.require_digit && !password.chars().any(|c| c.is_numeric()) {
            err.push("new_password", "no_digit", "Must contain digit");
        }

        if config.require_special && !password.chars().any(|c| "!@#$%^&*(),.?:{}|<>".contains(c)) {
            err.push("new_password", "no_special", "Must contain special character");
        }

        if err.is_empty() { Ok(()) } else { Err(err) }
    }
}

// Usage
let strict_config = PasswordConfig {
    min_length: 12,
    max_length: 128,
    require_uppercase: true,
    require_lowercase: true,
    require_digit: true,
    require_special: true,
};

let relaxed_config = PasswordConfig {
    min_length: 8,
    max_length: 128,
    require_uppercase: false,
    require_lowercase: false,
    require_digit: true,
    require_special: false,
};

password_change.validate_with_config(&strict_config)?;
```

### Environment-Based Validation

```rust
pub struct EnvironmentConfig {
    pub is_production: bool,
    pub allow_test_emails: bool,
    pub max_upload_size_mb: usize,
}

impl FileUpload {
    pub fn validate_with_env(
        &self,
        env: &EnvironmentConfig
    ) -> Result<(), ValidationError> {
        let mut err = ValidationError::new();

        // Size limit from environment
        let max_bytes = env.max_upload_size_mb * 1024 * 1024;
        if self.size > max_bytes {
            err.push(
                "size",
                "file_too_large",
                format!("File exceeds maximum size of {}MB", env.max_upload_size_mb)
            );
        }

        // Production-only restrictions
        if env.is_production {
            // Stricter file type validation in production
            let allowed_types = ["image/jpeg", "image/png", "application/pdf"];
            if !allowed_types.contains(&self.content_type.as_str()) {
                err.push("content_type", "invalid_type", "File type not allowed");
            }
        }

        if err.is_empty() { Ok(()) } else { Err(err) }
    }
}
```

## Multi-Branch Validation

Different validation paths based on type or category.

### Enum-Based Validation

```rust
pub enum PaymentMethod {
    CreditCard {
        number: String,
        expiry: String,
        cvv: String,
    },
    BankTransfer {
        iban: String,
        bic: String,
    },
    Cryptocurrency {
        wallet_address: String,
        network: String,
    },
}

impl Validate for PaymentMethod {
    fn validate(&self) -> Result<(), ValidationError> {
        let mut err = ValidationError::new();

        match self {
            PaymentMethod::CreditCard { number, expiry, cvv } => {
                // Credit card validation
                if let Err(e) = validate("number", number.as_str(),
                    &rules::numeric_string().and(rules::length(13, 19))) {
                    err.extend(e);
                }

                if let Err(e) = validate("expiry", expiry.as_str(),
                    &rules::matches_regex(r"^\d{2}/\d{2}$")) {
                    err.extend(e);
                }

                if let Err(e) = validate("cvv", cvv.as_str(),
                    &rules::matches_regex(r"^\d{3,4}$")) {
                    err.extend(e);
                }
            }

            PaymentMethod::BankTransfer { iban, bic } => {
                // IBAN/BIC validation
                if let Err(e) = validate("iban", iban.as_str(),
                    &rules::alphanumeric().and(rules::length(15, 34))) {
                    err.extend(e);
                }

                if let Err(e) = validate("bic", bic.as_str(),
                    &rules::alphanumeric().and(rules::length(8, 11))) {
                    err.extend(e);
                }
            }

            PaymentMethod::Cryptocurrency { wallet_address, network } => {
                // Crypto validation (simplified)
                if let Err(e) = validate("wallet_address", wallet_address.as_str(),
                    &rules::min_len(20).and(rules::max_len(100))) {
                    err.extend(e);
                }

                if let Err(e) = validate("network", network.as_str(),
                    &rules::one_of(&["ethereum", "bitcoin", "polygon"])) {
                    err.extend(e);
                }
            }
        }

        if err.is_empty() { Ok(()) } else { Err(err) }
    }
}
```

### Variant-Specific Context

```rust
impl PaymentMethod {
    pub fn validate_with_context(
        &self,
        ctx: &PaymentContext
    ) -> Result<(), ValidationError> {
        let mut err = ValidationError::new();

        // Base validation
        if let Err(e) = self.validate() {
            err.extend(e);
        }

        // Variant-specific context validation
        match self {
            PaymentMethod::CreditCard { number, .. } => {
                // Check if card is in blocked list
                if ctx.blocked_card_prefixes.iter().any(|p| number.starts_with(p)) {
                    err.push("number", "card_blocked", "This card cannot be used");
                }
            }

            PaymentMethod::BankTransfer { iban, .. } => {
                // Check if bank is supported
                let country_code = &iban[0..2];
                if !ctx.supported_countries.contains(country_code) {
                    err.push("iban", "country_unsupported", "Bank country not supported");
                }
            }

            PaymentMethod::Cryptocurrency { network, .. } => {
                // Check if network is enabled
                if !ctx.enabled_crypto_networks.contains(network.as_str()) {
                    err.push("network", "network_disabled", "This network is currently disabled");
                }
            }
        }

        if err.is_empty() { Ok(()) } else { Err(err) }
    }
}
```

## Optional Field Validation

Validate optional fields only when present.

### Option<T> Pattern

```rust
pub struct UserProfile {
    pub name: String,
    pub bio: Option<String>,
    pub website: Option<String>,
    pub age: Option<u8>,
}

impl Validate for UserProfile {
    fn validate(&self) -> Result<(), ValidationError> {
        let mut err = ValidationError::new();

        // Required field
        if let Err(e) = validate("name", &self.name, &rules::length(1, 100)) {
            err.extend(e);
        }

        // Optional: validate only if Some
        if let Some(ref bio) = self.bio {
            if let Err(e) = validate("bio", bio.as_str(), &rules::max_len(500)) {
                err.extend(e);
            }
        }

        if let Some(ref website) = self.website {
            if let Err(e) = validate("website", website.as_str(), &rules::url()) {
                err.extend(e);
            }
        }

        if let Some(age) = self.age {
            if let Err(e) = validate("age", &age, &rules::range(13, 120)) {
                err.extend(e);
            }
        }

        if err.is_empty() { Ok(()) } else { Err(err) }
    }
}
```

### Derive Macro with Optional Fields

```rust
#[derive(Validate)]
pub struct UserProfile {
    #[validate(length(min = 1, max = 100))]
    pub name: String,

    #[validate(max_len = 500)]  // Only validates if Some
    pub bio: Option<String>,

    #[validate(url)]  // Only validates if Some
    pub website: Option<String>,

    #[validate(nested)]  // Nested validation if Some
    pub address: Option<Address>,
}
```

## Best Practices

### 1. Validate Unconditionally First

```rust
// GOOD: Common validation first, then conditional
impl Validate for Order {
    fn validate(&self) -> Result<(), ValidationError> {
        let mut err = ValidationError::new();

        // Always validate (common to all orders)
        if self.items.is_empty() {
            err.push("items", "required", "At least one item required");
        }

        // Conditionally validate (depends on order type)
        if self.requires_shipping {
            // ...shipping validation
        }

        if err.is_empty() { Ok(()) } else { Err(err) }
    }
}
```

### 2. Document Conditional Logic

```rust
/// Validates the payment method.
///
/// # Validation Rules
///
/// - All methods: Basic format validation
/// - Credit Card: Luhn check, expiry validation
/// - Bank Transfer: IBAN format, country support (from context)
/// - Crypto: Wallet format, network availability (from context)
pub fn validate(&self) -> Result<(), ValidationError> { ... }
```

### 3. Use Descriptive Error Codes

```rust
// GOOD: Specific codes for conditional failures
err.push("shipping_address", "required_for_physical", "Required for physical products");
err.push("card_cvv", "required_for_card", "CVV required for card payments");

// [x] BAD: Generic codes
err.push("shipping_address", "required", "Required");
```

### 4. Avoid Deep Nesting

```rust
// [x] BAD: Deeply nested conditions
if self.is_premium {
    if self.order_type == OrderType::Express {
        if self.total > 100.0 {
            // validation...
        }
    }
}

// GOOD: Early returns or flat structure
if !self.is_premium || self.order_type != OrderType::Express {
    return validate_standard_order(self);
}

if self.total > 100.0 {
    // premium express validation...
}
```

### 5. Test All Branches

```rust
#[test]
fn test_payment_validation_credit_card() {
    let payment = PaymentMethod::CreditCard { ... };
    assert!(payment.validate().is_ok());
}

#[test]
fn test_payment_validation_bank_transfer() {
    let payment = PaymentMethod::BankTransfer { ... };
    assert!(payment.validate().is_ok());
}

#[test]
fn test_payment_validation_crypto() {
    let payment = PaymentMethod::Cryptocurrency { ... };
    assert!(payment.validate().is_ok());
}

// Also test invalid cases for each branch
```

## See Also

- [Cross-Field Validation](CROSS_FIELD_VALIDATION.md) - Relationships between fields
- [Async Validation](ASYNC_VALIDATION.md) - Database and API checks
- [Manual Validation](MANUAL_VALIDATION.md) - Implementing `Validate` trait
- [Derive Macro](DERIVE_MACRO.md) - Declarative validation
- [Rules Reference](RULES.md) - Built-in validation rules

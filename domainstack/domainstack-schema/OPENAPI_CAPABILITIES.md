# OpenAPI 3.0 Capabilities Reference

**Version:** domainstack-schema v0.8.0
**OpenAPI Version:** 3.0.0
**Last Updated:** 2025-12-24

This document provides a comprehensive reference of OpenAPI 3.0 features supported by `domainstack-schema`.

---

## Table of Contents

1. [Feature Coverage Matrix](#feature-coverage-matrix)
2. [Supported Features (Detailed)](#supported-features-detailed)
3. [Unsupported Features](#unsupported-features)
4. [Workarounds for Limitations](#workarounds-for-limitations)
5. [Best Practices](#best-practices)
6. [Complete Examples](#complete-examples)

---

## Feature Coverage Matrix

### OpenAPI Specification Components

| Component | Support Level | Notes |
|-----------|--------------|-------|
| **openapi** | ✅ Full | Fixed at "3.0.0" |
| **info** | ✅ Partial | Title, version, description only |
| **servers** | ❌ None | Out of scope |
| **paths** | ❌ None | Out of scope - use framework adapters |
| **components/schemas** | ✅ Full | **Core focus of this crate** |
| **components/responses** | ❌ None | Out of scope |
| **components/parameters** | ❌ None | Out of scope |
| **components/examples** | ❌ None | Out of scope |
| **components/requestBodies** | ❌ None | Out of scope |
| **components/headers** | ❌ None | Out of scope |
| **components/securitySchemes** | ❌ None | Out of scope |
| **components/links** | ❌ None | Out of scope |
| **components/callbacks** | ❌ None | Out of scope |
| **security** | ❌ None | Out of scope |
| **tags** | ❌ None | Out of scope |
| **externalDocs** | ❌ None | Out of scope |

**Summary:** This crate focuses exclusively on **schema generation** (components/schemas). For full API documentation, integrate with framework-specific tools.

---

### Schema Object Features

#### Core Schema Types

| Feature | Support | API | Example |
|---------|---------|-----|---------|
| **type: string** | ✅ Full | `Schema::string()` | Basic string type |
| **type: number** | ✅ Full | `Schema::number()` | Floating point numbers |
| **type: integer** | ✅ Full | `Schema::integer()` | Integer numbers |
| **type: boolean** | ✅ Full | `Schema::boolean()` | Boolean values |
| **type: array** | ✅ Full | `Schema::array(items)` | Arrays with item schema |
| **type: object** | ✅ Full | `Schema::object()` | Objects with properties |
| **type: null** | ⚠️ Partial | Use `Option<T>` | Via nullable in JSON |

#### String Constraints

| Constraint | Support | API | OpenAPI Field |
|------------|---------|-----|---------------|
| **minLength** | ✅ Full | `.min_length(n)` | `minLength` |
| **maxLength** | ✅ Full | `.max_length(n)` | `maxLength` |
| **pattern** | ✅ Full | `.pattern(regex)` | `pattern` |
| **format** | ✅ Full | `.format(format)` | `format` |
| **enum** | ✅ Full | `.enum_values(&[...])` | `enum` |

**Supported formats:** `email`, `date`, `date-time`, `password`, `uuid`, `uri`, `hostname`, `ipv4`, `ipv6`, `byte`, `binary`, or custom.

#### Numeric Constraints

| Constraint | Support | API | OpenAPI Field |
|------------|---------|-----|---------------|
| **minimum** | ✅ Full | `.minimum(n)` | `minimum` |
| **maximum** | ✅ Full | `.maximum(n)` | `maximum` |
| **exclusiveMinimum** | ❌ None | - | Not supported |
| **exclusiveMaximum** | ❌ None | - | Not supported |
| **multipleOf** | ✅ Full | `.multiple_of(n)` | `multipleOf` |

**Workaround for exclusive:** Use `minimum(n + 1)` for integers, document in description.

#### Array Constraints

| Constraint | Support | API | OpenAPI Field |
|------------|---------|-----|---------------|
| **items** | ✅ Full | `Schema::array(schema)` | `items` |
| **minItems** | ✅ Full | `.min_items(n)` | `minItems` |
| **maxItems** | ✅ Full | `.max_items(n)` | `maxItems` |
| **uniqueItems** | ✅ Full | `.unique_items(true)` | `uniqueItems` |
| **contains** | ❌ None | - | Not supported |

#### Object Constraints

| Constraint | Support | API | OpenAPI Field |
|------------|---------|-----|---------------|
| **properties** | ✅ Full | `.property(name, schema)` | `properties` |
| **required** | ✅ Full | `.required(&[...])` | `required` |
| **additionalProperties** | ⚠️ Partial | Manual JSON | Limited support |
| **minProperties** | ❌ None | - | Not supported |
| **maxProperties** | ❌ None | - | Not supported |
| **propertyNames** | ❌ None | - | Not supported |
| **patternProperties** | ❌ None | - | Not supported |

#### Schema Composition (v0.8+)

| Feature | Support | API | OpenAPI Field |
|---------|---------|-----|---------------|
| **anyOf** | ✅ Full | `Schema::any_of(vec![...])` | `anyOf` |
| **allOf** | ✅ Full | `Schema::all_of(vec![...])` | `allOf` |
| **oneOf** | ✅ Full | `Schema::one_of(vec![...])` | `oneOf` |
| **not** | ❌ None | - | Not supported |

#### Schema Metadata (v0.8+)

| Feature | Support | API | OpenAPI Field |
|---------|---------|-----|---------------|
| **title** | ⚠️ Via name | Schema name becomes title | Implicit |
| **description** | ✅ Full | `.description(text)` | `description` |
| **default** | ✅ Full | `.default(value)` | `default` |
| **example** | ✅ Full | `.example(value)` | `example` |
| **examples** | ✅ Full | `.examples(vec![...])` | `examples` |
| **deprecated** | ✅ Full | `.deprecated(true)` | `deprecated` |
| **readOnly** | ✅ Full | `.read_only(true)` | `readOnly` |
| **writeOnly** | ✅ Full | `.write_only(true)` | `writeOnly` |
| **externalDocs** | ❌ None | - | Not supported |

#### Advanced Features

| Feature | Support | API | OpenAPI Field |
|---------|---------|-----|---------------|
| **$ref** | ✅ Full | `Schema::reference(name)` | `$ref` |
| **discriminator** | ❌ None | - | Not supported |
| **xml** | ❌ None | - | Not supported |
| **nullable** | ⚠️ Via Option | Use Rust `Option<T>` | Implicit |
| **Vendor Extensions (x-*)** | ✅ Full | `.extension(key, value)` | Custom `x-*` fields |

---

## Supported Features (Detailed)

### 1. Basic Schema Types

```rust
use domainstack_schema::Schema;

// String
let name = Schema::string()
    .min_length(1)
    .max_length(100)
    .description("User name");

// Integer with range
let age = Schema::integer()
    .minimum(0)
    .maximum(150);

// Number (float/double)
let price = Schema::number()
    .minimum(0.0)
    .maximum(1000000.0)
    .multiple_of(0.01);  // Penny precision

// Boolean
let active = Schema::boolean()
    .default(json!(true));

// Array of strings
let tags = Schema::array(Schema::string())
    .min_items(1)
    .max_items(10)
    .unique_items(true);

// Object with properties
let user = Schema::object()
    .property("name", Schema::string())
    .property("age", Schema::integer())
    .required(&["name"]);
```

### 2. String Formats

```rust
// Email validation
let email = Schema::string()
    .format("email")
    .description("Must be a valid email address");

// Date/DateTime
let birthdate = Schema::string()
    .format("date")
    .description("ISO 8601 date (YYYY-MM-DD)");

let created_at = Schema::string()
    .format("date-time")
    .description("ISO 8601 timestamp");

// UUID
let id = Schema::string()
    .format("uuid")
    .read_only(true);

// URI/URL
let website = Schema::string()
    .format("uri")
    .description("Full URL including protocol");

// Password (write-only)
let password = Schema::string()
    .format("password")
    .min_length(8)
    .write_only(true)
    .description("User password (never returned in responses)");

// IP addresses
let ipv4 = Schema::string().format("ipv4");
let ipv6 = Schema::string().format("ipv6");

// Hostname
let host = Schema::string().format("hostname");
```

### 3. Pattern Validation

```rust
// Regex patterns
let username = Schema::string()
    .pattern("^[a-zA-Z0-9_]{3,20}$")
    .description("Alphanumeric and underscores, 3-20 characters");

// Phone number
let phone = Schema::string()
    .pattern(r"^\+?[1-9]\d{1,14}$")
    .description("E.164 format phone number");

// Hex color
let color = Schema::string()
    .pattern("^#[0-9A-Fa-f]{6}$")
    .description("Hex color code (e.g., #FF5733)");
```

### 4. Enumerations

```rust
// String enum
let status = Schema::string()
    .enum_values(&["draft", "published", "archived"])
    .default(json!("draft"))
    .description("Post status");

// Integer enum
let priority = Schema::integer()
    .enum_values(&[1, 2, 3, 4, 5])
    .description("Priority level (1=low, 5=critical)");

// Mixed types (use anyOf for this)
let flexible = Schema::any_of(vec![
    Schema::string().enum_values(&["auto"]),
    Schema::integer().minimum(0).maximum(100),
]);
```

### 5. Schema Composition

#### anyOf - Union Types

```rust
// Field can be string OR integer
let id = Schema::any_of(vec![
    Schema::string().format("uuid"),
    Schema::integer().minimum(1),
])
.description("User ID: UUID string or positive integer");

// Nullable by union
let optional_name = Schema::any_of(vec![
    Schema::string(),
    Schema::object().property("type", Schema::string().enum_values(&["null"])),
]);
```

#### allOf - Schema Inheritance

```rust
// Base schema
let base_entity = Schema::object()
    .property("id", Schema::string().format("uuid").read_only(true))
    .property("createdAt", Schema::string().format("date-time").read_only(true))
    .required(&["id", "createdAt"]);

// Extended schema
let user = Schema::all_of(vec![
    Schema::reference("BaseEntity"),
    Schema::object()
        .property("email", Schema::string().format("email"))
        .property("name", Schema::string())
        .required(&["email", "name"]),
])
.description("User extends BaseEntity with email and name");
```

#### oneOf - Discriminated Unions

```rust
// Payment methods with different shapes
let payment = Schema::one_of(vec![
    Schema::object()
        .property("type", Schema::string().enum_values(&["card"]))
        .property("cardNumber", Schema::string().pattern(r"^\d{16}$"))
        .property("cvv", Schema::string().pattern(r"^\d{3,4}$"))
        .required(&["type", "cardNumber", "cvv"])
        .description("Credit card payment"),

    Schema::object()
        .property("type", Schema::string().enum_values(&["paypal"]))
        .property("email", Schema::string().format("email"))
        .required(&["type", "email"])
        .description("PayPal payment"),

    Schema::object()
        .property("type", Schema::string().enum_values(&["cash"]))
        .property("amount", Schema::number().minimum(0))
        .required(&["type", "amount"])
        .description("Cash payment"),
])
.description("Payment method (exactly one type)");
```

### 6. Metadata & Documentation

```rust
let settings = Schema::object()
    .description("User preferences and configuration")
    .property(
        "theme",
        Schema::string()
            .enum_values(&["light", "dark", "auto"])
            .default(json!("auto"))
            .example(json!("dark"))
            .description("UI color theme")
    )
    .property(
        "language",
        Schema::string()
            .default(json!("en"))
            .examples(vec![json!("en"), json!("es"), json!("fr"), json!("de")])
            .description("Preferred language (ISO 639-1 code)")
    )
    .property(
        "fontSize",
        Schema::integer()
            .minimum(10)
            .maximum(24)
            .default(json!(14))
            .example(json!(16))
            .description("Font size in pixels")
    );
```

### 7. Request/Response Modifiers

```rust
let user_schema = Schema::object()
    .description("User account")

    // Read-only: Server-generated, never accepted in requests
    .property(
        "id",
        Schema::string()
            .format("uuid")
            .read_only(true)
            .description("Auto-generated user ID")
    )
    .property(
        "createdAt",
        Schema::string()
            .format("date-time")
            .read_only(true)
            .description("Account creation timestamp")
    )
    .property(
        "updatedAt",
        Schema::string()
            .format("date-time")
            .read_only(true)
            .description("Last update timestamp")
    )

    // Write-only: Accepted in requests, never returned
    .property(
        "password",
        Schema::string()
            .format("password")
            .min_length(8)
            .write_only(true)
            .description("User password (min 8 characters)")
    )

    // Regular fields
    .property(
        "email",
        Schema::string()
            .format("email")
            .description("User email address")
    )
    .property(
        "name",
        Schema::string()
            .min_length(1)
            .max_length(100)
            .description("Full name")
    )

    // Deprecated field
    .property(
        "username",
        Schema::string()
            .deprecated(true)
            .description("DEPRECATED: Use 'email' instead")
    )

    .required(&["email", "name", "password"]);
```

### 8. Schema References

```rust
use domainstack_schema::{OpenApiBuilder, ToSchema};

// Define reusable schemas
struct Address { /* ... */ }
impl ToSchema for Address {
    fn schema_name() -> &'static str { "Address" }
    fn schema() -> Schema {
        Schema::object()
            .property("street", Schema::string())
            .property("city", Schema::string())
            .property("zipCode", Schema::string())
            .required(&["street", "city", "zipCode"])
    }
}

struct User { /* ... */ }
impl ToSchema for User {
    fn schema_name() -> &'static str { "User" }
    fn schema() -> Schema {
        Schema::object()
            .property("name", Schema::string())
            // Reference to Address schema
            .property("address", Schema::reference("Address"))
            // Array of references
            .property("previousAddresses",
                Schema::array(Schema::reference("Address"))
            )
    }
}

// Build spec - references auto-resolved
let spec = OpenApiBuilder::new("API", "1.0.0")
    .register::<Address>()
    .register::<User>()
    .build();
```

### 9. Vendor Extensions

```rust
use serde_json::json;

// For validations that don't map to OpenAPI
let date_range = Schema::object()
    .property("startDate", Schema::string().format("date"))
    .property("endDate", Schema::string().format("date"))
    .required(&["startDate", "endDate"])

    // Cross-field validation via vendor extension
    .extension("x-domainstack-validations", json!({
        "cross_field": [{
            "rule": "endDate > startDate",
            "message": "End date must be after start date",
            "code": "invalid_date_range"
        }]
    }))

    // Custom metadata for your tools
    .extension("x-ui-hints", json!({
        "widget": "date-range-picker",
        "allowPast": false
    }));

// Conditional validation
let order = Schema::object()
    .property("total", Schema::number().minimum(0))
    .property("couponCode", Schema::string())
    .extension("x-domainstack-validations", json!({
        "conditional": [{
            "when": "couponCode.isPresent()",
            "then": {
                "rule": "total > 10.00",
                "message": "Coupon requires minimum $10 order"
            }
        }]
    }));

// Database-level validations
let user = Schema::object()
    .property("email", Schema::string().format("email"))
    .extension("x-domainstack-validations", json!({
        "async": [{
            "rule": "unique_in_db",
            "field": "email",
            "message": "Email already registered"
        }]
    }));
```

---

## Unsupported Features

### OpenAPI 3.0 Features NOT Supported

These features are intentionally out of scope for `domainstack-schema`:

#### 1. API Operations & Paths

```yaml
# NOT SUPPORTED - Use framework adapters instead
paths:
  /users:
    get:
      summary: List users
      responses:
        '200':
          description: Success
```

**Why:** Path/operation definitions are framework-specific. Use:
- `domainstack-axum` for Axum
- `domainstack-actix` for Actix-web
- Or integrate with `utoipa`, `aide`, etc.

#### 2. Security Schemes

```yaml
# NOT SUPPORTED
components:
  securitySchemes:
    bearerAuth:
      type: http
      scheme: bearer
```

**Why:** Authentication is framework/app-specific.

#### 3. Advanced Schema Features

- **discriminator**: Polymorphism hints
  - **Workaround:** Use `oneOf` with type field in each variant

- **not**: Schema negation
  - **Workaround:** Document in description, validate in code

- **exclusiveMinimum/Maximum**: Strict inequalities
  - **Workaround:** Use `minimum(n + 1)` for integers, document for floats

- **xml**: XML-specific metadata
  - **Not needed:** Focus is JSON APIs

#### 4. Object Constraints

- **minProperties/maxProperties**: Property count limits
  - **Workaround:** Document in description, validate in code

- **propertyNames**: Pattern for property names
  - **Use case rare:** Define explicit properties instead

- **patternProperties**: Dynamic property patterns
  - **Workaround:** Use `additionalProperties` with description

---

## Workarounds for Limitations

### 1. Nullable Types

**OpenAPI 3.0 has `nullable: true`**, but we represent this idiomatically in Rust:

```rust
// Instead of Schema with nullable: true
// Use Rust's Option<T>
struct User {
    required_field: String,
    optional_field: Option<String>,  // This is nullable
}

impl ToSchema for User {
    fn schema() -> Schema {
        Schema::object()
            .property("requiredField", Schema::string())
            .property("optionalField", Schema::string())  // Not in required
            .required(&["requiredField"])  // Only required field listed
    }
}

// For union with null, use anyOf
let nullable_string = Schema::any_of(vec![
    Schema::string(),
    Schema::object(),  // Represents null in JSON
]);
```

### 2. Exclusive Minimum/Maximum

```rust
// For integers: use minimum(n + 1)
let positive = Schema::integer()
    .minimum(1)  // equivalent to exclusiveMinimum: 0
    .description("Positive integer (> 0)");

// For floats: document in description
let price = Schema::number()
    .minimum(0.01)  // Close enough
    .description("Price (must be > 0.00)");
```

### 3. Conditional Schemas

```rust
// Use oneOf with explicit variants
let contact = Schema::one_of(vec![
    Schema::object()
        .property("type", Schema::string().enum_values(&["email"]))
        .property("address", Schema::string().format("email"))
        .required(&["type", "address"]),

    Schema::object()
        .property("type", Schema::string().enum_values(&["phone"]))
        .property("number", Schema::string().pattern(r"^\+?[0-9]{10,}$"))
        .required(&["type", "number"]),
])
.description("Contact method: email or phone");
```

### 4. Dependencies Between Fields

```rust
// Use vendor extensions to document
let billing = Schema::object()
    .property("method", Schema::string().enum_values(&["card", "invoice"]))
    .property("cardNumber", Schema::string())
    .property("billingAddress", Schema::reference("Address"))
    .extension("x-field-dependencies", json!({
        "cardNumber": {
            "requiredWhen": { "method": "card" },
            "message": "Card number required when method is 'card'"
        },
        "billingAddress": {
            "requiredWhen": { "method": "invoice" },
            "message": "Billing address required for invoices"
        }
    }))
    .required(&["method"]);
```

---

## Best Practices

### 1. Schema Naming

```rust
// ✅ GOOD: Clear, consistent naming
impl ToSchema for User {
    fn schema_name() -> &'static str { "User" }  // Singular, PascalCase
}

impl ToSchema for UserSettings {
    fn schema_name() -> &'static str { "UserSettings" }
}

// ❌ BAD: Inconsistent naming
fn schema_name() -> &'static str { "user" }  // lowercase
fn schema_name() -> &'static str { "Users" }  // Plural
fn schema_name() -> &'static str { "user_settings" }  // snake_case
```

### 2. Description Quality

```rust
// ✅ GOOD: Descriptive, helpful
Schema::string()
    .format("email")
    .description("User's primary email address for login and notifications")

Schema::integer()
    .minimum(0)
    .maximum(5)
    .description("Priority level: 0 (lowest) to 5 (critical)")

// ❌ BAD: Redundant or missing
Schema::string()
    .description("Email")  // Just restates the field name

Schema::integer()
    .minimum(0)
    .maximum(5)
    // No description - what do the numbers mean?
```

### 3. Use Appropriate Types

```rust
// ✅ GOOD: Semantic types
let id = Schema::string().format("uuid");
let created_at = Schema::string().format("date-time");
let price = Schema::number();  // Can have decimals

// ❌ BAD: Generic types
let id = Schema::string();  // Missing format hint
let created_at = Schema::string();  // Missing format
let price = Schema::integer();  // Loses cents!
```

### 4. Required vs Optional Fields

```rust
// ✅ GOOD: Clear requirements
Schema::object()
    .property("id", Schema::string())          // Required
    .property("email", Schema::string())       // Required
    .property("phone", Schema::string())       // Optional
    .property("middleName", Schema::string())  // Optional
    .required(&["id", "email"])  // Explicit list

// ❌ BAD: Everything required or unclear
Schema::object()
    // ... properties ...
    .required(&["id", "email", "phone", "middleName"])  // Too strict

// Or worse: no required at all
Schema::object()
    // ... properties ...
    // Missing .required() - nothing is required!
```

### 5. readOnly vs writeOnly

```rust
// ✅ GOOD: Appropriate modifiers
Schema::object()
    .property("id",
        Schema::string()
            .read_only(true)  // Server generates, never accept from client
    )
    .property("password",
        Schema::string()
            .write_only(true)  // Accept in request, never return
    )
    .property("email",
        Schema::string()  // Both read and write
    )

// ❌ BAD: Contradictory modifiers
Schema::string()
    .read_only(true)
    .write_only(true)  // Impossible! Can't be both
```

### 6. Vendor Extensions Naming

```rust
// ✅ GOOD: Prefixed with x-, clear purpose
.extension("x-domainstack-validations", json!({ /* ... */ }))
.extension("x-ui-hints", json!({ /* ... */ }))
.extension("x-api-gateway-config", json!({ /* ... */ }))

// ❌ BAD: Missing x- prefix (invalid OpenAPI)
.extension("custom-data", json!({ /* ... */ }))  // Must start with x-
.extension("validation", json!({ /* ... */ }))   // Reserved word
```

### 7. Examples for Better DX

```rust
// ✅ GOOD: Realistic, helpful examples
Schema::string()
    .format("email")
    .example(json!("user@example.com"))

Schema::object()
    .property("name", Schema::string().example(json!("Alice Johnson")))
    .property("age", Schema::integer().example(json!(28)))
    .property("tags", Schema::array(Schema::string())
        .examples(vec![
            json!(["rust", "api", "backend"]),
            json!(["frontend", "react"]),
        ])
    )

// ❌ BAD: Unhelpful examples
Schema::string()
    .format("email")
    .example(json!("string"))  // Not a real email!

Schema::integer()
    .minimum(18)
    .example(json!(0))  // Violates constraints!
```

---

## Complete Examples

### Example 1: E-commerce Product API

```rust
use domainstack_schema::{OpenApiBuilder, Schema, ToSchema};
use serde_json::json;

#[allow(dead_code)]
struct Product {
    id: String,
    name: String,
    description: String,
    price: f64,
    category: String,
    tags: Vec<String>,
    in_stock: bool,
    variants: Vec<ProductVariant>,
}

impl ToSchema for Product {
    fn schema_name() -> &'static str { "Product" }

    fn schema() -> Schema {
        Schema::object()
            .description("Product catalog item")
            .property(
                "id",
                Schema::string()
                    .format("uuid")
                    .read_only(true)
                    .description("Unique product identifier")
            )
            .property(
                "name",
                Schema::string()
                    .min_length(1)
                    .max_length(200)
                    .example(json!("Premium Wireless Headphones"))
                    .description("Product name")
            )
            .property(
                "description",
                Schema::string()
                    .max_length(2000)
                    .example(json!("High-quality wireless headphones with noise cancellation"))
                    .description("Full product description")
            )
            .property(
                "price",
                Schema::number()
                    .minimum(0.01)
                    .maximum(1000000.0)
                    .multiple_of(0.01)
                    .example(json!(199.99))
                    .description("Price in USD")
            )
            .property(
                "category",
                Schema::string()
                    .enum_values(&["electronics", "clothing", "books", "home", "sports"])
                    .example(json!("electronics"))
                    .description("Product category")
            )
            .property(
                "tags",
                Schema::array(Schema::string())
                    .min_items(1)
                    .max_items(10)
                    .unique_items(true)
                    .examples(vec![
                        json!(["wireless", "audio", "premium"]),
                        json!(["sale", "featured"]),
                    ])
                    .description("Search tags")
            )
            .property(
                "inStock",
                Schema::boolean()
                    .default(json!(true))
                    .description("Whether product is currently in stock")
            )
            .property(
                "variants",
                Schema::array(Schema::reference("ProductVariant"))
                    .min_items(1)
                    .description("Available product variants (color, size, etc.)")
            )
            .required(&["id", "name", "price", "category", "tags", "variants"])
    }
}

#[allow(dead_code)]
struct ProductVariant {
    sku: String,
    color: String,
    size: String,
    stock_quantity: u32,
}

impl ToSchema for ProductVariant {
    fn schema_name() -> &'static str { "ProductVariant" }

    fn schema() -> Schema {
        Schema::object()
            .description("Product variant with specific attributes")
            .property(
                "sku",
                Schema::string()
                    .pattern("^[A-Z0-9]{8,12}$")
                    .example(json!("PRD12345678"))
                    .description("Stock Keeping Unit")
            )
            .property(
                "color",
                Schema::string()
                    .example(json!("Midnight Black"))
                    .description("Color option")
            )
            .property(
                "size",
                Schema::string()
                    .enum_values(&["XS", "S", "M", "L", "XL", "XXL"])
                    .description("Size option")
            )
            .property(
                "stockQuantity",
                Schema::integer()
                    .minimum(0)
                    .default(json!(0))
                    .description("Available stock quantity")
            )
            .required(&["sku", "color", "size", "stockQuantity"])
    }
}

fn main() {
    let spec = OpenApiBuilder::new("E-commerce API", "1.0.0")
        .description("Product catalog and inventory management")
        .register::<Product>()
        .register::<ProductVariant>()
        .build();

    println!("{}", spec.to_json().unwrap());
}
```

### Example 2: User Management with Authentication

```rust
use domainstack_schema::{OpenApiBuilder, Schema, ToSchema};
use serde_json::json;

#[allow(dead_code)]
struct UserRegistration {
    email: String,
    password: String,
    name: String,
}

impl ToSchema for UserRegistration {
    fn schema_name() -> &'static str { "UserRegistration" }

    fn schema() -> Schema {
        Schema::object()
            .description("User registration request")
            .property(
                "email",
                Schema::string()
                    .format("email")
                    .example(json!("user@example.com"))
                    .description("User email address (must be unique)")
            )
            .property(
                "password",
                Schema::string()
                    .format("password")
                    .min_length(8)
                    .max_length(128)
                    .write_only(true)
                    .description("Password (min 8 characters, will be hashed)")
                    .extension("x-validation-rules", json!({
                        "pattern": "Must contain uppercase, lowercase, and number",
                        "forbidden": ["password123", "12345678"]
                    }))
            )
            .property(
                "name",
                Schema::string()
                    .min_length(1)
                    .max_length(100)
                    .example(json!("Alice Johnson"))
                    .description("Full name")
            )
            .required(&["email", "password", "name"])
    }
}

#[allow(dead_code)]
struct UserProfile {
    id: String,
    email: String,
    name: String,
    avatar_url: Option<String>,
    created_at: String,
    updated_at: String,
}

impl ToSchema for UserProfile {
    fn schema_name() -> &'static str { "UserProfile" }

    fn schema() -> Schema {
        Schema::object()
            .description("User profile (response only)")
            .property(
                "id",
                Schema::string()
                    .format("uuid")
                    .read_only(true)
                    .example(json!("550e8400-e29b-41d4-a716-446655440000"))
                    .description("Unique user ID")
            )
            .property(
                "email",
                Schema::string()
                    .format("email")
                    .read_only(true)
                    .description("User email (cannot be changed)")
            )
            .property(
                "name",
                Schema::string()
                    .min_length(1)
                    .max_length(100)
                    .description("User full name")
            )
            .property(
                "avatarUrl",
                Schema::string()
                    .format("uri")
                    .description("Profile picture URL (optional)")
            )
            .property(
                "createdAt",
                Schema::string()
                    .format("date-time")
                    .read_only(true)
                    .example(json!("2025-01-15T10:30:00Z"))
                    .description("Account creation timestamp")
            )
            .property(
                "updatedAt",
                Schema::string()
                    .format("date-time")
                    .read_only(true)
                    .description("Last profile update timestamp")
            )
            .required(&["id", "email", "name", "createdAt", "updatedAt"])
    }
}

fn main() {
    let spec = OpenApiBuilder::new("User Management API", "1.0.0")
        .description("User registration and profile management")
        .register::<UserRegistration>()
        .register::<UserProfile>()
        .build();

    println!("{}", spec.to_json().unwrap());
}
```

---

## Validation Coverage

### What Gets Validated at Compile Time

- ✅ Schema structure (correct types)
- ✅ Method chaining correctness
- ✅ ToSchema trait implementation

### What Gets Validated at Runtime

- ⚠️ Example values matching schema constraints (not enforced - your responsibility)
- ⚠️ Reference validity (schema name exists)
- ⚠️ Required fields are defined as properties

### What You Need to Validate Manually

- ❌ Examples match constraints
- ❌ Descriptions are helpful
- ❌ All referenced schemas are registered
- ❌ Vendor extension structure

**Recommendation:** Always run generated OpenAPI through a validator like [Swagger Editor](https://editor.swagger.io/) or `openapi-spec-validator`.

---

## Performance Characteristics

| Operation | Time Complexity | Notes |
|-----------|----------------|-------|
| Schema creation | O(1) | Builder pattern, no validation |
| Adding properties | O(1) | HashMap insert |
| ToJSON serialization | O(n) | Where n = total schema nodes |
| Reference resolution | O(1) | HashMap lookup |

**Memory:** Schemas are small in memory (~100-500 bytes each). Building a 100-schema spec takes <50KB RAM.

---

## Version Compatibility

| domainstack-schema | OpenAPI Spec | Rust MSRV |
|-------------------|--------------|-----------|
| 0.7.0 | 3.0.0 | 1.76+ |
| 0.8.0 | 3.0.0 | 1.76+ |
| Future 1.0.0 | 3.0.0 / 3.1.0 | 1.76+ |

**OpenAPI 3.1.0 support:** Planned for post-1.0 (adds JSON Schema 2020-12 alignment).

---

## Next Steps

1. **Read the README** - Quick start guide
2. **Run examples** - `cargo run --example v08_features`
3. **Implement ToSchema** - For your domain types
4. **Validate output** - Use Swagger Editor
5. **Integrate with framework** - Use domainstack-axum or similar

For questions or issues: https://github.com/blackwell-systems/domainstack/issues

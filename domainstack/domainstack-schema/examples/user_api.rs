//! Complete example of OpenAPI schema generation for a User API.
//!
//! This example demonstrates how to:
//! 1. Define domain types with validation
//! 2. Implement ToSchema to generate OpenAPI schemas
//! 3. Map validation rules to OpenAPI constraints
//! 4. Build a complete OpenAPI specification

use domainstack_schema::{OpenApiBuilder, Schema, ToSchema};

// Domain type (validation can be manual or via derive)
#[allow(dead_code)]
struct User {
    email: String,
    age: u8,
    name: String,
    status: String,
}

// Manual ToSchema implementation that mirrors validation rules
impl ToSchema for User {
    fn schema_name() -> &'static str {
        "User"
    }

    fn schema() -> Schema {
        Schema::object()
            .description("User account")
            .property(
                "email",
                Schema::string()
                    .format("email")
                    .description("User email address"),
            )
            .property(
                "age",
                Schema::integer()
                    .minimum(18)
                    .maximum(120)
                    .description("User age (18-120)"),
            )
            .property(
                "name",
                Schema::string()
                    .min_length(3)
                    .max_length(50)
                    .description("User full name"),
            )
            .property(
                "status",
                Schema::string()
                    .enum_values(&["active", "pending", "inactive"])
                    .description("User account status"),
            )
            .required(&["email", "age", "name", "status"])
    }
}

#[allow(dead_code)]
struct Address {
    street: String,
    zip_code: String,
    city: String,
}

impl ToSchema for Address {
    fn schema_name() -> &'static str {
        "Address"
    }

    fn schema() -> Schema {
        Schema::object()
            .description("Physical address")
            .property(
                "street",
                Schema::string()
                    .min_length(1)
                    .max_length(100)
                    .description("Street address"),
            )
            .property(
                "zipCode",
                Schema::string()
                    .min_length(5)
                    .max_length(5)
                    .pattern("^[0-9]{5}$")
                    .description("5-digit ZIP code"),
            )
            .property(
                "city",
                Schema::string()
                    .min_length(2)
                    .max_length(50)
                    .description("City name"),
            )
            .required(&["street", "zipCode", "city"])
    }
}

#[allow(dead_code)]
struct Team {
    name: String,
    members: Vec<String>, // Simplified - would be Vec<User> in real app
}

impl ToSchema for Team {
    fn schema_name() -> &'static str {
        "Team"
    }

    fn schema() -> Schema {
        Schema::object()
            .description("Team of users")
            .property(
                "name",
                Schema::string()
                    .min_length(1)
                    .max_length(50)
                    .description("Team name"),
            )
            .property(
                "members",
                Schema::array(Schema::reference("User"))
                    .min_items(1)
                    .max_items(10)
                    .description("Team members (1-10 users)"),
            )
            .required(&["name", "members"])
    }
}

fn main() {
    // Build OpenAPI specification
    let spec = OpenApiBuilder::new("User Management API", "1.0.0")
        .description("API for managing users, addresses, and teams with validation")
        .register::<User>()
        .register::<Address>()
        .register::<Team>()
        .build();

    // Output as JSON
    let json = spec.to_json().expect("Failed to serialize to JSON");
    println!("{}", json);

    println!("\n=== Schema Generation Complete ===");
    println!("Registered schemas:");
    println!("  - User (with email, age, name, status)");
    println!("  - Address (with street, zip code, city)");
    println!("  - Team (with name and members array)");
    println!("\nValidation constraints mapped to OpenAPI:");
    println!("  ✓ email → format: email");
    println!("  ✓ range(min, max) → minimum, maximum");
    println!("  ✓ length(min, max) → minLength, maxLength");
    println!("  ✓ one_of → enum");
    println!("  ✓ min_items, max_items → minItems, maxItems");
    println!("  ✓ numeric_string → pattern: ^[0-9]+$");
}

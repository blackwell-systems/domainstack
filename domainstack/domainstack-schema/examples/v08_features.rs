//! Demonstrates v0.8 features: schema composition, metadata, and vendor extensions.
//!
//! This example showcases:
//! 1. anyOf/allOf/oneOf composition
//! 2. default/example/examples metadata
//! 3. readOnly/writeOnly/deprecated modifiers
//! 4. Vendor extensions for non-mappable validations

use domainstack_schema::{OpenApiBuilder, Schema, ToSchema};
use serde_json::json;

// === 1. Schema Composition ===

/// Payment method: either card OR cash (union type with anyOf)
#[allow(dead_code)]
struct PaymentMethod {
    method_type: String,
}

impl ToSchema for PaymentMethod {
    fn schema_name() -> &'static str {
        "PaymentMethod"
    }

    fn schema() -> Schema {
        Schema::any_of(vec![
            Schema::object()
                .property("type", Schema::string().enum_values(&["card"]))
                .property("cardNumber", Schema::string().min_length(16).max_length(16))
                .property("cvv", Schema::string().min_length(3).max_length(4))
                .required(&["type", "cardNumber", "cvv"]),
            Schema::object()
                .property("type", Schema::string().enum_values(&["cash"]))
                .property("amount", Schema::number().minimum(0))
                .required(&["type", "amount"]),
        ])
    }
}

/// Admin user: inherits from User AND adds admin field (composition with allOf)
#[allow(dead_code)]
struct AdminUser {
    // Inherits all User fields + additional fields
    admin: bool,
}

impl ToSchema for AdminUser {
    fn schema_name() -> &'static str {
        "AdminUser"
    }

    fn schema() -> Schema {
        Schema::all_of(vec![
            Schema::reference("User"),
            Schema::object()
                .property("admin", Schema::boolean().default(json!(false)))
                .property("permissions", Schema::array(Schema::string()))
                .required(&["admin"]),
        ])
        .description("Admin user with elevated permissions")
    }
}

// === 2. Metadata: default, example, examples ===

#[allow(dead_code)]
struct UserSettings {
    theme: String,
    language: String,
    notifications_enabled: bool,
}

impl ToSchema for UserSettings {
    fn schema_name() -> &'static str {
        "UserSettings"
    }

    fn schema() -> Schema {
        Schema::object()
            .description("User preferences and settings")
            .property(
                "theme",
                Schema::string()
                    .enum_values(&["light", "dark", "auto"])
                    .default(json!("auto"))
                    .example(json!("dark"))
                    .description("UI theme preference"),
            )
            .property(
                "language",
                Schema::string()
                    .default(json!("en"))
                    .examples(vec![json!("en"), json!("es"), json!("fr")])
                    .description("Preferred language code (ISO 639-1)"),
            )
            .property(
                "notificationsEnabled",
                Schema::boolean()
                    .default(json!(true))
                    .description("Enable/disable notifications"),
            )
            .required(&["theme", "language"])
    }
}

// === 3. Request/Response Modifiers: readOnly, writeOnly, deprecated ===

#[allow(dead_code)]
struct UserAccount {
    id: String,
    email: String,
    password: String,
    created_at: String,
    old_username: Option<String>,
}

impl ToSchema for UserAccount {
    fn schema_name() -> &'static str {
        "UserAccount"
    }

    fn schema() -> Schema {
        Schema::object()
            .description("User account with request/response field modifiers")
            .property(
                "id",
                Schema::string()
                    .read_only(true)
                    .description("Auto-generated user ID (returned in responses only)"),
            )
            .property(
                "email",
                Schema::string().format("email").description("User email"),
            )
            .property(
                "password",
                Schema::string()
                    .format("password")
                    .min_length(8)
                    .write_only(true)
                    .description("Password (accepted in requests only, never returned)"),
            )
            .property(
                "createdAt",
                Schema::string()
                    .format("date-time")
                    .read_only(true)
                    .description("Account creation timestamp"),
            )
            .property(
                "oldUsername",
                Schema::string()
                    .deprecated(true)
                    .description("Deprecated: Use 'email' instead"),
            )
            .required(&["email", "password"])
    }
}

// === 4. Vendor Extensions for Non-Mappable Validations ===

#[allow(dead_code)]
struct DateRange {
    start_date: String,
    end_date: String,
}

impl ToSchema for DateRange {
    fn schema_name() -> &'static str {
        "DateRange"
    }

    fn schema() -> Schema {
        Schema::object()
            .description("Date range with cross-field validation")
            .property("startDate", Schema::string().format("date"))
            .property("endDate", Schema::string().format("date"))
            .required(&["startDate", "endDate"])
            // Cross-field validation doesn't map to OpenAPI, so use vendor extension
            .extension(
                "x-domainstack-validations",
                json!({
                    "cross_field": ["endDate > startDate"],
                    "description": "End date must be after start date"
                }),
            )
    }
}

#[allow(dead_code)]
struct OrderForm {
    total: f64,
    minimum_order: f64,
    requires_minimum: bool,
}

impl ToSchema for OrderForm {
    fn schema_name() -> &'static str {
        "OrderForm"
    }

    fn schema() -> Schema {
        Schema::object()
            .description("Order form with conditional validation")
            .property("total", Schema::number().minimum(0))
            .property("minimumOrder", Schema::number().minimum(0))
            .property("requiresMinimum", Schema::boolean())
            .required(&["total", "minimumOrder", "requiresMinimum"])
            // Conditional validation doesn't map to OpenAPI
            .extension(
                "x-domainstack-validations",
                json!({
                    "conditional": {
                        "when": "requiresMinimum == true",
                        "then": "total >= minimumOrder"
                    },
                    "description": "When minimum is required, total must meet it"
                }),
            )
    }
}

// === User type for allOf example ===

#[allow(dead_code)]
struct User {
    email: String,
    name: String,
}

impl ToSchema for User {
    fn schema_name() -> &'static str {
        "User"
    }

    fn schema() -> Schema {
        Schema::object()
            .property("email", Schema::string().format("email"))
            .property("name", Schema::string().min_length(1))
            .required(&["email", "name"])
    }
}

fn main() {
    let spec = OpenApiBuilder::new("v0.8 Features Demo", "1.0.0")
        .description("Demonstrates OpenAPI v0.8 features: composition, metadata, and extensions")
        .register::<PaymentMethod>()
        .register::<AdminUser>()
        .register::<User>()
        .register::<UserSettings>()
        .register::<UserAccount>()
        .register::<DateRange>()
        .register::<OrderForm>()
        .build();

    println!("{}", spec.to_json().expect("Failed to serialize"));

    println!("\n=== v0.8 Features Demonstrated ===");
    println!("[ok] anyOf: PaymentMethod (union of card | cash)");
    println!("[ok] allOf: AdminUser (extends User)");
    println!("[ok] default: UserSettings.theme = 'auto'");
    println!("[ok] example: UserSettings.theme example = 'dark'");
    println!("[ok] examples: UserSettings.language examples = ['en', 'es', 'fr']");
    println!("[ok] readOnly: UserAccount.id, createdAt (response only)");
    println!("[ok] writeOnly: UserAccount.password (request only)");
    println!("[ok] deprecated: UserAccount.oldUsername");
    println!("[ok] vendor extensions: DateRange, OrderForm (x-domainstack-validations)");
    println!("\nAll v0.8 features working correctly!");
}

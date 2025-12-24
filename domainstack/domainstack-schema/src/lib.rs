//! OpenAPI 3.0 schema generation for domainstack validation types.
//!
//! This crate provides tools to generate OpenAPI 3.0 component schemas from Rust types,
//! with a focus on mapping validation rules to OpenAPI constraints.
//!
//! # Features
//!
//! - **Schema Composition** - anyOf, allOf, oneOf for union and inheritance patterns
//! - **Rich Metadata** - default values, examples, deprecation markers
//! - **Request/Response Modifiers** - readOnly, writeOnly for accurate API modeling
//! - **Vendor Extensions** - Preserve non-mappable validations (cross-field, conditional, etc.)
//! - **Type-Safe** - Fluent builder API with compile-time guarantees
//! - **Framework Agnostic** - Works with any Rust web framework
//!
//! # Quick Start
//!
//! ```rust
//! use domainstack_schema::{OpenApiBuilder, Schema, ToSchema};
//! use serde_json::json;
//!
//! struct User {
//!     email: String,
//!     age: u8,
//! }
//!
//! impl ToSchema for User {
//!     fn schema_name() -> &'static str {
//!         "User"
//!     }
//!
//!     fn schema() -> Schema {
//!         Schema::object()
//!             .property("email", Schema::string().format("email"))
//!             .property("age", Schema::integer().minimum(18).maximum(120))
//!             .required(&["email", "age"])
//!     }
//! }
//!
//! let spec = OpenApiBuilder::new("My API", "1.0.0")
//!     .description("User management API")
//!     .register::<User>()
//!     .build();
//!
//! let json = spec.to_json().expect("Failed to serialize");
//! println!("{}", json);
//! ```
//!
//! # Validation Rule Mapping
//!
//! Maps domainstack validation rules to OpenAPI constraints:
//!
//! | Validation | OpenAPI | Example |
//! |------------|---------|---------|
//! | `length(min, max)` | `minLength`, `maxLength` | `.min_length(3).max_length(50)` |
//! | `range(min, max)` | `minimum`, `maximum` | `.minimum(0).maximum(100)` |
//! | `email()` | `format: "email"` | `.format("email")` |
//! | `one_of(...)` | `enum` | `.enum_values(&["a", "b"])` |
//! | `numeric_string()` | `pattern` | `.pattern("^[0-9]+$")` |
//! | `min_items(n)` | `minItems` | `.min_items(1)` |
//! | `max_items(n)` | `maxItems` | `.max_items(10)` |
//!
//! # Schema Composition (v0.8+)
//!
//! ## Union Types (anyOf)
//!
//! ```rust
//! use domainstack_schema::Schema;
//!
//! // Field can be string OR integer
//! let flexible = Schema::any_of(vec![
//!     Schema::string(),
//!     Schema::integer(),
//! ]);
//! ```
//!
//! ## Inheritance (allOf)
//!
//! ```rust
//! use domainstack_schema::Schema;
//!
//! // AdminUser extends User
//! let admin = Schema::all_of(vec![
//!     Schema::reference("User"),
//!     Schema::object().property("admin", Schema::boolean()),
//! ]);
//! ```
//!
//! ## Discriminated Unions (oneOf)
//!
//! ```rust
//! use domainstack_schema::Schema;
//!
//! let payment = Schema::one_of(vec![
//!     Schema::object()
//!         .property("type", Schema::string().enum_values(&["card"]))
//!         .property("cardNumber", Schema::string()),
//!     Schema::object()
//!         .property("type", Schema::string().enum_values(&["cash"]))
//!         .property("amount", Schema::number()),
//! ]);
//! ```
//!
//! # Metadata & Documentation (v0.8+)
//!
//! ```rust
//! use domainstack_schema::Schema;
//! use serde_json::json;
//!
//! let theme = Schema::string()
//!     .enum_values(&["light", "dark", "auto"])
//!     .default(json!("auto"))          // Default value
//!     .example(json!("dark"))           // Single example
//!     .examples(vec![                   // Multiple examples
//!         json!("light"),
//!         json!("dark"),
//!     ])
//!     .description("UI theme preference");
//! ```
//!
//! # Request/Response Modifiers (v0.8+)
//!
//! ```rust
//! use domainstack_schema::Schema;
//!
//! let user = Schema::object()
//!     .property("id",
//!         Schema::string()
//!             .read_only(true)         // Response only
//!             .description("Auto-generated ID")
//!     )
//!     .property("password",
//!         Schema::string()
//!             .write_only(true)        // Request only
//!             .min_length(8)
//!     )
//!     .property("oldField",
//!         Schema::string()
//!             .deprecated(true)        // Mark as deprecated
//!     );
//! ```
//!
//! # Vendor Extensions (v0.8+)
//!
//! For validations that don't map to OpenAPI (cross-field, conditional, async):
//!
//! ```rust
//! use domainstack_schema::Schema;
//! use serde_json::json;
//!
//! let date_range = Schema::object()
//!     .property("startDate", Schema::string().format("date"))
//!     .property("endDate", Schema::string().format("date"))
//!     .extension("x-domainstack-validations", json!({
//!         "cross_field": ["endDate > startDate"]
//!     }));
//! ```
//!
//! # Scope & Limitations
//!
//! **What this crate does:**
//! - Generates OpenAPI 3.0 component schemas for domain types
//! - Maps field-level validations to OpenAPI constraints
//! - Provides type-safe schema builders
//! - Exports to JSON/YAML
//!
//! **What this crate does NOT do:**
//! - API paths/operations (GET /users, POST /users, etc.)
//! - Request/response body definitions
//! - Security schemes or authentication
//! - Full API documentation generation
//!
//! For complete API documentation, use framework adapters:
//! - `domainstack-axum` for Axum
//! - `domainstack-actix` for Actix-web
//! - Or integrate with `utoipa`, `aide`, etc.
//!
//! # Complete Documentation
//!
//! See `OPENAPI_CAPABILITIES.md` for:
//! - Complete feature matrix
//! - Detailed examples for every feature
//! - Workarounds for limitations
//! - Best practices
//! - Performance characteristics

mod openapi;
mod schema;
mod traits;

pub use openapi::{OpenApiBuilder, OpenApiSpec};
pub use schema::{Schema, SchemaType};
pub use traits::ToSchema;

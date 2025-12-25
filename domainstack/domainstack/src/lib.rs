//! # domainstack
//!
//! Turn untrusted input into valid domain objects—with structured, field-level errors.
//!
//! Most validation crates answer: **"Is this DTO valid?"**
//! domainstack answers: **"How do I safely construct domain models from untrusted input, and return a stable error contract?"**
//!
//! ## Core Features
//!
//! - **Domain-first modeling** - Invalid states are unrepresentable
//! - **37 validation rules** - String, numeric, collection, date/time validation out of the box
//! - **Composable rule algebra** - `.and()`, `.or()`, `.when()` combinators
//! - **Structured error paths** - `rooms[0].adults`, `guest.email.value` for APIs/UIs
//! - **Auto-derived OpenAPI schemas** - Write validation once, get OpenAPI 3.0 automatically
//! - **Serde integration** - Validate during deserialization with `#[derive(ValidateOnDeserialize)]`
//! - **Async validation** - Database uniqueness checks with context passing
//! - **Cross-field validation** - Password confirmation, date ranges, business rules
//! - **Type-state tracking** - Compile-time guarantees with phantom types
//! - **Zero dependencies** - Lightweight core, optional features for regex/async/chrono
//!
//! ## Quick Start
//!
//! ```rust
//! use domainstack::prelude::*;
//! use domainstack::Validate;
//!
//! // Derive validation for your domain types
//! #[derive(Debug, Validate)]
//! struct User {
//!     #[validate(length(min = 2, max = 50))]
//!     name: String,
//!
//!     #[validate(range(min = 18, max = 120))]
//!     age: u8,
//!
//!     #[validate(each(nested))]
//!     emails: Vec<Email>,
//! }
//!
//! #[derive(Debug, Validate)]
//! struct Email {
//!     #[validate(email)]
//!     #[validate(max_len = 255)]
//!     value: String,
//! }
//!
//! // Validation happens with one call
//! let user = User {
//!     name: "Alice".to_string(),
//!     age: 30,
//!     emails: vec![Email { value: "alice@example.com".to_string() }],
//! };
//!
//! match user.validate() {
//!     Ok(_) => println!("Valid!"),
//!     Err(e) => {
//!         for v in &e.violations {
//!             println!("[{}] {} - {}", v.path, v.code, v.message);
//!         }
//!     }
//! }
//! ```
//!
//! ## Documentation
//!
//! - [API Guide](https://github.com/blackwell-systems/domainstack/blob/main/domainstack/domainstack/docs/api-guide.md) - Complete API documentation
//! - [Rules Reference](https://github.com/blackwell-systems/domainstack/blob/main/domainstack/domainstack/docs/RULES.md) - All 37 validation rules
//! - [Examples](https://github.com/blackwell-systems/domainstack/tree/main/domainstack/domainstack-examples) - 9 runnable examples

mod context;
mod error;
mod helpers;
mod path;
mod rule;
mod validate;
mod violation;

#[cfg(feature = "async")]
mod async_validate;

pub mod prelude;
pub mod rules;
pub mod typestate;

pub use context::RuleContext;
pub use error::ValidationError;
pub use helpers::validate;
pub use path::{Path, PathSegment};
pub use rule::Rule;
pub use validate::Validate;
pub use violation::{Meta, Violation};

#[cfg(feature = "async")]
pub use async_validate::{AsyncRule, AsyncValidate, ValidationContext};

#[cfg(feature = "derive")]
pub use domainstack_derive::Validate;

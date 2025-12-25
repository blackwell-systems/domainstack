//! # domainstack
//!
//! domainstack turns untrusted input into valid-by-construction domain objects, and returns stable,
//! field-level errors your APIs and UIs can depend on.
//!
//! It's built for the boundary you actually live at:
//!
//! **HTTP/JSON/etc. → DTOs → Domain (validated) → Business logic**
//!
//! Most validation crates ask: **"Is this DTO valid?"**
//! domainstack asks: **"How do I safely construct domain models from untrusted input—and report failures with a consistent error contract?"**
//!
//! ## What that gives you
//!
//! - **Domain-first modeling**: invalid states are hard/impossible to represent
//! - **Composable rule algebra**: reusable rules with `.and()`, `.or()`, `.when()`
//! - **Structured error paths**: `rooms[0].adults`, `guest.email.value` (UI-friendly)
//! - **Async validation with context**: DB/API checks like uniqueness, rate limits
//! - **Cross-field validation**: invariants like password confirmation, date ranges
//! - **Type-state tracking**: phantom types to enforce "validated" at compile time
//! - **Schema + client parity**: generate OpenAPI and TypeScript/Zod from the same Rust rules
//! - **Framework adapters**: one-line boundary extraction (Axum / Actix / Rocket)
//! - **Lean core**: zero-deps base, opt-in features for regex / async / chrono / serde
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

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
//! // Domain models with validation rules (invalid states impossible)
//! #[derive(Debug, Validate)]
//! #[validate(
//!     check = "self.check_in < self.check_out",
//!     message = "Check-out must be after check-in"
//! )]
//! struct Booking {
//!     #[validate(email, max_len = 255)]
//!     guest_email: String,
//!
//!     check_in: u32,  // Unix timestamp for example
//!     check_out: u32,
//!
//!     #[validate(min_items = 1, max_items = 5)]
//!     #[validate(each(nested))]
//!     rooms: Vec<Room>,
//! }
//!
//! #[derive(Debug, Validate)]
//! struct Room {
//!     #[validate(range(min = 1, max = 4))]
//!     adults: u8,
//!
//!     #[validate(range(min = 0, max = 3))]
//!     children: u8,
//! }
//!
//! // Build your booking
//! let booking = Booking {
//!     guest_email: "guest@example.com".to_string(),
//!     check_in: 1704067200,
//!     check_out: 1704153600,
//!     rooms: vec![
//!         Room { adults: 2, children: 1 },
//!         Room { adults: 5, children: 0 },  // Invalid: too many adults!
//!     ],
//! };
//!
//! // Validate all fields + cross-field rules in one call
//! match booking.validate() {
//!     Ok(_) => println!("Booking valid"),
//!     Err(e) => {
//!         // Structured errors with precise paths:
//!         // [rooms[1].adults] out_of_range - Must be between 1 and 4
//!         for v in &e.violations {
//!             println!("[{}] {} - {}", v.path, v.code, v.message);
//!         }
//!     }
//! }
//! ```
//!
//! With framework adapters (Axum/Actix/Rocket), this becomes a one-line extraction:
//! `DomainJson<Booking, BookingDto>` automatically deserializes, validates, and converts DTOs
//! to domain types—returning structured errors to clients on failure.
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

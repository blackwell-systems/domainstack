//! # domain-model
//!
//! A Rust validation framework for domain-driven design.
//!
//! ## Quick Start
//!
//! ```rust
//! use domain_model::prelude::*;
//!
//! struct Email(String);
//!
//! impl Email {
//!     pub fn new(raw: String) -> Result<Self, ValidationError> {
//!         validate("email", raw.as_str(), rules::email())?;
//!         Ok(Self(raw))
//!     }
//! }
//! ```
//!
//! ## Features
//!
//! - **Valid-by-construction types** - Invalid states can't exist
//! - **Composable rules** - Combine validation logic with `and`, `or`, `when`
//! - **Structured error paths** - Field-level error reporting
//! - **Zero dependencies** - Core crate uses only std (regex optional for email validation)
//!
//! ## Usage
//!
//! See examples/ directory for complete examples.

mod error;
mod helpers;
mod path;
mod rule;
mod validate;
mod violation;

pub mod prelude;
pub mod rules;

pub use error::ValidationError;
pub use helpers::validate;
pub use path::{Path, PathSegment};
pub use rule::Rule;
pub use validate::Validate;
pub use violation::{Meta, Violation};

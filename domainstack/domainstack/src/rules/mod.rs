pub mod numeric;
pub mod string;

pub use numeric::{max, min, multiple_of, negative, positive, range};
pub use string::{
    alpha_only, alphanumeric, contains, email, ends_with, length, max_len, min_len, non_empty,
    numeric_string, starts_with, url,
};

#[cfg(feature = "regex")]
pub use string::matches_regex;

pub mod choice;
pub mod collection;
pub mod numeric;
pub mod string;

pub use choice::{equals, not_equals, one_of};
pub use collection::{max_items, min_items, non_empty_items, unique};
pub use numeric::{
    finite, float_max, float_min, float_range, max, min, multiple_of, negative, non_zero, positive,
    range, try_multiple_of, FiniteCheck,
};
pub use string::{
    alpha_only, alphanumeric, ascii, contains, ends_with, len_chars, length, max_len, min_len,
    no_whitespace, non_blank, non_empty, numeric_string, starts_with,
};

#[cfg(feature = "regex")]
pub use string::{email, matches_regex, try_matches_regex, url};

#[cfg(feature = "chrono")]
pub mod datetime;

#[cfg(feature = "chrono")]
pub use datetime::{after, age_range, before, future, past};

pub mod numeric;
pub mod string;

pub use numeric::{max, min, range};
pub use string::{email, length, max_len, min_len, non_empty};

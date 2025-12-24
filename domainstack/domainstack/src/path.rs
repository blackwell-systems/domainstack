/// Represents a path to a field in a nested structure.
///
/// Paths are used to identify which field caused a validation error in nested
/// and collection structures. They support dot notation for nested fields and
/// bracket notation for array indices.
///
/// # Examples
///
/// ```
/// use domainstack::Path;
///
/// // Simple field path
/// let path = Path::root().field("email");
/// assert_eq!(path.to_string(), "email");
///
/// // Nested path
/// let path = Path::root().field("user").field("email");
/// assert_eq!(path.to_string(), "user.email");
///
/// // Collection path
/// let path = Path::root().field("items").index(0).field("name");
/// assert_eq!(path.to_string(), "items[0].name");
/// ```
///
/// # Memory Considerations
///
/// This implementation uses `Box::leak()` in the `parse()` method to create `'static`
/// references to field names. This is intentional - field names in validation paths need
/// static lifetime because:
///
/// 1. Paths can be stored in `ValidationError` which must be `'static`
/// 2. Field names are typically known at compile time (from derive macro)
/// 3. The number of unique field paths is bounded by your schema
///
/// **Memory Impact:** Each unique field name is leaked once when parsed from a string.
/// For a typical application with ~100 domain types and ~10 fields each, this is
/// ~1000 leaked strings (~50KB). This is negligible for server applications.
///
/// **When to worry:** If you're dynamically generating millions of unique field names
/// at runtime using `Path::parse()`, this could accumulate memory. In that case, use
/// the fluent API (`Path::root().field("name")`) which uses compile-time `&'static str`
/// without leaking memory.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Path(pub Vec<PathSegment>);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PathSegment {
    Field(&'static str),
    Index(usize),
}

impl Path {
    /// Creates an empty root path.
    ///
    /// # Examples
    ///
    /// ```
    /// use domainstack::Path;
    ///
    /// let path = Path::root();
    /// assert_eq!(path.to_string(), "");
    /// ```
    pub fn root() -> Self {
        Self(Vec::new())
    }

    /// Appends a field name to the path.
    ///
    /// # Examples
    ///
    /// ```
    /// use domainstack::Path;
    ///
    /// let path = Path::root().field("email");
    /// assert_eq!(path.to_string(), "email");
    ///
    /// let nested = Path::root().field("user").field("email");
    /// assert_eq!(nested.to_string(), "user.email");
    /// ```
    pub fn field(mut self, name: &'static str) -> Self {
        self.0.push(PathSegment::Field(name));
        self
    }

    /// Appends an array index to the path.
    ///
    /// # Examples
    ///
    /// ```
    /// use domainstack::Path;
    ///
    /// let path = Path::root().field("items").index(0);
    /// assert_eq!(path.to_string(), "items[0]");
    ///
    /// let nested = Path::root().field("items").index(0).field("name");
    /// assert_eq!(nested.to_string(), "items[0].name");
    /// ```
    pub fn index(mut self, idx: usize) -> Self {
        self.0.push(PathSegment::Index(idx));
        self
    }

    /// Parses a path from a string representation.
    ///
    /// **Note:** This method uses `Box::leak()` to create `'static` references to field names.
    /// See the [`Path`] documentation for memory considerations. Prefer using the fluent API
    /// (`Path::root().field("name")`) when field names are known at compile time.
    ///
    /// # Examples
    ///
    /// ```
    /// use domainstack::Path;
    ///
    /// let path = Path::parse("user.email");
    /// assert_eq!(path, Path::root().field("user").field("email"));
    ///
    /// let with_index = Path::parse("items[0].name");
    /// assert_eq!(with_index, Path::root().field("items").index(0).field("name"));
    /// ```
    pub fn parse(s: &str) -> Self {
        let mut segments = Vec::new();
        let mut current = String::new();

        let chars: Vec<char> = s.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            match chars[i] {
                '.' => {
                    if !current.is_empty() {
                        segments.push(PathSegment::Field(Box::leak(
                            current.clone().into_boxed_str(),
                        )));
                        current.clear();
                    }
                    i += 1;
                }
                '[' => {
                    if !current.is_empty() {
                        segments.push(PathSegment::Field(Box::leak(
                            current.clone().into_boxed_str(),
                        )));
                        current.clear();
                    }

                    i += 1;
                    let mut index_str = String::new();
                    while i < chars.len() && chars[i] != ']' {
                        index_str.push(chars[i]);
                        i += 1;
                    }

                    if let Ok(idx) = index_str.parse::<usize>() {
                        segments.push(PathSegment::Index(idx));
                    }

                    i += 1;
                }
                _ => {
                    current.push(chars[i]);
                    i += 1;
                }
            }
        }

        if !current.is_empty() {
            segments.push(PathSegment::Field(Box::leak(current.into_boxed_str())));
        }

        Path(segments)
    }
}

impl core::fmt::Display for Path {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        for (i, segment) in self.0.iter().enumerate() {
            match segment {
                PathSegment::Field(name) => {
                    if i > 0 {
                        write!(f, ".")?;
                    }
                    write!(f, "{}", name)?;
                }
                PathSegment::Index(idx) => write!(f, "[{}]", idx)?,
            }
        }
        Ok(())
    }
}

impl From<&'static str> for Path {
    fn from(s: &'static str) -> Self {
        Path(vec![PathSegment::Field(s)])
    }
}

impl From<String> for Path {
    fn from(s: String) -> Self {
        Path::parse(&s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_root() {
        let path = Path::root();
        assert!(path.0.is_empty());
        assert_eq!(path.to_string(), "");
    }

    #[test]
    fn test_field() {
        let path = Path::root().field("email");
        assert_eq!(path.0.len(), 1);
        assert_eq!(path.to_string(), "email");
    }

    #[test]
    fn test_nested_field() {
        let path = Path::root().field("guest").field("email");
        assert_eq!(path.0.len(), 2);
        assert_eq!(path.to_string(), "guest.email");
    }

    #[test]
    fn test_index() {
        let path = Path::root().field("guests").index(0);
        assert_eq!(path.0.len(), 2);
        assert_eq!(path.to_string(), "guests[0]");
    }

    #[test]
    fn test_complex_path() {
        let path = Path::root()
            .field("booking")
            .field("guests")
            .index(0)
            .field("email");
        assert_eq!(path.to_string(), "booking.guests[0].email");
    }

    #[test]
    fn test_from_str() {
        let path = Path::from("email");
        assert_eq!(path.0.len(), 1);
        assert_eq!(path.to_string(), "email");
    }

    #[test]
    fn test_parse_simple() {
        let path = Path::parse("email");
        assert_eq!(path.to_string(), "email");
    }

    #[test]
    fn test_parse_nested() {
        let path = Path::parse("guest.email");
        assert_eq!(path.to_string(), "guest.email");
    }

    #[test]
    fn test_parse_with_index() {
        let path = Path::parse("guests[0].email");
        assert_eq!(path.to_string(), "guests[0].email");
    }

    #[test]
    fn test_parse_complex() {
        let path = Path::parse("booking.guests[0].email");
        assert_eq!(path.to_string(), "booking.guests[0].email");
    }
}

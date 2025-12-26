use std::sync::Arc;

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
/// # Memory Management
///
/// Path uses `Arc<str>` for field names, providing:
/// - **No memory leaks:** Reference counting ensures proper cleanup
/// - **Efficient cloning:** Cloning a path is cheap (just incrementing reference counts)
/// - **Shared ownership:** Multiple errors can reference the same field names
///
/// Field names from compile-time literals (`"email"`) are converted to `Arc<str>`
/// on first use and reference-counted thereafter.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Path(Vec<PathSegment>);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PathSegment {
    Field(Arc<str>),
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
    pub fn field(mut self, name: impl Into<Arc<str>>) -> Self {
        self.0.push(PathSegment::Field(name.into()));
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
    /// Uses `Arc<str>` for field names, ensuring proper memory management
    /// without leaks. Field names are reference-counted and cleaned up
    /// when no longer needed.
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
                        segments.push(PathSegment::Field(Arc::from(current.as_str())));
                        current.clear();
                    }
                    i += 1;
                }
                '[' => {
                    if !current.is_empty() {
                        segments.push(PathSegment::Field(Arc::from(current.as_str())));
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
            segments.push(PathSegment::Field(Arc::from(current.as_str())));
        }

        Path(segments)
    }

    /// Returns a slice of the path segments.
    ///
    /// # Examples
    ///
    /// ```
    /// use domainstack::{Path, PathSegment};
    ///
    /// let path = Path::root().field("user").index(0).field("name");
    /// assert_eq!(path.segments().len(), 3);
    /// ```
    pub fn segments(&self) -> &[PathSegment] {
        &self.0
    }

    /// Pushes a field segment to the path.
    ///
    /// # Examples
    ///
    /// ```
    /// use domainstack::Path;
    ///
    /// let mut path = Path::root();
    /// path.push_field("email");
    /// assert_eq!(path.to_string(), "email");
    /// ```
    pub fn push_field(&mut self, name: impl Into<Arc<str>>) {
        self.0.push(PathSegment::Field(name.into()));
    }

    /// Pushes an index segment to the path.
    ///
    /// # Examples
    ///
    /// ```
    /// use domainstack::Path;
    ///
    /// let mut path = Path::root();
    /// path.push_field("items");
    /// path.push_index(0);
    /// assert_eq!(path.to_string(), "items[0]");
    /// ```
    pub fn push_index(&mut self, idx: usize) {
        self.0.push(PathSegment::Index(idx));
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
        Path(vec![PathSegment::Field(Arc::from(s))])
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
        assert!(path.segments().is_empty());
        assert_eq!(path.to_string(), "");
    }

    #[test]
    fn test_field() {
        let path = Path::root().field("email");
        assert_eq!(path.segments().len(), 1);
        assert_eq!(path.to_string(), "email");
    }

    #[test]
    fn test_nested_field() {
        let path = Path::root().field("guest").field("email");
        assert_eq!(path.segments().len(), 2);
        assert_eq!(path.to_string(), "guest.email");
    }

    #[test]
    fn test_index() {
        let path = Path::root().field("guests").index(0);
        assert_eq!(path.segments().len(), 2);
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
        assert_eq!(path.segments().len(), 1);
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

    // Tests for push methods
    #[test]
    fn test_push_field_basic() {
        let mut path = Path::root();
        path.push_field("email");
        assert_eq!(path.segments().len(), 1);
        assert_eq!(path.to_string(), "email");
    }

    #[test]
    fn test_push_field_multiple() {
        let mut path = Path::root();
        path.push_field("user");
        path.push_field("profile");
        path.push_field("email");
        assert_eq!(path.segments().len(), 3);
        assert_eq!(path.to_string(), "user.profile.email");
    }

    #[test]
    fn test_push_index_basic() {
        let mut path = Path::root();
        path.push_field("items");
        path.push_index(0);
        assert_eq!(path.segments().len(), 2);
        assert_eq!(path.to_string(), "items[0]");
    }

    #[test]
    fn test_push_index_multiple() {
        let mut path = Path::root();
        path.push_field("matrix");
        path.push_index(0);
        path.push_index(1);
        assert_eq!(path.segments().len(), 3);
        assert_eq!(path.to_string(), "matrix[0][1]");
    }

    #[test]
    fn test_push_field_and_index_mixed() {
        let mut path = Path::root();
        path.push_field("orders");
        path.push_index(5);
        path.push_field("items");
        path.push_index(3);
        path.push_field("sku");
        assert_eq!(path.segments().len(), 5);
        assert_eq!(path.to_string(), "orders[5].items[3].sku");
    }

    #[test]
    fn test_push_with_string() {
        let mut path = Path::root();
        path.push_field(String::from("dynamic_field"));
        assert_eq!(path.to_string(), "dynamic_field");
    }

    // Parse edge cases
    #[test]
    fn test_parse_empty_string() {
        let path = Path::parse("");
        assert!(path.segments().is_empty());
        assert_eq!(path.to_string(), "");
    }

    #[test]
    fn test_parse_leading_dot() {
        let path = Path::parse(".field");
        // Leading dot should be skipped, resulting in just "field"
        assert_eq!(path.to_string(), "field");
    }

    #[test]
    fn test_parse_trailing_dot() {
        let path = Path::parse("field.");
        // Trailing dot is ignored
        assert_eq!(path.to_string(), "field");
    }

    #[test]
    fn test_parse_consecutive_dots() {
        let path = Path::parse("a..b");
        // Empty segments between dots are skipped
        assert_eq!(path.to_string(), "a.b");
    }

    #[test]
    fn test_parse_consecutive_indices() {
        let path = Path::parse("items[0][1][2]");
        assert_eq!(path.segments().len(), 4);
        assert_eq!(path.to_string(), "items[0][1][2]");
    }

    #[test]
    fn test_parse_invalid_index_ignored() {
        let path = Path::parse("items[abc]");
        // Non-numeric index is ignored, only field is captured
        assert_eq!(path.segments().len(), 1);
        assert_eq!(path.to_string(), "items");
    }

    #[test]
    fn test_parse_negative_index_ignored() {
        let path = Path::parse("items[-1]");
        // Negative index can't be parsed as usize, ignored
        assert_eq!(path.segments().len(), 1);
        assert_eq!(path.to_string(), "items");
    }

    #[test]
    fn test_parse_unclosed_bracket() {
        let path = Path::parse("items[0");
        // Unclosed bracket - index still captured
        assert_eq!(path.segments().len(), 2);
        assert_eq!(path.to_string(), "items[0]");
    }

    #[test]
    fn test_parse_deep_nesting() {
        let path = Path::parse("a.b.c.d.e.f.g.h.i.j.k");
        assert_eq!(path.segments().len(), 11);
        assert_eq!(path.to_string(), "a.b.c.d.e.f.g.h.i.j.k");
    }

    #[test]
    fn test_parse_deep_mixed_nesting() {
        let path = Path::parse("a[0].b[1].c[2].d[3].e[4]");
        assert_eq!(path.segments().len(), 10);
        assert_eq!(path.to_string(), "a[0].b[1].c[2].d[3].e[4]");
    }

    #[test]
    fn test_parse_large_index() {
        let path = Path::parse("items[999999]");
        assert_eq!(path.to_string(), "items[999999]");
    }

    // Equality and hashing tests
    #[test]
    fn test_parsed_equals_constructed() {
        let parsed = Path::parse("user.items[0].name");
        let constructed = Path::root()
            .field("user")
            .field("items")
            .index(0)
            .field("name");
        assert_eq!(parsed, constructed);
    }

    #[test]
    fn test_path_hash_consistency() {
        use std::collections::HashMap;

        let path1 = Path::parse("user.email");
        let path2 = Path::root().field("user").field("email");

        let mut map = HashMap::new();
        map.insert(path1.clone(), "value1");

        // Same path constructed differently should access same entry
        assert_eq!(map.get(&path2), Some(&"value1"));
    }

    #[test]
    fn test_clone_independence() {
        let original = Path::root().field("test");
        let mut cloned = original.clone();
        cloned.push_field("extra");

        assert_eq!(original.segments().len(), 1);
        assert_eq!(cloned.segments().len(), 2);
        assert_eq!(original.to_string(), "test");
        assert_eq!(cloned.to_string(), "test.extra");
    }

    #[test]
    fn test_from_string() {
        let path: Path = String::from("user.email").into();
        assert_eq!(path.to_string(), "user.email");
    }

    #[test]
    fn test_segment_types() {
        let path = Path::root().field("items").index(0).field("name");
        let segments = path.segments();

        assert!(matches!(&segments[0], PathSegment::Field(_)));
        assert!(matches!(&segments[1], PathSegment::Index(0)));
        assert!(matches!(&segments[2], PathSegment::Field(_)));
    }
}

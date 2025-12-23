#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Path(pub Vec<PathSegment>);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PathSegment {
    Field(&'static str),
    Index(usize),
}

impl Path {
    pub fn root() -> Self {
        Self(Vec::new())
    }

    pub fn field(mut self, name: &'static str) -> Self {
        self.0.push(PathSegment::Field(name));
        self
    }

    pub fn index(mut self, idx: usize) -> Self {
        self.0.push(PathSegment::Index(idx));
        self
    }

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

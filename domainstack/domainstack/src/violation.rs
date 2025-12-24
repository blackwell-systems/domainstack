use crate::Path;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Violation {
    pub path: Path,
    pub code: &'static str,
    pub message: String,
    pub meta: Meta,
}

#[derive(Debug, Clone, Default)]
pub struct Meta {
    fields: HashMap<&'static str, String>,
}

impl Meta {
    pub fn new() -> Self {
        Self {
            fields: HashMap::new(),
        }
    }

    pub fn insert(&mut self, key: &'static str, value: impl ToString) {
        self.fields.insert(key, value.to_string());
    }

    pub fn get(&self, key: &'static str) -> Option<&str> {
        self.fields.get(key).map(|v| v.as_str())
    }

    pub fn is_empty(&self) -> bool {
        self.fields.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&'static str, &str)> + '_ {
        self.fields.iter().map(|(k, v)| (*k, v.as_str()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_meta_new() {
        let meta = Meta::new();
        assert!(meta.is_empty());
    }

    #[test]
    fn test_meta_default() {
        let meta = Meta::default();
        assert!(meta.is_empty());
    }

    #[test]
    fn test_meta_insert_get() {
        let mut meta = Meta::new();
        meta.insert("min", 5);
        meta.insert("max", 10);

        assert_eq!(meta.get("min"), Some("5"));
        assert_eq!(meta.get("max"), Some("10"));
        assert_eq!(meta.get("missing"), None);
    }

    #[test]
    fn test_meta_string_values() {
        let mut meta = Meta::new();
        meta.insert("key", "value");
        assert_eq!(meta.get("key"), Some("value"));
    }

    #[test]
    fn test_violation_creation() {
        let violation = Violation {
            path: Path::from("email"),
            code: "invalid_email",
            message: "Invalid email format".to_string(),
            meta: Meta::default(),
        };

        assert_eq!(violation.code, "invalid_email");
        assert_eq!(violation.message, "Invalid email format");
        assert_eq!(violation.path.to_string(), "email");
    }
}

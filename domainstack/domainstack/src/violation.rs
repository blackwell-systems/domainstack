use crate::Path;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Violation {
    pub path: Path,
    pub code: &'static str,
    pub message: String,
    pub meta: Meta,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
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

    #[test]
    fn test_meta_equality() {
        let mut meta1 = Meta::new();
        meta1.insert("min", 5);
        meta1.insert("max", 10);

        let mut meta2 = Meta::new();
        meta2.insert("min", 5);
        meta2.insert("max", 10);

        assert_eq!(meta1, meta2);

        let meta3 = Meta::new();
        assert_ne!(meta1, meta3);
    }

    #[test]
    fn test_violation_equality() {
        let v1 = Violation {
            path: Path::from("email"),
            code: "invalid_email",
            message: "Invalid email".to_string(),
            meta: Meta::default(),
        };

        let v2 = Violation {
            path: Path::from("email"),
            code: "invalid_email",
            message: "Invalid email".to_string(),
            meta: Meta::default(),
        };

        assert_eq!(v1, v2);

        let v3 = Violation {
            path: Path::from("age"),
            code: "invalid_email",
            message: "Invalid email".to_string(),
            meta: Meta::default(),
        };

        assert_ne!(v1, v3);
    }

    // Meta::iter() tests
    #[test]
    fn test_meta_iter_empty() {
        let meta = Meta::new();
        assert_eq!(meta.iter().count(), 0);
    }

    #[test]
    fn test_meta_iter_single() {
        let mut meta = Meta::new();
        meta.insert("key", "value");

        let entries: Vec<_> = meta.iter().collect();
        assert_eq!(entries.len(), 1);
        assert!(entries.contains(&("key", "value")));
    }

    #[test]
    fn test_meta_iter_multiple() {
        let mut meta = Meta::new();
        meta.insert("min", 1);
        meta.insert("max", 100);
        meta.insert("actual", 150);

        let entries: Vec<_> = meta.iter().collect();
        assert_eq!(entries.len(), 3);
        assert!(entries.contains(&("min", "1")));
        assert!(entries.contains(&("max", "100")));
        assert!(entries.contains(&("actual", "150")));
    }

    #[test]
    fn test_meta_iter_all_values_present() {
        let mut meta = Meta::new();
        for i in 0..10 {
            // Use static keys
            match i {
                0 => meta.insert("k0", i),
                1 => meta.insert("k1", i),
                2 => meta.insert("k2", i),
                3 => meta.insert("k3", i),
                4 => meta.insert("k4", i),
                5 => meta.insert("k5", i),
                6 => meta.insert("k6", i),
                7 => meta.insert("k7", i),
                8 => meta.insert("k8", i),
                9 => meta.insert("k9", i),
                _ => {}
            }
        }

        assert_eq!(meta.iter().count(), 10);
    }

    // Meta value overwriting
    #[test]
    fn test_meta_overwrite_value() {
        let mut meta = Meta::new();
        meta.insert("key", "first");
        meta.insert("key", "second");

        assert_eq!(meta.get("key"), Some("second"));
        assert_eq!(meta.iter().count(), 1);
    }

    #[test]
    fn test_meta_overwrite_different_types() {
        let mut meta = Meta::new();
        meta.insert("value", "string");
        meta.insert("value", 42);

        assert_eq!(meta.get("value"), Some("42"));
    }

    // Special characters in values
    #[test]
    fn test_meta_special_characters() {
        let mut meta = Meta::new();
        meta.insert("newline", "line1\nline2");
        meta.insert("tab", "col1\tcol2");
        meta.insert("quote", "\"quoted\"");
        meta.insert("backslash", "path\\to\\file");

        assert_eq!(meta.get("newline"), Some("line1\nline2"));
        assert_eq!(meta.get("tab"), Some("col1\tcol2"));
        assert_eq!(meta.get("quote"), Some("\"quoted\""));
        assert_eq!(meta.get("backslash"), Some("path\\to\\file"));
    }

    #[test]
    fn test_meta_unicode_values() {
        let mut meta = Meta::new();
        meta.insert("emoji", "🔒🔑");
        meta.insert("cjk", "日本語");
        meta.insert("rtl", "مرحبا");

        assert_eq!(meta.get("emoji"), Some("🔒🔑"));
        assert_eq!(meta.get("cjk"), Some("日本語"));
        assert_eq!(meta.get("rtl"), Some("مرحبا"));
    }

    #[test]
    fn test_meta_empty_value() {
        let mut meta = Meta::new();
        meta.insert("empty", "");

        assert_eq!(meta.get("empty"), Some(""));
        assert!(!meta.is_empty());
    }

    // Violation with complex meta
    #[test]
    fn test_violation_with_rich_meta() {
        let mut meta = Meta::new();
        meta.insert("min", 18);
        meta.insert("max", 120);
        meta.insert("actual", 15);
        meta.insert("message", "Age must be between 18 and 120");

        let violation = Violation {
            path: Path::root().field("user").field("age"),
            code: "out_of_range",
            message: "Value out of allowed range".to_string(),
            meta,
        };

        assert_eq!(violation.meta.get("min"), Some("18"));
        assert_eq!(violation.meta.get("max"), Some("120"));
        assert_eq!(violation.meta.get("actual"), Some("15"));
        assert_eq!(violation.path.to_string(), "user.age");
    }

    // Clone behavior
    #[test]
    fn test_meta_clone_independence() {
        let mut meta1 = Meta::new();
        meta1.insert("key", "original");

        let mut meta2 = meta1.clone();
        meta2.insert("key", "modified");

        assert_eq!(meta1.get("key"), Some("original"));
        assert_eq!(meta2.get("key"), Some("modified"));
    }

    #[test]
    fn test_violation_clone_independence() {
        let mut v1 = Violation {
            path: Path::from("field"),
            code: "error",
            message: "Original".to_string(),
            meta: Meta::default(),
        };
        v1.meta.insert("key", "original");

        let mut v2 = v1.clone();
        v2.meta.insert("key", "modified");
        v2.message = "Modified".to_string();

        assert_eq!(v1.message, "Original");
        assert_eq!(v1.meta.get("key"), Some("original"));
        assert_eq!(v2.message, "Modified");
        assert_eq!(v2.meta.get("key"), Some("modified"));
    }

    // Debug trait
    #[test]
    fn test_violation_debug_format() {
        let violation = Violation {
            path: Path::from("email"),
            code: "invalid",
            message: "Invalid".to_string(),
            meta: Meta::default(),
        };

        let debug_str = format!("{:?}", violation);
        assert!(debug_str.contains("Violation"));
        assert!(debug_str.contains("email"));
        assert!(debug_str.contains("invalid"));
    }

    #[test]
    fn test_meta_debug_format() {
        let mut meta = Meta::new();
        meta.insert("key", "value");

        let debug_str = format!("{:?}", meta);
        assert!(debug_str.contains("Meta"));
        assert!(debug_str.contains("key"));
    }

    // Empty message
    #[test]
    fn test_violation_empty_message() {
        let v = Violation {
            path: Path::from("field"),
            code: "error",
            message: String::new(),
            meta: Meta::default(),
        };

        assert!(v.message.is_empty());
        assert_eq!(v.code, "error");
    }

    // Large number of meta entries
    #[test]
    fn test_meta_many_entries() {
        let mut meta = Meta::new();
        // Add several entries with static keys
        meta.insert("a", "1");
        meta.insert("b", "2");
        meta.insert("c", "3");
        meta.insert("d", "4");
        meta.insert("e", "5");
        meta.insert("f", "6");
        meta.insert("g", "7");
        meta.insert("h", "8");
        meta.insert("i", "9");
        meta.insert("j", "10");

        assert_eq!(meta.iter().count(), 10);
        assert!(!meta.is_empty());
    }
}

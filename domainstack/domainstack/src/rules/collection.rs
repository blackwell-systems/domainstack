use crate::{Rule, RuleContext, ValidationError};
use std::collections::HashSet;
use std::hash::Hash;

/// Validates that a collection has at least the minimum number of items.
///
/// Works with any slice type `[T]`, including `Vec<T>` (which derefs to `[T]`).
///
/// # Examples
///
/// ```
/// use domainstack::prelude::*;
///
/// let rule = rules::min_items(2);
/// assert!(rule.apply(&[1, 2, 3]).is_empty());
/// assert!(rule.apply(&[1, 2]).is_empty());   // exactly min
/// assert!(!rule.apply(&[1]).is_empty());     // too few
/// assert!(!rule.apply(&Vec::<i32>::new()).is_empty()); // empty
/// ```
///
/// # Error Code
/// - Code: `too_few_items`
/// - Message: `"Must have at least {min} items"`
/// - Meta: `{"min": "2", "actual": "1"}`
pub fn min_items<T: 'static>(min: usize) -> Rule<[T]> {
    Rule::new(move |value: &[T], ctx: &RuleContext| {
        let count = value.len();
        if count < min {
            let mut err = ValidationError::single(
                ctx.full_path(),
                "too_few_items",
                format!("Must have at least {} items", min),
            );
            err.violations[0].meta.insert("min", min.to_string());
            err.violations[0].meta.insert("actual", count.to_string());
            err
        } else {
            ValidationError::default()
        }
    })
}

/// Validates that a collection has at most the maximum number of items.
///
/// Works with any slice type `[T]`, including `Vec<T>` (which derefs to `[T]`).
///
/// # Examples
///
/// ```
/// use domainstack::prelude::*;
///
/// let rule = rules::max_items(3);
/// assert!(rule.apply(&[1, 2]).is_empty());
/// assert!(rule.apply(&[1, 2, 3]).is_empty()); // exactly max
/// assert!(!rule.apply(&[1, 2, 3, 4]).is_empty()); // too many
/// ```
///
/// # Error Code
/// - Code: `too_many_items`
/// - Message: `"Must have at most {max} items"`
/// - Meta: `{"max": "3", "actual": "4"}`
pub fn max_items<T: 'static>(max: usize) -> Rule<[T]> {
    Rule::new(move |value: &[T], ctx: &RuleContext| {
        let count = value.len();
        if count > max {
            let mut err = ValidationError::single(
                ctx.full_path(),
                "too_many_items",
                format!("Must have at most {} items", max),
            );
            err.violations[0].meta.insert("max", max.to_string());
            err.violations[0].meta.insert("actual", count.to_string());
            err
        } else {
            ValidationError::default()
        }
    })
}

/// Validates that all items in a collection are unique (no duplicates).
///
/// Works with any slice type `[T]` where T implements `Eq` and `Hash`.
///
/// # Examples
///
/// ```
/// use domainstack::prelude::*;
///
/// let rule: Rule<[i32]> = rules::unique();
/// assert!(rule.apply(&[1, 2, 3]).is_empty());
/// assert!(!rule.apply(&[1, 2, 2, 3]).is_empty()); // duplicate 2
///
/// let rule_str: Rule<[&str]> = rules::unique();
/// assert!(rule_str.apply(&["a", "b", "c"]).is_empty());
/// assert!(!rule_str.apply(&["a", "b", "a"]).is_empty()); // duplicate "a"
/// ```
///
/// # Error Code
/// - Code: `duplicate_items`
/// - Message: `"All items must be unique (found {count} duplicates)"`
/// - Meta: `{"duplicates": "2"}`
pub fn unique<T>() -> Rule<[T]>
where
    T: Eq + Hash + 'static,
{
    Rule::new(|value: &[T], ctx: &RuleContext| {
        let mut seen = HashSet::new();
        let mut duplicate_count = 0;

        for item in value {
            if !seen.insert(item) {
                duplicate_count += 1;
            }
        }

        if duplicate_count > 0 {
            let mut err = ValidationError::single(
                ctx.full_path(),
                "duplicate_items",
                format!(
                    "All items must be unique (found {} duplicates)",
                    duplicate_count
                ),
            );
            err.violations[0]
                .meta
                .insert("duplicates", duplicate_count.to_string());
            err
        } else {
            ValidationError::default()
        }
    })
}

/// Validates that all string items in a collection are non-empty.
///
/// This is useful for ensuring no empty strings exist in a `Vec<String>`.
/// Commonly used for tags, keywords, or any list where empty strings are not allowed.
///
/// # Examples
///
/// ```
/// use domainstack::prelude::*;
///
/// let rule = rules::non_empty_items();
///
/// // Valid - all items are non-empty
/// let tags = vec!["tag1".to_string(), "tag2".to_string(), "tag3".to_string()];
/// assert!(rule.apply(&tags).is_empty());
///
/// let tags = vec!["rust".to_string(), "validation".to_string()];
/// assert!(rule.apply(&tags).is_empty());
///
/// // Invalid - contains empty string
/// let invalid_tags = vec!["tag1".to_string(), "".to_string(), "tag3".to_string()];
/// assert!(!rule.apply(&invalid_tags).is_empty()); // empty string at index 1
///
/// let invalid_tags = vec!["".to_string(), "tag2".to_string()];
/// assert!(!rule.apply(&invalid_tags).is_empty()); // empty string at index 0
/// ```
///
/// # Error Code
/// - Code: `empty_item`
/// - Message: `"All items must be non-empty (found {count} empty items)"`
/// - Meta: `{"empty_count": "2", "indices": "[0, 2]"}`
pub fn non_empty_items() -> Rule<[String]> {
    Rule::new(|value: &[String], ctx: &RuleContext| {
        let mut empty_indices = Vec::new();

        for (i, item) in value.iter().enumerate() {
            if item.is_empty() {
                empty_indices.push(i);
            }
        }

        if !empty_indices.is_empty() {
            let count = empty_indices.len();
            let mut err = ValidationError::single(
                ctx.full_path(),
                "empty_item",
                format!("All items must be non-empty (found {} empty items)", count),
            );
            err.violations[0]
                .meta
                .insert("empty_count", count.to_string());
            err.violations[0].meta.insert(
                "indices",
                format!("[{}]", empty_indices.iter().map(|i| i.to_string()).collect::<Vec<_>>().join(", ")),
            );
            err
        } else {
            ValidationError::default()
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_min_items_valid() {
        let rule: Rule<[i32]> = min_items(2);
        assert!(rule.apply(&[1, 2, 3]).is_empty());
        assert!(rule.apply(&[1, 2]).is_empty()); // exactly min

        let rule_str: Rule<[&str]> = min_items(2);
        assert!(rule_str.apply(&["a", "b", "c"]).is_empty());
    }

    #[test]
    fn test_min_items_invalid() {
        let rule = min_items(2);

        let result = rule.apply(&[1]);
        assert!(!result.is_empty());
        assert_eq!(result.violations[0].code, "too_few_items");
        assert_eq!(result.violations[0].meta.get("min"), Some("2"));
        assert_eq!(result.violations[0].meta.get("actual"), Some("1"));

        let result = rule.apply(&Vec::<i32>::new());
        assert!(!result.is_empty());
        assert_eq!(result.violations[0].meta.get("actual"), Some("0"));
    }

    #[test]
    fn test_max_items_valid() {
        let rule = max_items(3);
        assert!(rule.apply(&[1, 2]).is_empty());
        assert!(rule.apply(&[1, 2, 3]).is_empty()); // exactly max
        assert!(rule.apply(&Vec::<i32>::new()).is_empty()); // empty is ok
    }

    #[test]
    fn test_max_items_invalid() {
        let rule = max_items(3);

        let result = rule.apply(&[1, 2, 3, 4]);
        assert!(!result.is_empty());
        assert_eq!(result.violations[0].code, "too_many_items");
        assert_eq!(result.violations[0].meta.get("max"), Some("3"));
        assert_eq!(result.violations[0].meta.get("actual"), Some("4"));

        let result = rule.apply(&[1, 2, 3, 4, 5]);
        assert!(!result.is_empty());
        assert_eq!(result.violations[0].meta.get("actual"), Some("5"));
    }

    #[test]
    fn test_min_max_composition() {
        // Combine min and max for range validation
        let rule = min_items(2).and(max_items(4));

        assert!(rule.apply(&[1, 2]).is_empty()); // min boundary
        assert!(rule.apply(&[1, 2, 3]).is_empty()); // middle
        assert!(rule.apply(&[1, 2, 3, 4]).is_empty()); // max boundary

        let result = rule.apply(&[1]);
        assert!(!result.is_empty());
        assert_eq!(result.violations[0].code, "too_few_items");

        let result = rule.apply(&[1, 2, 3, 4, 5]);
        assert!(!result.is_empty());
        assert_eq!(result.violations[0].code, "too_many_items");
    }

    #[test]
    fn test_unique_valid() {
        let rule: Rule<[i32]> = unique();
        assert!(rule.apply(&[1, 2, 3, 4]).is_empty());
        assert!(rule.apply(&Vec::<i32>::new()).is_empty()); // empty is unique
        assert!(rule.apply(&[42]).is_empty()); // single item is unique

        let rule_str: Rule<[&str]> = unique();
        assert!(rule_str.apply(&["a", "b", "c"]).is_empty());
    }

    #[test]
    fn test_unique_invalid_numbers() {
        let rule = unique();

        let result = rule.apply(&[1, 2, 2, 3]);
        assert!(!result.is_empty());
        assert_eq!(result.violations[0].code, "duplicate_items");
        assert_eq!(result.violations[0].meta.get("duplicates"), Some("1"));

        let result = rule.apply(&[1, 1, 2, 2, 3, 3]);
        assert!(!result.is_empty());
        assert_eq!(result.violations[0].meta.get("duplicates"), Some("3"));
    }

    #[test]
    fn test_unique_invalid_strings() {
        let rule = unique();

        let result = rule.apply(&["a", "b", "a", "c"]);
        assert!(!result.is_empty());
        assert_eq!(result.violations[0].code, "duplicate_items");
    }

    #[test]
    fn test_unique_all_duplicates() {
        let rule = unique();

        let result = rule.apply(&[1, 1, 1, 1]);
        assert!(!result.is_empty());
        assert_eq!(result.violations[0].meta.get("duplicates"), Some("3"));
    }

    #[test]
    fn test_collection_with_min_and_unique() {
        // Realistic example: tags must have at least 1 item and be unique
        let rule = min_items(1).and(unique());

        assert!(rule.apply(&["rust", "validation"]).is_empty());

        let result = rule.apply(&Vec::<&str>::new());
        assert!(!result.is_empty());
        assert_eq!(result.violations[0].code, "too_few_items");

        let result = rule.apply(&["rust", "rust"]);
        assert!(!result.is_empty());
        assert_eq!(result.violations[0].code, "duplicate_items");
    }

    #[test]
    fn test_non_empty_items_valid() {
        let rule = non_empty_items();

        // Vec<String> works
        let tags = vec!["tag1".to_string(), "tag2".to_string(), "tag3".to_string()];
        assert!(rule.apply(&tags).is_empty());

        let tags = vec!["a".to_string()];
        assert!(rule.apply(&tags).is_empty());

        let tags = vec!["rust".to_string(), "validation".to_string()];
        assert!(rule.apply(&tags).is_empty());

        // Empty collection is valid (no empty items)
        assert!(rule.apply(&Vec::<String>::new()).is_empty());
    }

    #[test]
    fn test_non_empty_items_invalid() {
        let rule = non_empty_items();

        // Single empty string
        let tags = vec!["tag1".to_string(), "".to_string(), "tag3".to_string()];
        let result = rule.apply(&tags);
        assert!(!result.is_empty());
        assert_eq!(result.violations[0].code, "empty_item");
        assert_eq!(result.violations[0].meta.get("empty_count"), Some("1"));
        assert!(result.violations[0]
            .meta
            .get("indices")
            .unwrap()
            .contains("1"));

        // Multiple empty strings
        let tags = vec!["".to_string(), "tag2".to_string(), "".to_string(), "tag4".to_string()];
        let result = rule.apply(&tags);
        assert!(!result.is_empty());
        assert_eq!(result.violations[0].meta.get("empty_count"), Some("2"));

        // All empty
        let tags = vec!["".to_string(), "".to_string(), "".to_string()];
        let result = rule.apply(&tags);
        assert!(!result.is_empty());
        assert_eq!(result.violations[0].meta.get("empty_count"), Some("3"));

        // Vec<String> with empty
        let invalid_tags = vec!["rust".to_string(), "".to_string()];
        let result = rule.apply(&invalid_tags);
        assert!(!result.is_empty());
        assert_eq!(result.violations[0].code, "empty_item");
    }

    #[test]
    fn test_non_empty_items_composition() {
        // Realistic: tags must have items, be unique, and non-empty
        let rule = min_items(1).and(unique()).and(non_empty_items());

        let tags = vec!["rust".to_string(), "validation".to_string()];
        assert!(rule.apply(&tags).is_empty());

        // Fails min_items
        let result = rule.apply(&Vec::<String>::new());
        assert!(!result.is_empty());
        assert_eq!(result.violations[0].code, "too_few_items");

        // Fails unique
        let tags = vec!["rust".to_string(), "rust".to_string()];
        let result = rule.apply(&tags);
        assert!(!result.is_empty());
        assert_eq!(result.violations[0].code, "duplicate_items");

        // Fails non_empty_items
        let tags = vec!["rust".to_string(), "".to_string()];
        let result = rule.apply(&tags);
        assert!(!result.is_empty());
        assert_eq!(result.violations[0].code, "empty_item");
    }
}

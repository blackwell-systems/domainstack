use crate::{Rule, RuleContext, ValidationError};
use chrono::{DateTime, Datelike, NaiveDate, Utc};

/// Validates that a datetime is in the past (before now).
///
/// Useful for validating birth dates, historical events, or any datetime
/// that must have already occurred.
///
/// # Examples
///
/// ```
/// use domainstack::prelude::*;
/// use chrono::{DateTime, Utc, Duration};
///
/// let rule = rules::past();
///
/// // Yesterday is valid
/// let yesterday = Utc::now() - Duration::days(1);
/// assert!(rule.apply(&yesterday).is_empty());
///
/// // Tomorrow is invalid
/// let tomorrow = Utc::now() + Duration::days(1);
/// assert!(!rule.apply(&tomorrow).is_empty());
/// ```
///
/// # Error Code
/// - Code: `not_in_past`
/// - Message: `"Must be in the past"`
pub fn past() -> Rule<DateTime<Utc>> {
    Rule::new(|value: &DateTime<Utc>, ctx: &RuleContext| {
        let now = Utc::now();
        if *value < now {
            ValidationError::default()
        } else {
            ValidationError::single(ctx.full_path(), "not_in_past", "Must be in the past")
        }
    })
}

/// Validates that a datetime is in the future (after now).
///
/// Useful for validating event dates, deadlines, or any datetime
/// that must not have occurred yet.
///
/// # Examples
///
/// ```
/// use domainstack::prelude::*;
/// use chrono::{DateTime, Utc, Duration};
///
/// let rule = rules::future();
///
/// // Tomorrow is valid
/// let tomorrow = Utc::now() + Duration::days(1);
/// assert!(rule.apply(&tomorrow).is_empty());
///
/// // Yesterday is invalid
/// let yesterday = Utc::now() - Duration::days(1);
/// assert!(!rule.apply(&yesterday).is_empty());
/// ```
///
/// # Error Code
/// - Code: `not_in_future`
/// - Message: `"Must be in the future"`
pub fn future() -> Rule<DateTime<Utc>> {
    Rule::new(|value: &DateTime<Utc>, ctx: &RuleContext| {
        let now = Utc::now();
        if *value > now {
            ValidationError::default()
        } else {
            ValidationError::single(ctx.full_path(), "not_in_future", "Must be in the future")
        }
    })
}

/// Validates that a datetime is before the specified datetime.
///
/// Useful for validating ranges, ensuring an event occurs before another,
/// or checking temporal constraints.
///
/// # Examples
///
/// ```
/// use domainstack::prelude::*;
/// use chrono::{DateTime, Utc, NaiveDate};
///
/// let deadline = NaiveDate::from_ymd_opt(2025, 12, 31)
///     .unwrap()
///     .and_hms_opt(23, 59, 59)
///     .unwrap()
///     .and_utc();
///
/// let rule = rules::before(deadline);
///
/// let valid = NaiveDate::from_ymd_opt(2025, 6, 15)
///     .unwrap()
///     .and_hms_opt(12, 0, 0)
///     .unwrap()
///     .and_utc();
/// assert!(rule.apply(&valid).is_empty());
///
/// let invalid = NaiveDate::from_ymd_opt(2026, 1, 1)
///     .unwrap()
///     .and_hms_opt(0, 0, 0)
///     .unwrap()
///     .and_utc();
/// assert!(!rule.apply(&invalid).is_empty());
/// ```
///
/// # Error Code
/// - Code: `not_before`
/// - Message: `"Must be before {limit}"`
/// - Meta: `{"limit": "2025-12-31T23:59:59Z"}`
pub fn before(limit: DateTime<Utc>) -> Rule<DateTime<Utc>> {
    Rule::new(move |value: &DateTime<Utc>, ctx: &RuleContext| {
        if *value < limit {
            ValidationError::default()
        } else {
            let mut err = ValidationError::single(
                ctx.full_path(),
                "not_before",
                format!("Must be before {}", limit.to_rfc3339()),
            );
            err.violations[0].meta.insert("limit", limit.to_rfc3339());
            err
        }
    })
}

/// Validates that a datetime is after the specified datetime.
///
/// Useful for validating ranges, ensuring an event occurs after another,
/// or checking temporal constraints.
///
/// # Examples
///
/// ```
/// use domainstack::prelude::*;
/// use chrono::{DateTime, Utc, NaiveDate};
///
/// let start_date = NaiveDate::from_ymd_opt(2025, 1, 1)
///     .unwrap()
///     .and_hms_opt(0, 0, 0)
///     .unwrap()
///     .and_utc();
///
/// let rule = rules::after(start_date);
///
/// let valid = NaiveDate::from_ymd_opt(2025, 6, 15)
///     .unwrap()
///     .and_hms_opt(12, 0, 0)
///     .unwrap()
///     .and_utc();
/// assert!(rule.apply(&valid).is_empty());
///
/// let invalid = NaiveDate::from_ymd_opt(2024, 12, 31)
///     .unwrap()
///     .and_hms_opt(23, 59, 59)
///     .unwrap()
///     .and_utc();
/// assert!(!rule.apply(&invalid).is_empty());
/// ```
///
/// # Error Code
/// - Code: `not_after`
/// - Message: `"Must be after {limit}"`
/// - Meta: `{"limit": "2025-01-01T00:00:00Z"}`
pub fn after(limit: DateTime<Utc>) -> Rule<DateTime<Utc>> {
    Rule::new(move |value: &DateTime<Utc>, ctx: &RuleContext| {
        if *value > limit {
            ValidationError::default()
        } else {
            let mut err = ValidationError::single(
                ctx.full_path(),
                "not_after",
                format!("Must be after {}", limit.to_rfc3339()),
            );
            err.violations[0].meta.insert("limit", limit.to_rfc3339());
            err
        }
    })
}

/// Validates that a birth date corresponds to an age within the specified range.
///
/// Calculates age based on the current date and validates it falls within min-max years.
/// Useful for age verification, eligibility checks, etc.
///
/// # Examples
///
/// ```
/// use domainstack::prelude::*;
/// use chrono::{NaiveDate, Utc, Datelike};
///
/// let rule = rules::age_range(18, 120);
///
/// // Someone born 25 years ago is valid (18-120)
/// let today = Utc::now().date_naive();
/// let birth_date = NaiveDate::from_ymd_opt(
///     today.year() - 25,
///     today.month(),
///     today.day()
/// ).unwrap();
/// assert!(rule.apply(&birth_date).is_empty());
///
/// // Someone born 10 years ago is invalid (under 18)
/// let too_young = NaiveDate::from_ymd_opt(
///     today.year() - 10,
///     today.month(),
///     today.day()
/// ).unwrap();
/// assert!(!rule.apply(&too_young).is_empty());
/// ```
///
/// # Error Code
/// - Code: `age_out_of_range`
/// - Message: `"Age must be between {min} and {max} years"`
/// - Meta: `{"min": "18", "max": "120", "age": "10"}`
pub fn age_range(min: u32, max: u32) -> Rule<NaiveDate> {
    Rule::new(move |birth_date: &NaiveDate, ctx: &RuleContext| {
        let today = Utc::now().date_naive();
        let age = calculate_age(*birth_date, today);

        if age >= min && age <= max {
            ValidationError::default()
        } else {
            let mut err = ValidationError::single(
                ctx.full_path(),
                "age_out_of_range",
                format!("Age must be between {} and {} years", min, max),
            );
            err.violations[0].meta.insert("min", min.to_string());
            err.violations[0].meta.insert("max", max.to_string());
            err.violations[0].meta.insert("age", age.to_string());
            err
        }
    })
}

/// Helper function to calculate age from birth date to a given date
fn calculate_age(birth_date: NaiveDate, current_date: NaiveDate) -> u32 {
    let mut age = current_date.year() - birth_date.year();

    // Adjust if birthday hasn't occurred yet this year
    if current_date.month() < birth_date.month()
        || (current_date.month() == birth_date.month() && current_date.day() < birth_date.day())
    {
        age -= 1;
    }

    age as u32
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn test_past_valid() {
        let rule = past();

        let yesterday = Utc::now() - Duration::days(1);
        assert!(rule.apply(&yesterday).is_empty());

        let last_year = Utc::now() - Duration::days(365);
        assert!(rule.apply(&last_year).is_empty());
    }

    #[test]
    fn test_past_invalid() {
        let rule = past();

        let tomorrow = Utc::now() + Duration::days(1);
        let result = rule.apply(&tomorrow);
        assert!(!result.is_empty());
        assert_eq!(result.violations[0].code, "not_in_past");
    }

    #[test]
    fn test_future_valid() {
        let rule = future();

        let tomorrow = Utc::now() + Duration::days(1);
        assert!(rule.apply(&tomorrow).is_empty());

        let next_year = Utc::now() + Duration::days(365);
        assert!(rule.apply(&next_year).is_empty());
    }

    #[test]
    fn test_future_invalid() {
        let rule = future();

        let yesterday = Utc::now() - Duration::days(1);
        let result = rule.apply(&yesterday);
        assert!(!result.is_empty());
        assert_eq!(result.violations[0].code, "not_in_future");
    }

    #[test]
    fn test_before_valid() {
        let limit = Utc::now() + Duration::days(30);
        let rule = before(limit);

        let today = Utc::now();
        assert!(rule.apply(&today).is_empty());

        let tomorrow = Utc::now() + Duration::days(1);
        assert!(rule.apply(&tomorrow).is_empty());
    }

    #[test]
    fn test_before_invalid() {
        let limit = Utc::now();
        let rule = before(limit);

        let tomorrow = Utc::now() + Duration::days(1);
        let result = rule.apply(&tomorrow);
        assert!(!result.is_empty());
        assert_eq!(result.violations[0].code, "not_before");
        assert!(result.violations[0].meta.get("limit").is_some());
    }

    #[test]
    fn test_after_valid() {
        let limit = Utc::now() - Duration::days(30);
        let rule = after(limit);

        let today = Utc::now();
        assert!(rule.apply(&today).is_empty());

        let tomorrow = Utc::now() + Duration::days(1);
        assert!(rule.apply(&tomorrow).is_empty());
    }

    #[test]
    fn test_after_invalid() {
        let limit = Utc::now();
        let rule = after(limit);

        let yesterday = Utc::now() - Duration::days(1);
        let result = rule.apply(&yesterday);
        assert!(!result.is_empty());
        assert_eq!(result.violations[0].code, "not_after");
        assert!(result.violations[0].meta.get("limit").is_some());
    }

    #[test]
    fn test_age_range_valid() {
        let rule = age_range(18, 120);

        // Person born 25 years ago
        let birth_date = NaiveDate::from_ymd_opt(Utc::now().year() - 25, 6, 15).unwrap();
        assert!(rule.apply(&birth_date).is_empty());

        // Person born exactly 18 years ago
        let birth_date = NaiveDate::from_ymd_opt(Utc::now().year() - 18, 1, 1).unwrap();
        assert!(rule.apply(&birth_date).is_empty());
    }

    #[test]
    fn test_age_range_too_young() {
        let rule = age_range(18, 120);

        // Person born 10 years ago
        let birth_date = NaiveDate::from_ymd_opt(Utc::now().year() - 10, 6, 15).unwrap();
        let result = rule.apply(&birth_date);
        assert!(!result.is_empty());
        assert_eq!(result.violations[0].code, "age_out_of_range");
        assert_eq!(result.violations[0].meta.get("min"), Some("18"));
        assert_eq!(result.violations[0].meta.get("age"), Some("10"));
    }

    #[test]
    fn test_age_range_too_old() {
        let rule = age_range(18, 120);

        // Person born 130 years ago
        let birth_date = NaiveDate::from_ymd_opt(Utc::now().year() - 130, 6, 15).unwrap();
        let result = rule.apply(&birth_date);
        assert!(!result.is_empty());
        assert_eq!(result.violations[0].code, "age_out_of_range");
        assert_eq!(result.violations[0].meta.get("max"), Some("120"));
        assert_eq!(result.violations[0].meta.get("age"), Some("130"));
    }

    #[test]
    fn test_calculate_age() {
        let birth_date = NaiveDate::from_ymd_opt(2000, 6, 15).unwrap();

        // Before birthday
        let current = NaiveDate::from_ymd_opt(2025, 3, 10).unwrap();
        assert_eq!(calculate_age(birth_date, current), 24);

        // After birthday
        let current = NaiveDate::from_ymd_opt(2025, 8, 20).unwrap();
        assert_eq!(calculate_age(birth_date, current), 25);

        // On birthday
        let current = NaiveDate::from_ymd_opt(2025, 6, 15).unwrap();
        assert_eq!(calculate_age(birth_date, current), 25);
    }

    #[test]
    fn test_before_after_composition() {
        // Event must be within a specific window
        let start = Utc::now();
        let end = Utc::now() + Duration::days(30);

        let rule = after(start).and(before(end));

        // Within window
        let event = Utc::now() + Duration::days(15);
        assert!(rule.apply(&event).is_empty());

        // Before window
        let event = Utc::now() - Duration::days(1);
        let result = rule.apply(&event);
        assert!(!result.is_empty());

        // After window
        let event = Utc::now() + Duration::days(31);
        let result = rule.apply(&event);
        assert!(!result.is_empty());
    }
}

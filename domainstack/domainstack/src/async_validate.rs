//! Async validation support for database checks and external API validation.
//!
//! This module provides async validation capabilities, allowing validation rules
//! to perform asynchronous operations like database queries or external API calls.
//!
//! # Example
//!
//! ```rust,ignore
//! use domainstack::{AsyncValidate, ValidationContext, ValidationError};
//! use async_trait::async_trait;
//!
//! struct User {
//!     email: String,
//! }
//!
//! #[async_trait]
//! impl AsyncValidate for User {
//!     async fn validate_async(&self, ctx: &ValidationContext) -> Result<(), ValidationError> {
//!         // Perform async validation (e.g., check email uniqueness in database)
//!         Ok(())
//!     }
//! }
//! ```

use crate::error::ValidationError;
use crate::RuleContext;
use async_trait::async_trait;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

/// Context passed to async validation functions.
///
/// Contains references to external resources needed for async validation,
/// such as database connections, cache clients, or HTTP clients.
///
/// # Example
///
/// ```rust,ignore
/// use domainstack::ValidationContext;
/// use std::sync::Arc;
///
/// // Define your own database trait
/// trait Database: Send + Sync {
///     async fn email_exists(&self, email: &str) -> bool;
/// }
///
/// // Create context with database
/// let ctx = ValidationContext::new()
///     .with_resource("db", Arc::new(my_database));
/// ```
#[derive(Clone, Default)]
pub struct ValidationContext {
    /// Shared resources accessible during validation
    resources: std::collections::HashMap<&'static str, Arc<dyn std::any::Any + Send + Sync>>,
}

impl ValidationContext {
    /// Creates a new empty validation context.
    ///
    /// # Example
    ///
    /// ```
    /// use domainstack::ValidationContext;
    ///
    /// let ctx = ValidationContext::new();
    /// ```
    pub fn new() -> Self {
        Self {
            resources: std::collections::HashMap::new(),
        }
    }

    /// Adds a resource to the validation context.
    ///
    /// Resources are stored as `Arc<dyn Any + Send + Sync>` and can be retrieved
    /// using [`get_resource`](Self::get_resource).
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use domainstack::ValidationContext;
    /// use std::sync::Arc;
    ///
    /// let ctx = ValidationContext::new()
    ///     .with_resource("db", Arc::new(my_database));
    /// ```
    pub fn with_resource<T: Send + Sync + 'static>(
        mut self,
        key: &'static str,
        resource: Arc<T>,
    ) -> Self {
        self.resources.insert(key, resource);
        self
    }

    /// Retrieves a resource from the validation context.
    ///
    /// Returns `None` if the resource doesn't exist or if the type doesn't match.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use domainstack::ValidationContext;
    ///
    /// let db = ctx.get_resource::<MyDatabase>("db")
    ///     .expect("Database not found in context");
    /// ```
    pub fn get_resource<T: Send + Sync + 'static>(&self, key: &'static str) -> Option<Arc<T>> {
        self.resources
            .get(key)
            .and_then(|any| any.clone().downcast::<T>().ok())
    }
}

/// Trait for types that support asynchronous validation.
///
/// Implement this trait to perform validation that requires async operations,
/// such as database queries, external API calls, or other I/O operations.
///
/// # Example
///
/// ```rust,ignore
/// use domainstack::{AsyncValidate, ValidationContext, ValidationError, Path};
/// use async_trait::async_trait;
///
/// struct User {
///     email: String,
/// }
///
/// #[async_trait]
/// impl AsyncValidate for User {
///     async fn validate_async(&self, ctx: &ValidationContext) -> Result<(), ValidationError> {
///         // Get database from context
///         let db = ctx.get_resource::<Database>("db")
///             .expect("Database not in context");
///
///         // Check if email already exists
///         if db.email_exists(&self.email).await {
///             return Err(ValidationError::single(
///                 Path::from("email"),
///                 "email_taken",
///                 "Email address is already registered"
///             ));
///         }
///
///         Ok(())
///     }
/// }
///
/// // Usage
/// async fn register_user(user: User, ctx: ValidationContext) -> Result<(), ValidationError> {
///     user.validate_async(&ctx).await?;
///     // ... proceed with registration
///     Ok(())
/// }
/// ```
#[async_trait]
pub trait AsyncValidate {
    /// Performs asynchronous validation.
    ///
    /// # Parameters
    ///
    /// * `ctx` - Validation context containing external resources
    ///
    /// # Returns
    ///
    /// * `Ok(())` if validation passes
    /// * `Err(ValidationError)` if validation fails
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// #[async_trait]
    /// impl AsyncValidate for User {
    ///     async fn validate_async(&self, ctx: &ValidationContext) -> Result<(), ValidationError> {
    ///         // Async validation logic here
    ///         Ok(())
    ///     }
    /// }
    /// ```
    async fn validate_async(&self, ctx: &ValidationContext) -> Result<(), ValidationError>;
}

/// Type alias for async validation functions.
///
/// An async rule is a function that takes a value, a validation context, and
/// an async context, then returns a future that resolves to a ValidationError.
pub type AsyncRuleFn<T> = Arc<
    dyn Fn(
            &T,
            &RuleContext,
            &ValidationContext,
        ) -> Pin<Box<dyn Future<Output = ValidationError> + Send>>
        + Send
        + Sync,
>;

/// An async validation rule that can perform I/O operations.
///
/// Similar to [`Rule`](crate::Rule), but supports async validation functions
/// for database queries, external API calls, and other async operations.
///
/// # Example
///
/// ```rust,ignore
/// use domainstack::{AsyncRule, ValidationContext, ValidationError, Path};
/// use std::sync::Arc;
///
/// // Create an async rule that checks email uniqueness
/// fn email_unique() -> AsyncRule<str> {
///     AsyncRule::new(|email: &str, ctx: &RuleContext, vctx: &ValidationContext| {
///         Box::pin(async move {
///             let db = vctx.get_resource::<Database>("db")
///                 .expect("Database not in context");
///
///             if db.email_exists(email).await {
///                 ValidationError::single(
///                     ctx.full_path(),
///                     "email_taken",
///                     "Email is already registered"
///                 )
///             } else {
///                 ValidationError::default()
///             }
///         })
///     })
/// }
/// ```
pub struct AsyncRule<T: ?Sized> {
    func: AsyncRuleFn<T>,
}

impl<T: ?Sized> AsyncRule<T> {
    /// Creates a new async validation rule from a function.
    ///
    /// # Parameters
    ///
    /// * `func` - An async function that takes a value reference, rule context,
    ///   validation context, and returns a future resolving to ValidationError
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use domainstack::{AsyncRule, ValidationContext, ValidationError};
    ///
    /// let rule = AsyncRule::new(|value: &str, ctx, vctx| {
    ///     Box::pin(async move {
    ///         // Async validation logic
    ///         ValidationError::default()
    ///     })
    /// });
    /// ```
    pub fn new<F, Fut>(func: F) -> Self
    where
        F: Fn(&T, &RuleContext, &ValidationContext) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ValidationError> + Send + 'static,
    {
        Self {
            func: Arc::new(move |value, ctx, vctx| Box::pin(func(value, ctx, vctx))),
        }
    }

    /// Applies the async rule to a value.
    ///
    /// # Parameters
    ///
    /// * `value` - The value to validate
    /// * `ctx` - Rule context with field information
    /// * `vctx` - Validation context with external resources
    ///
    /// # Returns
    ///
    /// A future that resolves to a ValidationError (empty if valid)
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let rule = email_unique();
    /// let ctx = RuleContext::root("email");
    /// let vctx = ValidationContext::new();
    /// let result = rule.apply("test@example.com", &ctx, &vctx).await;
    /// ```
    pub async fn apply(
        &self,
        value: &T,
        ctx: &RuleContext,
        vctx: &ValidationContext,
    ) -> ValidationError {
        (self.func)(value, ctx, vctx).await
    }
}

impl<T: ?Sized> Clone for AsyncRule<T> {
    fn clone(&self) -> Self {
        Self {
            func: Arc::clone(&self.func),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_new() {
        let ctx = ValidationContext::new();
        assert!(ctx.resources.is_empty());
    }

    #[test]
    fn test_context_with_resource() {
        let value = Arc::new(42i32);
        let ctx = ValidationContext::new().with_resource("test", value.clone());

        let retrieved = ctx.get_resource::<i32>("test");
        assert!(retrieved.is_some());
        assert_eq!(*retrieved.unwrap(), 42);
    }

    #[test]
    fn test_context_get_resource_wrong_type() {
        let value = Arc::new(42i32);
        let ctx = ValidationContext::new().with_resource("test", value);

        let retrieved = ctx.get_resource::<String>("test");
        assert!(retrieved.is_none());
    }

    #[test]
    fn test_context_get_resource_missing_key() {
        let ctx = ValidationContext::new();
        let retrieved = ctx.get_resource::<i32>("missing");
        assert!(retrieved.is_none());
    }

    // Example async validate implementation for testing
    struct TestUser {
        email: String,
    }

    #[async_trait]
    impl AsyncValidate for TestUser {
        async fn validate_async(&self, _ctx: &ValidationContext) -> Result<(), ValidationError> {
            if self.email.is_empty() {
                return Err(ValidationError::single(
                    crate::Path::from("email"),
                    "required",
                    "Email is required",
                ));
            }
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_async_validate_success() {
        let user = TestUser {
            email: "test@example.com".to_string(),
        };
        let ctx = ValidationContext::new();
        assert!(user.validate_async(&ctx).await.is_ok());
    }

    #[tokio::test]
    async fn test_async_validate_failure() {
        let user = TestUser {
            email: String::new(),
        };
        let ctx = ValidationContext::new();
        let result = user.validate_async(&ctx).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_async_rule_success() {
        let rule = AsyncRule::new(
            |value: &str, _ctx: &RuleContext, _vctx: &ValidationContext| {
                let len = value.len();
                async move {
                    if len >= 3 {
                        ValidationError::default()
                    } else {
                        ValidationError::single(
                            crate::Path::root(),
                            "too_short",
                            "Value must be at least 3 characters",
                        )
                    }
                }
            },
        );

        let ctx = RuleContext::root("test");
        let vctx = ValidationContext::new();
        let result = rule.apply("hello", &ctx, &vctx).await;
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn test_async_rule_failure() {
        let rule = AsyncRule::new(
            |value: &str, _ctx: &RuleContext, _vctx: &ValidationContext| {
                let len = value.len();
                async move {
                    if len >= 3 {
                        ValidationError::default()
                    } else {
                        ValidationError::single(
                            crate::Path::root(),
                            "too_short",
                            "Value must be at least 3 characters",
                        )
                    }
                }
            },
        );

        let ctx = RuleContext::root("test");
        let vctx = ValidationContext::new();
        let result = rule.apply("ab", &ctx, &vctx).await;
        assert!(!result.is_empty());
        assert_eq!(result.violations.len(), 1);
        assert_eq!(result.violations[0].code, "too_short");
    }

    #[tokio::test]
    async fn test_async_rule_with_context_resource() {
        // Simulate a database check
        #[derive(Clone)]
        struct MockDatabase {
            taken_emails: Vec<String>,
        }

        impl MockDatabase {
            fn new() -> Self {
                Self {
                    taken_emails: vec!["taken@example.com".to_string()],
                }
            }

            fn exists(&self, email: &str) -> bool {
                self.taken_emails.contains(&email.to_string())
            }
        }

        let rule = AsyncRule::new(|email: &str, ctx: &RuleContext, vctx: &ValidationContext| {
            let db = vctx
                .get_resource::<MockDatabase>("db")
                .expect("Database not in context");
            let email = email.to_string();
            let path = ctx.full_path();

            async move {
                if db.exists(&email) {
                    ValidationError::single(path, "email_taken", "Email is already registered")
                } else {
                    ValidationError::default()
                }
            }
        });

        let db = Arc::new(MockDatabase::new());
        let vctx = ValidationContext::new().with_resource("db", db);
        let ctx = RuleContext::root("email");

        // Test with taken email
        let result = rule.apply("taken@example.com", &ctx, &vctx).await;
        assert!(!result.is_empty());
        assert_eq!(result.violations[0].code, "email_taken");

        // Test with available email
        let result = rule.apply("available@example.com", &ctx, &vctx).await;
        assert!(result.is_empty());
    }
}

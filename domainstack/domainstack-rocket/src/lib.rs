//! Rocket integration for domainstack validation
//!
//! This crate provides Rocket request guards for automatic validation and domain conversion:
//!
//! - [`DomainJson<T, Dto>`] - Deserialize JSON, convert DTO to domain type, return structured errors
//! - [`ValidatedJson<Dto>`] - Deserialize and validate a DTO without domain conversion
//! - [`ErrorResponse`] - RFC 9457 compliant error responses
//!
//! ## Example
//!
//! ```rust,no_run
//! use domainstack::prelude::*;
//! use domainstack_rocket::{DomainJson, ErrorResponse};
//! use rocket::{post, routes, serde::json::Json};
//! use serde::Deserialize;
//!
//! #[derive(Deserialize)]
//! struct CreateUserDto {
//!     name: String,
//!     email: String,
//!     age: u8,
//! }
//!
//! struct User {
//!     name: String,
//!     email: String,
//!     age: u8,
//! }
//!
//! impl TryFrom<CreateUserDto> for User {
//!     type Error = domainstack::ValidationError;
//!
//!     fn try_from(dto: CreateUserDto) -> Result<Self, Self::Error> {
//!         validate("name", dto.name.as_str(), &rules::min_len(2).and(rules::max_len(50)))?;
//!         validate("email", dto.email.as_str(), &rules::email())?;
//!         validate("age", &dto.age, &rules::range(18, 120))?;
//!         Ok(Self { name: dto.name, email: dto.email, age: dto.age })
//!     }
//! }
//!
//! #[post("/users", data = "<user>")]
//! fn create_user(user: DomainJson<User, CreateUserDto>) -> Result<Json<String>, ErrorResponse> {
//!     // user.domain is guaranteed valid here!
//!     Ok(Json(format!("Created user: {}", user.domain.name)))
//! }
//!
//! #[rocket::main]
//! async fn main() {
//!     rocket::build()
//!         .mount("/", routes![create_user])
//!         .launch()
//!         .await
//!         .unwrap();
//! }
//! ```

use domainstack::ValidationError;
use rocket::{
    data::{self, Data, FromData},
    http::{ContentType, Status},
    request::Request,
    response::{self, Responder, Response},
    serde::json::Json,
};
use std::io::Cursor;
use std::marker::PhantomData;

/// Request guard for domain type conversion with automatic validation
///
/// # Type Parameters
///
/// - `T` - The domain type to convert to (must implement `TryFrom<Dto, Error = ValidationError>`)
/// - `Dto` - The DTO type to deserialize from JSON (must implement `DeserializeOwned`)
///
/// # Example
///
/// ```rust,no_run
/// use domainstack::prelude::*;
/// use domainstack_rocket::DomainJson;
/// use rocket::{post, serde::json::Json};
/// use serde::Deserialize;
///
/// #[derive(Deserialize)]
/// struct CreateUserDto {
///     name: String,
/// }
///
/// struct User {
///     name: String,
/// }
///
/// impl TryFrom<CreateUserDto> for User {
///     type Error = domainstack::ValidationError;
///     fn try_from(dto: CreateUserDto) -> Result<Self, Self::Error> {
///         validate("name", dto.name.as_str(), &rules::min_len(2))?;
///         Ok(Self { name: dto.name })
///     }
/// }
///
/// #[post("/users", data = "<user>")]
/// fn create_user(user: DomainJson<User, CreateUserDto>) -> Json<String> {
///     let domain = user.domain; // Guaranteed valid!
///     Json(domain.name)
/// }
/// ```
pub struct DomainJson<T, Dto = ()> {
    /// The validated domain object
    pub domain: T,
    _dto: PhantomData<Dto>,
}

impl<T, Dto> DomainJson<T, Dto> {
    /// Create a new DomainJson wrapper
    pub fn new(domain: T) -> Self {
        Self {
            domain,
            _dto: PhantomData,
        }
    }
}

#[rocket::async_trait]
impl<'r, T, Dto> FromData<'r> for DomainJson<T, Dto>
where
    Dto: serde::de::DeserializeOwned,
    T: TryFrom<Dto, Error = ValidationError>,
{
    type Error = ErrorResponse;

    async fn from_data(req: &'r Request<'_>, data: Data<'r>) -> data::Outcome<'r, Self> {
        // Use Rocket's JSON extractor
        let json_outcome = Json::<Dto>::from_data(req, data).await;

        let dto = match json_outcome {
            data::Outcome::Success(Json(dto)) => dto,
            data::Outcome::Forward(f) => return data::Outcome::Forward(f),
            data::Outcome::Error((status, e)) => {
                let err = ErrorResponse(error_envelope::Error::bad_request(format!(
                    "Invalid JSON: {}",
                    e
                )));
                // Store error in request-local state so catcher can access it
                req.local_cache(|| Some(err.clone()));
                return data::Outcome::Error((status, err));
            }
        };

        // Convert DTO to domain using domainstack-http helper
        match domainstack_http::into_domain(dto) {
            Ok(domain) => data::Outcome::Success(DomainJson::new(domain)),
            Err(err) => {
                let error_resp = ErrorResponse(err);
                // Store error in request-local state so catcher can access it
                req.local_cache(|| Some(error_resp.clone()));
                data::Outcome::Error((Status::BadRequest, error_resp))
            }
        }
    }
}

/// Request guard for DTO validation without domain conversion
///
/// Use this when you want to validate a DTO but don't need to convert it to a domain type.
///
/// # Example
///
/// ```rust,no_run
/// use domainstack::Validate;
/// use domainstack_rocket::ValidatedJson;
/// use rocket::{post, serde::json::Json};
/// use serde::Deserialize;
///
/// #[derive(Deserialize, Validate)]
/// struct UpdateUserDto {
///     #[validate(length(min = 2, max = 50))]
///     name: String,
/// }
///
/// #[post("/users/<id>", data = "<dto>")]
/// fn update_user(id: u64, dto: ValidatedJson<UpdateUserDto>) -> Json<String> {
///     let validated = dto.0; // Guaranteed valid!
///     Json(validated.name)
/// }
/// ```
pub struct ValidatedJson<Dto>(pub Dto);

#[rocket::async_trait]
impl<'r, Dto> FromData<'r> for ValidatedJson<Dto>
where
    Dto: serde::de::DeserializeOwned + domainstack::Validate,
{
    type Error = ErrorResponse;

    async fn from_data(req: &'r Request<'_>, data: Data<'r>) -> data::Outcome<'r, Self> {
        // Use Rocket's JSON extractor
        let json_outcome = Json::<Dto>::from_data(req, data).await;

        let dto = match json_outcome {
            data::Outcome::Success(Json(dto)) => dto,
            data::Outcome::Forward(f) => return data::Outcome::Forward(f),
            data::Outcome::Error((status, e)) => {
                let err = ErrorResponse(error_envelope::Error::bad_request(format!(
                    "Invalid JSON: {}",
                    e
                )));
                req.local_cache(|| Some(err.clone()));
                return data::Outcome::Error((status, err));
            }
        };

        // Validate using domainstack-http helper
        match domainstack_http::validate_dto(dto) {
            Ok(dto) => data::Outcome::Success(ValidatedJson(dto)),
            Err(err) => {
                let error_resp = ErrorResponse(err);
                req.local_cache(|| Some(error_resp.clone()));
                data::Outcome::Error((Status::BadRequest, error_resp))
            }
        }
    }
}

/// RFC 9457 compliant error response for Rocket
///
/// Automatically converts ValidationError into structured JSON error responses.
///
/// # Example
///
/// ```rust,no_run
/// use domainstack::prelude::*;
/// use domainstack_rocket::{DomainJson, ErrorResponse};
/// use rocket::{post, serde::json::Json};
/// use serde::Deserialize;
///
/// #[derive(Deserialize)]
/// struct CreateUserDto {
///     name: String,
/// }
///
/// struct User {
///     name: String,
/// }
///
/// impl TryFrom<CreateUserDto> for User {
///     type Error = domainstack::ValidationError;
///     fn try_from(dto: CreateUserDto) -> Result<Self, Self::Error> {
///         validate("name", dto.name.as_str(), &rules::min_len(2))?;
///         Ok(Self { name: dto.name })
///     }
/// }
///
/// #[post("/users", data = "<user>")]
/// fn create_user(user: DomainJson<User, CreateUserDto>) -> Result<Json<String>, ErrorResponse> {
///     // ErrorResponse is automatically returned on validation failure
///     Ok(Json(user.domain.name))
/// }
/// ```
#[derive(Debug, Clone)]
pub struct ErrorResponse(pub error_envelope::Error);

impl From<error_envelope::Error> for ErrorResponse {
    fn from(err: error_envelope::Error) -> Self {
        ErrorResponse(err)
    }
}

impl From<ValidationError> for ErrorResponse {
    fn from(err: ValidationError) -> Self {
        use domainstack_envelope::IntoEnvelopeError;
        ErrorResponse(err.into_envelope_error())
    }
}

impl<'r> Responder<'r, 'static> for ErrorResponse {
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'static> {
        let status = Status::from_code(self.0.status).unwrap_or(Status::InternalServerError);

        let body = serde_json::to_string(&self.0).unwrap_or_else(|_| {
            r#"{"code":"INTERNAL","message":"Serialization failed"}"#.to_string()
        });

        Response::build()
            .status(status)
            .header(ContentType::JSON)
            .sized_body(body.len(), Cursor::new(body))
            .ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use domainstack::prelude::*;
    use domainstack::Validate;
    use rocket::{
        catch, catchers,
        http::{ContentType, Status},
        local::blocking::Client,
        post, routes,
        serde::json::Json,
    };
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Deserialize)]
    struct CreateUserDto {
        name: String,
        email: String,
        age: u8,
    }

    #[derive(Debug, Clone, Serialize)]
    struct User {
        name: String,
        email: String,
        age: u8,
    }

    impl TryFrom<CreateUserDto> for User {
        type Error = ValidationError;

        fn try_from(dto: CreateUserDto) -> Result<Self, Self::Error> {
            let mut err = ValidationError::new();

            let name_rule = rules::min_len(2).and(rules::max_len(50));
            if let Err(e) = validate("name", dto.name.as_str(), &name_rule) {
                err.extend(e);
            }

            let email_rule = rules::email();
            if let Err(e) = validate("email", dto.email.as_str(), &email_rule) {
                err.extend(e);
            }

            let age_rule = rules::range(18, 120);
            if let Err(e) = validate("age", &dto.age, &age_rule) {
                err.extend(e);
            }

            if !err.is_empty() {
                return Err(err);
            }

            Ok(Self {
                name: dto.name,
                email: dto.email,
                age: dto.age,
            })
        }
    }

    #[post("/users", data = "<user>")]
    fn create_user(user: DomainJson<User, CreateUserDto>) -> Result<Json<User>, ErrorResponse> {
        Ok(Json(user.domain))
    }

    #[derive(Debug, Clone, Deserialize, Serialize, Validate)]
    struct UpdateUserDto {
        #[validate(length(min = 2, max = 50))]
        name: String,
    }

    #[post("/users/<_id>/update", data = "<dto>")]
    fn update_user(_id: u64, dto: ValidatedJson<UpdateUserDto>) -> Json<UpdateUserDto> {
        Json(dto.0)
    }

    #[catch(400)]
    fn bad_request_catcher(req: &Request) -> ErrorResponse {
        // Extract the error from the request local cache if it exists
        req.local_cache(|| None::<ErrorResponse>)
            .clone()
            .unwrap_or_else(|| ErrorResponse(error_envelope::Error::bad_request("Bad Request")))
    }

    #[test]
    fn test_domain_json_success() {
        let rocket = rocket::build()
            .mount("/", routes![create_user])
            .register("/", catchers![bad_request_catcher]);
        let client = Client::tracked(rocket).expect("valid rocket instance");

        let response = client
            .post("/users")
            .header(ContentType::JSON)
            .body(r#"{"name":"Alice","email":"alice@example.com","age":30}"#)
            .dispatch();

        assert_eq!(response.status(), Status::Ok);
        let body = response.into_string().unwrap();
        assert!(body.contains("Alice"));
        assert!(body.contains("alice@example.com"));
    }

    #[test]
    fn test_domain_json_validation_failure() {
        let rocket = rocket::build()
            .mount("/", routes![create_user])
            .register("/", catchers![bad_request_catcher]);
        let client = Client::tracked(rocket).expect("valid rocket instance");

        // Invalid: name too short, invalid email, age too young
        let response = client
            .post("/users")
            .header(ContentType::JSON)
            .body(r#"{"name":"A","email":"not-an-email","age":10}"#)
            .dispatch();

        assert_eq!(response.status(), Status::BadRequest);
        let body = response.into_string().unwrap();
        assert!(body.contains("VALIDATION"));
        assert!(body.contains("name"));
        assert!(body.contains("email"));
        assert!(body.contains("age"));
    }

    #[test]
    fn test_domain_json_invalid_json() {
        let rocket = rocket::build()
            .mount("/", routes![create_user])
            .register("/", catchers![bad_request_catcher]);
        let client = Client::tracked(rocket).expect("valid rocket instance");

        let response = client
            .post("/users")
            .header(ContentType::JSON)
            .body(r#"{"invalid json"#)
            .dispatch();

        assert_eq!(response.status(), Status::BadRequest);
        let body = response.into_string().unwrap();
        assert!(body.contains("Invalid JSON"));
    }

    #[test]
    fn test_validated_json_success() {
        let rocket = rocket::build()
            .mount("/", routes![update_user])
            .register("/", catchers![bad_request_catcher]);
        let client = Client::tracked(rocket).expect("valid rocket instance");

        let response = client
            .post("/users/1/update")
            .header(ContentType::JSON)
            .body(r#"{"name":"Alice"}"#)
            .dispatch();

        assert_eq!(response.status(), Status::Ok);
        let body = response.into_string().unwrap();
        assert!(body.contains("Alice"));
    }

    #[test]
    fn test_validated_json_failure() {
        let rocket = rocket::build()
            .mount("/", routes![update_user])
            .register("/", catchers![bad_request_catcher]);
        let client = Client::tracked(rocket).expect("valid rocket instance");

        // Invalid: name too short
        let response = client
            .post("/users/1/update")
            .header(ContentType::JSON)
            .body(r#"{"name":"A"}"#)
            .dispatch();

        assert_eq!(response.status(), Status::BadRequest);
        let body = response.into_string().unwrap();
        assert!(body.contains("VALIDATION"));
        assert!(body.contains("name"));
    }
}

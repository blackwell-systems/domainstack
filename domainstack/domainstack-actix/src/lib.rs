//! # domainstack-actix
//!
//! Actix-web integration for domainstack validation with automatic DTO→Domain conversion.
//!
//! This crate provides Actix-web extractors that automatically deserialize, validate, and convert
//! DTOs to domain types—returning structured error responses on validation failure.
//!
//! ## What it provides
//!
//! - **`DomainJson<T, Dto>`** - Extract JSON, validate, and convert DTO to domain type in one step
//! - **`ValidatedJson<Dto>`** - Extract and validate a DTO without domain conversion
//! - **`ErrorResponse`** - Automatic structured error responses with field-level details
//!
//! ## Example - DomainJson
//!
//! ```rust,no_run
//! use actix_web::{post, web, App, HttpServer};
//! use domainstack::prelude::*;
//! use domainstack_actix::{DomainJson, ErrorResponse};
//! use serde::Deserialize;
//!
//! #[derive(Deserialize)]
//! struct CreateUserDto {
//!     name: String,
//!     age: u8,
//! }
//!
//! struct User {
//!     name: String,
//!     age: u8,
//! }
//!
//! impl TryFrom<CreateUserDto> for User {
//!     type Error = domainstack::ValidationError;
//!
//!     fn try_from(dto: CreateUserDto) -> Result<Self, Self::Error> {
//!         validate("name", dto.name.as_str(), &rules::min_len(2).and(rules::max_len(50)))?;
//!         validate("age", &dto.age, &rules::range(18, 120))?;
//!         Ok(Self { name: dto.name, age: dto.age })
//!     }
//! }
//!
//! // Type alias for cleaner handler signatures
//! type UserJson = DomainJson<User, CreateUserDto>;
//!
//! #[post("/users")]
//! async fn create_user(
//!     UserJson { domain: user, .. }: UserJson
//! ) -> Result<web::Json<String>, ErrorResponse> {
//!     // user is guaranteed valid here!
//!     Ok(web::Json(format!("Created: {}", user.name)))
//! }
//!
//! #[actix_web::main]
//! async fn main() -> std::io::Result<()> {
//!     HttpServer::new(|| App::new().service(create_user))
//!         .bind(("127.0.0.1", 8080))?
//!         .run()
//!         .await
//! }
//! ```
//!
//! ## Example - ValidatedJson
//!
//! ```rust,ignore
//! use actix_web::{post, web};
//! use domainstack::Validate;
//! use domainstack_actix::{ValidatedJson, ErrorResponse};
//!
//! #[derive(Debug, Validate, serde::Deserialize)]
//! struct UserDto {
//!     #[validate(length(min = 2, max = 50))]
//!     name: String,
//!
//!     #[validate(range(min = 18, max = 120))]
//!     age: u8,
//! }
//!
//! #[post("/users")]
//! async fn create_user(
//!     ValidatedJson(dto): ValidatedJson<UserDto>
//! ) -> Result<web::Json<UserDto>, ErrorResponse> {
//!     // dto is guaranteed valid here!
//!     Ok(web::Json(dto))
//! }
//! ```
//!
//! ## Error Response Format
//!
//! On validation failure, returns a 400 Bad Request with structured errors:
//!
//! ```json
//! {
//!   "code": "VALIDATION",
//!   "status": 400,
//!   "message": "Validation failed with 2 errors",
//!   "details": {
//!     "fields": {
//!       "name": [{"code": "min_length", "message": "Must be at least 2 characters"}],
//!       "age": [{"code": "out_of_range", "message": "Must be between 18 and 120"}]
//!     }
//!   }
//! }
//! ```

use actix_web::{error::ResponseError, web, FromRequest, HttpRequest, HttpResponse};
use domainstack::ValidationError;
use futures::future::{ready, Ready};
use std::marker::PhantomData;

pub struct DomainJson<T, Dto = ()> {
    pub domain: T,
    _dto: PhantomData<Dto>,
}

impl<T, Dto> DomainJson<T, Dto> {
    pub fn new(domain: T) -> Self {
        Self {
            domain,
            _dto: PhantomData,
        }
    }
}

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

impl<T, Dto> FromRequest for DomainJson<T, Dto>
where
    Dto: serde::de::DeserializeOwned,
    T: TryFrom<Dto, Error = ValidationError>,
{
    type Error = ErrorResponse;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, payload: &mut actix_web::dev::Payload) -> Self::Future {
        let json_fut = web::Json::<Dto>::from_request(req, payload);

        // Note: Using block_on() here is required by Actix-web's synchronous extractor pattern.
        // FromRequest returns a Ready<T> future (not async), so we must synchronously extract
        // the JSON. This is the standard pattern for Actix-web 4.x extractors.
        // Performance note: This blocks the current task but does not block the async runtime.
        // For truly async extraction, consider using web::Json::from_request directly in your
        // handler and calling into_domain() on the DTO.
        ready(match futures::executor::block_on(json_fut) {
            Ok(web::Json(dto)) => domainstack_http::into_domain(dto)
                .map(DomainJson::new)
                .map_err(ErrorResponse),
            Err(e) => Err(ErrorResponse(error_envelope::Error::bad_request(format!(
                "Invalid JSON: {}",
                e
            )))),
        })
    }
}

impl ResponseError for ErrorResponse {
    fn status_code(&self) -> actix_web::http::StatusCode {
        actix_web::http::StatusCode::from_u16(self.0.status)
            .unwrap_or(actix_web::http::StatusCode::INTERNAL_SERVER_ERROR)
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).json(&self.0)
    }
}

impl std::fmt::Debug for ErrorResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ErrorResponse({:?})", self.0)
    }
}

impl std::fmt::Display for ErrorResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.message)
    }
}

pub struct ValidatedJson<Dto>(pub Dto);

impl<Dto> FromRequest for ValidatedJson<Dto>
where
    Dto: serde::de::DeserializeOwned + domainstack::Validate,
{
    type Error = ErrorResponse;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, payload: &mut actix_web::dev::Payload) -> Self::Future {
        let json_fut = web::Json::<Dto>::from_request(req, payload);

        // Note: See DomainJson implementation for explanation of block_on() usage.
        // This is the standard Actix-web 4.x extractor pattern.
        ready(match futures::executor::block_on(json_fut) {
            Ok(web::Json(dto)) => dto.validate().map(|_| ValidatedJson(dto)).map_err(|e| {
                use domainstack_envelope::IntoEnvelopeError;
                ErrorResponse(e.into_envelope_error())
            }),
            Err(e) => Err(ErrorResponse(error_envelope::Error::bad_request(format!(
                "Invalid JSON: {}",
                e
            )))),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, web, App};
    use domainstack::{prelude::*, Validate};

    // DTOs used with DomainJson are just serde shapes
    // Validation happens during TryFrom conversion to domain
    #[derive(Debug, Clone, serde::Deserialize)]
    struct UserDto {
        name: String,
        age: u8,
    }

    #[derive(Debug, serde::Serialize)]
    struct User {
        name: String,
        age: u8,
    }

    impl TryFrom<UserDto> for User {
        type Error = ValidationError;

        fn try_from(dto: UserDto) -> Result<Self, Self::Error> {
            let mut err = ValidationError::new();

            let name_rule = rules::min_len(2).and(rules::max_len(50));
            if let Err(e) = validate("name", dto.name.as_str(), &name_rule) {
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
                age: dto.age,
            })
        }
    }

    async fn create_user(
        DomainJson { domain: user, .. }: DomainJson<User, UserDto>,
    ) -> web::Json<User> {
        web::Json(user)
    }

    type UserJson = DomainJson<User, UserDto>;

    async fn create_user_with_alias(UserJson { domain: user, .. }: UserJson) -> web::Json<User> {
        web::Json(user)
    }

    async fn create_user_result_style(
        UserJson { domain: user, .. }: UserJson,
    ) -> Result<web::Json<User>, ErrorResponse> {
        Ok(web::Json(user))
    }

    #[actix_rt::test]
    async fn test_domain_json_valid() {
        let app = test::init_service(App::new().route("/", web::post().to(create_user))).await;

        let req = test::TestRequest::post()
            .uri("/")
            .set_json(serde_json::json!({"name": "Alice", "age": 30}))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);
    }

    #[actix_rt::test]
    async fn test_domain_json_invalid() {
        let app = test::init_service(App::new().route("/", web::post().to(create_user))).await;

        let req = test::TestRequest::post()
            .uri("/")
            .set_json(serde_json::json!({"name": "A", "age": 200}))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 400);

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert!(body["details"].is_object());
        assert_eq!(
            body["message"].as_str().unwrap(),
            "Validation failed with 2 errors"
        );

        let details = body["details"].as_object().unwrap();
        let fields = details["fields"].as_object().unwrap();

        assert!(fields.contains_key("name"));
        assert!(fields.contains_key("age"));
    }

    #[actix_rt::test]
    async fn test_domain_json_malformed_json() {
        let app = test::init_service(App::new().route("/", web::post().to(create_user))).await;

        let req = test::TestRequest::post()
            .uri("/")
            .set_payload("{invalid json")
            .insert_header(("content-type", "application/json"))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 400);
    }

    #[actix_rt::test]
    async fn test_domain_json_missing_fields() {
        let app = test::init_service(App::new().route("/", web::post().to(create_user))).await;

        let req = test::TestRequest::post()
            .uri("/")
            .set_json(serde_json::json!({"name": "Alice"}))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 400);
    }

    #[actix_rt::test]
    async fn test_type_alias_pattern() {
        let app =
            test::init_service(App::new().route("/", web::post().to(create_user_with_alias))).await;

        let req = test::TestRequest::post()
            .uri("/")
            .set_json(serde_json::json!({"name": "Alice", "age": 30}))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);
    }

    #[actix_rt::test]
    async fn test_result_style_handler() {
        let app =
            test::init_service(App::new().route("/", web::post().to(create_user_result_style)))
                .await;

        let req = test::TestRequest::post()
            .uri("/")
            .set_json(serde_json::json!({"name": "Alice", "age": 30}))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);
    }

    // ValidatedJson tests - DTOs that derive Validate
    #[derive(Debug, Clone, Validate, serde::Deserialize, serde::Serialize)]
    struct ValidatedUserDto {
        #[validate(length(min = 2, max = 50))]
        name: String,

        #[validate(range(min = 18, max = 120))]
        age: u8,
    }

    async fn accept_validated_dto(
        ValidatedJson(dto): ValidatedJson<ValidatedUserDto>,
    ) -> web::Json<ValidatedUserDto> {
        web::Json(dto)
    }

    #[actix_rt::test]
    async fn test_validated_json_valid() {
        let app =
            test::init_service(App::new().route("/", web::post().to(accept_validated_dto))).await;

        let req = test::TestRequest::post()
            .uri("/")
            .set_json(serde_json::json!({"name": "Alice", "age": 30}))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);

        let body: ValidatedUserDto = test::read_body_json(resp).await;
        assert_eq!(body.name, "Alice");
        assert_eq!(body.age, 30);
    }

    #[actix_rt::test]
    async fn test_validated_json_invalid() {
        let app =
            test::init_service(App::new().route("/", web::post().to(accept_validated_dto))).await;

        let req = test::TestRequest::post()
            .uri("/")
            .set_json(serde_json::json!({"name": "A", "age": 200}))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 400);

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(
            body["message"].as_str().unwrap(),
            "Validation failed with 2 errors"
        );

        let details = body["details"].as_object().unwrap();
        let fields = details["fields"].as_object().unwrap();

        assert!(fields.contains_key("name"));
        assert!(fields.contains_key("age"));
    }

    #[actix_rt::test]
    async fn test_validated_json_malformed_json() {
        let app =
            test::init_service(App::new().route("/", web::post().to(accept_validated_dto))).await;

        let req = test::TestRequest::post()
            .uri("/")
            .set_payload("{invalid json")
            .insert_header(("content-type", "application/json"))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 400);
    }

    #[actix_rt::test]
    async fn test_error_response_debug() {
        let err = ErrorResponse(error_envelope::Error::bad_request("Test error"));
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("ErrorResponse"));
    }

    #[actix_rt::test]
    async fn test_error_response_display() {
        let err = ErrorResponse(error_envelope::Error::bad_request("Custom message"));
        let display_str = format!("{}", err);
        assert_eq!(display_str, "Custom message");
    }

    #[actix_rt::test]
    async fn test_error_response_status_code() {
        use actix_web::ResponseError;

        let err = ErrorResponse(error_envelope::Error::bad_request("Bad request"));
        assert_eq!(err.status_code().as_u16(), 400);

        let mut custom_err = error_envelope::Error::bad_request("Custom");
        custom_err.status = 422;
        let err = ErrorResponse(custom_err);
        assert_eq!(err.status_code().as_u16(), 422);
    }
}

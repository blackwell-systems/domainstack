use axum::{
    extract::{FromRequest, Request},
    response::{IntoResponse, Response},
    Json,
};
use domainstack::ValidationError;
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

#[axum::async_trait]
impl<T, Dto, S> FromRequest<S> for DomainJson<T, Dto>
where
    Dto: serde::de::DeserializeOwned,
    T: TryFrom<Dto, Error = ValidationError>,
    S: Send + Sync,
{
    type Rejection = ErrorResponse;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Json(dto) = Json::<Dto>::from_request(req, state).await.map_err(|e| {
            ErrorResponse(error_envelope::Error::bad_request(format!(
                "Invalid JSON: {}",
                e
            )))
        })?;

        let domain = domainstack_http::into_domain(dto).map_err(ErrorResponse)?;

        Ok(DomainJson::new(domain))
    }
}

impl IntoResponse for ErrorResponse {
    fn into_response(self) -> Response {
        let status = axum::http::StatusCode::from_u16(self.0.status)
            .unwrap_or(axum::http::StatusCode::INTERNAL_SERVER_ERROR);

        let body = serde_json::to_string(&self.0).unwrap_or_else(|_| {
            r#"{"code":"INTERNAL","message":"Serialization failed"}"#.to_string()
        });

        (status, body).into_response()
    }
}

pub struct ValidatedJson<Dto>(pub Dto);

#[axum::async_trait]
impl<Dto, S> FromRequest<S> for ValidatedJson<Dto>
where
    Dto: serde::de::DeserializeOwned + domainstack::Validate,
    S: Send + Sync,
{
    type Rejection = ErrorResponse;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Json(dto) = Json::<Dto>::from_request(req, state).await.map_err(|e| {
            ErrorResponse(error_envelope::Error::bad_request(format!(
                "Invalid JSON: {}",
                e
            )))
        })?;

        dto.validate().map(|_| ValidatedJson(dto)).map_err(|e| {
            use domainstack_envelope::IntoEnvelopeError;
            ErrorResponse(e.into_envelope_error())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{routing::post, Router};
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

    async fn create_user(DomainJson { domain: user, .. }: DomainJson<User, UserDto>) -> Json<User> {
        Json(user)
    }

    type UserJson = DomainJson<User, UserDto>;

    async fn create_user_with_alias(UserJson { domain: user, .. }: UserJson) -> Json<User> {
        Json(user)
    }

    async fn create_user_result_style(
        UserJson { domain: user, .. }: UserJson,
    ) -> Result<Json<User>, ErrorResponse> {
        Ok(Json(user))
    }

    #[tokio::test]
    async fn test_domain_json_valid() {
        let app = Router::new().route("/", post(create_user));

        let server = axum_test::TestServer::new(app).unwrap();

        let response = server
            .post("/")
            .json(&serde_json::json!({"name": "Alice", "age": 30}))
            .await;

        response.assert_status_ok();
    }

    #[tokio::test]
    async fn test_domain_json_invalid() {
        let app = Router::new().route("/", post(create_user));

        let server = axum_test::TestServer::new(app).unwrap();

        let response = server
            .post("/")
            .json(&serde_json::json!({"name": "A", "age": 200}))
            .await;

        response.assert_status_bad_request();

        let body: serde_json::Value = response.json();

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

    #[tokio::test]
    async fn test_domain_json_malformed_json() {
        let app = Router::new().route("/", post(create_user));

        let server = axum_test::TestServer::new(app).unwrap();

        let response = server.post("/").text("{invalid json").await;

        response.assert_status_bad_request();
    }

    #[tokio::test]
    async fn test_domain_json_missing_fields() {
        let app = Router::new().route("/", post(create_user));

        let server = axum_test::TestServer::new(app).unwrap();

        let response = server
            .post("/")
            .json(&serde_json::json!({"name": "Alice"}))
            .await;

        response.assert_status_bad_request();
    }

    #[tokio::test]
    async fn test_type_alias_pattern() {
        let app = Router::new().route("/", post(create_user_with_alias));

        let server = axum_test::TestServer::new(app).unwrap();

        let response = server
            .post("/")
            .json(&serde_json::json!({"name": "Alice", "age": 30}))
            .await;

        response.assert_status_ok();
    }

    #[tokio::test]
    async fn test_result_style_handler() {
        let app = Router::new().route("/", post(create_user_result_style));

        let server = axum_test::TestServer::new(app).unwrap();

        let response = server
            .post("/")
            .json(&serde_json::json!({"name": "Alice", "age": 30}))
            .await;

        response.assert_status_ok();
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
    ) -> Json<ValidatedUserDto> {
        Json(dto)
    }

    #[tokio::test]
    async fn test_validated_json_valid() {
        let app = Router::new().route("/", post(accept_validated_dto));

        let server = axum_test::TestServer::new(app).unwrap();

        let response = server
            .post("/")
            .json(&serde_json::json!({"name": "Alice", "age": 30}))
            .await;

        response.assert_status_ok();
        let body: ValidatedUserDto = response.json();
        assert_eq!(body.name, "Alice");
        assert_eq!(body.age, 30);
    }

    #[tokio::test]
    async fn test_validated_json_invalid() {
        let app = Router::new().route("/", post(accept_validated_dto));

        let server = axum_test::TestServer::new(app).unwrap();

        let response = server
            .post("/")
            .json(&serde_json::json!({"name": "A", "age": 200}))
            .await;

        response.assert_status_bad_request();

        let body: serde_json::Value = response.json();
        assert_eq!(
            body["message"].as_str().unwrap(),
            "Validation failed with 2 errors"
        );

        let details = body["details"].as_object().unwrap();
        let fields = details["fields"].as_object().unwrap();

        assert!(fields.contains_key("name"));
        assert!(fields.contains_key("age"));
    }

    #[tokio::test]
    async fn test_validated_json_malformed_json() {
        let app = Router::new().route("/", post(accept_validated_dto));

        let server = axum_test::TestServer::new(app).unwrap();

        let response = server.post("/").text("{invalid json").await;

        response.assert_status_bad_request();
    }

    #[tokio::test]
    async fn test_error_response_into_response() {
        let err = ErrorResponse(error_envelope::Error::bad_request("Test error"));
        let response = err.into_response();
        assert_eq!(response.status(), axum::http::StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_error_response_custom_status() {
        let mut err = error_envelope::Error::bad_request("Test");
        err.status = 422;
        let response = ErrorResponse(err).into_response();
        assert_eq!(
            response.status(),
            axum::http::StatusCode::UNPROCESSABLE_ENTITY
        );
    }
}

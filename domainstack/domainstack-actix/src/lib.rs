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

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, web, App};
    use domainstack::{prelude::*, Validate};

    #[derive(Debug, Clone, Validate, serde::Deserialize)]
    struct UserDto {
        #[validate(length(min = 2, max = 50))]
        name: String,

        #[validate(range(min = 18, max = 120))]
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
}

use domainstack::{Validate, ValidationError};
use domainstack_envelope::IntoEnvelopeError;

#[allow(clippy::result_large_err)]
pub fn into_domain<T, Dto>(dto: Dto) -> Result<T, error_envelope::Error>
where
    T: TryFrom<Dto, Error = ValidationError>,
{
    T::try_from(dto).map_err(|e| e.into_envelope_error())
}

#[allow(clippy::result_large_err)]
pub fn validate_dto<Dto>(dto: Dto) -> Result<Dto, error_envelope::Error>
where
    Dto: Validate,
{
    dto.validate()
        .map(|_| dto)
        .map_err(|e| e.into_envelope_error())
}

#[cfg(test)]
mod tests {
    use super::*;
    use domainstack::prelude::*;
    use domainstack::Validate;

    #[derive(Debug, Clone, Validate)]
    struct EmailDto {
        #[validate(length(min = 5, max = 255))]
        value: String,
    }

    #[derive(Debug, Clone)]
    struct Email(#[allow(dead_code)] String);

    impl Email {
        #[allow(clippy::result_large_err)]
        pub fn new(raw: String) -> Result<Self, ValidationError> {
            let rule = rules::min_len(5).and(rules::max_len(255));
            validate("email", raw.as_str(), &rule)?;
            Ok(Self(raw))
        }
    }

    impl TryFrom<EmailDto> for Email {
        type Error = ValidationError;

        fn try_from(dto: EmailDto) -> Result<Self, Self::Error> {
            Email::new(dto.value)
        }
    }

    #[test]
    fn test_into_domain_valid() {
        let dto = EmailDto {
            value: "test@example.com".to_string(),
        };

        let result = into_domain::<Email, EmailDto>(dto);
        assert!(result.is_ok());
    }

    #[test]
    fn test_into_domain_invalid_too_short() {
        let dto = EmailDto {
            value: "abc".to_string(),
        };

        let result = into_domain::<Email, EmailDto>(dto);
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert_eq!(err.status, 400);
        assert!(err.details.is_some());
    }

    #[test]
    fn test_into_domain_invalid_too_long() {
        let dto = EmailDto {
            value: "a".repeat(300),
        };

        let result = into_domain::<Email, EmailDto>(dto);
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert_eq!(err.status, 400);
        assert!(err.details.is_some());
    }

    #[test]
    fn test_validate_dto_valid() {
        let dto = EmailDto {
            value: "test@example.com".to_string(),
        };

        let result = validate_dto(dto.clone());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().value, dto.value);
    }

    #[test]
    fn test_validate_dto_invalid() {
        let dto = EmailDto {
            value: "abc".to_string(),
        };

        let result = validate_dto(dto);
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert_eq!(err.status, 400);
        assert!(err.details.is_some());
    }

    #[derive(Debug, Clone, Validate)]
    struct UserDto {
        #[validate(length(min = 2, max = 50))]
        name: String,

        #[validate(range(min = 18, max = 120))]
        age: u8,
    }

    #[derive(Debug)]
    #[allow(dead_code)]
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

    #[test]
    fn test_into_domain_user_valid() {
        let dto = UserDto {
            name: "Alice".to_string(),
            age: 30,
        };

        let result = into_domain::<User, UserDto>(dto);
        assert!(result.is_ok());
    }

    #[test]
    fn test_into_domain_user_invalid_multiple_errors() {
        let dto = UserDto {
            name: "A".to_string(),
            age: 200,
        };

        let result = into_domain::<User, UserDto>(dto);
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert_eq!(err.status, 400);

        let details = err.details.as_ref().unwrap();
        let fields = details.as_object().unwrap().get("fields").unwrap();
        let fields_obj = fields.as_object().unwrap();

        assert!(fields_obj.contains_key("name"));
        assert!(fields_obj.contains_key("age"));
    }

    #[test]
    fn test_validate_dto_user_valid() {
        let dto = UserDto {
            name: "Alice".to_string(),
            age: 30,
        };

        let result = validate_dto(dto.clone());
        assert!(result.is_ok());
        let validated = result.unwrap();
        assert_eq!(validated.name, dto.name);
        assert_eq!(validated.age, dto.age);
    }

    #[test]
    fn test_validate_dto_user_invalid() {
        let dto = UserDto {
            name: "A".to_string(),
            age: 200,
        };

        let result = validate_dto(dto);
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert_eq!(err.status, 400);

        let details = err.details.as_ref().unwrap();
        let fields = details.as_object().unwrap().get("fields").unwrap();
        let fields_obj = fields.as_object().unwrap();

        assert!(fields_obj.contains_key("name"));
        assert!(fields_obj.contains_key("age"));
    }
}

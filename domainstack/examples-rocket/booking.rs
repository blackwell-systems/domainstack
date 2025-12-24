use domainstack::prelude::*;
use domainstack_rocket::{DomainJson, ErrorResponse};
use rocket::{catch, catchers, get, post, routes, serde::json::Json, Request, State};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize)]
pub struct CreateBookingDto {
    pub guest_name: String,
    pub email: String,
    pub nights: u8,
    pub adults: u8,
}

#[derive(Debug, Clone)]
pub struct Email(String);

impl Email {
    #[allow(clippy::result_large_err)]
    pub fn new(raw: String) -> Result<Self, ValidationError> {
        // Use Path::root() in primitives - caller will prefix with field name
        let rule = rules::min_len(5)
            .and(rules::max_len(255))
            .and(rules::email());
        validate(Path::root(), raw.as_str(), &rule)?;

        Ok(Self(raw))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone)]
pub struct CreateBookingCommand {
    guest_name: String,
    email: Email,
    nights: u8,
    adults: u8,
}

impl CreateBookingCommand {
    pub fn guest_name(&self) -> &str {
        &self.guest_name
    }

    pub fn email(&self) -> &Email {
        &self.email
    }

    pub fn nights(&self) -> u8 {
        self.nights
    }

    pub fn adults(&self) -> u8 {
        self.adults
    }
}

impl TryFrom<CreateBookingDto> for CreateBookingCommand {
    type Error = ValidationError;

    fn try_from(dto: CreateBookingDto) -> Result<Self, Self::Error> {
        let mut err = ValidationError::new();

        if let Err(e) = validate(
            "guest_name",
            dto.guest_name.as_str(),
            &rules::min_len(1).and(rules::max_len(100)),
        ) {
            err.extend(e);
        }

        let email = match Email::new(dto.email) {
            Ok(email) => Some(email),
            Err(e) => {
                err.extend(e.prefixed("email"));
                None
            }
        };

        if let Err(e) = validate("nights", &dto.nights, &rules::range(1, 30)) {
            err.extend(e);
        }

        if let Err(e) = validate("adults", &dto.adults, &rules::range(1, 4)) {
            err.extend(e);
        }

        if !err.is_empty() {
            return Err(err);
        }

        Ok(Self {
            guest_name: dto.guest_name,
            email: email.unwrap(),
            nights: dto.nights,
            adults: dto.adults,
        })
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Booking {
    pub id: Uuid,
    pub guest_name: String,
    pub email: String,
    pub nights: u8,
    pub adults: u8,
}

#[derive(Clone)]
pub struct BookingService {
    bookings: Arc<Mutex<Vec<Booking>>>,
}

impl Default for BookingService {
    fn default() -> Self {
        Self::new()
    }
}

impl BookingService {
    pub fn new() -> Self {
        Self {
            bookings: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn create(&self, cmd: CreateBookingCommand) -> Booking {
        let booking = Booking {
            id: Uuid::new_v4(),
            guest_name: cmd.guest_name().to_string(),
            email: cmd.email().as_str().to_string(),
            nights: cmd.nights(),
            adults: cmd.adults(),
        };

        self.bookings.lock().unwrap().push(booking.clone());
        booking
    }

    pub fn list(&self) -> Vec<Booking> {
        self.bookings.lock().unwrap().clone()
    }
}

type CreateBookingJson = DomainJson<CreateBookingCommand, CreateBookingDto>;

#[post("/bookings", data = "<booking>")]
fn create_booking(
    service: &State<BookingService>,
    booking: CreateBookingJson,
) -> Result<Json<Booking>, ErrorResponse> {
    let created = service.create(booking.domain);
    Ok(Json(created))
}

#[get("/bookings")]
fn list_bookings(service: &State<BookingService>) -> Json<Vec<Booking>> {
    Json(service.list())
}

#[get("/health")]
fn health() -> &'static str {
    "OK"
}

/// Error catcher for validation failures
///
/// This is required to properly handle validation errors from DomainJson guards.
/// The guard stores the ErrorResponse in request-local state when validation fails,
/// and this catcher retrieves it to return as JSON.
#[catch(400)]
fn validation_catcher(req: &Request) -> ErrorResponse {
    req.local_cache(|| None::<ErrorResponse>)
        .clone()
        .unwrap_or_else(|| ErrorResponse(error_envelope::Error::bad_request("Bad Request")))
}

#[rocket::main]
async fn main() {
    let service = BookingService::new();

    let rocket = rocket::build()
        .manage(service)
        .mount("/", routes![health, create_booking, list_bookings])
        .register("/", catchers![validation_catcher]);

    println!("🚀 Server launching on http://127.0.0.1:8000");
    println!();
    println!("Try:");
    println!("  curl -X POST http://127.0.0.1:8000/bookings \\");
    println!("    -H 'Content-Type: application/json' \\");
    println!("    -d '{{\"guest_name\":\"Alice\",\"email\":\"alice@example.com\",\"nights\":3,\"adults\":2}}'");
    println!();
    println!("  curl http://127.0.0.1:8000/bookings");
    println!();
    println!("Test validation errors:");
    println!("  curl -X POST http://127.0.0.1:8000/bookings \\");
    println!("    -H 'Content-Type: application/json' \\");
    println!("    -d '{{\"guest_name\":\"A\",\"email\":\"bad\",\"nights\":0,\"adults\":10}}'");

    rocket.launch().await.unwrap();
}

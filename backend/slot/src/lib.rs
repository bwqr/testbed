use actix_web::http::StatusCode;
use actix_web::web;

use core::error::{ErrorMessaging, HttpError};
use user::middlewares::auth::Auth;

mod handlers;
mod models;
mod requests;

pub fn register(config: &mut web::ServiceConfig) {
    config
        .service(
            web::scope("/api/slot")
                .wrap(Auth)
                .service(handlers::fetch_slots)
                .service(handlers::fetch_slot)
                .service(handlers::reserve_slot)
        );
}

#[derive(Debug)]
pub enum ErrorMessage {
    InvalidSlotInterval,
    AlreadyReserved,
}

impl ErrorMessaging for ErrorMessage {
    fn value(&self) -> HttpError {
        match self {
            ErrorMessage::InvalidSlotInterval => HttpError {
                code: StatusCode::UNPROCESSABLE_ENTITY,
                error_code: 100,
                message: String::from("invalid_slot_interval"),
            },
            ErrorMessage::AlreadyReserved => HttpError {
                code: StatusCode::UNPROCESSABLE_ENTITY,
                error_code: 101,
                message: String::from("already_reserved"),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

use actix_web::http::StatusCode;
use actix_web::web;

use core::error::{ErrorMessaging, HttpError};

mod handlers;
mod requests;
mod templates;

pub fn register(config: &mut web::ServiceConfig) {
    config
        .service(
            web::scope("/api/auth")
                .service(handlers::login)
                .service(handlers::sign_up)
        );
}

#[derive(Debug)]
pub enum ErrorMessage {
    UserExists,
    InvalidCredentialsOrUser,
    NotVerified,
    Banned,
}

impl ErrorMessaging for ErrorMessage {
    fn value(&self) -> HttpError {
        match self {
            ErrorMessage::UserExists => HttpError {
                code: StatusCode::UNPROCESSABLE_ENTITY,
                error_code: 100,
                message: String::from("user_exists"),
            },
            ErrorMessage::InvalidCredentialsOrUser => HttpError {
                code: StatusCode::UNPROCESSABLE_ENTITY,
                error_code: 101,
                message: String::from("invalid_credentials"),
            },
            ErrorMessage::NotVerified => HttpError {
                code: StatusCode::UNPROCESSABLE_ENTITY,
                error_code: 102,
                message: String::from("not_verified"),
            },
            ErrorMessage::Banned => HttpError {
                code: StatusCode::UNPROCESSABLE_ENTITY,
                error_code: 102,
                message: String::from("banned"),
            }
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

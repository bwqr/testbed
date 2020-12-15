#[macro_use]
extern crate diesel;
#[macro_use]
extern crate lazy_static;

use std::fmt::Debug;

use actix_web::http::StatusCode;

use crate::error::{ErrorMessaging, HttpError};

pub mod db;
pub mod error;
pub mod middlewares;
pub mod models;
pub mod schema;
pub mod types;
pub mod utils;
pub mod websocket_messages;

#[derive(Debug)]
pub enum SocketErrorKind {
    InvalidMessage,
}

#[derive(Debug)]
pub enum ErrorMessage {
    DBError,
    TokenNotFound,
    InvalidToken,
    ExpiredToken,
    UserNotFound,
    BlockingCanceled,
    UniqueViolation,
    ItemNotFound,
    UnknownError,
}

impl ErrorMessaging for ErrorMessage {
    fn value(&self) -> HttpError {
        match self {
            ErrorMessage::DBError => HttpError {
                code: StatusCode::INTERNAL_SERVER_ERROR,
                error_code: 100,
                message: String::from("db_error"),
            },
            ErrorMessage::UserNotFound => HttpError {
                code: StatusCode::UNAUTHORIZED,
                error_code: 101,
                message: String::from("user_not_found"),
            },
            ErrorMessage::TokenNotFound => HttpError {
                code: StatusCode::UNAUTHORIZED,
                error_code: 104,
                message: String::from("token_not_found"),
            },
            ErrorMessage::InvalidToken => HttpError {
                code: StatusCode::UNAUTHORIZED,
                error_code: 105,
                message: String::from("invalid_token"),
            },
            ErrorMessage::ExpiredToken => HttpError {
                code: StatusCode::UNAUTHORIZED,
                error_code: 106,
                message: String::from("expired_token"),
            },
            ErrorMessage::BlockingCanceled => HttpError {
                code: StatusCode::INTERNAL_SERVER_ERROR,
                error_code: 106,
                message: String::from("blocking_canceled"),
            },
            ErrorMessage::UniqueViolation => HttpError {
                code: StatusCode::CONFLICT,
                error_code: 110,
                message: String::from("unique_violation"),
            },
            ErrorMessage::ItemNotFound => HttpError {
                code: StatusCode::NOT_FOUND,
                error_code: 101,
                message: String::from("item_not_found"),
            },
            ErrorMessage::UnknownError => HttpError {
                code: StatusCode::INTERNAL_SERVER_ERROR,
                error_code: 103,
                message: String::from("unknown_error"),
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

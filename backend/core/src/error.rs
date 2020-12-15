use core::fmt::Debug;
use std::fmt;

use actix_web::http::StatusCode;
use actix_web::HttpResponse;
use diesel::result::{DatabaseErrorKind, Error};
pub use jsonwebtoken::Algorithm;
use serde::{ser::SerializeStruct, Serialize, Serializer};

use crate::ErrorMessage;

#[derive(Debug)]
pub struct HttpError {
    pub code: StatusCode,
    pub error_code: i32,
    pub message: String,
}

impl Serialize for HttpError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
    {
        // 3 is the number of fields in the struct.
        let mut state = serializer.serialize_struct("HttpError", 3)?;
        state.serialize_field("code", &StatusCode::as_u16(&self.code))?;
        state.serialize_field("errorCode", &self.error_code)?;
        state.serialize_field("message", &self.message)?;
        state.end()
    }
}

impl fmt::Display for HttpError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "code: {}, error_code: {}, message: {}", self.code, self.error_code, self.message)
    }
}

pub trait ErrorMessaging: Debug + Send {
    fn error(&self) -> HttpResponse {
        let http_error = self.value();
        HttpResponse::build(http_error.code).json(http_error)
    }

    fn value(&self) -> HttpError;
}

impl ErrorMessaging for Error {
    fn value(&self) -> HttpError {
        diesel_error_into_messaging(self).value()
    }
}


fn diesel_error_into_messaging(e: &Error) -> ErrorMessage {
    match e {
        Error::DatabaseError(kind, _) => match kind {
            DatabaseErrorKind::UniqueViolation => ErrorMessage::UniqueViolation,
            _ => ErrorMessage::DBError
        },
        Error::NotFound => ErrorMessage::ItemNotFound,
        _ => ErrorMessage::UnknownError
    }
}
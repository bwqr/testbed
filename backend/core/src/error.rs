use core::fmt::Debug;
use std::fmt;

use actix_web::{HttpResponse, ResponseError};
use actix_web::http::StatusCode;
use actix_web::rt::blocking::BlockingError;
use diesel::result::{DatabaseErrorKind, Error};
use jsonwebtoken::errors::Error as JWTErrors;
use serde::{ser::SerializeStruct, Serialize, Serializer};
use validator::ValidationErrors;

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

#[derive(Debug)]
pub struct ValidationError {
    pub code: StatusCode,
    pub error_code: i32,
    pub message: String,
    pub validation_errors: ValidationErrors,
}

impl Serialize for ValidationError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
    {
        // 4 is the number of fields in the struct.
        let mut state = serializer.serialize_struct("ValidationError", 4)?;
        state.serialize_field("code", &StatusCode::as_u16(&self.code))?;
        state.serialize_field("errorCode", &self.error_code)?;
        state.serialize_field("message", &self.message)?;
        state.serialize_field("validationErrors", &self.validation_errors)?;
        state.end()
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "code: {}, error_code: {}, message: {}, validation_errors: omitted", self.code, self.error_code, self.message)
    }
}

impl From<ValidationErrors> for ValidationError {
    fn from(validation_errors: ValidationErrors) -> Self {
        ValidationError {
            code: StatusCode::UNPROCESSABLE_ENTITY,
            error_code: 1,
            message: "validation_errors".to_string(),
            validation_errors,
        }
    }
}

impl ErrorMessaging for ValidationError {
    fn error(&self) -> HttpResponse {
        HttpResponse::UnprocessableEntity().json(self)
    }

    fn value(&self) -> HttpError {
        HttpError {
            code: self.code,
            error_code: self.error_code,
            message: self.message.clone(),
        }
    }
}

pub trait ErrorMessaging: Debug + Send {
    fn error(&self) -> HttpResponse {
        let http_error = self.value();
        HttpResponse::build(http_error.code).json(http_error)
    }

    fn value(&self) -> HttpError;
}

impl ErrorMessaging for JWTErrors {
    fn value(&self) -> HttpError {
        ErrorMessage::HashFailed.value()
    }
}

impl fmt::Display for Box<dyn ErrorMessaging> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.value(), f)
    }
}

impl ResponseError for Box<dyn ErrorMessaging> {
    fn status_code(&self) -> StatusCode {
        self.value().code
    }

    fn error_response(&self) -> HttpResponse {
        self.error()
    }
}

impl<T> ErrorMessaging for BlockingError<T> where T: ErrorMessaging {
    fn value(&self) -> HttpError {
        match self {
            BlockingError::Error(t) => t.value(),
            BlockingError::Canceled => ErrorMessage::BlockingCanceled.value()
        }
    }
}

impl<T> From<T> for Box<dyn ErrorMessaging> where T: ErrorMessaging + 'static {
    fn from(m: T) -> Box<dyn ErrorMessaging> {
        Box::new(m)
    }
}

impl ErrorMessaging for BlockingError<Box<dyn ErrorMessaging>> {
    fn value(&self) -> HttpError {
        match self {
            BlockingError::Error(t) => t.value(),
            BlockingError::Canceled => ErrorMessage::BlockingCanceled.value()
        }
    }
}

impl ErrorMessaging for Error {
    fn value(&self) -> HttpError {
        match self {
            Error::DatabaseError(kind, _) => match kind {
                DatabaseErrorKind::UniqueViolation => ErrorMessage::UniqueViolation,
                _ => ErrorMessage::DBError
            },
            Error::NotFound => ErrorMessage::ItemNotFound,
            _ => ErrorMessage::UnknownError
        }.value()
    }
}

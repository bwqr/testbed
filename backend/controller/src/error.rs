use serde::Serialize;

#[derive(Serialize)]
pub enum ErrorCause {
    User,
    Abort,
    Internal
}

#[derive(Serialize)]
pub struct Error {
    pub kind: &'static str,
    pub cause: ErrorCause,
    pub detail: Option<String>,
    pub context: Option<&'static str>,
    pub output: Option<String>,
}

impl Error {
    pub fn new(kind: &'static str, cause: ErrorCause) -> Error {
        Error {
            kind, cause,
            detail: None,
            context: None,
            output: None
        }
    }
}

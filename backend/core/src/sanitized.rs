use actix_web::{FromRequest, HttpRequest, web::{Json, Path, Query}};
use actix_web::dev::Payload;
use futures::future::Map;
use futures::FutureExt;
use serde::de::DeserializeOwned;

pub trait Sanitize {
    fn sanitize(self) -> Self;
}

impl Sanitize for String {
    fn sanitize(self) -> Self {
        crate::encode_minimal(self.as_str())
    }
}

impl<T> Sanitize for Option<T> where T: Sanitize {
    fn sanitize(self) -> Self {
        if let Some(t) = self {
            Some(t.sanitize())
        } else {
            None
        }
    }
}

pub struct SanitizedPath<T>(T);

impl<T> SanitizedPath<T> {
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> From<T> for SanitizedPath<T> {
    fn from(inner: T) -> Self {
        SanitizedPath(inner)
    }
}

impl<T> FromRequest for SanitizedPath<T> where T: Sanitize + DeserializeOwned {
    type Error = <Path<T> as FromRequest>::Error;
    type Future = Map<<Path<T> as FromRequest>::Future, fn(Result<Path<T>, Self::Error>) -> Result<SanitizedPath<T>, Self::Error>>;
    type Config = ();

    fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
        <Path<T> as FromRequest>::from_request(req, payload)
            .map(|res| res.map(|t| SanitizedPath(t.into_inner().sanitize())))
    }
}

pub struct SanitizedJson<T>(pub T);

impl<T> SanitizedJson<T> {
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> From<T> for SanitizedJson<T> {
    fn from(inner: T) -> Self {
        SanitizedJson(inner)
    }
}

impl<T> FromRequest for SanitizedJson<T> where T: Sanitize + DeserializeOwned + 'static {
    type Error = <Json<T> as FromRequest>::Error;
    type Future = Map<<Json<T> as FromRequest>::Future, fn(Result<Json<T>, Self::Error>) -> Result<SanitizedJson<T>, Self::Error>>;
    type Config = ();

    fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
        <Json<T> as FromRequest>::from_request(req, payload)
            .map(|res| res.map(|t| SanitizedJson(t.into_inner().sanitize())))
    }
}

pub struct SanitizedQuery<T>(T);

impl<T> SanitizedQuery<T> {
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> From<T> for SanitizedQuery<T> {
    fn from(inner: T) -> Self {
        SanitizedQuery(inner)
    }
}

impl<T> FromRequest for SanitizedQuery<T> where T: Sanitize + DeserializeOwned {
    type Error = <Query<T> as FromRequest>::Error;
    type Future = Map<<Query<T> as FromRequest>::Future, fn(Result<Query<T>, Self::Error>) -> Result<SanitizedQuery<T>, Self::Error>>;
    type Config = ();

    fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
        <Query<T> as FromRequest>::from_request(req, payload)
            .map(|res| res.map(|t| SanitizedQuery(t.into_inner().sanitize())))
    }
}
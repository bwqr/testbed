use actix_web::{Error, HttpMessage, web};
use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use futures::future::{Either, ok, Ready};
use futures::task::{Context, Poll};
use regex::Regex;

use crate::ErrorMessage;
use crate::models::AuthToken;
use crate::utils::{Hash, JWTErrorKind};
use crate::error::ErrorMessaging;

pub struct Auth;


impl<S, B> Transform<S> for Auth
    where
        S: Service<Request=ServiceRequest, Response=ServiceResponse<B>, Error=Error>,
        S::Future: 'static,
        B: 'static
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = AuthMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(AuthMiddleware { service })
    }
}


pub struct AuthMiddleware<S> {
    service: S
}

impl<S, B> Service for AuthMiddleware<S>
    where
        S: Service<Request=ServiceRequest, Response=ServiceResponse<B>, Error=Error>,
        S::Future: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Either<S::Future, Ready<Result<Self::Response, Self::Error>>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&mut self, req: ServiceRequest) -> Self::Future {
        match parse_user(&req) {
            Ok(auth_token) => {
                req.extensions_mut().insert(auth_token);
                Either::Left(self.service.call(req))
            }
            Err(e) => Either::Right(ok(req.into_response(e.error().into_body())))
        }
    }
}

fn parse_user(req: &ServiceRequest) -> Result<AuthToken, ErrorMessage> {
    lazy_static! {
                    static ref HEADER_RE: Regex = Regex::new(r"^Bearer ([A-Za-z0-9-_=]+\.[A-Za-z0-9-_=]+\.?[A-Za-z0-9-_.+/=]*$)").unwrap();
                    static ref QUERY_RE: Regex = Regex::new(r"token=([A-Za-z0-9-_=]+\.[A-Za-z0-9-_=]+\.?[A-Za-z0-9-_.+/=]*$)").unwrap();
                }

    let hash = req.app_data::<web::Data<Hash>>().unwrap();

    let token = if let Some(authorization_header) = req.headers().get::<String>(String::from("Authorization")) {
        let bearer_token = authorization_header.to_str()
            .map_err(|_e| ErrorMessage::TokenNotFound)?;

        HEADER_RE.captures(bearer_token)
            .ok_or_else(|| ErrorMessage::InvalidToken)?
            .get(1)
            .ok_or_else(|| ErrorMessage::InvalidToken)?
    } else {
        let query_string = req.query_string();

        QUERY_RE.captures(query_string)
            .ok_or_else(|| ErrorMessage::TokenNotFound)?
            .get(1)
            .ok_or_else(|| ErrorMessage::InvalidToken)?
    };


    let auth_token = hash.decode::<AuthToken>(token.as_str())
        .map_err(|e| {
            match e.kind() {
                JWTErrorKind::ExpiredSignature => ErrorMessage::ExpiredToken,
                _ => ErrorMessage::InvalidToken,
            }
        })?;

    Ok(auth_token)
}

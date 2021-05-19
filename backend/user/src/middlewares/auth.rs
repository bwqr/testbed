use std::cell::RefCell;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;

use actix_web::{Error, error::BlockingError, HttpMessage, web};
use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use diesel::prelude::*;
use diesel::result::Error as DieselError;
use futures::future::{ok, Ready};
use futures::task::{Context, Poll};
use regex::Regex;

use core::error::ErrorMessaging;
use core::ErrorMessage;
use core::models::token::AuthToken;
use core::schema::users;
use core::types::DBPool;
use core::utils::{Hash, JWTErrorKind};

use crate::models::user::User;

pub struct Auth;


impl<S, B> Transform<S> for Auth
    where
        S: Service<Request=ServiceRequest, Response=ServiceResponse<B>, Error=Error> + 'static,
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
        ok(AuthMiddleware { service: Rc::new(RefCell::new(service)) })
    }
}


pub struct AuthMiddleware<S> {
    service: Rc<RefCell<S>>,
}

impl<S, B> Service for AuthMiddleware<S>
    where
        S: Service<Request=ServiceRequest, Response=ServiceResponse<B>, Error=Error> + 'static,
        S::Future: 'static,
        B: 'static
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output=Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&mut self, req: ServiceRequest) -> Self::Future {
        let mut service = self.service.clone();
        Box::pin(async move {
            match parse_token(&req) {
                Ok(auth_token) => {
                    let conn = if let Some(pool) = req.app_data::<web::Data<DBPool>>() {
                        pool.get().unwrap()
                    } else {
                        return Ok(req.into_response(ErrorMessage::MiddlewareFailed.error().into_body()));
                    };

                    match web::block(move || users::table.find(auth_token.user_id).first::<User>(&conn))
                        .await {
                        Ok(user) => {
                            req.extensions_mut().insert(user);
                            service.call(req).await
                        }
                        Err(BlockingError::Error(DieselError::NotFound)) => Ok(req.into_response(ErrorMessage::UserNotFound.error().into_body())),
                        Err(_) => Ok(req.into_response(ErrorMessage::MiddlewareFailed.error().into_body())),
                    }
                }
                Err(e) => Ok(req.into_response(e.error().into_body()))
            }
        })
    }
}

fn parse_token(req: &ServiceRequest) -> Result<AuthToken, ErrorMessage> {
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

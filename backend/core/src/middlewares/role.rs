use std::task::{Context, Poll};

use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::Error;
use futures::future::{Either, ok, Ready};

use crate::error::ErrorMessaging;
use crate::ErrorMessage;
use crate::models::token::AuthToken;
use crate::types::ModelId;

pub struct AdminUser;

impl<S, B> Transform<S> for AdminUser
    where
        S: Service<Request=ServiceRequest, Response=ServiceResponse<B>, Error=Error>,
        S::Future: 'static,
        B: 'static
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = RoleMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(RoleMiddleware { service, role_id: 1 })
    }
}

pub struct RoleMiddleware<S> {
    service: S,
    role_id: ModelId,
}

impl<S, B> Service for RoleMiddleware<S>
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
        let role_id: Option<ModelId> = if let Some(token) = req.head().extensions().get::<AuthToken>() {
            Some(token.role_id)
        } else {
            None
        };

        if let Some(role_id) = role_id {
            if role_id == self.role_id {
                Either::Left(self.service.call(req))
            } else {
                Either::Right(ok(req.into_response(ErrorMessage::Forbidden.error().into_body())))
            }
        } else {
            Either::Right(ok(req.into_response(ErrorMessage::MiddlewareFailed.error().into_body())))
        }
    }
}

use std::cell::RefCell;
use std::rc::Rc;
use std::task::{Context, Poll};

use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::Error;
use futures::future::{Either, ok, Ready};

use core::error::ErrorMessaging;
use core::ErrorMessage;
use core::types::ModelId;

use crate::models::user::User;

pub struct AdminUser;

impl<S, B> Transform<S> for AdminUser
    where
        S: Service<Request=ServiceRequest, Response=ServiceResponse<B>, Error=Error> + 'static,
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
        ok(RoleMiddleware { service: Rc::new(RefCell::new(service)), role_id: 1 })
    }
}

pub struct RoleMiddleware<S> {
    service: Rc<RefCell<S>>,
    role_id: ModelId,
}

impl<S, B> Service for RoleMiddleware<S>
    where
        S: Service<Request=ServiceRequest, Response=ServiceResponse<B>, Error=Error> + 'static,
        S::Future: 'static,
        B: 'static
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Either<S::Future, Ready<Result<Self::Response, Self::Error>>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&mut self, req: ServiceRequest) -> Self::Future {
        let res = check_role(req.head().extensions().get::<User>(), self.role_id);

        match res {
            Ok(()) => Either::Left(self.service.call(req)),
            Err(e) => Either::Right(ok(req.into_response(e.error().into_body())))
        }
    }
}

fn check_role(user: Option<&User>, role_id: ModelId) -> Result<(), ErrorMessage> {
    if let Some(user) = user {
        if user.role_id == role_id {
            Ok(())
        } else {
            Err(ErrorMessage::Forbidden)
        }
    } else {
        Err(ErrorMessage::UserNotFound)
    }
}

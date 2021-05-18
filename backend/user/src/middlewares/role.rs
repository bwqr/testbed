use std::cell::RefCell;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll};

use actix_web::{Error, error::BlockingError, web};
use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use diesel::prelude::*;
use diesel::result::Error as DieselError;
use futures::future::{Future, ok, Ready};

use core::error::ErrorMessaging;
use core::ErrorMessage;
use core::models::token::AuthToken;
use core::schema::users;
use core::types::{DBPool, ModelId};

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
    type Future = Pin<Box<dyn Future<Output=Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&mut self, req: ServiceRequest) -> Self::Future {
        let mut service = self.service.clone();
        let expected_role_id = self.role_id;

        Box::pin(async move {
            let conn = if let Some(pool) = req.app_data::<web::Data<DBPool>>() {
                pool.get().unwrap()
            } else {
                return Err(ErrorMessage::MiddlewareFailed.error().into());
            };

            let user_id = if let Some(token) = req.head().extensions().get::<AuthToken>() {
                token.user_id
            } else {
                return Err(ErrorMessage::MiddlewareFailed.error().into());
            };

            match web::block(move || users::table.find(user_id).select(users::role_id).first::<ModelId>(&conn))
                .await {
                Ok(role_id) => {
                    if role_id == expected_role_id {
                        service.call(req).await
                    } else {
                        Err(ErrorMessage::Forbidden.error().into())
                    }
                }
                Err(BlockingError::Error(DieselError::NotFound)) => Err(ErrorMessage::UserNotFound.error().into()),
                Err(_) => Err(ErrorMessage::MiddlewareFailed.error().into()),
            }
        })

        // let role_id: Option<ModelId> = if let Some(token) = req.head().extensions().get::<AuthToken>() {
        //     Some(token.role_id)
        // } else {
        //     None
        // };
        //
        // if let Some(role_id) = role_id {
        //     if role_id == self.role_id {
        //         Either::Left(self.service.call(req))
        //     } else {
        //         Either::Right(ok(req.into_response(ErrorMessage::Forbidden.error().into_body())))
        //     }
        // } else {
        //     Either::Right(ok(req.into_response(ErrorMessage::MiddlewareFailed.error().into_body())))
        // }
    }
}

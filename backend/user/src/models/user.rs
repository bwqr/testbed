use actix_web::{error::BlockingError, FromRequest, HttpRequest, HttpResponse, web};
use actix_web::dev::Payload;
use diesel::{Identifiable, Queryable};
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::result::Error;
use diesel::sql_types::VarChar;
use futures::future::LocalBoxFuture;
use futures::FutureExt;
use serde::{Deserialize, Serialize};

use core::db::DieselEnum;
use core::error::ErrorMessaging;
use core::ErrorMessage;
use core::models::AuthToken;
use core::schema::users;
use core::types::{DBPool, ModelId};

#[derive(Queryable, Identifiable, Deserialize, Serialize)]
pub struct User {
    pub id: ModelId,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub status: UserStatus,
    pub role_id: ModelId,
}

impl FromRequest for User {
    type Error = HttpResponse;
    type Future = LocalBoxFuture<'static, Result<Self, Self::Error>>;
    type Config = ();

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let conn = req.app_data::<web::Data<DBPool>>()
            .ok_or_else(|| ErrorMessage::DBError.error())
            .map(|c| c.get().unwrap());

        let user_id = req.head().extensions().get::<AuthToken>()
            .ok_or_else(|| ErrorMessage::UserNotFound.error())
            .map(|a| a.user_id);


        async move {
            let user_id = user_id?;
            let conn = conn?;

            web::block(move || users::table.find(user_id).first::<User>(&conn))
                .await
                .map_err(|e| match e {
                    BlockingError::Error(e) => {
                        match e {
                            Error::NotFound => ErrorMessage::UserNotFound.error(),
                            _ => e.error()
                        }
                    }
                    BlockingError::Canceled => ErrorMessage::BlockingCanceled.error()
                })
        }.boxed_local()
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum UserStatus {
    NotVerified,
    Verified,
    Banned,
}

impl Default for UserStatus {
    fn default() -> Self { UserStatus::NotVerified }
}

impl Queryable<VarChar, Pg> for UserStatus {
    type Row = String;

    fn build(row: Self::Row) -> Self {
        Self::build_from_string(row)
    }
}

use actix_web::{FromRequest, HttpRequest, HttpResponse};
use actix_web::dev::Payload;
use diesel::{Identifiable, Insertable, Queryable};
use diesel::pg::Pg;
use diesel::sql_types::VarChar;
use futures::future::{err, ok, Ready};
use serde::{Deserialize, Serialize};

use core::db::DieselEnum;
use core::error::ErrorMessaging;
use core::ErrorMessage;
use core::schema::users;
use core::types::ModelId;

#[derive(Queryable, Identifiable, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: ModelId,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub password: String,
    pub status: UserStatus,
    pub role_id: ModelId,
}

impl User {
    pub fn full_name(&self) -> String {
        self.first_name.clone() + " " + self.last_name.as_str()
    }
}

impl FromRequest for User {
    type Error = HttpResponse;
    type Future = Ready<Result<Self, Self::Error>>;
    type Config = ();

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        if let Some(user) = req.head().extensions().get::<User>() {
            ok((*user).clone())
        } else {
            err(ErrorMessage::UserNotFound.error())
        }
    }
}

#[derive(Queryable, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SlimUser {
    pub id: ModelId,
    pub first_name: String,
    pub last_name: String,
    pub role_id: ModelId,
}

impl From<User> for SlimUser {
    fn from(user: User) -> Self {
        SlimUser {
            id: user.id,
            first_name: user.first_name,
            last_name: user.last_name,
            role_id: user.role_id,
        }
    }
}

pub const SLIM_USER_COLUMNS: (users::id, users::first_name, users::last_name, users::role_id) = (
    users::id,
    users::first_name,
    users::last_name,
    users::role_id,
);

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
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

#[derive(Insertable)]
#[table_name = "users"]
pub struct UserInsert {
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub password: String,
    pub role_id: ModelId,
}
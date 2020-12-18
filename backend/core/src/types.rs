use actix_web::HttpResponse;
use diesel::PgConnection;
use diesel::r2d2::{self, ConnectionManager};

use crate::error::ErrorMessaging;

pub type ModelId = i32;
pub type SessionId = i64;

pub type DBPool = r2d2::Pool<ConnectionManager<PgConnection>>;

pub type Result<T> = std::result::Result<T, Box<dyn ErrorMessaging>>;

pub type DefaultResponse = Result<HttpResponse>;

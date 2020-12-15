use diesel::PgConnection;
use diesel::r2d2::{self, ConnectionManager};

pub type ModelId = i32;
pub type SessionId = i64;

pub type DBPool = r2d2::Pool<ConnectionManager<PgConnection>>;

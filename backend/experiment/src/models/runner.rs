use chrono::NaiveDateTime;
use diesel::Queryable;
use serde::{Deserialize, Serialize};

use core::types::ModelId;

#[derive(Queryable)]
pub struct Runner {
    pub id: ModelId,
    pub access_key: String,
    pub created_at: NaiveDateTime,
}

#[derive(Deserialize, Serialize)]
pub struct RunnerToken {
    pub access_key: String,
    pub exp: i64,
}
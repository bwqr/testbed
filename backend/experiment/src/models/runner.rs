use chrono::NaiveDateTime;
use diesel::Queryable;
use serde::{Deserialize, Serialize};

use core::schema::runners;
use core::types::ModelId;

#[derive(Queryable)]
pub struct Runner {
    pub id: ModelId,
    pub name: String,
    pub access_key: String,
    pub created_at: NaiveDateTime,
}

#[derive(Queryable, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SlimRunner {
    pub id: ModelId,
    pub name: String,
    pub created_at: NaiveDateTime,
}

pub const SLIM_RUNNER_COLUMNS: (runners::id, runners::name, runners::created_at) = (
    runners::id,
    runners::name,
    runners::created_at
);

#[derive(Deserialize, Serialize)]
pub struct RunnerToken {
    pub access_key: String,
    pub exp: i64,
}
use chrono::NaiveDateTime;
use diesel::Queryable;
use serde::{Deserialize, Serialize};

use core::schema::controllers;
use core::types::ModelId;

#[derive(Queryable)]
pub struct Controller {
    pub id: ModelId,
    pub name: String,
    pub access_key: String,
    pub created_at: NaiveDateTime,
}

#[derive(Queryable, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SlimController {
    pub id: ModelId,
    pub name: String,
    pub created_at: NaiveDateTime,
}

pub const SLIM_CONTROLLER_COLUMNS: (controllers::id, controllers::name, controllers::created_at) = (
    controllers::id,
    controllers::name,
    controllers::created_at
);

#[derive(Deserialize, Serialize)]
pub struct ControllerToken {
    pub access_key: String,
    pub exp: i64,
}

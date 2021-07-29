use chrono::NaiveDateTime;
use diesel::{Identifiable, Queryable};
use serde::Serialize;

use core::schema::slots;
use core::types::ModelId;

#[derive(Identifiable, Queryable, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Slot {
    pub id: ModelId,
    pub user_id: ModelId,
    pub controller_id: ModelId,
    pub start_at: NaiveDateTime,
    pub end_at: NaiveDateTime,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

use chrono::NaiveDateTime;
use diesel::{Identifiable, Queryable};
use serde::Serialize;

use core::schema::experiments;
use core::types::ModelId;

#[derive(Identifiable, Queryable, Serialize)]
pub struct Experiment {
    pub id: ModelId,
    pub user_id: ModelId,
    pub name: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

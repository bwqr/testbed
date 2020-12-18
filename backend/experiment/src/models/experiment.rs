use chrono::NaiveDateTime;
use diesel::{Identifiable, Queryable};
use diesel::pg::Pg;
use diesel::sql_types::VarChar;
use serde::{Deserialize, Serialize};

use core::db::DieselEnum;
use core::schema::experiments;
use core::types::ModelId;

#[derive(Identifiable, Queryable, Serialize)]
pub struct Experiment {
    pub id: ModelId,
    pub user_id: ModelId,
    pub name: String,
    pub status: ExperimentStatus,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub enum ExperimentStatus {
    Pending,
    Running,
    NoAction,
    Failed,
}

impl Default for ExperimentStatus {
    fn default() -> Self {
        ExperimentStatus::Pending
    }
}

impl Queryable<VarChar, Pg> for ExperimentStatus {
    type Row = String;

    fn build(row: Self::Row) -> Self {
        Self::build_from_string(row)
    }
}

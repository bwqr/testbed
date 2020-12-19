use chrono::NaiveDateTime;
use diesel::{Identifiable, Queryable};
use diesel::pg::Pg;
use diesel::sql_types::VarChar;
use serde::{Deserialize, Serialize};

use core::db::DieselEnum;
use core::schema::runs;
use core::types::ModelId;

#[derive(Identifiable, Queryable, Serialize)]
pub struct Run {
    pub id: ModelId,
    pub experiment_id: ModelId,
    pub status: RunStatus,
    pub created_at: NaiveDateTime,
}


#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub enum RunStatus {
    Pending,
    Running,
    Successful,
    Failed,
}

impl Default for RunStatus {
    fn default() -> Self {
        RunStatus::Pending
    }
}

impl Queryable<VarChar, Pg> for RunStatus {
    type Row = String;

    fn build(row: Self::Row) -> Self {
        Self::build_from_string(row)
    }
}

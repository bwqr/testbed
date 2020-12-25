use chrono::NaiveDateTime;
use diesel::{Identifiable, Queryable};
use diesel::pg::Pg;
use diesel::sql_types::VarChar;
use serde::{Deserialize, Serialize};

use core::db::DieselEnum;
use core::schema::jobs;
use core::types::ModelId;

#[derive(Identifiable, Queryable, Serialize)]
pub struct Job {
    pub id: ModelId,
    pub experiment_id: ModelId,
    pub runner_id: ModelId,
    pub code: String,
    pub status: JobStatus,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}


#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub enum JobStatus {
    Pending,
    Running,
    Successful,
    Failed,
}

impl Default for JobStatus {
    fn default() -> Self {
        JobStatus::Pending
    }
}

impl Queryable<VarChar, Pg> for JobStatus {
    type Row = String;

    fn build(row: Self::Row) -> Self {
        Self::build_from_string(row)
    }
}

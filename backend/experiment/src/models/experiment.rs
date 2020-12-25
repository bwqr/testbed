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
    pub code: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Queryable, Serialize)]
pub struct SlimExperiment {
    pub id: ModelId,
    pub user_id: ModelId,
    pub name: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

pub const SLIM_EXPERIMENT_COLUMNS: (experiments::id, experiments::user_id, experiments::name, experiments::created_at, experiments::updated_at) = (
    experiments::id,
    experiments::user_id,
    experiments::name,
    experiments::created_at,
    experiments::updated_at
);


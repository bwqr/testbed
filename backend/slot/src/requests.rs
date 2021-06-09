use chrono::NaiveDateTime;
use serde::Deserialize;
use core::types::ModelId;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReservedQueryRequest {
    pub start_at: NaiveDateTime,
    pub runner_id: ModelId,
    pub count: u32,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SlotReserveRequest {
    pub start_at: NaiveDateTime,
    pub runner_id: ModelId,
}
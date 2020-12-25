use serde::Deserialize;

use core::sanitized::Sanitize;
use derive::Sanitize;

#[derive(Deserialize, Sanitize)]
pub struct ExperimentNameRequest {
    pub name: String,
}

#[derive(Deserialize, Sanitize)]
pub struct ExperimentCodeRequest {
    pub code: String,
}
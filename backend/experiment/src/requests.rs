use serde::Deserialize;

use core::sanitized::Sanitize;
use derive::Sanitize;

#[derive(Deserialize, Sanitize)]
pub struct ExperimentRequest {
    pub name: String,
}
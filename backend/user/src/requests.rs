use serde::Deserialize;

use core::sanitized::Sanitize;
use core::schema::users;
use derive::Sanitize;

#[derive(AsChangeset, Sanitize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[table_name = "users"]
pub struct UpdateProfileRequest {
    pub first_name: String,
    pub last_name: String,
}

#[derive(Deserialize)]
pub struct UpdatePasswordRequest {
    pub password: String
}
use std::fmt::Debug;

use serde::Deserialize;
use validator::Validate;

use core::models::role::Roles;
use core::sanitized::Sanitize;
use core::types::ModelId;
use core::utils::Hash;
use derive::Sanitize;
use user::models::user::UserInsert;

#[derive(Debug, Deserialize, Sanitize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize, Sanitize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct SignUpRequest {
    #[validate(length(max = 122))]
    pub first_name: String,
    #[validate(length(max = 122))]
    pub last_name: String,
    #[validate(length(max = 255), email)]
    pub email: String,
    #[validate(length(min = 8, max = 128))]
    pub password: String,
}

impl SignUpRequest {
    pub fn as_insert_model(self, hash: &Hash) -> UserInsert {
        UserInsert {
            first_name: self.first_name,
            last_name: self.last_name,
            email: self.email,
            password: hash.sign512(&self.password),
            role_id: Roles::User as ModelId,
        }
    }
}

#[derive(Deserialize, Sanitize)]
pub struct ForgotPasswordRequest {
    pub email: String
}

#[derive(Deserialize, Sanitize, Validate)]
pub struct ResetPasswordRequest {
    pub token: String,
    #[validate(length(min = 8, max = 128))]
    pub password: String,
}

#[derive(Deserialize, Sanitize)]
pub struct VerifyAccountRequest {
    pub token: String
}

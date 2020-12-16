use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct TokenResponse {
    pub token: String
}

#[derive(Serialize)]
pub struct SuccessResponse {
    pub message: String
}

impl Default for SuccessResponse {
    fn default() -> Self {
        SuccessResponse {
            message: String::from("success")
        }
    }
}

use serde::{Deserialize, Serialize};

use crate::types::ModelId;

pub mod experiment;
pub mod token;


#[derive(Clone, Serialize, Deserialize)]
pub struct AuthToken {
    // issued at
    pub iat: i64,
    // expire time
    pub exp: i64,
    // user id
    pub user_id: ModelId,
    // role id
    pub role_id: ModelId,
}
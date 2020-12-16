use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::types::ModelId;

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

impl AuthToken {
    pub fn new(user_id: ModelId, role_id: ModelId, timeout: i64) -> Self {
        let now = Utc::now().timestamp();

        AuthToken {
            iat: now,
            exp: now + timeout,
            user_id,
            role_id,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct IdentityToken {
    pub user_id: ModelId,
    pub iat: i64,
    pub exp: i64,
    pub kind: IdentityTokenKind,
}

#[derive(Deserialize, Serialize, PartialEq)]
pub enum IdentityTokenKind {
    ForgotPassword,
    VerifyAccount,
}

impl IdentityToken {
    pub fn new(user_id: ModelId, kind: IdentityTokenKind, timeout: i64) -> Self {
        let now = Utc::now().timestamp();

        IdentityToken {
            user_id,
            iat: now,
            exp: now + timeout,
            kind,
        }
    }
}
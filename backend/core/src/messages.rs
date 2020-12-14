use serde::{Deserialize, Serialize};

use crate::types::ModelId;

#[derive(Deserialize, Serialize)]
pub enum SocketMessageKind {
    RegisterBackend,
}

#[derive(Deserialize, Serialize)]
pub struct BaseMessage {
    pub kind: SocketMessageKind
}

#[derive(Deserialize, Serialize)]
pub struct SocketMessage<T> {
    pub kind: SocketMessageKind,
    pub data: T,
}

pub mod bidirect {
    use super::{Deserialize, Serialize};
    use super::ModelId;

    #[derive(Deserialize, Serialize)]
    pub struct RegisterBackend {
        pub id: ModelId,
        pub access_key: String,
    }
}
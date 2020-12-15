use serde::{Deserialize, Serialize};

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

pub mod server {
    use super::{Deserialize, Serialize};

    #[derive(Deserialize, Serialize)]
    pub struct RegisterBackend {
        pub access_key: String,
    }
}
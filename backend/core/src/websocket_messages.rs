use serde::{Deserialize, Serialize};

use crate::types::ModelId;

pub mod server {
    use super::{Deserialize, ModelId, Serialize};

    #[derive(Deserialize, Serialize)]
    pub enum SocketMessageKind {
        RunResult
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

    #[derive(Deserialize, Serialize)]
    pub struct RunResult {
        pub run_id: ModelId,
        pub successful: bool,
    }
}

pub mod client {
    use super::{Deserialize, ModelId, Serialize};

    #[derive(Deserialize, Serialize)]
    pub enum SocketMessageKind {
        RunExperiment
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

    #[derive(Deserialize, Serialize)]
    pub struct RunExperiment {
        pub run_id: ModelId
    }
}
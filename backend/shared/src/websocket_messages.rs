use serde::{Deserialize, Serialize};

type ModelId = i32;

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
        pub job_id: ModelId,
        pub output: String,
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
        pub job_id: ModelId,
        pub code: String,
    }
}
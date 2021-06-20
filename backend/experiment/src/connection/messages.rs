use actix::{Addr, Message};

use core::types::ModelId;
use shared::RunnerState;

use crate::connection::session::Session;

#[derive(Message)]
#[rtype(result = "()")]
pub struct RunMessage {
    pub job_id: ModelId,
    pub code: String,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct JoinServerMessage {
    pub state: RunnerState,
    pub runner_id: ModelId,
    pub addr: Addr<Session>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct DisconnectServerMessage {
    pub runner_id: ModelId,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct RunResultMessage {
    pub runner_id: ModelId,
    pub job_id: ModelId,
    pub successful: bool,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct UpdateRunnerValue {
    pub runner_id: ModelId,
    pub values: Vec<u8>,
}

pub struct ReceiverValues {
    pub runner_id: ModelId,
}

impl Message for ReceiverValues {
    type Result = Result<Option<Vec<u8>>, ()>;
}
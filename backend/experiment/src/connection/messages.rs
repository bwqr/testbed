use actix::{Addr, Message};

use core::types::ModelId;
use shared::ControllerState;

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
    pub state: ControllerState,
    pub controller_id: ModelId,
    pub addr: Addr<Session>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct DisconnectServerMessage {
    pub controller_id: ModelId,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct RunResultMessage {
    pub controller_id: ModelId,
    pub job_id: ModelId,
    pub successful: bool,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct UpdateControllerValue {
    pub controller_id: ModelId,
    pub values: Vec<u8>,
}

pub struct ReceiverValues {
    pub controller_id: ModelId,
}

impl Message for ReceiverValues {
    type Result = Result<Option<Vec<u8>>, ()>;
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct AbortRunningJob {
    pub job_id: ModelId,
    pub controller_id: ModelId,
}


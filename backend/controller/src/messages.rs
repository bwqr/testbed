use actix::{Message, Recipient};

use crate::ModelId;

#[derive(Message)]
#[rtype(result = "()")]
pub struct RunMessage {
    pub job_id: ModelId,
    pub code: String,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct RunResultMessage {
    pub job_id: ModelId,
    pub output: String,
    pub successful: bool,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct ControllerReceiversValueMessage {
    pub values: Vec<u32>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct UpdateExecutorMessage {
    pub executor: Recipient<RunMessage>,
}

#[derive(Message)]
#[rtype(result = "bool")]
pub struct IsJobAborted;

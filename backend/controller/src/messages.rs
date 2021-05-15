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
pub struct RunnerReceiversValueMessage {
    pub values: Vec<u8>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct UpdateExecutorMessage {
    pub executor: Recipient<RunMessage>,
}

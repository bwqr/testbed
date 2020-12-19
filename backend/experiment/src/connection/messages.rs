use actix::{Addr, Message};

use core::types::ModelId;

use crate::connection::session::Session;

#[derive(Message)]
#[rtype(result = "()")]
pub struct RunMessage {
    pub run_id: ModelId
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct JoinServerMessage {
    pub runner_id: ModelId,
    pub addr: Addr<Session>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct RunResultMessage {
    pub runner_id: ModelId,
    pub run_id: ModelId,
    pub successful: bool,
}
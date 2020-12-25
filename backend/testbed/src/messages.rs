use actix::{Addr, Message, Recipient};

use crate::connection::Connection;
use crate::ModelId;

#[derive(Message)]
#[rtype(result = "()")]
pub struct RunMessage {
    pub job_id: ModelId,
    pub code: String,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct UpdateExecutorMessage {
    pub executor: Recipient<RunMessage>
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct UpdateConnectionMessage {
    pub connection: Addr<Connection>
}
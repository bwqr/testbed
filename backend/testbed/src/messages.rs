use actix::{Addr, Message};

use crate::connection::Connection;
use crate::executor::Executor;

#[derive(Message)]
#[rtype(result = "()")]
pub struct UpdateExecutorMessage {
    pub executor: Addr<Executor>
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct UpdateConnectionMessage {
    pub connection: Addr<Connection>
}
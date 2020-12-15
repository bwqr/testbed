use actix::prelude::*;

use crate::connection::Connection;
use crate::messages::UpdateConnectionMessage;

pub struct Executor {
    connection: Option<Addr<Connection>>
}

impl Executor {
    pub fn new() -> Self {
        Executor {
            connection: None
        }
    }
}

impl Actor for Executor {
    type Context = Context<Self>;

    fn started(&mut self, _: &mut Self::Context) {}

    fn stopped(&mut self, _: &mut Self::Context) {}
}

impl Handler<UpdateConnectionMessage> for Executor {
    type Result = ();

    fn handle(&mut self, msg: UpdateConnectionMessage, _: &mut Self::Context) {
        self.connection = Some(msg.connection);
    }
}
use std::collections::HashMap;

use actix::prelude::*;
use log::info;
use serde::Serialize;

use core::types::ModelId;

use crate::Notification;
use crate::ws::messages::{internal, outgoing};
use crate::ws::session::{Session, SessionId};

pub struct NotificationServer {
    // user_id -> (session_id -> session)
    users: HashMap<ModelId, HashMap<SessionId, Addr<Session>>>,
}

impl NotificationServer {
    pub fn new() -> Self {
        NotificationServer {
            users: HashMap::new()
        }
    }
}

impl Actor for NotificationServer {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Context<Self>) {
        info!("NotificationServer is started!");
    }

    fn stopped(&mut self, _ctx: &mut Context<Self>) {
        info!("NotificationServer is stopped");
    }
}

impl Handler<internal::server::ConnectServer> for NotificationServer {
    type Result = ();

    fn handle(&mut self, msg: internal::server::ConnectServer, _: &mut Self::Context) -> Self::Result {
        if let Some(sessions) = self.users.get_mut(&msg.user_id) {
            sessions.insert(msg.session_id, msg.addr);
        } else {
            let mut sessions: HashMap<SessionId, Addr<Session>> = HashMap::new();
            sessions.insert(msg.session_id, msg.addr);

            self.users.insert(msg.user_id, sessions);
        }
    }
}

impl Handler<internal::server::DisconnectServer> for NotificationServer {
    type Result = ();

    fn handle(&mut self, msg: internal::server::DisconnectServer, _: &mut Self::Context) -> Self::Result {
        if let Some(mut sessions) = self.users.remove(&msg.user_id) {
            sessions.remove(&msg.session_id);
        }
    }
}

impl<T> Handler<Notification<T>> for NotificationServer where T: Serialize + Send + Clone + 'static {
    type Result = ();

    fn handle(&mut self, msg: Notification<T>, _: &mut Self::Context) -> Self::Result {
        if let Some(sessions) = self.users.get(&msg.user_id) {
            for (_, addr) in sessions {
                addr.do_send(outgoing::NotifyUser {
                    user_id: msg.user_id,
                    message: msg.message.clone(),
                })
            }
        }
    }
}
use std::collections::HashSet;

use actix::{Actor, Addr, AsyncContext, Context, Handler};
use log::info;
use rand::prelude::*;

use crate::notification::NotificationServer;

use super::messages::internal;
use super::session::{Session, SessionId};
use crate::CreateSession;

/// Manages session creation. In order to ensure the uniqueness of session ids, one manager should
/// handle the creation of sessions.
pub struct SessionManager {
    sessions: HashSet<SessionId>,
    rng: ThreadRng,
    pub notification: Addr<NotificationServer>,
}

impl SessionManager {
    pub fn new(notification: Addr<NotificationServer>) -> Self {
        SessionManager {
            sessions: HashSet::new(),
            rng: rand::thread_rng(),
            notification,
        }
    }
}

impl Actor for SessionManager {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Context<Self>) {
        info!("SessionManager is started!");
    }

    fn stopped(&mut self, _ctx: &mut Context<Self>) {
        info!("SessionManager is stopped");
    }
}

impl Handler<CreateSession> for SessionManager {
    type Result = Session;

    fn handle(&mut self, msg: CreateSession, ctx: &mut Self::Context) -> Self::Result {
        let mut session_id: SessionId = self.rng.gen();

        // Check if generated session id already exists
        loop {
            if self.sessions.contains(&session_id) {
                session_id = self.rng.gen();
            } else {
                break;
            }
        }

        self.sessions.insert(session_id);

        Session::new(ctx.address(), self.notification.clone(), session_id, msg.user_id)
    }
}

impl Handler<internal::server::DisconnectServer> for SessionManager {
    type Result = ();

    fn handle(&mut self, msg: internal::server::DisconnectServer, _: &mut Self::Context) {
        self.sessions.remove(&msg.session_id);
    }
}

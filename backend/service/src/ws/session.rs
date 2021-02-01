use std::time::{Duration, Instant};

use actix::dev::{MessageResponse, ResponseChannel};
use actix::prelude::*;
use actix_web_actors::ws::{Message as WebActorMessage, ProtocolError, WebsocketContext};

use core::types::ModelId;

use crate::notification::NotificationServer;

use super::messages::*;
use super::session_manager::SessionManager;

const CONNECTION_TIMEOUT: Duration = Duration::from_secs(120);
const HB_CHECK_INTERVAL: Duration = Duration::from_secs(60);

pub struct Session {
    id: SessionId,
    user_id: ModelId,
    hb: Instant,
    manager: Addr<SessionManager>,
    notification: Addr<NotificationServer>,
}

impl Session {
    pub fn new(manager: Addr<SessionManager>, notification: Addr<NotificationServer>, session_id: SessionId, user_id: ModelId) -> Self {
        Session {
            id: session_id,
            hb: Instant::now(),
            user_id,
            manager,
            notification,
        }
    }

    fn handle_msg(&mut self, msg: WebActorMessage, ctx: &mut <Self as Actor>::Context) -> Result<(), ErrorKind> {
        match msg {
            WebActorMessage::Ping(_) => {
                self.hb = Instant::now()
            }
            WebActorMessage::Pong(_) => {
                self.hb = Instant::now()
            }
            WebActorMessage::Text(text) => {
                let text = text.as_str();

                let base = serde_json::from_str::<'_, incoming::BaseMessage>(text)
                    .map_err(|_| ErrorKind::InvalidMessage)?;

                match base.kind {
                    IncomingMessageKind::KeepAlive => {
                        self.hb = Instant::now();
                    }
                }
            }
            WebActorMessage::Close(_) => ctx.stop(),
            _ => {}
        }

        Ok(())
    }

    fn check_hb(&mut self) -> bool {
        Instant::now().duration_since(self.hb) < CONNECTION_TIMEOUT
    }
}

impl<A, M> MessageResponse<A, M> for Session
    where
        A: Actor,
        M: Message<Result=Session>, {
    fn handle<R: ResponseChannel<M>>(self, _: &mut A::Context, tx: Option<R>) {
        if let Some(tx) = tx {
            tx.send(self);
        }
    }
}

pub type SessionId = i64;

impl Actor for Session {
    type Context = WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.notification.send(internal::server::ConnectServer {
            session_id: self.id,
            user_id: self.user_id,
            addr: ctx.address(),
        })
            .into_actor(self)
            .then(|_, _, _| fut::ready(()))
            .wait(ctx);

        ctx.run_interval(HB_CHECK_INTERVAL, |act, ctx| {
            if !act.check_hb() {
                ctx.stop();
            }
        });
    }

    fn stopping(&mut self, ctx: &mut Self::Context) -> Running {
        // If we are stopping due to timeout, send a timeout message
        if !self.check_hb() {
            ctx.text(outgoing::Timeout.value());
        }

        self.notification.do_send(internal::server::DisconnectServer {
            session_id: self.id,
            user_id: self.user_id,
        });

        self.manager.do_send(internal::server::DisconnectServer {
            session_id: self.id,
            user_id: self.user_id,
        });

        Running::Stop
    }
}

impl StreamHandler<Result<WebActorMessage, ProtocolError>> for Session {
    fn handle(&mut self, msg: Result<WebActorMessage, ProtocolError>, ctx: &mut Self::Context) {
        let msg = match msg {
            Ok(m) => m,
            Err(_) => {
                ctx.stop();
                return;
            }
        };

        if let Err(e) = self.handle_msg(msg, ctx) {
            ctx.text(e.value());
        }
    }
}

impl<T> Handler<T> for Session where T: WebSocketMessaging {
    type Result = ();

    fn handle(&mut self, msg: T, ctx: &mut Self::Context) -> Self::Result {
        ctx.text(msg.value())
    }
}
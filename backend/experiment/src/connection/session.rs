use actix::{Actor, ActorContext, Addr, StreamHandler};
use actix_web_actors::ws::{Message, ProtocolError, WebsocketContext};
use log::error;

use core::websocket_messages::{BaseMessage, SocketMessage, SocketMessageKind};
use core::websocket_messages::server::RegisterBackend;
use core::SocketErrorKind;

use crate::connection::server::ExperimentServer;

pub struct Session {
    // experiment_server: Addr<ExperimentServer>
}

impl Session {
    pub fn new(experiment_server: Addr<ExperimentServer>) -> Self {
        Session {
            // experiment_server
        }
    }

    fn handle_msg(&self, msg: Message, ctx: &mut WebsocketContext<Self>) -> Result<(), SocketErrorKind> {
        match msg {
            Message::Ping(_) => {
                // self.hb = Instant::now()
            }
            Message::Pong(_) => {
                // self.hb = Instant::now()
            }
            Message::Text(text) => {
                let text = text.as_str();

                let base = serde_json::from_str::<'_, BaseMessage>(text)
                    .map_err(|_| SocketErrorKind::InvalidMessage)?;

                match base.kind {
                    SocketMessageKind::RegisterBackend => {
                        let register_backend = serde_json::from_str::<'_, SocketMessage<RegisterBackend>>(text)
                            .map_err(|_| SocketErrorKind::InvalidMessage)?;
                        println!("access_key: {}", register_backend.data.access_key);

                        ctx.text("How are you");
                    }
                }
            }
            Message::Close(_) => ctx.stop(),
            _ => {}
        }

        Ok(())
    }
}

impl Actor for Session {
    type Context = WebsocketContext<Self>;

    fn started(&mut self, _: &mut Self::Context) {}

    fn stopped(&mut self, _: &mut Self::Context) {}
}

impl StreamHandler<Result<Message, ProtocolError>> for Session {
    fn handle(&mut self, msg: Result<Message, ProtocolError>, ctx: &mut Self::Context) {
        let msg = match msg {
            Ok(m) => m,
            Err(_) => {
                ctx.stop();
                return;
            }
        };

        if let Err(e) = self.handle_msg(msg, ctx) {
            error!("{:?}", e);
            // ctx.text(e.value());
        }
    }
}
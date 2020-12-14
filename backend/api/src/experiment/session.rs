use actix::{Actor, ActorContext, Running, StreamHandler};
use actix_web_actors::ws::{Message, ProtocolError, WebsocketContext};

use core::messages::{BaseMessage, SocketMessage, SocketMessageKind};
use core::messages::bidirect::RegisterBackend;
use core::SocketErrorKind;

pub struct Session {}

impl Session {
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
                        println!("id: {}, access_key: {}", register_backend.data.id, register_backend.data.access_key)
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

    fn started(&mut self, ctx: &mut Self::Context) {}

    fn stopped(&mut self, ctx: &mut Self::Context) {}
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
            // ctx.text(e.value());
        }
    }
}
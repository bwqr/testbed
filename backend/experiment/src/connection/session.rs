use actix::clock::Duration;
use actix::prelude::*;
use actix_web_actors::ws::{Message, ProtocolError, WebsocketContext};
use log::{error, info};

use core::SocketErrorKind;
use core::types::ModelId;
use core::websocket_messages::{BaseMessage, SocketMessage, SocketMessageKind};
use core::websocket_messages::server::RegisterBackend;

use crate::connection::messages::{JoinServerMessage, RunMessage, RunResultMessage};
use crate::connection::server::ExperimentServer;

pub struct Session {
    experiment_server: Addr<ExperimentServer>,
    runner_id: ModelId,
}

impl Session {
    pub fn new(experiment_server: Addr<ExperimentServer>, runner_id: ModelId) -> Self {
        Session {
            experiment_server,
            runner_id,
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
                        println!("access_key: {}", register_backend.data.access_token);

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

    fn started(&mut self, ctx: &mut Self::Context) {
        let runner_id = self.runner_id;
        let session_addr = ctx.address();
        let exp_addr = self.experiment_server.clone();

        async move {
            exp_addr.send(JoinServerMessage {
                runner_id,
                addr: session_addr,
            })
                .await;
        }
            .into_actor(self)
            .spawn(ctx);
    }

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
        }
    }
}

impl Handler<RunMessage> for Session {
    type Result = ();

    fn handle(&mut self, msg: RunMessage, ctx: &mut Self::Context) {
        info!("got run message {}", msg.run_id);

        ctx.run_later(Duration::from_secs(60), |act, ctx| {
            let runner_id = act.runner_id;
            let exp_addr = act.experiment_server.clone();

            async move {
                exp_addr.send(RunResultMessage {
                    run_id: msg.run_id,
                    runner_id,
                    successful: false,
                })
                    .await;
            }
                .into_actor(act)
                .spawn(ctx);
        });
    }
}
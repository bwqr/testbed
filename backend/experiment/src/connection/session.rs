use actix::prelude::*;
use actix_web_actors::ws::{Message, ProtocolError, WebsocketContext};
use log::{error, info};

use core::types::ModelId;
use shared::SocketErrorKind;
use shared::websocket_messages::{client, server};

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
            Message::Ping(_) | Message::Pong(_) => {
                // self.hb = Instant::now()
            }
            Message::Text(text) => {
                let text = text.as_str();

                let base = serde_json::from_str::<'_, server::BaseMessage>(text)
                    .map_err(|_| SocketErrorKind::InvalidMessage)?;

                match base.kind {
                    server::SocketMessageKind::RunResult => {
                        let run_result = serde_json::from_str::<'_, server::SocketMessage<server::RunResult>>(text)
                            .map_err(|_| SocketErrorKind::InvalidMessage)?;

                        info!("received run result from runner, successful {}", run_result.data.successful);

                        let exp_addr = self.experiment_server.clone();

                        let msg = RunResultMessage {
                            job_id: run_result.data.job_id,
                            runner_id: self.runner_id,
                            output: run_result.data.output,
                            successful: run_result.data.successful,
                        };

                        async move {
                            if let Err(e) = exp_addr.send(msg)
                                .await {
                                error!("Sending message to experiment server is failed: {:?}", e);
                            }
                        }
                            .into_actor(self)
                            .spawn(ctx);
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
        let exp_addr = self.experiment_server.clone();

        let msg = JoinServerMessage {
            runner_id: self.runner_id,
            addr: ctx.address(),
        };

        async move {
            if let Err(e) = exp_addr.send(msg)
                .await {
                error!("joining into server is failed: {:?}", e);
            }
        }
            .into_actor(self)
            .spawn(ctx);
    }

    fn stopped(&mut self, _: &mut Self::Context) {}
}

impl StreamHandler<Result<Message, ProtocolError>> for Session {
    fn handle(&mut self, msg: Result<Message, ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(msg) => {
                if let Err(e) = self.handle_msg(msg, ctx) {
                    error!("{:?}", e);
                }
            }
            Err(_) => ctx.stop()
        };
    }
}

impl Handler<RunMessage> for Session {
    type Result = ();

    fn handle(&mut self, msg: RunMessage, ctx: &mut Self::Context) {
        info!("got run message {}", msg.job_id);

        // TODO we can send directly message to client, instead of copying msg into RunExperiment
        ctx.text(serde_json::to_string(&client::SocketMessage {
            kind: client::SocketMessageKind::RunExperiment,
            data: client::RunExperiment { job_id: msg.job_id, code: msg.code },
        }).unwrap());
    }
}
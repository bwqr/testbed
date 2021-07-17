use actix::prelude::*;
use actix_web_actors::ws::{Message, ProtocolError, WebsocketContext};
use log::{error, info};

use core::types::ModelId;
use shared::RunnerState;
use shared::SocketErrorKind;
use shared::websocket_messages::{client, server};

use crate::connection::messages::{DisconnectServerMessage, JoinServerMessage, RunMessage, RunResultMessage, UpdateRunnerValue, AbortRunningJob};
use crate::connection::server::ExperimentServer;

pub struct Session {
    experiment_server: Addr<ExperimentServer>,
    // this is used for joining into Experiment Server, after that point, it does not reflect runner state
    initial_runner_state: RunnerState,
    runner_id: ModelId,
}

impl Session {
    pub fn new(experiment_server: Addr<ExperimentServer>, runner_id: ModelId, initial_runner_state: RunnerState) -> Self {
        Session {
            experiment_server,
            initial_runner_state,
            runner_id,
        }
    }

    fn handle_msg(&self, msg: Message, ctx: &mut WebsocketContext<Self>) -> Result<(), SocketErrorKind> {
        match msg {
            Message::Ping(msg) => ctx.pong(&msg),
            Message::Pong(_) => {}
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
                    server::SocketMessageKind::ReceiverStatus => {
                        let receiver_value = serde_json::from_str::<'_, server::SocketMessage<server::RunnerReceiverValue>>(text)
                            .map_err(|_| SocketErrorKind::InvalidMessage)?;

                        let exp_addr = self.experiment_server.clone();

                        let msg = UpdateRunnerValue {
                            runner_id: self.runner_id,
                            values: receiver_value.data.values,
                        };

                        async move {
                            if let Err(e) = exp_addr.send(msg)
                                .await {
                                error!("Sending receiver status to experiment server is failed: {:?}", e);
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
            state: self.initial_runner_state.clone(),
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

    fn stopped(&mut self, _: &mut Self::Context) {
        self.experiment_server.do_send(DisconnectServerMessage {
            runner_id: self.runner_id
        });
    }
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

        ctx.text(serde_json::to_string(&client::SocketMessage {
            kind: client::SocketMessageKind::RunExperiment,
            data: client::RunExperiment { job_id: msg.job_id, code: msg.code },
        }).unwrap());
    }
}

impl Handler<AbortRunningJob> for Session {
    type Result = ();

    fn handle(&mut self, msg: AbortRunningJob, ctx: &mut Self::Context) {
        ctx.text(serde_json::to_string(&client::SocketMessage {
            kind: client::SocketMessageKind::AbortRunningJob,
            data: client::AbortRunningJob {job_id: msg.job_id}
        }).unwrap());
    }
}

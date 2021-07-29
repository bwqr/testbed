use std::cmp::min;

use actix::clock::Duration;
use actix::io::SinkWrite;
use actix::prelude::*;
use actix::{Actor, Context, StreamHandler, WrapFuture};
use actix_codec::Framed;
use awc::error::{WsClientError, WsProtocolError};
use awc::ws::{Codec, Frame, Message};
use awc::{BoxedSocket, Client};
use futures::stream::{SplitSink, StreamExt};
use log::{error, info};

use shared::websocket_messages::{client, server};
use shared::SocketErrorKind;
use shared::{JoinServerRequest, ControllerState};

use crate::messages::{
    IsJobAborted, RunMessage, RunResultMessage, ControllerReceiversValueMessage, UpdateExecutorMessage,
};

type Write = SinkWrite<Message, SplitSink<Framed<BoxedSocket, Codec>, Message>>;

const MAX_TIMING: usize = 5;

const TIMINGS: [u8; MAX_TIMING] = [
    0, 2, 4, 6, 8,
];

pub struct Connection {
    server_url: String,
    access_token: String,
    sink: Option<Write>,
    // this is the delay until we retry connecting to the server
    current_timing_index: usize,
    executor: Option<Recipient<RunMessage>>,
    pending_messages: Vec<RunResultMessage>,
    controller_state: ControllerState,
    is_job_aborted: bool,
}

impl Connection {
    pub fn new(server_url: String, access_token: String) -> Self {
        Connection {
            server_url,
            access_token,
            sink: None,
            current_timing_index: 0,
            executor: None,
            pending_messages: Vec::new(),
            controller_state: ControllerState::Idle,
            is_job_aborted: false,
        }
    }

    fn handle_frame(
        &mut self,
        frame: Frame,
        ctx: &mut <Self as Actor>::Context,
    ) -> Result<(), SocketErrorKind> {
        match frame {
            Frame::Ping(_) | Frame::Pong(_) => {}
            Frame::Text(bytes) => {
                let text = String::from_utf8(bytes.to_vec())
                    .map_err(|_| SocketErrorKind::InvalidMessage)?;

                let text = text.as_str();

                let base = serde_json::from_str::<'_, client::BaseMessage>(text)
                    .map_err(|_| SocketErrorKind::InvalidMessage)?;

                match base.kind {
                    client::SocketMessageKind::RunExperiment => {
                        let run_experiment = serde_json::from_str::<'_, client::SocketMessage<client::RunExperiment>>(text)
                        .map_err(|_| SocketErrorKind::InvalidMessage)?;

                        info!(
                            "received run from server, id {}",
                            run_experiment.data.job_id
                        );

                        if let Some(executor) = &self.executor {
                            self.controller_state = ControllerState::Running(run_experiment.data.job_id);
                            // in any case, clear the is_job_aborted
                            self.is_job_aborted = false;

                            let msg = RunMessage {
                                job_id: run_experiment.data.job_id,
                                code: run_experiment.data.code,
                            };
                            let addr = executor.clone();

                            async move {
                                if let Err(e) = addr.send(msg).await {
                                    error!("sending run message to executor is failed: {:?}", e);
                                }
                            }
                            .into_actor(self)
                            .spawn(ctx);
                        }
                    }
                    client::SocketMessageKind::AbortRunningJob => {
                         let abort_job = serde_json::from_str::<'_, client::SocketMessage<client::AbortRunningJob>>(text)
                        .map_err(|_| SocketErrorKind::InvalidMessage)?;

                        info!("Received abort message");
                        match self.controller_state {
                            ControllerState::Running(job_id) if job_id == abort_job.data.job_id => {
                                info!("Setting is_job_aborted to true");
                                self.is_job_aborted = true;
                            },
                            ControllerState::Running(job_id) => error!("Server sent an abort message for a different job from current running job, received {}, running {}", abort_job.data.job_id, job_id),
                            ControllerState::Idle => error!("Server sent an abort message even though controller is idle")
                        }
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    async fn connect(
        server_url: String,
        access_token: String,
        running_job_id: Option<i32>
    ) -> Result<Framed<BoxedSocket, Codec>, WsClientError> {
        let queries = serde_urlencoded::to_string(JoinServerRequest {
            token: access_token,
            running_job_id,
        })
        .unwrap();

        Client::new()
            .ws(format!("{}/experiment/ws?{}", server_url, queries))
            .connect()
            .await
            .map(|f| f.1)
    }

    fn try_connect(act: &mut Connection, ctx: &mut <Self as Actor>::Context) {
        let running_job_id = if let ControllerState::Running(job_id) = act.controller_state {
            Some(job_id)
        } else {
            None
        };

        Self::connect(
            act.server_url.clone(),
            act.access_token.clone(),
            running_job_id,
        )
        .into_actor(act)
        .then(move |framed, act, ctx| {
            match framed {
                Ok(framed) => {
                    info!("Connected to server");

                    let (sink, stream) = framed.split();
                    Self::add_stream(stream, ctx);
                    act.sink = Some(SinkWrite::new(sink, ctx));

                    let mut pending_messages = Vec::<RunResultMessage>::new();
                    std::mem::swap(&mut pending_messages, &mut act.pending_messages);

                    while let Some(msg) = pending_messages.pop() {
                        Self::upload_output_to_server(
                            msg,
                            act.server_url.clone(),
                            act.access_token.clone(),
                        )
                        .into_actor(act)
                        .then(|res, act, _| {
                            let (msg, sent) = res;
                            if sent {
                                act.send_msg_to_server(msg);
                            } else {
                                act.pending_messages.push(msg);
                            }

                            fut::ready(())
                        })
                        .spawn(ctx);
                    }

                    // we have connected now, reset timing
                    act.current_timing_index = 0;
                }
                Err(e) => {
                    error!("{:?}", e);

                    act.current_timing_index = min(act.current_timing_index + 1, MAX_TIMING - 1);

                    info!(
                        "Could not connect to server, will retry in {} seconds",
                        TIMINGS[act.current_timing_index]
                    );

                    ctx.run_later(
                        Duration::from_secs(TIMINGS[act.current_timing_index] as u64),
                        |act, ctx| {
                            Self::try_connect(act, ctx);
                        },
                    );
                }
            }

            fut::ready(())
        })
        .spawn(ctx);
    }

    fn serialize_result(msg: &RunResultMessage) -> Message {
        Message::Text(
            serde_json::to_string(&server::SocketMessage {
                kind: server::SocketMessageKind::RunResult,
                data: server::RunResult {
                    job_id: msg.job_id,
                    successful: msg.successful,
                },
            })
            .unwrap(),
        )
    }

    async fn upload_output_to_server(
        msg: RunResultMessage,
        server_url: String,
        access_token: String,
    ) -> (RunResultMessage, bool) {
        let res = Client::new()
            .post(format!(
                "{}/experiment/job/{}/output?token={}",
                server_url, msg.job_id, access_token
            ))
            .send_body(&msg.output)
            .await;

        (msg, res.is_ok())
    }

    fn send_msg_to_server(&mut self, msg: RunResultMessage) {
        if let Some(sink) = &mut self.sink {
            if let Some(_) = sink.write(Self::serialize_result(&msg)) {
                self.pending_messages.push(msg);
            }
        } else {
            self.pending_messages.push(msg);
        }
    }
}

impl Actor for Connection {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        info!("Connection is started!");

        Self::try_connect(self, ctx);
    }

    fn stopped(&mut self, _: &mut Self::Context) {
        info!("Connection is stopped!");
    }
}

impl StreamHandler<Result<Frame, WsProtocolError>> for Connection {
    fn handle(&mut self, frame: Result<Frame, WsProtocolError>, ctx: &mut Context<Self>) {
        match frame {
            Ok(frame) => {
                if let Err(e) = self.handle_frame(frame, ctx) {
                    error!("{:?}", e);
                }
            }
            Err(e) => error!("{:?}", e),
        };
    }

    fn finished(&mut self, ctx: &mut Context<Self>) {
        info!("Server disconnected, trying to reconnect");
        self.sink = None;
        Self::try_connect(self, ctx);
    }
}

impl Handler<UpdateExecutorMessage> for Connection {
    type Result = ();

    fn handle(&mut self, msg: UpdateExecutorMessage, _: &mut Self::Context) {
        self.executor = Some(msg.executor);
    }
}

impl Handler<ControllerReceiversValueMessage> for Connection {
    type Result = ();

    fn handle(&mut self, msg: ControllerReceiversValueMessage, _: &mut Self::Context) {
        let message = Message::Text(
            serde_json::to_string(&server::SocketMessage {
                kind: server::SocketMessageKind::ReceiverStatus,
                data: server::ControllerReceiverValue { values: msg.values },
            })
            .unwrap(),
        );

        if let Some(sink) = &mut self.sink {
            if let Some(_) = sink.write(message) {
                error!("unable to send receiver values to server");
            }
        }
    }
}

impl Handler<RunResultMessage> for Connection {
    type Result = ();

    fn handle(&mut self, msg: RunResultMessage, ctx: &mut Self::Context) {
        self.controller_state = ControllerState::Idle;
        self.is_job_aborted = false;

        Self::upload_output_to_server(msg, self.server_url.clone(), self.access_token.clone())
            .into_actor(self)
            .then(move |res, act, _| {
                let (msg, sent) = res;
                if sent {
                    act.send_msg_to_server(msg)
                } else {
                    act.pending_messages.push(msg);
                }

                fut::ready(())
            })
            .spawn(ctx);
    }
}

impl Handler<IsJobAborted> for Connection {
    type Result = bool;
    fn handle(&mut self, _: IsJobAborted, _: &mut Self::Context) -> bool {
        self.is_job_aborted
    }
}

impl actix::io::WriteHandler<WsProtocolError> for Connection {}

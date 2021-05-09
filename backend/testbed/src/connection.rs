use std::cmp::min;

use actix::{Actor, Context, StreamHandler, WrapFuture};
use actix::clock::Duration;
use actix::io::SinkWrite;
use actix::prelude::*;
use actix_codec::Framed;
use awc::{BoxedSocket, Client};
use awc::error::{WsClientError, WsProtocolError};
use awc::ws::{Codec, Frame, Message};
use futures::stream::{SplitSink, StreamExt};
use log::{error, info};

use shared::SocketErrorKind;
use shared::websocket_messages::{client, server};

use crate::messages::{ReceiverStatusMessage, RunMessage, RunResultMessage, UpdateExecutorMessage};

type Write = SinkWrite<Message, SplitSink<Framed<BoxedSocket, Codec>, Message>>;

const MAX_TIMING: usize = 5;

const TIMINGS: [u8; MAX_TIMING] = [
    // 0, 15, 30, 75, 120
    0, 2, 4, 6, 8
];

pub struct Connection {
    server_url: String,
    access_token: String,
    sink: Option<Write>,
    // this is the delay until we try connecting again
    current_timing_index: usize,
    executor: Option<Recipient<RunMessage>>,
    pending_messages: Vec<Message>,
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
        }
    }

    fn handle_frame(&mut self, frame: Frame, ctx: &mut <Self as Actor>::Context) -> Result<(), SocketErrorKind> {
        match frame {
            Frame::Ping(_) | Frame::Pong(_) => {
                //update hb
            }
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

                        info!("received run from server, id {}", run_experiment.data.job_id);

                        if let Some(executor) = &self.executor {
                            let msg = RunMessage {
                                job_id: run_experiment.data.job_id,
                                code: run_experiment.data.code,
                            };
                            let addr = executor.clone();

                            async move {
                                if let Err(e) = addr.send(msg)
                                    .await {
                                    error!("sending run message to executor is failed: {:?}", e);
                                }
                            }
                                .into_actor(self)
                                .spawn(ctx);
                        }
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    async fn connect(server_url: String, access_token: String) -> Result<Framed<BoxedSocket, Codec>, WsClientError> {
        Client::new()
            .ws(format!("{}?token={}", server_url, access_token))
            .connect()
            .await
            .map(|f| f.1)
    }

    fn try_connect(act: &mut Connection, ctx: &mut <Self as Actor>::Context) {
        Self::connect(act.server_url.clone(), act.access_token.clone())
            .into_actor(act)
            .then(move |framed, act, ctx| {
                match framed {
                    Ok(framed) => {
                        info!("Connected to server");

                        let (sink, stream) = framed.split();
                        Self::add_stream(stream, ctx);
                        act.sink = Some(SinkWrite::new(sink, ctx));

                        // try to send pending messages
                        let mut failed_messages = Vec::<Message>::new();
                        // in order to borrow the sink, we have to do 'let Some' pattern. It should always pass this check
                        if let Some(sink) = &mut act.sink {
                            act.pending_messages.drain(..).for_each(|message| {
                                if let Some(message) = sink.write(message) {
                                    failed_messages.push(message);
                                }
                            });
                        }
                        act.pending_messages = failed_messages;

                        // we have connected now, reset timing
                        act.current_timing_index = 0;
                    }
                    Err(e) => {
                        error!("{:?}", e);

                        act.current_timing_index = min(act.current_timing_index + 1, MAX_TIMING - 1);

                        info!("Could not connect to server, will retry in {} seconds", TIMINGS[act.current_timing_index]);

                        ctx.run_later(Duration::from_secs(TIMINGS[act.current_timing_index] as u64), |act, ctx| {
                            Self::try_connect(act, ctx);
                        });
                    }
                }

                fut::ready(())
            })
            .spawn(ctx);
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
            Err(e) => error!("{:?}", e)
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

impl Handler<ReceiverStatusMessage> for Connection {
    type Result = ();

    fn handle(&mut self, msg: ReceiverStatusMessage, _: &mut Self::Context) {
        let message = Message::Text(serde_json::to_string(&server::SocketMessage {
            kind: server::SocketMessageKind::ReceiverStatus,
            data: server::ReceiverStatus {
                outputs: msg.outputs,
            },
        }).unwrap());

        if let Some(sink) = &mut self.sink {
            if let Some(_) = sink.write(message) {
                error!("unable to send receiver status to server");
            }
        }
    }
}

impl Handler<RunResultMessage> for Connection {
    type Result = ();

    fn handle(&mut self, msg: RunResultMessage, _: &mut Self::Context) {
        let message = Message::Text(serde_json::to_string(&server::SocketMessage {
            kind: server::SocketMessageKind::RunResult,
            data: server::RunResult {
                job_id: msg.job_id,
                output: msg.output,
                successful: msg.successful,
            },
        }).unwrap());

        if let Some(sink) = &mut self.sink {
            // if write returns the message, sending was not successful, try to send it next time
            if let Some(message) = sink.write(message) {
                self.pending_messages.push(message);
            }
        } else { // if connection is not available, send it next time
            self.pending_messages.push(message);
        }
    }
}

impl actix::io::WriteHandler<WsProtocolError> for Connection {}

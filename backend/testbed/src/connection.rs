use std::cmp::min;

use actix::{Actor, Context, StreamHandler, WrapFuture};
use actix::clock::Duration;
use actix::io::SinkWrite;
use actix::prelude::*;
use actix_codec::Framed;
use awc::{BoxedSocket, Client};
use awc::error::WsProtocolError;
use awc::ws::{Codec, Frame, Message};
use futures::stream::{SplitSink, StreamExt};
use log::{error, info};

use core::websocket_messages::{SocketMessage, SocketMessageKind};
use core::websocket_messages::server::RegisterBackend;

use crate::executor::Executor;
use crate::messages::UpdateExecutorMessage;

type Write = SinkWrite<Message, SplitSink<Framed<BoxedSocket, Codec>, Message>>;

const MAX_TIMING: usize = 5;

const TIMINGS: [u8; MAX_TIMING] = [
    // 0, 15, 30, 75, 120
    0, 2, 4, 6, 8
];

pub struct Connection {
    server_url: String,
    access_key: String,
    sink: Option<Write>,
    // this is the delay until we try connecting again
    current_timing_index: usize,
    executor: Option<Addr<Executor>>,
}

impl Connection {
    pub fn new(server_url: String, access_key: String) -> Self {
        Connection {
            server_url,
            access_key,
            sink: None,
            current_timing_index: 0,
            executor: None,
        }
    }

    async fn connect(server_url: String) -> Result<Framed<BoxedSocket, Codec>, Error> {
        Ok(Client::new()
            .ws(server_url)
            .connect()
            .await
            .map_err(|e| {
                error!("{:?}", e);
                Error::ServerNotReachable
            })?.1)
    }

    fn try_connect(act: &mut Connection, ctx: &mut <Self as Actor>::Context) {
        Self::connect(act.server_url.clone())
            .into_actor(act)
            .then(move |framed, act, ctx| {
                if let Ok(framed) = framed {
                    info!("Connected to server");

                    let (sink, stream) = framed.split();
                    Self::add_stream(stream, ctx);
                    act.sink = Some(SinkWrite::new(sink, ctx));
                    // we have connected now, reset timing
                    act.current_timing_index = 0;
                } else {
                    act.current_timing_index = min(act.current_timing_index + 1, MAX_TIMING - 1);

                    info!("Could not connect to server, will retry in {} seconds", TIMINGS[act.current_timing_index]);

                    ctx.run_later(Duration::from_secs(TIMINGS[act.current_timing_index] as u64), |act, ctx| {
                        Self::try_connect(act, ctx);
                    });
                }

                fut::ready(())
            })
            .spawn(ctx);
    }
}

impl Actor for Connection {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        Self::try_connect(self, ctx);
    }

    fn stopped(&mut self, _: &mut Self::Context) {}
}

impl StreamHandler<Result<Frame, WsProtocolError>> for Connection {
    fn handle(&mut self, msg: Result<Frame, WsProtocolError>, _: &mut Context<Self>) {
        if let Ok(Frame::Text(txt)) = msg {
            info!("Server: {:?}", txt)
        }
    }

    fn started(&mut self, _ctx: &mut Context<Self>) {
        // After connection is established between server and testbed, register this backend
        let message = SocketMessage {
            kind: SocketMessageKind::RegisterBackend,
            data: RegisterBackend {
                access_key: self.access_key.clone(),
            },
        };

        if let Some(sink) = &mut self.sink {
            sink.write(Message::Text(serde_json::to_string(&message).unwrap()));
        }
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

impl actix::io::WriteHandler<WsProtocolError> for Connection {}

pub enum Error {
    ServerNotReachable
}
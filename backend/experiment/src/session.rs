use actix::{Actor, ActorContext, Context, StreamHandler};
use actix::io::SinkWrite;
use actix_codec::Framed;
use awc::BoxedSocket;
use awc::error::WsProtocolError;
use awc::ws::{Codec, Frame, Message};
use futures::stream::{SplitSink, StreamExt};

use core::messages::{SocketMessage, SocketMessageKind};
use core::messages::bidirect::RegisterBackend;

pub struct Session(SinkWrite<Message, SplitSink<Framed<BoxedSocket, Codec>, Message>>);

impl Session {
    pub fn new(sink: SinkWrite<Message, SplitSink<Framed<BoxedSocket, Codec>, Message>>) -> Self {
        Session(sink)
    }
}

impl Actor for Session {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {}

    fn stopped(&mut self, ctx: &mut Self::Context) {}
}

impl StreamHandler<Result<Frame, WsProtocolError>> for Session {
    fn handle(&mut self, msg: Result<Frame, WsProtocolError>, _: &mut Context<Self>) {
        if let Ok(Frame::Text(txt)) = msg {
            println!("Server: {:?}", txt)
        }
    }

    fn started(&mut self, _ctx: &mut Context<Self>) {
        let message = SocketMessage {
            kind: SocketMessageKind::RegisterBackend,
            data: RegisterBackend {
                id: 2,
                access_key: String::from("Hola Hermano"),
            },
        };

        self.0.write(Message::Text(serde_json::to_string(&message).unwrap()));
    }

    fn finished(&mut self, ctx: &mut Context<Self>) {
        println!("Server disconnected");
        ctx.stop()
    }
}

impl actix::io::WriteHandler<WsProtocolError> for Session {}
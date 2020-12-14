use actix::io::SinkWrite;
use actix::prelude::*;
use actix_codec::Framed;
use awc::{BoxedSocket, Client};
use awc::error::WsProtocolError;
use awc::ws::{Codec, Frame, Message};
use futures::stream::{SplitSink, StreamExt};
use log::{error, log};
use bytes::Bytes;

struct WSSession(SinkWrite<Message, SplitSink<Framed<BoxedSocket, Codec>, Message>>);

impl Actor for WSSession {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Context<Self>) {}

    fn stopped(&mut self, _: &mut Context<Self>) {
        println!("Disconnected");

        // Stop application on disconnect
        System::current().stop();
    }
}

/// Handle server websocket messages
impl StreamHandler<Result<Frame, WsProtocolError>> for WSSession {
    fn handle(&mut self, msg: Result<Frame, WsProtocolError>, _: &mut Context<Self>) {
        if let Ok(Frame::Text(txt)) = msg {
            println!("Server: {:?}", txt)
        }
    }

    fn started(&mut self, _ctx: &mut Context<Self>) {
        println!("Connected");
        self.0.write(Message::Text(String::from("Hola Herman")));
    }

    fn finished(&mut self, ctx: &mut Context<Self>) {
        println!("Server disconnected");
        ctx.stop()
    }
}

impl actix::io::WriteHandler<WsProtocolError> for WSSession {}

fn main() {
    // Load .env
    dotenv::dotenv().ok();

    // Enable logger
    env_logger::init();

    let sys = System::new("websocket-client");

    Arbiter::spawn(async {
        let (response, framed) = Client::new()
            .ws("http://169.254.163.46:8040/ws")
            .connect()
            .await
            .map_err(|e| {
                error!("{:?}", e);
            })
            .unwrap();

        println!("{:?}", response);

        let (sink, stream) = framed.split();

        let addr = WSSession::create(|ctx| {
            WSSession::add_stream(stream, ctx);
            WSSession(SinkWrite::new(sink, ctx))
        });
    });

    sys.run().unwrap();
}

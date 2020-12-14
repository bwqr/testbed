use actix::{Actor, Arbiter, StreamHandler, System};
use actix::io::SinkWrite;
use awc::Client;
use futures::StreamExt;
use log::{error, log};

use crate::session::Session;

pub mod session;

fn main() {
    // Load .env
    dotenv::dotenv().ok();

    // Enable logger
    env_logger::init();

    let sys = System::new("websocket-client");

    Arbiter::spawn(async {
        let (response, framed) = Client::new()
            .ws(std::env::var("SERVER_URL").expect("SERVER_URL is not provided in env"))
            .connect()
            .await
            .map_err(|e| {
                error!("{:?}", e);
            })
            .unwrap();

        println!("{:?}", response);

        let (sink, stream) = framed.split();

        let addr = Session::create(|ctx| {
            Session::add_stream(stream, ctx);
            Session::new(SinkWrite::new(sink, ctx))
        });
    });

    sys.run().unwrap();
}
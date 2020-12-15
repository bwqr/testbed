use actix::{Actor, Arbiter, System};

use crate::connection::Connection;
use crate::executor::Executor;
use crate::messages::{UpdateConnectionMessage, UpdateExecutorMessage};

mod connection;
mod executor;
mod messages;

fn main() {
    // Load .env
    dotenv::dotenv().ok();

    let access_key = std::env::var("BACKEND_ACCESS_KEY").expect("BACKEND_ACCESS_KEY is not provided in env");
    let server_url = std::env::var("SERVER_URL").expect("SERVER_URL is not provided in env");

    // Enable logger
    env_logger::init();

    let sys = System::new("websocket-client");

    Arbiter::spawn(async move {
        let connection = Connection::new(server_url, access_key).start();
        let executor = Executor::new().start();

        connection
            .send(UpdateExecutorMessage { executor: executor.clone() })
            .await
            .unwrap();

        executor
            .send(UpdateConnectionMessage { connection: connection.clone() })
            .await
            .unwrap();
    });

    sys.run().unwrap();
}
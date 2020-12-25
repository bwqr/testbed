use actix::{Actor, Arbiter, System};

use crate::connection::Connection;
use crate::executor::{Executor, ExecutorMock};
use crate::messages::{RunMessage, UpdateConnectionMessage, UpdateExecutorMessage};

mod connection;
mod executor;
mod messages;

type ModelId = i32;

fn main() {
    // Load .env
    dotenv::dotenv().ok();

    let access_token = std::env::var("BACKEND_ACCESS_TOKEN").expect("BACKEND_ACCESS_TOKEN is not provided in env");
    let server_url = std::env::var("SERVER_URL").expect("SERVER_URL is not provided in env");

    // Enable logger
    env_logger::init();

    let sys = System::new("websocket-client");

    Arbiter::spawn(async move {
        let connection = Connection::new(server_url, access_token).start();

        // TODO move executor into another thread. It should not block connection
        #[cfg(target_arch = "x86_64")]
            let executor = ExecutorMock::new().start();
        #[cfg(target_arch = "arm")]
            let executor = Executor::new().start();

        executor
            .send(UpdateConnectionMessage { connection: connection.clone() })
            .await
            .unwrap();

        connection
            .send(UpdateExecutorMessage { executor: executor.recipient::<RunMessage>() })
            .await
            .unwrap();
    });

    sys.run().unwrap();
}
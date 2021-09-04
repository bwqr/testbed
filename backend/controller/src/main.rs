use std::sync::mpsc::channel;

use actix::{Actor, Addr, Arbiter, Recipient, System};

use crate::connection::Connection;
use crate::executor::Executor;
use crate::messages::{RunMessage, UpdateExecutorMessage};

mod connection;
mod error;
mod executor;
mod process;
mod messages;
mod state;

type ModelId = i32;

fn setup_executor(connection: Addr<Connection>, docker_path: String, tx_dev_path: String, rx_dev_paths: Vec<String>, python_lib_path: String) -> Recipient<RunMessage> {
    let (tx, rx) = channel::<Recipient<RunMessage>>();

    std::thread::Builder::new().name("executor".to_string()).spawn(move || {
        let sys = System::new("executor");
        let executor = Executor::new(connection, docker_path, tx_dev_path, rx_dev_paths, python_lib_path).start();
        tx.send(executor.recipient::<RunMessage>()).expect("Failed to send Executor from thread");
        sys.run()
    }).expect("Failed to initialize thread");

    rx.recv().expect("Failed to receive Executor from thread")
}

fn main() {
    // Load .env
    dotenv::dotenv().ok();

    let access_token = std::env::var("BACKEND_ACCESS_TOKEN").expect("BACKEND_ACCESS_TOKEN is not provided in env");
    let server_url = std::env::var("SERVER_URL").expect("SERVER_URL is not provided in env");
    let docker_path = std::env::var("DOCKER_PATH").expect("DOCKER_PATH is not provided in env");
    let tx_dev_path = std::env::var("TRANSMITTER_DEVICE_PATH").expect("TRANSMITTER_DEVICE_PATH is not provided in env");
    let python_lib_path = std::env::var("PYTHON_LIB_PATH").expect("PYTHON_LIB_PATH is not provided in env");

    let rx_dev_paths = std::env::var("RECEIVER_DEVICE_PATHS").expect("RECEIVER_DEVICE_PATHS is not provided in env")
        .split(",")
        .into_iter()
        .map(|reference| String::from(reference))
        .collect::<Vec<String>>();

    // Enable logger
    env_logger::init();

    let sys = System::new("websocket-client");

    Arbiter::spawn(async move {
        let connection = Connection::new(server_url, access_token).start();

        let executor = setup_executor(connection.clone(), docker_path, tx_dev_path, rx_dev_paths, python_lib_path);

        connection
            .send(UpdateExecutorMessage { executor })
            .await
            .unwrap();
    });

    sys.run().unwrap();
}

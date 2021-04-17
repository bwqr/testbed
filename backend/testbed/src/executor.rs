use std::io::{Read, Write};
use std::process::Stdio;
use std::time::Duration;

use actix::prelude::*;
use log::{error, info};
use serial::core::SerialDevice;

use crate::connection::Connection;
use crate::messages::{RunMessage, RunResultMessage};
use crate::ModelId;

const PYTHON_VERSION: &str = "3.9";
const ALPINE_VERSION: &str = "3.13";

mod incoming {
    pub const START_MESSAGE: &str = "arduino_avaiable";
    pub const END_MESSAGE: &str = "end_of_experiment";
}

mod outgoing {
    pub const ABORT_MESSAGE: &str = "abort_experiment\n";
    pub const END_MESSAGE: &str = "end_of_experiment";
}

pub struct Executor {
    connection: Addr<Connection>,
    docker_path: String,
    tx_dev_path: String,
    rx_dev_path: String,
    python_lib_path: String,
}

impl Executor {
    pub fn new(connection: Addr<Connection>, docker_path: String, tx_dev_path: String, rx_dev_path: String, python_lib_path: String) -> Self {
        Executor {
            connection,
            docker_path,
            tx_dev_path,
            rx_dev_path,
            python_lib_path,
        }
    }

    fn gen_tmp_dir(job_id: ModelId) -> String {
        format!("/tmp/testbed/{}", job_id)
    }

    fn create_dir_and_files(script_dir: &str, code: String) -> Result<(), Error> {
        let file = String::from(script_dir) + "/job.py";

        std::fs::create_dir_all(script_dir)
            .map_err(|e| Error::IO(e))?;

        let mut f = std::fs::File::create(file.as_str())
            .map_err(|e| Error::IO(e))?;

        f.write(code.as_bytes())
            .map_err(|e| Error::IO(e))?;

        Ok(())
    }

    fn remove_dir(&self, script_dir: &str) -> Result<(), Error> {
        std::fs::remove_dir_all(script_dir)
            .map_err(|e| Error::IO(e))
    }

    fn run_transmitter_code(&self, script_dir: &str) -> Result<String, Error> {
        let output = std::process::Command::new(self.docker_path.as_str())
            .args(&["run", "--rm", "-e PYTHONDONTWRITEBYTECODE=1"])
            .args(&["--mount", format!("type=bind,source={},target=/usr/local/lib/python{}/site-packages/,readonly", self.python_lib_path.as_str(), PYTHON_VERSION).as_str()])
            .args(&["--mount", format!("type=bind,source={},target=/usr/local/scripts/,readonly", script_dir).as_str()])
            .arg(format!("python:{}-alpine{}", PYTHON_VERSION, ALPINE_VERSION).as_str())
            .args(&["python", "/usr/local/scripts/job.py", "--transmitter"])
            .output()
            .map_err(|e| Error::IO(e))?;

        if !output.status.success() {
            // concatenate stdout and stderr
            let mut err = String::from_utf8(output.stdout).map_err(|e| Error::String(e))?;
            err += String::from_utf8(output.stderr).map_err(|e| Error::String(e))?.as_str();

            return Err(Error::Output(err));
        }

        String::from_utf8(output.stdout)
            .map_err(|e| Error::String(e))
    }

    fn start_receiver(&self, script_dir: &str) -> Result<std::process::Child, Error> {
        std::process::Command::new(self.docker_path.as_str())
            .args(&["run", "--rm", "-e PYTHONDONTWRITEBYTECODE=1", "-p", "8011:8011"])
            .arg(format!("--device={}", self.rx_dev_path.as_str()))
            .args(&["--mount", format!("type=bind,source={},target=/usr/local/lib/python{}/site-packages/,readonly", self.python_lib_path.as_str(), PYTHON_VERSION).as_str()])
            .args(&["--mount", format!("type=bind,source={},target=/usr/local/scripts/,readonly", script_dir).as_str()])
            .arg(format!("python:{}-alpine{}", PYTHON_VERSION, ALPINE_VERSION).as_str())
            .args(&["python", "/usr/local/scripts/job.py", "--receiver", self.rx_dev_path.as_str()])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| Error::Custom(format!("Failed to spawn receiver process, {:?}", e)))
    }

    fn send_to_transmitter(&self, port: &mut serial::SystemPort, command_buffer: String) -> Result<(), Error> {
        let mut buffer = [0 as u8; incoming::START_MESSAGE.len()];

        let mut error: Option<Error> = None;

        // read the START_MESSAGE message, we do not check if received buffer equals to START_MESSAGE since it is mostly equal.
        // try 5 times to read a message
        for _ in 0..5 {
            match port.read(&mut buffer) {
                Ok(_) => {
                    error = None;
                    break;
                }
                Err(e) => error = Some(Error::IO(e))
            };
        }

        if let Some(error) = error {
            return Err(error);
        }

        port.write(command_buffer.as_bytes())
            .map_err(|e| Error::IO(e))?;

        Ok(())
    }

    fn handle_execution(&self, job_id: ModelId, code: String) -> Result<String, Error> {
        let script_dir = Self::gen_tmp_dir(job_id);

        Self::create_dir_and_files(script_dir.as_str(), code)?;

        let command_buffer = self.run_transmitter_code(script_dir.as_str())?;

        let mut port = serial::open(self.tx_dev_path.as_str())
            .map_err(|e| Error::Serial(e))?;
        port.set_timeout(Duration::from_secs(1))
            .map_err(|e| Error::Serial(e))?;

        let mut child = self.start_receiver(script_dir.as_str())?;

        self.send_to_transmitter(&mut port, command_buffer)?;
        let mut buff = [0 as u8; incoming::END_MESSAGE.len()];

        let mut tx_success = false;
        let mut child_exited = false;

        loop {
            // wait end of experiment message from transmitter
            match port.read(&mut buff) {
                Ok(_) => {
                    if buff == incoming::END_MESSAGE.as_bytes() {
                        tx_success = true;
                        break;
                    }
                }
                Err(e) => {
                    match e.kind() {
                        // ignore TimedOut error
                        std::io::ErrorKind::TimedOut => {}
                        e => error!("error while reading from tx device{:?}", e)
                    }
                }
            }
            match child.try_wait() {
                Ok(Some(status)) => {
                    child_exited = true;
                    break;
                }
                Ok(None) => {}
                Err(e) => {}
            };
        }

        // if the loop did not exited with end of message, send abort experiment message to transmitter
        if !tx_success {
            port.write(outgoing::ABORT_MESSAGE.as_bytes())
                .map_err(|e| Error::IO(e))?;
        }

        if !child_exited {
            std::net::TcpStream::connect("127.0.0.1:8011")
                .unwrap()
                .write(outgoing::END_MESSAGE.as_bytes())
                .unwrap();
        }

        // wait one minute until receiver exits

        let mut output = String::new();
        child.stdout.unwrap().read_to_string(&mut output)
            .map_err(|e| Error::IO(e))?;
        child.stderr.unwrap().read_to_string(&mut output)
            .map_err(|e| Error::IO(e))?;

        self.remove_dir(script_dir.as_str())?;

        Ok(output)
    }
}

impl Actor for Executor {
    type Context = Context<Self>;
}

impl Handler<RunMessage> for Executor {
    type Result = ();

    fn handle(&mut self, msg: RunMessage, ctx: &mut Self::Context) {
        info!("got some run for Executor, id: {}, code: {}", msg.job_id, msg.code);

        let job_id = msg.job_id;

        let addr = self.connection.clone();

        let (output, successful) = match self.handle_execution(msg.job_id, msg.code) {
            Ok(output) => (output, true),
            Err(e) => {
                error!("could not execute the job, {:?}", e);
                // just try to remove script files, even error originated from remove_script_files, we should try it.
                let _ = self.remove_dir(Self::gen_tmp_dir(job_id).as_str());

                (e.to_string(), false)
            }
        };

        async move {
            if let Err(e) = addr.send(RunResultMessage { job_id, output, successful })
                .await {
                error!("could not send run result to connection, {:?}", e);
            }
        }
            .into_actor(self)
            .spawn(ctx);
    }
}

#[derive(Debug)]
pub enum Error {
    IO(std::io::Error),
    Serial(serial::Error),
    String(std::string::FromUtf8Error),
    Output(String),
    Custom(String),
}

impl Error {
    pub fn to_string(self) -> String {
        match self {
            Error::IO(io) => format!("{:?}", io),
            Error::String(utf8) => format!("{:?}", utf8),
            Error::Output(output) => output,
            Error::Serial(serial) => format!("{:?}", serial),
            Error::Custom(message) => format!("Custom {:?}", message),
        }
    }
}
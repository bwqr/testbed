use std::io::{Read, Write};
use std::process::{ExitStatus, Stdio};
use std::time::Duration;

use actix::prelude::*;
use log::error;
use serial::core::SerialDevice;

use crate::connection::Connection;
use crate::messages::{RunMessage, RunResultMessage};
use crate::ModelId;
use crate::state::Decoder;

const PYTHON_VERSION: &str = "3.9";
const ALPINE_VERSION: &str = "3.13";

// in milliseconds
const MAX_EXECUTION_TIME: u32 = 50000;

// in milliseconds
const MAX_EMIT_TIME: u32 = 10000;

mod incoming {
    pub const SETUP_MESSAGE: &str = "arduino_available";
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
        let mut child = std::process::Command::new(self.docker_path.as_str())
            .args(&["run", "--rm", "-e", "PYTHONDONTWRITEBYTECODE=1"])
            .args(&["--mount", format!("type=bind,source={},target=/usr/local/lib/python{}/site-packages/,readonly", self.python_lib_path.as_str(), PYTHON_VERSION).as_str()])
            .args(&["--mount", format!("type=bind,source={},target=/usr/local/scripts/,readonly", script_dir).as_str()])
            .arg(format!("python:{}-alpine{}", PYTHON_VERSION, ALPINE_VERSION).as_str())
            .args(&["python", "/usr/local/scripts/job.py", "--transmitter"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| Error::Custom(format!("Failed to spawn transmitter process, {:?}", e)))?;

        // wait 10 second until receiver code exits
        let mut exited = false;
        for _ in 0..10 {
            match child.try_wait() {
                Ok(Some(status)) => {
                    exited = true;

                    // if not successful, gather the outputs
                    if !status.success() {
                        let mut output = String::new();
                        child.stdout.unwrap().read_to_string(&mut output)
                            .map_err(|e| Error::IO(e))?;
                        child.stderr.unwrap().read_to_string(&mut output)
                            .map_err(|e| Error::IO(e))?;

                        return Err(Error::Output(output));
                    }

                    break;
                }
                Ok(None) => {}
                Err(e) => return Err(Error::IO(e))
            }

            std::thread::sleep(Duration::from_secs(1));
        }

        if !exited {
            let _ = child.kill();
            return Err(Error::Custom(String::from("transmitter code did not exit in 10 seconds")));
        }

        let mut output = String::new();
        child.stdout.unwrap().read_to_string(&mut output)
            .map_err(|e| Error::IO(e))?;

        Ok(output)
    }

    fn start_receiver(&self, script_dir: &str) -> Result<std::process::Child, Error> {
        std::process::Command::new(self.docker_path.as_str())
            .args(&["run", "--rm", "-e", "PYTHONDONTWRITEBYTECODE=1", "-p", "8011:8011"])
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

    fn start_transmitter(&self) -> Result<serial::SystemPort, Error> {
        let mut port = serial::open(self.tx_dev_path.as_str())
            .map_err(|e| Error::Serial(e))?;
        port.set_timeout(Duration::from_secs(1))
            .map_err(|e| Error::Serial(e))?;

        let mut buffer = [0 as u8; incoming::SETUP_MESSAGE.len()];

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

        Ok(port)
    }

    fn send_to_transmitter(&self, port: &mut serial::SystemPort, command_buffer: String) -> Result<(), Error> {
        port.write(command_buffer.as_bytes())
            .map(|_| ())
            .map_err(|e| Error::IO(e))
    }

    fn wait_exit(&self, port: &mut serial::SystemPort, child: &mut std::process::Child) -> Result<ExitReason, Error> {
        let mut buff = [0 as u8; incoming::END_MESSAGE.len()];

        let mut exit_reason: Option<ExitReason> = None;

        while exit_reason.is_none() {
            match port.read(&mut buff) {
                Ok(_) => {
                    if buff == incoming::END_MESSAGE.as_bytes() {
                        exit_reason = Some(ExitReason::EndOfExperiment);
                    }
                }
                Err(e) => {
                    match e.kind() {
                        // ignore TimedOut error
                        std::io::ErrorKind::TimedOut => {}
                        e => return Err(Error::IO(std::io::Error::from(e)))
                    }
                }
            }

            match child.try_wait() {
                Ok(Some(status)) => {
                    exit_reason = Some(ExitReason::ChildExit(status));
                }
                Ok(None) => {}
                Err(e) => error!("Error while trying to wait for child, {:?}", e)
            };
        }
        // end of experiment is default exit reason
        Ok(exit_reason.unwrap_or(ExitReason::EndOfExperiment))
    }

    fn handle_execution(&self, job_id: ModelId, code: String) -> Result<String, Error> {
        let script_dir = Self::gen_tmp_dir(job_id);

        Self::create_dir_and_files(script_dir.as_str(), code)?;

        let command_buffer = self.run_transmitter_code(script_dir.as_str())?;

        let state = Decoder::decode(command_buffer.as_str())
            .map_err(|e| Error::Custom(format!("Unable to decode state, {:?}", e)))?;

        let execution_time = state.execution_time();
        if execution_time > MAX_EXECUTION_TIME {
            return Err(Error::Custom(format!("Max execution time is reached, execution time: {}", execution_time)));
        }

        let emit_time = state.emit_time();
        if emit_time > MAX_EMIT_TIME {
            return Err(Error::Custom(format!("Max emit time is reached, emit time: {}", emit_time)));
        }

        let mut port = self.start_transmitter()?;

        let mut child = self.start_receiver(script_dir.as_str())?;

        self.send_to_transmitter(&mut port, command_buffer)?;

        match self.wait_exit(&mut port, &mut child) {
            Ok(exit_reason) => {
                match exit_reason {
                    ExitReason::EndOfExperiment => {
                        std::net::TcpStream::connect("127.0.0.1:8011")
                            .map_err(|_| {
                                let _ = child.kill();
                                Error::Custom(String::from("Failed to connect receiver code over tcp"))
                            })?
                            .write(outgoing::END_MESSAGE.as_bytes())
                            .map_err(|_| {
                                let _ = child.kill();
                                Error::Custom(String::from("Failed to send END_MESSAGE to receiver code"))
                            })?;

                        // wait 10 second until receiver code exits
                        let mut exited = false;
                        for _ in 0..10 {
                            match child.try_wait() {
                                Ok(Some(status)) => {
                                    exited = true;

                                    // if not successful, gather the outputs
                                    if !status.success() {
                                        let mut output = String::new();
                                        child.stdout.unwrap().read_to_string(&mut output)
                                            .map_err(|e| Error::IO(e))?;
                                        child.stderr.unwrap().read_to_string(&mut output)
                                            .map_err(|e| Error::IO(e))?;

                                        return Err(Error::Output(output));
                                    }

                                    break;
                                }
                                Ok(None) => {}
                                Err(e) => return Err(Error::IO(e))
                            }

                            std::thread::sleep(Duration::from_secs(1));
                        }

                        if !exited {
                            let _ = child.kill();
                            return Err(Error::Custom(String::from("receiver code did not exit in 10 seconds")));
                        }
                    }
                    ExitReason::ChildExit(status) => {
                        port.write(outgoing::ABORT_MESSAGE.as_bytes())
                            .map_err(|e| Error::IO(e))?;

                        // user receiver code should not exit until receiving an END_MESSAGE over tcp
                        if status.success() {
                            return Err(Error::Custom(String::from("User receiver code exited successfully without waiting end of experiment, this should not be the case")));
                        }

                        // if not successful, gather the outputs
                        let mut output = String::new();
                        child.stdout.unwrap().read_to_string(&mut output)
                            .map_err(|e| Error::IO(e))?;
                        child.stderr.unwrap().read_to_string(&mut output)
                            .map_err(|e| Error::IO(e))?;

                        return Err(Error::Output(output));
                    }
                }
            }
            Err(e) => {
                // just kill everything without checking error and return error
                let _ = port.write(outgoing::ABORT_MESSAGE.as_bytes());
                let _ = child.kill();

                return Err(e);
            }
        }

        let mut output = String::new();
        child.stdout.unwrap().read_to_string(&mut output)
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
        let job_id = msg.job_id;

        let addr = self.connection.clone();

        let (output, successful) = match self.handle_execution(msg.job_id, msg.code) {
            Ok(output) => (output, true),
            Err(e) => {
                // just try to remove script files, even error originated from remove_script_files, we should try it.
                let _ = self.remove_dir(Self::gen_tmp_dir(job_id).as_str());

                (format!("{:?}", e), false)
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
enum Error {
    IO(std::io::Error),
    Serial(serial::Error),
    String(std::string::FromUtf8Error),
    Output(String),
    Custom(String),
}

enum ExitReason {
    EndOfExperiment,
    ChildExit(ExitStatus),
}
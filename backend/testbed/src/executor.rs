use std::io::{Read, Write};
use std::time::Duration;

use actix::prelude::*;
use log::{error, info};
use serial::core::SerialDevice;

use crate::connection::Connection;
use crate::messages::{RunMessage, RunResultMessage};
use crate::ModelId;

const PYTHON_VERSION: &str = "3.9";
const ALPINE_VERSION: &str = "3.13";

const START_MESSAGE: &str = "arduino_avaiable";

pub struct Executor {
    connection: Addr<Connection>,
    docker_path: String,
    serial_path: String,
    python_lib_path: String,
}

impl Executor {
    pub fn new(connection: Addr<Connection>, docker_path: String, serial_path: String, python_lib_path: String) -> Self {
        Executor {
            connection,
            docker_path,
            serial_path,
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

    fn start_receiver(&self, script_dir: &str) -> Result<String, Error> {
        let (tx, rx) = std::sync::mpsc::channel();
        let docker_path = self.docker_path.clone();
        let python_lib_path = self.python_lib_path.clone();
        let script_dir = String::from(script_dir);
        std::thread::Builder::new().name(String::from("testbed-receiver")).spawn(move || {
            tx.send(
                std::process::Command::new(docker_path.as_str())
                    .args(&["run", "--rm", "-e PYTHONDONTWRITEBYTECODE=1", "--device=/dev/ttyUSB0", "-p", "8011:8011"])
                    .args(&["--mount", format!("type=bind,source={},target=/usr/local/lib/python{}/site-packages/,readonly", python_lib_path.as_str(), PYTHON_VERSION).as_str()])
                    .args(&["--mount", format!("type=bind,source={},target=/usr/local/scripts/,readonly", script_dir.as_str()).as_str()])
                    .arg(format!("python:{}-alpine{}", PYTHON_VERSION, ALPINE_VERSION).as_str())
                    .args(&["python", "/usr/local/scripts/job.py", "--receiver"])
                    .output()
                    .map_err(|e| Error::IO(e))
            )
        }).map_err(|e| Error::IO(e))?;

        std::thread::sleep(Duration::from_secs(10));

        match std::net::TcpStream::connect("127.0.0.1:8011")
            .map_err(|e| Error::Custom(String::from("Failed to connect tcp stream"))) {
            Ok(mut tcp) => tcp.write("end_of_experiment".as_bytes())
                .map_err(|e| Error::IO(e))?,
            Err(e) => Err(e)?
        };

        let output = rx.recv().map_err(|e| Error::Custom(String::from("failed to receive from thread")))??;

        if !output.status.success() {
            // concatenate stdout and stderr
            let mut err = String::from_utf8(output.stdout).map_err(|e| Error::String(e))?;
            err += String::from_utf8(output.stderr).map_err(|e| Error::String(e))?.as_str();

            return Err(Error::Output(err));
        }

        String::from_utf8(output.stdout)
            .map_err(|e| Error::String(e))
    }

    fn send_to_transmitter(&self, port: &mut serial::SystemPort, command_buffer: String) -> Result<(), Error> {
        let mut buffer: Vec<u8> = Vec::with_capacity(START_MESSAGE.len());

        port.set_timeout(Duration::from_secs(5))
            .map_err(|e| Error::Serial(e))?;

        // read the START_MESSAGE message, we do not check if received buffer equals to START_MESSAGE since it is mostly equal.
        port.read(buffer.as_mut_slice())
            .map_err(|e| Error::IO(e))?;

        port.write(command_buffer.as_bytes())
            .map_err(|e| Error::IO(e))?;

        Ok(())
    }

    fn handle_execution(&self, job_id: ModelId, code: String) -> Result<String, Error> {
        let script_dir = Self::gen_tmp_dir(job_id);

        Self::create_dir_and_files(script_dir.as_str(), code)?;

        // let command_buffer = self.run_transmitter_code(script_dir)?;

        // let mut port = serial::open(self.serial_path.as_str())
        //     .map_err(|e| Error::Serial(e))?;

        let output = self.start_receiver(script_dir.as_str())?;

        // self.send_to_transmitter(&mut port, command_buffer)?;

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
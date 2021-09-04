use std::io::{self, Read, Write};
use std::net::SocketAddr;
use std::sync::Mutex;
use std::time::Duration;

use actix::prelude::*;
use log::{error, info};
use serial::core::SerialDevice;

use crate::connection::Connection;
use crate::messages::{RunMessage, ControllerReceiversValueMessage, RunResultMessage, IsJobAborted};
use crate::ModelId;
use crate::state::{self, Decoder, END_DELIMITER_NEW_LINE, START_DELIMITER_NEW_LINE, State};
use crate::process::{Error as ProcessError, ErrorKind as ProcessErrorKind, DockerBuilder, DockerProcess};
use crate::error::ErrorCause;

// in seconds
const SEND_RECEIVERS_VALUES_INTERVAL: u64 = 10;

mod incoming {
    pub mod arduino {
        pub const SETUP_MESSAGE: &str = "arduino_available";
        pub const END_MESSAGE: &str = "end_of_experiment";
    }
}

mod outgoing {
    pub mod tcp {
        pub const END_MESSAGE: &str = "end_of_experiment";
    }
}

pub struct Executor {
    connection: Addr<Connection>,
    docker_path: String,
    tx_dev_path: String,
    rx_dev_paths: Vec<String>,
    python_lib_path: String,
    rx_lock: Mutex<()>,
}

impl Executor {
    pub fn new(connection: Addr<Connection>, docker_path: String, tx_dev_path: String, rx_dev_paths: Vec<String>, python_lib_path: String) -> Self {
        Executor {
            connection,
            docker_path,
            tx_dev_path,
            rx_dev_paths,
            python_lib_path,
            rx_lock: Mutex::new(()),
        }
    }

    fn gen_tmp_dir(job_id: ModelId) -> String {
        format!("/tmp/controller/{}", job_id)
    }

    fn create_dir_and_files(script_dir: &str, code: String) -> Result<(), Error> {
        let file = String::from(script_dir) + "/job.py";

        std::fs::create_dir_all(script_dir)
            .map_err(|e| Error::IOError(e, "creating script dir"))?;

        let mut f = std::fs::File::create(file.as_str())
            .map_err(|e| Error::IOError(e, "creating script file"))?;

        f.write(code.as_bytes())
            .map_err(|e| Error::IOError(e, "writing script file"))?;

        Ok(())
    }

    fn remove_dir(script_dir: &str) -> Result<(), Error> {
        std::fs::remove_dir_all(script_dir)
            .map_err(|e| Error::IOError(e, "removing script dir"))
    }

    fn run_transmitter_code(&self, script_dir: &str) -> Result<String, Error> {
        let process = DockerBuilder::new(
            self.docker_path.as_str(),
            self.python_lib_path.as_str(),
            script_dir,
            &["python", "/usr/local/scripts/job.py", "--transmitter"]
        )
            .name("nrgtestbed-transmitter")
            .build()
            .map_err(|e| Error::ProcessErrorKind(e))?;

        process.wait(60)
            .map_err(|e| Error::ProcessError(e))
    }

    fn start_receiver(&self, script_dir: &str) -> Result<DockerProcess, Error> {
        let devices = (&self.rx_dev_paths)
            .into_iter()
            .map(|dev| dev.as_str())
            .collect::<Vec<&str>>();

        DockerBuilder::new(
            self.docker_path.as_str(),
            self.python_lib_path.as_str(),
            script_dir,
            &["python", "/usr/local/scripts/job.py", "--receiver"]
        )
            .name("nrgtestbed-receiver")
            .devices(&devices)
            .build()
            .map_err(|e| Error::ProcessErrorKind(e))
    }

    fn start_transmitter(&self) -> Result<serial::SystemPort, Error> {
        let mut port = serial::open(self.tx_dev_path.as_str())
            .map_err(|e| Error::SerialError(e, "opening serial port"))?;

        // Wait 5 seconds for SETUP_MESSAGE
        port.set_timeout(Duration::from_secs(5))
            .map_err(|e| Error::SerialError(e, "setting serial port timeout"))?;

        let mut buffer = [0 as u8; incoming::arduino::SETUP_MESSAGE.len()];

        // read the START_MESSAGE message, we do not check if received buffer equals to START_MESSAGE since it is mostly equal.
        port.read(&mut buffer)
            .map_err(|e| Error::IOError(e, "reading from serial port"))?;

        // Other IO operations should have 1 second for timeout
        port.set_timeout(Duration::from_secs(1))
            .map_err(|e| Error::SerialError(e, "setting serial port timeout"))?;

        Ok(port)
    }

    fn run_commands(&self, state: State, port: &mut serial::SystemPort, receiver: &mut DockerProcess) -> Result<ExitReason, Error> {
        let mut buff = [0 as u8; incoming::arduino::END_MESSAGE.len()];

        // clear previous characters from transmitter
        port.write("\n".as_bytes())
            .map_err(|e| Error::IOError(e, "writing new line char"))?;
        port.write(START_DELIMITER_NEW_LINE.as_bytes())
            .map_err(|e| Error::IOError(e, "writing start delimiter new line"))?;


        for command in state.into_iter() {
            info!("{:?}", command);
            let connection = self.connection.clone();

            let res = futures::executor::block_on(async move {
                connection.send(IsJobAborted)
                    .await
            });

            match res {
                Ok(true) => return Err(Error::JobAborted),
                Ok(false) => {},
                Err(e) => error!("Error while checking if job is aborted, {:?}", e)
            }

            port.write(command.as_bytes())
                .map_err(|e| Error::IOError(e, "writing command to serial port"))?;

            let mut total_read_size = 0;

            // Loop until command is executed or receiver is terminated
            loop {
                if receiver.is_terminated() {
                    port.write(END_DELIMITER_NEW_LINE.as_bytes())
                       .map_err(|e| Error::IOError(e, "writing end delimiter new line"))?;

                    return Ok(ExitReason::ProcessExit);
                }

                receiver.read_pipes()
                    .map_err(|e| Error::ProcessErrorKind(e))?;

                match port.read(&mut buff) {
                    Ok(size) => {
                        total_read_size += size;

                        if total_read_size >= incoming::arduino::END_MESSAGE.len() {
                            break;
                        }
                    }
                    Err(e) if std::io::ErrorKind::TimedOut == e.kind() => { },
                    Err(e) => return Err(Error::IOError(e, "reading end message from serial port"))
                }
            }
        }

        port.write(END_DELIMITER_NEW_LINE.as_bytes())
            .map_err(|e| Error::IOError(e, "writing end delimiter new line to end experiment"))?;

        Ok(ExitReason::EndOfExperiment)
    }

    fn send_end_of_experiment() -> Result<(), io::Error> {
        std::net::TcpStream::connect_timeout(&SocketAddr::from(([127, 0, 0, 1], 8011)), Duration::from_secs(10))?
            .write(outgoing::tcp::END_MESSAGE.as_bytes())?;

        Ok(())
    }

    fn handle_execution(&self, job_id: ModelId, code: String) -> Result<String, Error> {
        info!("generating tmp dirs");
        let script_dir = Self::gen_tmp_dir(job_id);

        info!("creating dirs and files");
        Self::create_dir_and_files(script_dir.as_str(), code)?;

        info!("running the transmitter code");
        let serialized_state = self.run_transmitter_code(script_dir.as_str())?;

        info!("decoding the state");
        let state = Decoder::decode(serialized_state.as_str())
            .map_err(|e| Error::Decoding(e, serialized_state))?;

        info!("starting the transmitter");
        let mut port = self.start_transmitter()?;

        info!("starting the receiver");
        let mut receiver = self.start_receiver(script_dir.as_str())?;

        info!("running commands");
        let output = match self.run_commands(state, &mut port, &mut receiver) {
            Ok(ExitReason::EndOfExperiment) => {
                info!("experiment is ended");

                if let Err(e) = Self::send_end_of_experiment() {
                    error!("failed to send end of experiment to receiver");

                    receiver.kill()
                        .map_err(|e| Error::ProcessErrorKind(e))?;

                    return Err(Error::IOError(e, "sending end of experiment to receiver"));
                }

                info!("waiting for receiver to exit and generating the output");
                receiver.wait(60)
                   .map_err(|e| Error::ProcessError(e))?
            }
            Ok(ExitReason::ProcessExit) => {
                info!("process exited before experiment end");

                receiver.wait(1)
                    .map_err(|e| Error::ProcessError(e))?
            }
            Err(e) => {
                // just kill everything without checking error and return error
                let _ = port.write(END_DELIMITER_NEW_LINE.as_bytes());
                let _ = receiver.kill();

                return Err(e);
            }
        };

        info!("removing script dir");
        Self::remove_dir(script_dir.as_str())?;

        info!("returning");
        Ok(output)
    }
}

impl Executor {
    fn send_receivers_values(act: &mut Executor, ctx: &mut <Self as Actor>::Context) {
        if let std::sync::TryLockResult::Ok(_) = act.rx_lock.try_lock() {
            let serials: Vec<Option<serial::SystemPort>> = act.rx_dev_paths.iter()
                .map(|path| {
                    match serial::open(path) {
                        Ok(mut port) => {
                            match port.set_timeout(Duration::from_secs(5)) {
                                Ok(_) => Some(port),
                                Err(_) => None
                            }
                        }
                        Err(_) => None
                    }
                })
                .collect();

            let mut values: Vec<u8> = Vec::with_capacity(serials.len());

            for serial in serials {
                match serial {
                    Some(mut serial) => {
                        let mut buff: [u8; 1] = [0];

                        match serial.read(&mut buff) {
                            Ok(_) => values.push(buff[0]),
                            Err(_) => values.push(0)
                        }
                    }
                    None => values.push(0)
                }
            }

            act.connection.do_send(ControllerReceiversValueMessage { values });

            ctx.run_later(Duration::from_secs(SEND_RECEIVERS_VALUES_INTERVAL), Self::send_receivers_values);
        }
    }
}

impl Actor for Executor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Context<Self>) {
        info!("Executor is started!");
        ctx.run_later(Duration::from_secs(SEND_RECEIVERS_VALUES_INTERVAL), Self::send_receivers_values);
    }

    fn stopped(&mut self, _ctx: &mut Context<Self>) {
        info!("Executor is stopped!");
    }
}

impl Handler<RunMessage> for Executor {
    type Result = ();

    fn handle(&mut self, msg: RunMessage, ctx: &mut Self::Context) {
        let job_id = msg.job_id;

        let addr = self.connection.clone();

        // lock the receiver
        let _ = self.rx_lock.lock().unwrap();

        let (output, successful) = match self.handle_execution(msg.job_id, msg.code) {
            Ok(output) => (output, true),
            Err(e) => {
                info!("failed to execute experiment, {:?}", e);
                // just try to remove script files, even error originated from remove_script_files, we should try it.
                let _ = Self::remove_dir(Self::gen_tmp_dir(job_id).as_str());

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
    ProcessError(ProcessError),
    ProcessErrorKind(ProcessErrorKind),
    IOError(io::Error, &'static str),
    SerialError(serial::Error, &'static str),
    JobAborted,
    Decoding(state::Error, String)
}

impl Error {
    fn cause(&self) -> ErrorCause {
        match self{
            Error::ProcessError(e) => e.cause(),
            Error::ProcessErrorKind(e) => e.cause(),
            Error::IOError(_, _) | Error::SerialError(_, _) => ErrorCause::Internal,
            Error::JobAborted => ErrorCause::Abort,
            Error::Decoding(_, _) => ErrorCause::User,
        }
    }
}

enum ExitReason {
    EndOfExperiment,
    ProcessExit,
}

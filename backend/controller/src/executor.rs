use std::io::{Read, Write};
use std::net::SocketAddr;
use std::process::{Child, ExitStatus, Stdio};
use std::sync::Mutex;
use std::time::Duration;

use actix::prelude::*;
use log::{error, info};
use serial::core::SerialDevice;

use crate::connection::Connection;
use crate::executor::limits::{MAX_CPU_CORE, MAX_MEMORY_USAGE};
use crate::messages::{RunMessage, ControllerReceiversValueMessage, RunResultMessage, IsJobAborted};
use crate::ModelId;
use crate::state::{Decoder, END_DELIMITER_NEW_LINE, START_DELIMITER_NEW_LINE, State};

const PYTHON_VERSION: &str = "3.9";
const ALPINE_VERSION: &str = "3.13";

// in seconds
const SEND_RECEIVERS_VALUES_INTERVAL: u64 = 10;

mod limits {
    pub const MAX_MEMORY_USAGE: u32 = 512;
    // number of core
    pub const MAX_CPU_CORE: u32 = 1;
}

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

    fn gather_outputs(child: Child) -> Result<String, Error> {
        let mut output = String::new();
        child.stdout.unwrap().read_to_string(&mut output)
            .map_err(|e| Error::IO(e))?;
        child.stderr.unwrap().read_to_string(&mut output)
            .map_err(|e| Error::IO(e))?;

        Ok(output)
    }

    fn wait_child_exit(mut child: Child, seconds: u32) -> Result<String, Error> {
        // wait given amount of seconds until transmitter code exits
        let mut exited = false;
        for _ in 0..seconds {
            match child.try_wait() {
                Ok(Some(status)) => {
                    exited = true;

                    if let Some(137) = status.code() {
                        return Err(Error::Custom(String::from("child is killed, probably due to out of memory")));
                    }

                    // if not successful, gather the outputs
                    if !status.success() {
                        return Err(Error::Output(Self::gather_outputs(child)?));
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
            return Err(Error::Custom(format!("Child is failed to exit in {} seconds\n {}", seconds, Self::gather_outputs(child)?)));
        }

        Self::gather_outputs(child)
    }

    fn run_transmitter_code(&self, script_dir: &str) -> Result<String, Error> {
        let child = std::process::Command::new(self.docker_path.as_str())
            .args(&["run", "--rm", "-e", "PYTHONDONTWRITEBYTECODE=1", "--memory-swap", "-1", "-m", format!("{}m", MAX_MEMORY_USAGE).as_str(), format!("--cpus={}", MAX_CPU_CORE).as_str()])
            .args(&["--mount", format!("type=bind,source={},target=/usr/local/lib/python{}/site-packages/,readonly", self.python_lib_path.as_str(), PYTHON_VERSION).as_str()])
            .args(&["--mount", format!("type=bind,source={},target=/usr/local/scripts/,readonly", script_dir).as_str()])
            .arg(format!("python:{}-alpine{}", PYTHON_VERSION, ALPINE_VERSION).as_str())
            .args(&["python", "/usr/local/scripts/job.py", "--transmitter"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| Error::Custom(format!("Failed to spawn transmitter process, {:?}", e)))?;

        Self::wait_child_exit(child, 60)
    }

    fn start_receiver(&self, script_dir: &str) -> Result<std::process::Child, Error> {
        let devices = (&self.rx_dev_paths)
            .into_iter()
            .map(|dev| format!("--device={}", dev))
            .collect::<Vec<String>>();

        std::process::Command::new(self.docker_path.as_str())
            .args(&["run", "--rm", "-e", "PYTHONDONTWRITEBYTECODE=1", "-p", "8011:8011", "--memory-swap", "-1", "-m", format!("{}m", MAX_MEMORY_USAGE).as_str(), format!("--cpus={}", MAX_CPU_CORE).as_str()])
            .args(&devices)
            .args(&["--mount", format!("type=bind,source={},target=/usr/local/lib/python{}/site-packages/,readonly", self.python_lib_path.as_str(), PYTHON_VERSION).as_str()])
            .args(&["--mount", format!("type=bind,source={},target=/usr/local/scripts/,readonly", script_dir).as_str()])
            .arg(format!("python:{}-alpine{}", PYTHON_VERSION, ALPINE_VERSION).as_str())
            .args(&["python", "/usr/local/scripts/job.py", "--receiver"])
            .args(&self.rx_dev_paths)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| Error::Custom(format!("Failed to spawn receiver process, {:?}", e)))
    }

    fn start_transmitter(&self) -> Result<serial::SystemPort, Error> {
        let mut port = serial::open(self.tx_dev_path.as_str())
            .map_err(|e| Error::Serial(e))?;

        // Wait 5 seconds for SETUP_MESSAGE
        port.set_timeout(Duration::from_secs(5))
            .map_err(|e| Error::Serial(e))?;

        let mut buffer = [0 as u8; incoming::arduino::SETUP_MESSAGE.len()];

        // read the START_MESSAGE message, we do not check if received buffer equals to START_MESSAGE since it is mostly equal.
        port.read(&mut buffer)
            .map_err(|e| Error::IO(e))?;

        // Other IO operations should have 1 second for timeout
        port.set_timeout(Duration::from_secs(1))
            .map_err(|e| Error::Serial(e))?;

        Ok(port)
    }

    fn run_commands(&self, state: State, port: &mut serial::SystemPort, receiver: &mut std::process::Child) -> Result<ExitReason, Error> {
        let mut buff = [0 as u8; incoming::arduino::END_MESSAGE.len()];

        // clear previous characters from transmitter
        port.write("\n".as_bytes())
            .map_err(|e| Error::IO(e))?;
        port.write(START_DELIMITER_NEW_LINE.as_bytes())
            .map_err(|e| Error::IO(e))?;


        for command in state.into_iter() {
            info!("{:?}", command);
            let connection = self.connection.clone();

            let res = futures::executor::block_on(async move {
                connection.send(IsJobAborted)
                    .await
            });

            match res {
                Ok(true) => return Err(Error::Custom(String::from("User aborted the job"))),
                Ok(false) => {},
                Err(e) => error!("Error while checking if job is aborted, {:?}", e)
            }

            port.write(command.as_bytes())
                .map_err(|e| Error::IO(e))?;

            let mut total_read_size = 0;

            // Loop until command is executed or receiver is terminated
            loop {
                match receiver.try_wait() {
                    Ok(Some(status)) => {
                        port.write(END_DELIMITER_NEW_LINE.as_bytes())
                            .map_err(|e| Error::IO(e))?;

                        return Ok(ExitReason::ChildExit(status));
                    }
                    Ok(None) => {}
                    Err(e) => error!("Error while trying to wait for child, {:?}", e)
                }

                match port.read(&mut buff) {
                    Ok(size) => {
                        total_read_size += size;

                        if total_read_size >= incoming::arduino::END_MESSAGE.len() {
                            break;
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
            }
        }

        port.write(END_DELIMITER_NEW_LINE.as_bytes())
            .map_err(|e| Error::IO(e))?;

        Ok(ExitReason::EndOfExperiment)
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
            .map_err(|e| Error::Custom(format!("Unable to decode state, {:?}", e)))?;

        info!("starting the transmitter");
        let mut port = self.start_transmitter()?;

        info!("starting the receiver");
        let mut receiver = self.start_receiver(script_dir.as_str())?;

        info!("running commands");
        let output = match self.run_commands(state, &mut port, &mut receiver) {
            Ok(ExitReason::EndOfExperiment) => {
                info!("experiment is ended");


                std::net::TcpStream::connect_timeout(&SocketAddr::from(([127, 0, 0, 1], 8011)), Duration::from_secs(10))
                    .map_err(|_| {
                        let _ = receiver.kill();
                        Error::Custom(String::from("Failed tp connect receiver"))
                    })?
                    .write(outgoing::tcp::END_MESSAGE.as_bytes())
                    .map_err(|_| {
                        let _ = receiver.kill();
                        Error::Custom(String::from("Failed to send END_MESSAGE to receiver code"))
                    })?;

                info!("waiting for receiver to exit and generating the output");

                Self::wait_child_exit(receiver, 60)?
            }
            Ok(ExitReason::ChildExit(status)) => {
                info!("child exited before experiment end");

                // user receiver code should not exit until receiving an END_MESSAGE over tcp
                if status.success() {
                    return Err(Error::Custom(String::from("User receiver code exited successfully without waiting end of experiment, this should not be the case")));
                }

                if let Some(137) = status.code() {
                    return Err(Error::Custom(String::from("child is killed, probably due to out of memory")));
                }

                info!("child crashed");
                return Err(Error::Output(Self::gather_outputs(receiver)?));
            }
            Err(e) => {
                // just kill everything without checking error and return error
                let _ = port.write(END_DELIMITER_NEW_LINE.as_bytes());
                let _ = receiver.kill();

                return Err(e);
            }
        };

        info!("removing script dir");
        self.remove_dir(script_dir.as_str())?;

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
    Output(String),
    Custom(String),
}

enum ExitReason {
    EndOfExperiment,
    ChildExit(ExitStatus),
}

use std::io::{Read, Write};
use std::process::{ExitStatus, Stdio};
use std::sync::Mutex;
use std::time::Duration;

use actix::prelude::*;
use log::{error, info};
use serial::core::SerialDevice;

use crate::connection::Connection;
use crate::executor::limits::{MAX_CPU_CORE, MAX_MEMORY_USAGE};
use crate::messages::{RunMessage, RunnerReceiversValueMessage, RunResultMessage};
use crate::ModelId;
use crate::state::{Decoder, END_DELIMITER_NEW_LINE, START_DELIMITER_NEW_LINE, State};

const PYTHON_VERSION: &str = "3.9";
const ALPINE_VERSION: &str = "3.13";

const SEND_RECEIVERS_VALUES_INTERVAL: u64 = 10;

mod limits {
    // in milliseconds
    pub const MAX_EXECUTION_TIME: u32 = 50000;
    // in milliseconds
    pub const MAX_EMIT_TIME: u32 = 10000;
    // in MB
    pub const MAX_MEMORY_USAGE: u32 = 512;
    // number of core
    pub const MAX_CPU_CORE: u32 = 1;
}

mod incoming {
    pub const SETUP_MESSAGE: &str = "arduino_available";
    pub const END_MESSAGE: &str = "end_of_experiment";
}

mod outgoing {
    pub const END_MESSAGE: &str = "end_of_experiment";
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
            .args(&["run", "--rm", "-e", "PYTHONDONTWRITEBYTECODE=1", "-m", format!("{}m", MAX_MEMORY_USAGE).as_str(), format!("--cpus={}", MAX_CPU_CORE).as_str()])
            .args(&["--mount", format!("type=bind,source={},target=/usr/local/lib/python{}/site-packages/,readonly", self.python_lib_path.as_str(), PYTHON_VERSION).as_str()])
            .args(&["--mount", format!("type=bind,source={},target=/usr/local/scripts/,readonly", script_dir).as_str()])
            .arg(format!("python:{}-alpine{}", PYTHON_VERSION, ALPINE_VERSION).as_str())
            .args(&["python", "/usr/local/scripts/job.py", "--transmitter"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| Error::Custom(format!("Failed to spawn transmitter process, {:?}", e)))?;

        // wait 10 second until transmitter code exits
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
        let devices = (&self.rx_dev_paths)
            .into_iter()
            .map(|dev| format!("--device={}", dev))
            .collect::<Vec<String>>();

        std::process::Command::new(self.docker_path.as_str())
            .args(&["run", "--rm", "-e", "PYTHONDONTWRITEBYTECODE=1", "-p", "8011:8011", "-m", format!("{}m", MAX_MEMORY_USAGE).as_str(), format!("--cpus={}", MAX_CPU_CORE).as_str()])
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

    fn run_commands(&self, state: State, port: &mut serial::SystemPort, child: &mut std::process::Child) -> Result<ExitReason, Error> {
        let mut buff = [0 as u8; incoming::END_MESSAGE.len()];

        // clear previous characters from transmitter
        port.write("\n".as_bytes())
            .map_err(|e| Error::IO(e))?;
        port.write(START_DELIMITER_NEW_LINE.as_bytes())
            .map_err(|e| Error::IO(e))?;

        for command in state.into_iter() {
            port.write(command.as_bytes())
                .map_err(|e| Error::IO(e))?;

            loop {
                match child.try_wait() {
                    Ok(Some(status)) => {
                        port.write(END_DELIMITER_NEW_LINE.as_bytes())
                            .map_err(|e| Error::IO(e))?;

                        return Ok(ExitReason::ChildExit(status));
                    }
                    Ok(None) => {}
                    Err(e) => error!("Error while trying to wait for child, {:?}", e)
                }

                match port.read(&mut buff) {
                    Ok(_) => {
                        // transmitter most likely sent an end of experiment message, not need to check it
                        break;
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

        port.write("end_delimiter\n".as_bytes())
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

        // apply sanity checks
        info!("applying sanity checks");
        let execution_time = state.execution_time();
        if execution_time > limits::MAX_EXECUTION_TIME {
            return Err(Error::Custom(format!("Max execution time is reached, execution time: {}", execution_time)));
        }

        let emit_time = state.emit_time();
        if emit_time > limits::MAX_EMIT_TIME {
            return Err(Error::Custom(format!("Max emit time is reached, emit time: {}", emit_time)));
        }

        info!("starting the transmitter");
        let mut port = self.start_transmitter()?;

        info!("starting the receiver");
        let mut child = self.start_receiver(script_dir.as_str())?;

        info!("running commands");
        match self.run_commands(state, &mut port, &mut child) {
            Ok(ExitReason::EndOfExperiment) => {
                info!("experiment is ended");

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

                let mut exited = false;
                for _ in 0..10 {
                    match child.try_wait() {
                        Ok(Some(status)) => {
                            info!("child is exited");
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
                    info!("killing the child");
                    let _ = child.kill();
                    return Err(Error::Custom(String::from("receiver code did not exit in 10 seconds")));
                }
            }
            Ok(ExitReason::ChildExit(status)) => {
                info!("child exited before experiment end");

                // user receiver code should not exit until receiving an END_MESSAGE over tcp
                if status.success() {
                    return Err(Error::Custom(String::from("User receiver code exited successfully without waiting end of experiment, this should not be the case")));
                }
                info!("child crashed");

                // if not successful, gather the outputs
                let mut output = String::new();
                child.stdout.unwrap().read_to_string(&mut output)
                    .map_err(|e| Error::IO(e))?;
                child.stderr.unwrap().read_to_string(&mut output)
                    .map_err(|e| Error::IO(e))?;

                return Err(Error::Output(output));
            }
            Err(e) => {
                error!("unknown error while waiting for child exit");
                // just kill everything without checking error and return error
                let _ = port.write(END_DELIMITER_NEW_LINE.as_bytes());
                let _ = child.kill();

                return Err(e);
            }
        }

        let mut output = String::new();
        info!("generating the output");
        child.stdout.unwrap().read_to_string(&mut output)
            .map_err(|e| Error::IO(e))?;

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

            act.connection.do_send(RunnerReceiversValueMessage { values });

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
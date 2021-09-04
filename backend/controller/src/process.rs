use std::os::unix::io::{AsRawFd, RawFd};
use std::process::{Stdio, ChildStdout, ChildStderr};
use std::time::Duration;
use std::{io, io::Read, process::Child};

use log::{error, info};

const PYTHON_VERSION: &str = "3.9";
const ALPINE_VERSION: &str = "3.13";

mod limits {
    pub const MEMORY: &str = "512m";
    pub const CPU: &str = "1";
    pub const OUTPUT: usize = 1024 * 1024 * 1;
}

pub struct DockerBuilder<'a> {
    docker_path: &'a str,
    python_lib_path: &'a str,
    script_dir: &'a str,
    exec: &'a [&'a str],
    name: Option<&'a str>,
    devices: Option<&'a [&'a str]>,
}

impl<'a> DockerBuilder<'a> {
    pub fn new(docker_path: &'a str, python_lib_path: &'a str, script_dir: &'a str, exec: &'a [&'static str]) -> DockerBuilder<'a> {
        DockerBuilder {
            docker_path,
            python_lib_path,
            script_dir,
            exec,
            name: None,
            devices: None,
        }
    }

    pub fn name(mut self, name: &'a str) -> DockerBuilder {
        self.name = Some(name);

        self
    }

    pub fn devices(mut self, devices: &'a [&'a str]) -> DockerBuilder {
        self.devices = Some(devices);

        self
    }

    pub fn build(self) -> Result<DockerProcess, ChildErrorKind> {
        let mut command = std::process::Command::new(self.docker_path);

        command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .arg("run")
            .arg("--rm")
            .args(&["-e", "PYTHONUNBUFFERED=1"])
            .args(&["-e", "PYTHONDONTWRITEBYTECODE=1"])
            .args(&["-p", "8011:8011"])
            .args(&["--memory-swap", "-1"])
            .args(&["--memory", limits::MEMORY])
            .args(&["--cpus", limits::CPU])
            .args(&["--mount", format!("type=bind,source={},target=/usr/local/lib/python{}/site-packages/,readonly", self.python_lib_path, PYTHON_VERSION).as_str()])
            .args(&["--mount", format!("type=bind,source={},target=/usr/local/scripts/,readonly", self.script_dir).as_str()]);

        if let Some(devices) = self.devices {
            let devices = devices
                .into_iter()
                .map(|dev| format!("--device={}", dev))
                .collect::<Vec<String>>();

            command.args(&devices);
        }

        let name = self.name.unwrap_or("nrgtestbed-container");

        command
            .args(&["--name", name])
            .arg(format!("python:{}-alpine{}", PYTHON_VERSION, ALPINE_VERSION).as_str())
            .args(self.exec);

        if let Some(devices) = self.devices {
            command.args(devices);
        }

        let mut child = command.spawn()
            .map_err(|e| ChildErrorKind::IOError(e, "spawning child"))?;

        let stdout = child.stdout.take().unwrap();
        let stderr = child.stderr.take().unwrap();

        unsafe {
            set_non_blocking(stdout.as_raw_fd())
                .map_err(|e| ChildErrorKind::IOError(e, "setting stdout to non blocking"))?;

            set_non_blocking(stderr.as_raw_fd())
                .map_err(|e| ChildErrorKind::IOError(e, "setting stderr to non blocking"))?;
        }

        Ok(DockerProcess {
            child,
            stdout,
            stderr,
            output: String::new(),
            docker_path: String::from(self.docker_path),
            name: String::from(name)
        })
    }
}

pub struct DockerProcess {
    child: Child,
    stdout: ChildStdout,
    stderr: ChildStderr,
    output: String,
    docker_path: String,
    name: String
}

impl DockerProcess {
    pub fn wait(mut self, seconds: u64) -> Result<String, ChildError> {
        match self._wait(seconds) {
            Ok(_) => Ok(self.output),
            Err(e) => {
                // only out of memory and crashed kinds do not need to kill the child process
                match e {
                    ChildErrorKind::OutOfMemory | ChildErrorKind::Crashed => {},
                    _ => self.kill().map_err(|e| ChildError { output: self.output.clone(), kind: e })?
                }

                Err(ChildError { output: self.output, kind: e })
            }
        }
    }

    fn _wait(&mut self, seconds: u64) -> Result<(), ChildErrorKind> {
        for _ in 0..seconds {
            self.read_pipes()?;

            match self.child.try_wait() {
                Ok(Some(status)) => {
                    if let Some(137) = status.code() {
                        return Err(ChildErrorKind::OutOfMemory);
                    }

                    if !status.success() {
                        return Err(ChildErrorKind::Crashed);
                    }

                    return Ok(());
                }
                Ok(None) => {}
                Err(e) => {
                    error!("an error occurred while try_wait on child, {:?}", e);
                }
            }

            std::thread::sleep(Duration::from_secs(1));
        }

        info!("process did not exit in given time limit");

        Err(ChildErrorKind::TimeOut)
    }

    pub fn read_pipes(&mut self) -> Result<(), ChildErrorKind> {
        let max_read = self.remaining_output_limit();
        Self::read(&mut self.stdout, &mut self.output, max_read)?;

        let max_read = self.remaining_output_limit();
        Self::read(&mut self.stderr, &mut self.output, max_read)?;

        if limits::OUTPUT == self.output.len() {
            Err(ChildErrorKind::OutputLimitReached)
        } else {
            Ok(())
        }
    }

    fn remaining_output_limit(&self) -> usize {
        let opt = limits::OUTPUT.checked_sub(self.output.len());

        if let None = opt {
            error!(
                "BUG output limit length is smaller than output length, limit {}, output {}",
                limits::OUTPUT,
                self.output.len()
            )
        }

        opt.unwrap_or(0)
    }

    fn read<T: Read>(
        src: &mut T,
        output: &mut String,
        max_read: usize,
    ) -> Result<(), ChildErrorKind> {
        const BUFF_LENGTH: usize = 1024;
        let mut buff = [0; BUFF_LENGTH];
        let mut total_read = 0;

        loop {
            match src.read(&mut buff[0..std::cmp::min(max_read - total_read, BUFF_LENGTH)]) {
                Ok(0) => break,
                Ok(n) => {
                    total_read += n;
                    output.push_str(
                        std::str::from_utf8(&mut buff[0..n]).map_err(|_| ChildErrorKind::InvalidUtf8Character("reading from src"))?,
                    );
                }
                Err(e) if std::io::ErrorKind::WouldBlock == e.kind() => break,
                Err(e) => {
                    error!("failed to read from fd, {:?}", e);
                    return Err(ChildErrorKind::IOError(e, "reading from src"));
                }
            }
        }

        Ok(())
    }

    pub fn kill(&mut self) -> Result<(), ChildErrorKind> {
        self.child.kill()
            .map_err(|e| ChildErrorKind::IOError(e, "killing child process"))?;

        let output = std::process::Command::new(self.docker_path.as_str())
            .args(&["kill", self.name.as_str()])
            .output()
            .map_err(|e| ChildErrorKind::IOError(e, "calling kill command on docker"))?;

        if !output.status.success() {
            error!("failed to kill container");

            let stdout = std::str::from_utf8(&output.stdout)
                .map_err(|_| ChildErrorKind::InvalidUtf8Character("reading stdout of docker kill command"))?;

            let stderr = std::str::from_utf8(&output.stderr)
                .map_err(|_| ChildErrorKind::InvalidUtf8Character("reading stderr of docker kill command"))?;

            error!("stdout: {}, stderr: {}", stdout, stderr);
        }

        // collect the child exit status. This is done to prevent child becoming zombie
        self.child.try_wait()
            .map_err(|e| ChildErrorKind::IOError(e, "try waiting on child after killing"))?;

        Ok(())
    }

    pub fn is_terminated(&mut self) -> bool {
        match self.child.try_wait() {
            Ok(Some(_)) => true,
            Ok(None) => false,
            Err(e) => {
                error!(
                    "failed to try wait on child while checking it is terminated, {:?}",
                    e
                );
                false
            }
        }
    }
}

#[derive(Debug)]
pub struct ChildError {
    output: String,
    kind: ChildErrorKind
}

#[derive(Debug)]
pub enum ChildErrorKind {
    OutputLimitReached,
    InvalidUtf8Character(&'static str),
    IOError(io::Error, &'static str),
    Crashed,
    OutOfMemory,
    TimeOut,
}

unsafe fn set_non_blocking(fd: RawFd) -> Result<(), io::Error> {
    if libc::fcntl(
        fd,
        libc::F_SETFL,
        libc::fcntl(fd, libc::F_GETFL) | libc::O_NONBLOCK,
    ) < 0
    {
        error!(
            "call to fcntl for setting fd to non blocking failed, errno {}",
            *libc::__errno_location()
        );

        return Err(io::Error::from_raw_os_error(*libc::__errno_location()));
    }

    Ok(())
}

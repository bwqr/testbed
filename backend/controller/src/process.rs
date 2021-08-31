use std::os::unix::io::{AsRawFd, RawFd};
use std::process::{ExitStatus, Stdio, ChildStdout, ChildStderr};
use std::time::Duration;
use std::{io, io::Read, process::Child};

use log::error;

const PYTHON_VERSION: &str = "3.9";
const ALPINE_VERSION: &str = "3.13";

const INVALID_UTF8_ENCODING: &'static str = "invalid utf-8 encoding in the output";

mod limits {
    pub const MEMORY: &str = "512m";
    pub const CPU: &str = "1";
    pub const OUTPUT: usize = 1024 * 1024 * 1;
}

pub struct DockerBuilder<'a> {
    docker_path: &'a str,
    python_lib_path: &'a str,
    script_dir: &'a str,
    exec: &'a [&'static str],
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

    pub fn build(self) -> Result<DockerProcess, &'static str> {
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
            .map_err(|_| "faile to spawn child")?;

        let stdout = child.stdout.take().unwrap();
        let stderr = child.stderr.take().unwrap();

        unsafe {
            set_non_blocking(stdout.as_raw_fd())
                .map_err(|_| "failed to set stdout to non blocking")?;

            set_non_blocking(stderr.as_raw_fd())
                .map_err(|_| "failed to set stderr to non blocking")?;
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
    pub fn wait(mut self, seconds: u64) -> Result<String, &'static str> {
        let mut exited = false;
        let mut fail_reason: Option<&'static str> = None;

        for _ in 0..seconds {
            self.read_pipes()?;

            match self.child.try_wait() {
                Ok(Some(status)) => {
                    exited = true;

                    if let Some(137) = status.code() {
                        fail_reason = Some("child is killed, probably due to out of memory");
                    }

                    if !status.success() {
                        fail_reason = Some("child is crashed");
                    }

                    break;
                }
                Ok(None) => {}
                Err(e) => {
                    error!("an error occurred while try_wait on child, {:?}", e);
                }
            }

            std::thread::sleep(Duration::from_secs(1));
        }

        if !exited {
            let _ = self.kill();

            fail_reason = Some("child did not exit for given time, it is killed");
        }

        if let Some(reason) = fail_reason {
            error!("{}", self.output);
            return Err(reason);
        }

        Ok(self.output)
    }

    pub fn read_pipes(&mut self) -> Result<(), &'static str> {
        let max_read = self.remaining_output_limit()?;
        Self::read(&mut self.stdout, &mut self.output, max_read)?;

        let max_read = self.remaining_output_limit()?;
        Self::read(&mut self.stderr, &mut self.output, max_read)?;

        if limits::OUTPUT == self.output.len() {
            Err("output limit is reached")
        } else {
            Ok(())
        }
    }

    fn remaining_output_limit(&self) -> Result<usize, &'static str> {
        let opt = limits::OUTPUT.checked_sub(self.output.len());

        if let Some(res) = opt {
            Ok(res)
        } else {
            // Controller should not put itself into this state. This is just just to inform us.
            error!(
                "BUG output limit length is smaller than output length, limit {}, output {}",
                limits::OUTPUT,
                self.output.len()
            );
            Err("overflow occurred while calculating the remaining output limit")
        }
    }

    fn read<T: Read>(
        src: &mut T,
        output: &mut String,
        max_read: usize,
    ) -> Result<(), &'static str> {
        const BUFF_LENGTH: usize = 1024;
        let mut buff = [0; BUFF_LENGTH];
        let mut total_read = 0;

        loop {
            match src.read(&mut buff[0..std::cmp::min(max_read - total_read, BUFF_LENGTH)]) {
                Ok(0) => break,
                Ok(n) => {
                    total_read += n;
                    output.push_str(
                        std::str::from_utf8(&mut buff[0..n]).map_err(|_| INVALID_UTF8_ENCODING)?,
                    );
                }
                Err(e) if std::io::ErrorKind::WouldBlock == e.kind() => break,
                Err(e) => {
                    error!("failed to read from fd, {:?}", e);
                    return Err("an error occurred while reading from fd");
                }
            }
        }

        Ok(())
    }

    pub fn kill(&mut self) -> io::Result<()> {
        self.child.kill()?;

        std::process::Command::new(self.docker_path.as_str())
            .args(&["kill", self.name.as_str()])
            .output()?;

        // collect the child exit status. This is done to prevent child becoming zombie
        self.child.try_wait()?;

        Ok(())
    }

    pub fn is_terminated(&mut self) -> Option<ExitStatus> {
        match self.child.try_wait() {
            Ok(Some(status)) => Some(status),
            Ok(None) => None,
            Err(e) => {
                error!(
                    "failed to try wait on child while checking is alive, {:?}",
                    e
                );
                None
            }
        }
    }
}

unsafe fn set_non_blocking(fd: RawFd) -> Result<(), ()> {
    if libc::fcntl(
        fd,
        libc::F_SETFL,
        libc::fcntl(fd, libc::F_GETFL) | libc::O_NONBLOCK,
    ) < 0
    {
        error!(
            "call to fcntl for setting non blocking stdout failed, errno {}",
            *libc::__errno_location()
        );

        return Err(());
    }

    Ok(())
}

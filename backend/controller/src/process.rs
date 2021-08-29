use std::os::unix::io::{AsRawFd, RawFd};
use std::process::ExitStatus;
use std::time::Duration;
use std::{io, io::Read, process::Child};

use log::{error, info};

const INVALID_UTF8_ENCODING: &'static str = "invalid utf-8 encoding in the output";

pub struct ChildWrapper {
    child: Child,
    output: String,
}

impl ChildWrapper {
    pub fn new(child: Child) -> Result<Self, &'static str> {
        if let Some(stdout) = &child.stdout {
            unsafe {
                Self::set_non_blocking(stdout.as_raw_fd())
                    .map_err(|_| "failed to set stdout to non blocking")?;
            }
        }

        if let Some(stderr) = &child.stderr {
            unsafe {
                Self::set_non_blocking(stderr.as_raw_fd())
                    .map_err(|_| "failed to set stderr to non blocking")?;
            }
        }

        Ok(ChildWrapper {
            child,
            output: String::new(),
        })
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

    pub fn wait(mut self, seconds: u32) -> Result<String, &'static str> {
        let mut exited = false;
        let mut fail_reason: Option<&'static str> = None;

        for _ in 0..seconds {
            self.read_pipes()?;

            match self.child.try_wait() {
                Ok(Some(status)) => {
                    if let Some(137) = status.code() {
                        fail_reason = Some("child is killed, probably due to out of memory");
                        break;
                    }

                    if !status.success() {
                        fail_reason = Some("child is crashed");
                        break;
                    }

                    exited = true;

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
            let _ = self.child.kill();

            fail_reason = Some("child did not exit for given time, it is killed");
        }

        if let Some(reason) = fail_reason {
            self.output.push_str(reason);
        }

        Ok(self.output)
    }

    pub fn read_pipes(&mut self) -> Result<(), &'static str> {
        if let Some(stdout) = &mut self.child.stdout {
            Self::read(stdout, &mut self.output)?;
        }

        if let Some(stderr) = &mut self.child.stderr {
            Self::read(stderr, &mut self.output)?;
        }

        Ok(())
    }

    pub fn kill(&mut self) -> io::Result<()> {
        self.child.kill()
    }

    fn read<T: Read>(src: &mut T, output: &mut String) -> Result<(), &'static str> {
        let mut buf = [0; 1024];

        loop {
            match src.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    output.push_str(
                        std::str::from_utf8(&mut buf[0..n]).map_err(|_| INVALID_UTF8_ENCODING)?,
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
}

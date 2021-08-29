use std::os::unix::io::AsRawFd;
use std::process::ExitStatus;
use std::{io, io::Read, process::Child};

use log::error;

const INVALID_UTF8_ENCODING: &'static str = "invalid utf-8 encoding in the output";

pub struct ChildWrapper(Child);

impl ChildWrapper {
    pub fn new(mut child: Child) -> Result<Self, &'static str> {
        unsafe {
            Self::set_non_blocking(&mut child).map_err(|_| "failed to set fds to non blocking")?;
        }

        Ok(ChildWrapper(child))
    }

    pub fn read_into(&mut self, output: &mut String) -> Result<(), &'static str> {
        if let Some(stdout) = &mut self.0.stdout {
            Self::read(stdout, output)?;
        }

        if let Some(stderr) = &mut self.0.stderr {
            Self::read(stderr, output)?;
        }

        Ok(())
    }

    pub fn read_until_eof(&mut self) -> Result<String, &'static str> {
        unsafe {
            Self::set_blocking(&mut self.0).map_err(|_| "failed to set fds to blocking")?;
        }

        let mut output = String::from("");

        if let Some(stdout) = &mut self.0.stdout {
            stdout
                .read_to_string(&mut output)
                .map_err(|_| INVALID_UTF8_ENCODING)?;
        }

        if let Some(stderr) = &mut self.0.stderr {
            stderr
                .read_to_string(&mut output)
                .map_err(|_| INVALID_UTF8_ENCODING)?;
        }

        unsafe {
            Self::set_non_blocking(&mut self.0).map_err(|_| "failed to set fds to non blocking")?;
        }

        Ok(output)
    }

    pub fn try_wait(&mut self) -> io::Result<Option<ExitStatus>> {
        self.0.try_wait()
    }

    pub fn kill(&mut self) -> io::Result<()> {
        self.0.kill()
    }

    fn read<T: Read>(src: &mut T, output: &mut String) -> Result<(), &'static str> {
        loop {
            let mut buf = [0; 16000];

            match src.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    output.push_str(
                        std::str::from_utf8(&mut buf[0..n]).map_err(|_| INVALID_UTF8_ENCODING)?,
                    );
                }
                Err(e) if std::io::ErrorKind::WouldBlock == e.kind() => break,
                Err(e) => error!("failed to read from fd, {:?}", e),
            }
        }

        Ok(())
    }

    unsafe fn set_non_blocking(child: &mut Child) -> Result<(), ()> {
        if let Some(stdout) = &mut child.stdout {
            let fd = stdout.as_raw_fd();

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
        }

        if let Some(stderr) = &mut child.stderr {
            let fd = stderr.as_raw_fd();

            if libc::fcntl(
                fd,
                libc::F_SETFL,
                libc::fcntl(fd, libc::F_GETFL) | libc::O_NONBLOCK,
            ) < 0
            {
                error!(
                    "call to fcntl for setting non blocking  on stderr failed, errno {}",
                    *libc::__errno_location()
                );

                return Err(());
            }
        }

        Ok(())
    }

    unsafe fn set_blocking(child: &mut Child) -> Result<(), ()> {
        if let Some(stdout) = &mut child.stdout {
            let fd = stdout.as_raw_fd();

            if libc::fcntl(
                fd,
                libc::F_SETFL,
                libc::fcntl(fd, libc::F_GETFL) & !libc::O_NONBLOCK,
            ) < 0
            {
                error!(
                    "call to fcntl for setting non blocking on stdout failed, errno {}",
                    *libc::__errno_location()
                );

                return Err(());
            }
        }

        if let Some(stderr) = &mut child.stderr {
            let fd = stderr.as_raw_fd();

            if libc::fcntl(
                fd,
                libc::F_SETFL,
                libc::fcntl(fd, libc::F_GETFL) & !libc::O_NONBLOCK,
            ) < 0
            {
                error!(
                    "call to fcntl for setting non blocking on stderr failed, errno {}",
                    *libc::__errno_location()
                );

                return Err(());
            }
        }

        Ok(())
    }
}

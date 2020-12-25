use std::io::Write;
use std::thread;

use actix::prelude::*;
use log::{error, info};

use crate::connection::Connection;
use crate::messages::{RunMessage, UpdateConnectionMessage};

pub struct ExecutorMock {
    connection: Option<Addr<Connection>>
}

impl ExecutorMock {
    pub fn new() -> Self {
        ExecutorMock {
            connection: None
        }
    }
}

impl Actor for ExecutorMock {
    type Context = Context<Self>;
}

impl Handler<UpdateConnectionMessage> for ExecutorMock {
    type Result = ();

    fn handle(&mut self, msg: UpdateConnectionMessage, _: &mut Self::Context) {
        self.connection = Some(msg.connection);
    }
}

impl Handler<RunMessage> for ExecutorMock {
    type Result = ();

    fn handle(&mut self, msg: RunMessage, _: &mut Self::Context) {
        info!("got some run for ExecutorMock, id: {}, code: {}", msg.job_id, msg.code);

        std::fs::create_dir_all("/tmp/testbed").unwrap();

        let mut f = std::fs::File::create(format!("/tmp/testbed/job_{}.py", msg.job_id)).unwrap();

        f.write(msg.code.as_bytes()).unwrap();

        match std::process::Command::new("/usr/bin/docker")
            .arg("run")
            .arg("--rm")
            .args(&["--volume", "/tmp/testbed/:/usr/local/scripts/"])
            .arg("python:rc-alpine")
            .args(&["python", format!("/usr/local/scripts/job_{}.py", msg.job_id).as_str()])
            .arg(msg.code)
            .output() {
            Ok(output) => info!(
                "successful execution, status {:?}, stdout {:?}, stderr {:?}",
                output.status,
                String::from_utf8(output.stdout),
                String::from_utf8(output.stderr)
            ),
            Err(e) => error!("failed to execute, {}", e)
        };

        // std::fs::remove_file(format!("/tmp/testbed/job_{}.py", msg.job_id)).unwrap();
    }
}

pub struct Executor {
    connection: Option<Addr<Connection>>
}

impl Executor {
    pub fn new() -> Self {
        Executor {
            connection: None
        }
    }
}

impl Actor for Executor {
    type Context = Context<Self>;

    fn started(&mut self, _: &mut Self::Context) {}

    fn stopped(&mut self, _: &mut Self::Context) {}
}

impl Handler<UpdateConnectionMessage> for Executor {
    type Result = ();

    fn handle(&mut self, msg: UpdateConnectionMessage, _: &mut Self::Context) {
        self.connection = Some(msg.connection);
    }
}

impl Handler<RunMessage> for Executor {
    type Result = ();

    fn handle(&mut self, msg: RunMessage, _: &mut Self::Context) {
        info!("got some run for Executor {}", msg.job_id);

        let gpio = sysfs_gpio::Pin::new(18);

        gpio.with_exported(|| {
            gpio.set_direction(sysfs_gpio::Direction::Out)?;
            gpio.set_value(1)?;
            thread::sleep(std::time::Duration::from_secs(5));
            gpio.set_value(1)?;

            Ok(())
        })
            .map_err(|e| error!("Some error occurred {:?}", e));
    }
}
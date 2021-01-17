use std::io::Write;

use actix::prelude::*;
use log::{error, info};

use crate::connection::Connection;
use crate::messages::{RunMessage, RunResultMessage};
use crate::ModelId;

pub struct Executor {
    connection: Addr<Connection>
}

impl Executor {
    pub fn new(connection: Addr<Connection>) -> Self {
        Executor {
            connection
        }
    }

    fn handle_execution(job_id: ModelId, code: String) -> Result<String, Error> {
        let dir = format!("/tmp/testbed/{}", job_id);
        let file = dir.clone() + "/job.py";

        std::fs::create_dir_all(dir.as_str())
            .map_err(|e| Error::IO(e))?;

        let mut f = std::fs::File::create(file.as_str())
            .map_err(|e| Error::IO(e))?;

        f.write(code.as_bytes())
            .map_err(|e| Error::IO(e))?;

        let output = std::process::Command::new("/usr/bin/docker")
            .arg("run")
            .arg("--rm")
            .args(&["--volume", (dir.clone() + ":/usr/local/scripts/").as_str()])
            .arg("python:rc-alpine")
            .args(&["python", "/usr/local/scripts/job.py"])
            .output()
            .map_err(|e| Error::IO(e))?;

        if !output.status.success() {
            return Err(Error::Output(String::from_utf8(output.stderr).map_err(|e| Error::String(e))?));
        }

        info!(
            "successful execution, status {:?}, stdout {:?}, stderr {:?}",
            output.status, String::from_utf8(output.stdout.clone()).map_err(|e| Error::String(e))?,
            String::from_utf8(output.stderr).map_err(|e| Error::String(e))?);

        std::fs::remove_dir_all(dir.as_str())
            .map_err(|e| Error::IO(e))?;

        Ok(String::from_utf8(output.stdout)
            .map_err(|e| Error::String(e))?)
    }
}

impl Actor for Executor {
    type Context = Context<Self>;
}

impl Handler<RunMessage> for Executor {
    type Result = ();

    fn handle(&mut self, msg: RunMessage, ctx: &mut Self::Context) {
        info!("got some run for ExecutorMock, id: {}, code: {}", msg.job_id, msg.code);

        let job_id = msg.job_id;

        let addr = self.connection.clone();

        let (output, successful) = match Self::handle_execution(msg.job_id, msg.code) {
            Ok(output) => (output, true),
            Err(e) => {
                error!("could not execute the job, {:?}", e);

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
pub enum Error {
    IO(std::io::Error),
    String(std::string::FromUtf8Error),
    Output(String),
}
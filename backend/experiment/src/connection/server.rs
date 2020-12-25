use std::collections::HashMap;

use actix::prelude::*;
use actix_web::web;
use diesel::prelude::*;
use log::{error, info};

use core::db::DieselEnum;
use core::schema::jobs;
use core::types::{DBPool, ModelId};

use crate::connection::messages::{JoinServerMessage, RunMessage, RunResultMessage};
use crate::connection::session::Session;
use crate::models::job::{Job, JobStatus};

#[derive(Message)]
#[rtype(result = "()")]
pub struct RunExperimentMessage {
    pub job_id: ModelId
}

pub struct ExperimentServer {
    pool: DBPool,
    pending_runs: Vec<ModelId>,
    // run_id -> (session, run_id)
    runners: HashMap<ModelId, (Addr<Session>, Option<ModelId>)>,
}

impl ExperimentServer {
    pub fn new(pool: DBPool) -> Self {
        ExperimentServer {
            pool,
            pending_runs: Vec::new(),
            runners: HashMap::new(),
        }
    }

    fn run(&mut self, job_id: ModelId, ctx: &mut <Self as Actor>::Context) {
        let mut inactive_runner_id: Option<ModelId> = None;

        for (id, v) in &self.runners {
            if let None = v.1 {
                inactive_runner_id = Some(*id);
                break;
            }
        }

        // If there is an inactive runner
        if let Some(runner_id) = inactive_runner_id {
            let runner = self.runners.get_mut(&runner_id).unwrap();
            // mark this runner as active
            runner.1 = Some(job_id);

            let addr = runner.0.clone();

            let conn = self.pool.get().unwrap();
            async move {
                let job = web::block(move || jobs::table.find(job_id).first::<Job>(&conn))
                    .await
                    .map_err(|_| Error::DB(job_id))?;

                // We have to decode the job.code in order to replace encoded html characters like < char
                addr.send(RunMessage { job_id, code: core::decode_html(job.code.as_str()).unwrap() })
                    .await
                    .map_err(|_| Error::Send(job_id))?;

                Ok(job_id)
            }
                .into_actor(self)
                .then(|result, act, _| {
                    let conn = act.pool.get().unwrap();

                    async move {
                        let (status, job_id) = match result {
                            Ok(job_id) => (JobStatus::Running, job_id),
                            Err(Error::Send(job_id)) | Err(Error::DB(job_id)) => (JobStatus::Failed, job_id),
                        };

                        if let Err(e) = web::block(move || diesel::update(jobs::table.find(job_id))
                            .set(jobs::status.eq(status.value()))
                            .execute(&conn)
                        )
                            .await {
                            error!("setting job status is failed: {:?}", e);
                        }
                    }
                        .into_actor(act)
                })
                .spawn(ctx)
        }
        // Otherwise push it into pending
        else {
            self.pending_runs.push(job_id);
        }
    }
}

impl Actor for ExperimentServer {
    type Context = Context<Self>;

    fn started(&mut self, _: &mut Self::Context) {}

    fn stopped(&mut self, _: &mut Self::Context) {}
}

impl Handler<RunExperimentMessage> for ExperimentServer {
    type Result = ();

    fn handle(&mut self, msg: RunExperimentMessage, ctx: &mut Self::Context) {
        info!("Job with id {} received ", msg.job_id);

        self.run(msg.job_id, ctx);
    }
}

impl Handler<JoinServerMessage> for ExperimentServer {
    type Result = ();

    fn handle(&mut self, msg: JoinServerMessage, _: &mut Self::Context) {
        self.runners.insert(msg.runner_id, (msg.addr, None));
    }
}

impl Handler<RunResultMessage> for ExperimentServer {
    type Result = ();

    fn handle(&mut self, msg: RunResultMessage, ctx: &mut Self::Context) {
        info!("got result {} id {}", msg.successful, msg.job_id);

        if let Some(runner) = self.runners.get_mut(&msg.runner_id) {
            let correct_run = match runner.1 {
                Some(job_id) => job_id == msg.job_id,
                None => false
            };

            if correct_run {
                runner.1 = None;

                if let Some(job_id) = self.pending_runs.pop() {
                    self.run(job_id, ctx);
                }
            }
        }

        let status = match msg.successful {
            true => JobStatus::Successful,
            false => JobStatus::Failed,
        };

        let conn = self.pool.get().unwrap();

        async move {
            if let Err(e) = web::block(move ||
                diesel::update(jobs::table.find(msg.job_id))
                    .set(jobs::status.eq(status.value()))
                    .execute(&conn)
            )
                .await {
                error!("updating jobs status is failed: {:?}", e);
            }
        }.into_actor(self)
            .spawn(ctx);
    }
}

pub enum Error {
    DB(ModelId),
    Send(ModelId),
}
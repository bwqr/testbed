use std::collections::HashMap;

use actix::prelude::*;
use actix_web::web;
use diesel::prelude::*;
use log::{error, info};
use serde::Serialize;

use core::db::DieselEnum;
use core::schema::{experiments, jobs};
use core::types::{DBPool, ModelId};
use service::{Notification, NotificationKind, NotificationMessage, NotificationServer};

use crate::connection::messages::{JoinServerMessage, RunMessage, RunResultMessage};
use crate::connection::session::Session;
use crate::models::job::{Job, JobStatus};

#[derive(Message)]
#[rtype(result = "()")]
pub struct RunExperimentMessage {
    pub job: Job
}

pub struct ExperimentServer {
    pool: DBPool,
    // run_id -> (session, running job, pending jobs)
    runners: HashMap<ModelId, (Addr<Session>, Option<ModelId>, Vec<Job>)>,
    notification: Addr<NotificationServer>,
}

impl ExperimentServer {
    pub fn new(pool: DBPool, notification: Addr<NotificationServer>) -> Self {
        ExperimentServer {
            pool,
            runners: HashMap::new(),
            notification,
        }
    }

    async fn send_status_notification(addr: Addr<NotificationServer>, user_id: ModelId, job_id: ModelId, status: JobStatus) {
        addr.do_send(Notification {
            user_id,
            message: NotificationMessage {
                kind: NotificationKind::JobUpdate,
                data: JobUpdate {
                    job_id,
                    status,
                },
            },
        });
    }

    fn run(&mut self, job: Job, ctx: &mut <Self as Actor>::Context) -> Result<(), String> {
        let runner = self.runners.get_mut(&job.runner_id)
            .ok_or(format!("Unknown runner {} for job {}", job.runner_id, job.id))?;

        // if runner is idle
        if let None = runner.1 {
            // set runner in progress
            runner.1 = Some(job.id);

            let addr = runner.0.clone();
            async move {
                // We have to decode the job.code in order to replace encoded html characters like '<' char
                addr.send(RunMessage { job_id: job.id, code: core::decode_html(job.code.as_str()).unwrap() })
                    .await
                    .map_err(|_| Error::Send(job.id))?;

                Ok(job.id)
            }
                .into_actor(self)
                .then(|result, act, _| {
                    let conn = act.pool.get().unwrap();

                    async move {
                        let (status, job_id) = match result {
                            Ok(job_id) => (JobStatus::Running, job_id),
                            Err(Error::Send(job_id)) => (JobStatus::Failed, job_id),
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
                .spawn(ctx);
        } else {
            // push job into pending queue
            runner.2.push(job)
        }

        Ok(())
    }
}

impl Actor for ExperimentServer {
    type Context = Context<Self>;

    fn started(&mut self, _: &mut Self::Context) {
        info!("ExperimentServer is started!");
    }

    fn stopped(&mut self, _: &mut Self::Context) {
        info!("ExperimentServer is stopped!");
    }
}

impl Handler<RunExperimentMessage> for ExperimentServer {
    type Result = ();

    fn handle(&mut self, msg: RunExperimentMessage, ctx: &mut Self::Context) {
        info!("Job with id {} received ", msg.job.id);

        if let Err(e) = self.run(msg.job, ctx) {
            error!("Error while running job, error {}", e);
        }
    }
}

impl Handler<JoinServerMessage> for ExperimentServer {
    type Result = ();

    fn handle(&mut self, msg: JoinServerMessage, _: &mut Self::Context) {
        self.runners.insert(msg.runner_id, (msg.addr, None, Vec::new()));
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

            // if runner is this one, mark it empty and if there is pending jobs, run it
            if correct_run {
                runner.1 = None;

                if let Some(job) = runner.2.pop() {
                    if let Err(e) = self.run(job, ctx) {
                        error!("Error while running job, error {}", e);
                    }
                }
            }
        }

        let status = match msg.successful {
            true => JobStatus::Successful,
            false => JobStatus::Failed,
        };

        let conn = self.pool.get().unwrap();
        let notification = self.notification.clone();

        async move {
            // clone required things
            let job_id = msg.job_id;
            let status_clone = status.clone();

            let res = web::block(move || {
                diesel::update(jobs::table.find(msg.job_id))
                    .set((jobs::status.eq(status.value()), jobs::output.eq(Some(msg.output))))
                    .execute(&conn)?;

                experiments::table
                    .inner_join(jobs::table)
                    .filter(jobs::id.eq(msg.job_id))
                    .select(experiments::user_id)
                    .first::<ModelId>(&conn)
            })
                .await;

            //notify the user
            if let Ok(user_id) = res {
                Self::send_status_notification(notification, user_id, job_id, status_clone)
                    .await;
            }
        }.into_actor(self)
            .spawn(ctx);
    }
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct JobUpdate {
    job_id: ModelId,
    status: JobStatus,
}

pub enum Error {
    Send(ModelId),
}
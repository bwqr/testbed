use std::collections::HashMap;

use actix::prelude::*;
use actix_web::web;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, PooledConnection};
use log::{error, info};
use serde::Serialize;

use core::db::DieselEnum;
use core::schema::{experiments, jobs};
use core::types::{DBPool, ModelId};
use service::{Notification, NotificationKind, NotificationMessage, NotificationServer};
use shared::RunnerState;

use crate::connection::messages::{DisconnectServerMessage, JoinServerMessage, RunMessage, RunResultMessage, UpdateRunnerValue};
use crate::connection::ReceiverValues;
use crate::connection::session::Session;
use crate::models::job::JobStatus;

pub use crate::connection::messages::AbortRunningJob;

struct ConnectedRunner {
    session: Addr<Session>,
    state: RunnerState,
    receiver_values: Option<Vec<u8>>,
}

#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct RunExperiment {
    pub code: String,
    pub job_id: ModelId,
    pub runner_id: ModelId,
    pub user_id: ModelId,
}

pub struct ExperimentServer {
    pool: DBPool,
    runners: HashMap<ModelId, ConnectedRunner>,
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

    async fn try_next_job(conn: PooledConnection<ConnectionManager<PgConnection>>, runner_id: ModelId) -> Option<RunExperiment> {
        web::block(move || {
            // TODO https://trello.com/c/1qCVgmn6
            jobs::table
                .inner_join(experiments::table)
                .filter(jobs::status.eq(JobStatus::Pending.value()))
                .filter(jobs::runner_id.eq(runner_id))
                .select((experiments::user_id, jobs::id, jobs::code))
                .first::<(ModelId, ModelId, String)>(&conn)
                .map(|job| RunExperiment {
                    code: job.2,
                    job_id: job.1,
                    runner_id,
                    user_id: job.0,
                })
        })
            .await
            .ok()
    }

    async fn send_status_notification(addr: Addr<NotificationServer>, user_id: ModelId, job_id: ModelId, status: JobStatus) {
        let res = addr.send(Notification {
            user_id,
            message: NotificationMessage {
                kind: NotificationKind::JobUpdate,
                data: JobUpdate {
                    job_id,
                    status,
                },
            },
        })
            .await;

        if let Err(e) = res {
            error!("Error while sending notification, {:?}", e);
        }
    }

    fn run(&mut self, experiment: RunExperiment, ctx: &mut <Self as Actor>::Context) -> Result<(), &'static str> {
        let mut runner: &mut ConnectedRunner = self.runners.get_mut(&experiment.runner_id)
            .ok_or("runner is not yet connected")?;

        if let RunnerState::Running(_) = &runner.state {
            return Err("runner is already running an experiment");
        }

        runner.state = RunnerState::Running(experiment.job_id);

        // otherwise try to run experiment
        // clone some necessary vars
        let session = runner.session.clone();
        let user_id = experiment.user_id;
        let job_id = experiment.job_id;
        // send experiment to the runner
        async move {
            session.send(RunMessage {
                job_id: experiment.job_id,
                // We have to decode the code in order to replace encoded html characters like '<' char
                code: core::decode_html(experiment.code.as_str()).unwrap(),
            })
                .await?;

            Ok(())
        }
            .into_actor(self)
            .then(move |res: Result<(), MailboxError>, act, ctx| {
                let status = match res {
                    Ok(_) => JobStatus::Running,
                    Err(e) => {
                        error!("Error while sending job to runner, {:?}", e);
                        // TODO https://trello.com/c/nqKV1x8B
                        // schedule another job, update runner state to Idle
                        JobStatus::Failed
                    }
                };

                Self::send_status_notification(act.notification.clone(), user_id, job_id, status.clone())
                    .into_actor(act)
                    .spawn(ctx);

                Self::update_job(act.pool.get().unwrap(), job_id, status)
                    .into_actor(act)
                    .spawn(ctx);

                fut::ready(())
            })
            .spawn(ctx);

        Ok(())
    }

    async fn update_job(conn: PooledConnection<ConnectionManager<PgConnection>>, job_id: ModelId, status: JobStatus) {
        let res = web::block(move ||
            diesel::update(jobs::table.find(job_id))
                .set(jobs::status.eq(status.value()))
                .execute(&conn)
        )
            .await;

        if let Err(e) = res {
            error!("Error while updating job, {:?}", e);
        }
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

impl Handler<RunExperiment> for ExperimentServer {
    type Result = ();

    fn handle(&mut self, msg: RunExperiment, ctx: &mut Self::Context) {
        info!("Job with id {} received ", msg.job_id);

        if let Err(e) = self.run(msg, ctx) {
            info!("Error while trying to run experiment, {:?}", e);
        }
    }
}

impl Handler<JoinServerMessage> for ExperimentServer {
    type Result = ();

    fn handle(&mut self, msg: JoinServerMessage, ctx: &mut Self::Context) {
        // insert runner
        self.runners.insert(msg.runner_id, ConnectedRunner {
            state: msg.state.clone(),
            session: msg.addr,
            receiver_values: None,
        });

        if let RunnerState::Idle = msg.state {
            // copy some necessary vals
            let runner_id = msg.runner_id;

           // try to run a job
            let conn = self.pool.get().unwrap();
            async move {
                Self::try_next_job(conn, runner_id)
                    .await
            }
                .into_actor(self)
                .then(move |res: Option<RunExperiment>, act: &mut Self, ctx: _| {
                    if let Some(run) = res {
                        // maybe runner disconnected, so check it
                        if act.runners.contains_key(&runner_id) {
                            if let Err(e) = act.run(run, ctx) {
                                error!("Unexpectedly Runner is in running state, {}", e)
                            }
                        }
                    }

                    fut::ready(())
                })
                .spawn(ctx);
       }
    }
}

impl Handler<DisconnectServerMessage> for ExperimentServer {
    type Result = ();

    fn handle(&mut self, msg: DisconnectServerMessage, _: &mut Self::Context) {
        self.runners.remove(&msg.runner_id);
    }
}

impl Handler<RunResultMessage> for ExperimentServer {
    type Result = ();

    fn handle(&mut self, msg: RunResultMessage, ctx: &mut Self::Context) {
        info!("got result {} id {}", msg.successful, msg.job_id);

        if let Some(runner) = self.runners.get_mut(&msg.runner_id) {
            // TODO Maybe runner sent an older job's result, so we should check if currently running job
            // is equal to received msg.job_id
            runner.state = RunnerState::Idle;
            let conn = self.pool.get().unwrap();
            let runner_id = msg.runner_id;
            async move {
                Self::try_next_job(conn, runner_id)
                    .await
            }
                .into_actor(self)
                .then(move |res: Option<RunExperiment>, act: &mut Self, ctx: _| {
                    if let Some(run) = res {
                        // maybe runner disconnected, so check it
                        if act.runners.contains_key(&runner_id) {
                            if let Err(e) = act.run(run, ctx) {
                                error!("Failed to run experiment, {}", e);
                            }
                        }
                    }

                    fut::ready(())
                })
                .spawn(ctx);
        }

        let conn = self.pool.get().unwrap();
        let notification_server = self.notification.clone();

        let status = match msg.successful {
            true => JobStatus::Successful,
            false => JobStatus::Failed,
        };

        async move {
            // clone required vals
            let job_id = msg.job_id;
            let status_clone = status.clone();

            let res = web::block(move || {
                diesel::update(jobs::table.find(msg.job_id))
                    .set(jobs::status.eq(status.value()))
                    .execute(&conn)?;

                experiments::table
                    .inner_join(jobs::table)
                    .filter(jobs::id.eq(msg.job_id))
                    .select(experiments::user_id)
                    .first::<ModelId>(&conn)
            })
                .await;

            // try to notify the user
            match res {
                Ok(user_id) => Self::send_status_notification(notification_server, user_id, job_id, status_clone)
                    .await,
                Err(e) => error!("Error while updating job with run result, {:?}", e)
            }
        }
            .into_actor(self)
            .spawn(ctx);
    }
}

impl Handler<ReceiverValues> for ExperimentServer {
    type Result = <ReceiverValues as Message>::Result;
    fn handle(&mut self, msg: ReceiverValues, _: &mut Self::Context) -> Self::Result {
        Ok(self.runners.get(&msg.runner_id).ok_or(())?.receiver_values.clone())
    }
}

impl Handler<UpdateRunnerValue> for ExperimentServer {
    type Result = ();
    fn handle(&mut self, msg: UpdateRunnerValue, _: &mut Self::Context) -> Self::Result {
        if let Some(runner) = self.runners.get_mut(&msg.runner_id) {
            runner.receiver_values = Some(msg.values)
        }
    }
}

impl Handler<AbortRunningJob> for ExperimentServer {
    type Result = ();
    fn handle(&mut self, msg: AbortRunningJob, _: &mut Self::Context) -> Self::Result {
        if let Some(runner) = self.runners.get_mut(&msg.runner_id) {
            runner.session.do_send(msg)
        }
    }
}
#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct JobUpdate {
    job_id: ModelId,
    status: JobStatus,
}

use std::collections::{HashMap, VecDeque};

use actix::prelude::*;
use actix_web::web;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, PooledConnection};
use log::{error, info};
use serde::Serialize;

use core::db::DieselEnum;
use core::schema::{experiments, jobs, runners};
use core::types::{DBPool, ModelId};
use service::{Notification, NotificationKind, NotificationMessage, NotificationServer};

use crate::connection::messages::{DisconnectServerMessage, JoinServerMessage, RunMessage, RunResultMessage};
use crate::connection::session::Session;
use crate::models::job::JobStatus;

#[derive(PartialEq, Eq)]
enum RunnerState {
    Initializing,
    Connected,
}

struct ConnectedRunner {
    session: Addr<Session>,
    active_run: Option<RunExperiment>,
    queue: VecDeque<RunExperiment>,
    state: RunnerState,
}

#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct RunExperiment {
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

    fn run(&mut self, experiment: RunExperiment, ctx: &mut <Self as Actor>::Context) {
        let mut runner = if let Some(runner) = self.runners.get_mut(&experiment.runner_id) {
            runner
        } else {
            // runner is not connected, this experiment will be handled when runner connects, just return
            return;
        };

        // check if runner is in initializing state or there are some active run, if so, push the experiment into queue
        if runner.state == RunnerState::Initializing || runner.active_run.is_some() {
            runner.queue.push_back(experiment);
            return;
        }

        // otherwise try to run experiment
        // clone some necessary vars
        let session = runner.session.clone();
        let job_id = experiment.job_id;
        let conn = self.pool.get().unwrap();

        // set runner in progress
        runner.active_run = Some(experiment.clone());

        // send experiment to the runner
        async move {
            let code = web::block(move || jobs::table.find(job_id).select(jobs::code).first::<String>(&conn))
                .await
                .map_err(|e| format!("{:?}", e))?;

            session.send(RunMessage {
                job_id,
                // We have to decode the code in order to replace encoded html characters like '<' char
                code: core::decode_html(code.as_str()).unwrap(),
            })
                .await
                .map_err(|e| format!("{:?}", e))?;

            Ok(())
        }
            .into_actor(self)
            .then(move |res: Result<(), String>, act, ctx| {
                let status = match res {
                    Ok(_) => JobStatus::Running,
                    Err(e) => {
                        error!("Error while sending job to runner, {:?}", e);
                        // TODO schedule another job
                        JobStatus::Failed
                    }
                };

                Self::send_status_notification(act.notification.clone(), experiment.user_id, experiment.job_id, status.clone())
                    .into_actor(act)
                    .spawn(ctx);

                Self::update_job(act.pool.get().unwrap(), experiment.job_id, status)
                    .into_actor(act)
                    .spawn(ctx);

                fut::ready(())
            })
            .spawn(ctx);
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

        self.run(msg, ctx);
    }
}

impl Handler<JoinServerMessage> for ExperimentServer {
    type Result = ();

    /// Here are the steps that taken while handling joining the server
    /// 1. Insert the runner into collection
    /// 2. Fetch the pending jobs from database asynchronously
    /// 3. Insert fetched pending jobs into runner while keeping every element in the queue unique
    /// This procedure will eliminate any dangling jobs in the database. Here are cases which cause dangling jobs in database
    /// 1. First fetch the jobs from database, then insert the runner into collection, insert the jobs into queue. This will
    /// cause dangling jobs in the database since new jobs can be inserted into database between the event of fetching from database
    /// and inserting runner into collection
    /// 2. Insert the runner into collection, fetch the jobs from database, finally insert the fetched jobs into queue. This may cause some jobs
    /// to be run twice, since a job can already be in the queue and in the pending state, thus will be fetched from database again.
    fn handle(&mut self, msg: JoinServerMessage, ctx: &mut Self::Context) {
        // clone some necessary vals
        let runner_id = msg.runner_id;

        // insert runner
        self.runners.insert(msg.runner_id, ConnectedRunner {
            session: msg.addr,
            active_run: None,
            queue: VecDeque::new(),
            state: RunnerState::Initializing,
        });

        // fetch pending jobs and ,if exist, running job from database
        let conn = self.pool.get().unwrap();
        async move {
            web::block(move || {
                jobs::table
                    .inner_join(experiments::table)
                    .inner_join(runners::table)
                    .filter(
                        jobs::status.eq(JobStatus::Pending.value())
                            .or(jobs::status.eq(JobStatus::Running.value()))
                    )
                    .filter(runners::id.eq(runner_id))
                    .select((jobs::id, jobs::status, experiments::user_id, runners::id))
                    .load::<(ModelId, JobStatus, ModelId, ModelId)>(&conn)
                    .map(|experiments|
                        experiments.into_iter()
                            // vec for pending jobs, option for if running job exist
                            .fold::<(Vec<RunExperiment>, Option<RunExperiment>), _>((Vec::new(), None), |mut tuple, job| {
                                let run_experiment = RunExperiment {
                                    job_id: job.0,
                                    user_id: job.2,
                                    runner_id: job.3,
                                };

                                if job.1 == JobStatus::Running {
                                    tuple.1 = Some(run_experiment);
                                } else {
                                    tuple.0.push(run_experiment);
                                }

                                tuple
                            })
                    )
            })
                .await
                .map_err(|e| format!("{:?}", e))
        }
            .into_actor(self)
            .then(move |res: Result<(Vec<RunExperiment>, Option<RunExperiment>), String>, act: _, ctx: _| {
                match res {
                    Ok(experiments) => {
                        // maybe runner disconnected, so check it
                        if let Some(runner) = act.runners.get_mut(&runner_id) {
                            runner.state = RunnerState::Connected;

                            // push jobs into pending queue
                            experiments.0.into_iter().for_each(|experiment| {
                                // check if job is already pushed into queue, since job can be sent to experiment server while we are fetching from database
                                if let None = runner.queue.iter().find(|exp| exp.job_id == experiment.job_id) {
                                    runner.queue.push_back(experiment)
                                }
                            });

                            // assign current running job
                            runner.active_run = experiments.1;

                            // run a job if runner is idle and there is a pending job
                            if let None = runner.active_run {
                                if let Some(experiment) = runner.queue.pop_front() {
                                    act.run(experiment, ctx);
                                }
                            }
                        }
                    }
                    Err(e) => error!("Error while fetching jobs, {:?}", e)
                }

                fut::ready(())
            })
            .spawn(ctx);
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
            let correct_run = match &runner.active_run {
                Some(experiment) => experiment.job_id == msg.job_id,
                None => false
            };

            // if runner is this one, mark it empty and if there is pending jobs, run it
            if correct_run {
                runner.active_run = None;
                if let Some(experiment) = runner.queue.pop_front() {
                    self.run(experiment, ctx);
                }
            }
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
            // encode the received output from runner
            let encoded_output = core::encode_minimal(msg.output.as_str());

            let res = web::block(move || {
                diesel::update(jobs::table.find(msg.job_id))
                    .set((jobs::status.eq(status.value()), jobs::output.eq(Some(encoded_output))))
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

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct JobUpdate {
    job_id: ModelId,
    status: JobStatus,
}

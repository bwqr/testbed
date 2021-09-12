use std::collections::HashMap;

use actix::prelude::*;
use actix_web::web;
use chrono::Utc;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, PooledConnection};
use log::{error, info, warn};
use serde::Serialize;

use core::db::DieselEnum;
use core::schema::{experiments, jobs, slots};
use core::types::{DBPool, ModelId};
use service::{Notification, NotificationKind, NotificationMessage, NotificationServer};
use shared::ControllerState;

use crate::connection::messages::{DisconnectServerMessage, JoinServerMessage, RunMessage, RunResultMessage, UpdateControllerValue};
use crate::connection::ReceiverValues;
use crate::connection::session::Session;
use crate::models::job::JobStatus;

pub use crate::connection::messages::AbortRunningJob;

struct ConnectedController {
    session: Addr<Session>,
    state: ControllerState,
    receiver_values: Option<Vec<u32>>,
}

#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct RunExperiment {
    pub code: String,
    pub job_id: ModelId,
    pub controller_id: ModelId,
    pub user_id: ModelId,
}

pub struct ExperimentServer {
    pool: DBPool,
    controllers: HashMap<ModelId, ConnectedController>,
    notification: Addr<NotificationServer>,
}

impl ExperimentServer {
    pub fn new(pool: DBPool, notification: Addr<NotificationServer>) -> Self {
        ExperimentServer {
            pool,
            controllers: HashMap::new(),
            notification,
        }
    }

    async fn try_next_job(conn: PooledConnection<ConnectionManager<PgConnection>>, controller_id: ModelId) -> Option<RunExperiment> {
        web::block(move || {
            let now = Utc::now().naive_utc();

            let slot_owner_id = slots::table
                .filter(slots::start_at.le(&now).and(slots::end_at.ge(&now)))
                .select(slots::user_id)
                .first::<ModelId>(&conn)?;

            jobs::table
                .inner_join(experiments::table)
                .filter(jobs::status.eq(JobStatus::Pending.value()))
                .filter(jobs::controller_id.eq(controller_id))
                .filter(experiments::user_id.eq(slot_owner_id))
                .select((experiments::user_id, jobs::id, jobs::code))
                .first::<(ModelId, ModelId, String)>(&conn)
                .map(|job| RunExperiment {
                    code: job.2,
                    job_id: job.1,
                    controller_id,
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
        let mut controller: &mut ConnectedController = self.controllers.get_mut(&experiment.controller_id)
            .ok_or("controller is not yet connected")?;

        if let ControllerState::Running(_) = &controller.state {
            return Err("controller is already running an experiment");
        }

        controller.state = ControllerState::Running(experiment.job_id);

        // otherwise try to run experiment
        // clone some necessary vars
        let session = controller.session.clone();
        let user_id = experiment.user_id;
        let job_id = experiment.job_id;
        // send experiment to the controller
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
                        error!("Error while sending job to controller, {:?}", e);
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
        // insert controller
        self.controllers.insert(msg.controller_id, ConnectedController {
            state: msg.state.clone(),
            session: msg.addr,
            receiver_values: None,
        });

        if let ControllerState::Idle = msg.state {
            // copy some necessary vals
            let controller_id = msg.controller_id;

           // try to run a job
            let conn = self.pool.get().unwrap();
            async move {
                Self::try_next_job(conn, controller_id)
                    .await
            }
                .into_actor(self)
                .then(move |res: Option<RunExperiment>, act: &mut Self, ctx: _| {
                    if let Some(run) = res {
                        // maybe controller disconnected, so check it
                        if act.controllers.contains_key(&controller_id) {
                            if let Err(e) = act.run(run, ctx) {
                                error!("Unexpectedly Controller is in running state, {}", e)
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
        self.controllers.remove(&msg.controller_id);
    }
}

impl Handler<RunResultMessage> for ExperimentServer {
    type Result = ();

    fn handle(&mut self, msg: RunResultMessage, ctx: &mut Self::Context) {
        info!("got result {} id {}", msg.successful, msg.job_id);

        if let Some(controller) = self.controllers.get_mut(&msg.controller_id) {
            match controller.state {
                ControllerState::Running(job_id) if job_id != msg.job_id => {
                    warn!("controller sent a job result other than it is running currently, running {} received {}", job_id, msg.job_id)
                },
                _ => {},
            }

            controller.state = ControllerState::Idle;

            let conn = self.pool.get().unwrap();
            let controller_id = msg.controller_id;

            async move {
                Self::try_next_job(conn, controller_id)
                    .await
            }
                .into_actor(self)
                .then(move |res: Option<RunExperiment>, act: &mut Self, ctx: _| {
                    if let Some(run) = res {
                        // maybe controller disconnected, so check it
                        if act.controllers.contains_key(&controller_id) {
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
        Ok(self.controllers.get(&msg.controller_id).ok_or(())?.receiver_values.clone())
    }
}

impl Handler<UpdateControllerValue> for ExperimentServer {
    type Result = ();
    fn handle(&mut self, msg: UpdateControllerValue, _: &mut Self::Context) -> Self::Result {
        if let Some(controller) = self.controllers.get_mut(&msg.controller_id) {
            controller.receiver_values = Some(msg.values)
        }
    }
}

impl Handler<AbortRunningJob> for ExperimentServer {
    type Result = ();
    fn handle(&mut self, msg: AbortRunningJob, _: &mut Self::Context) -> Self::Result {
        if let Some(controller) = self.controllers.get_mut(&msg.controller_id) {
            controller.session.do_send(msg)
        }
    }
}
#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct JobUpdate {
    job_id: ModelId,
    status: JobStatus,
}

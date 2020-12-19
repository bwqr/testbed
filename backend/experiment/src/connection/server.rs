use std::collections::HashMap;
use std::time::SystemTime;

use actix::prelude::*;
use log::{error, info};

use core::types::ModelId;

use crate::connection::messages::{JoinServerMessage, RunMessage, RunResultMessage};
use crate::connection::session::Session;

struct Run {
    run_id: ModelId,
    runner_id: ModelId,
    created_at: SystemTime,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct RunExperimentMessage {
    pub run_id: ModelId
}

pub struct ExperimentServer {
    active_runs: Vec<Run>,
    pending_runs: Vec<ModelId>,
    // run_id -> (session, run_id)
    runners: HashMap<ModelId, (Addr<Session>, Option<ModelId>)>,
}

impl ExperimentServer {
    pub fn new() -> Self {
        ExperimentServer {
            active_runs: Vec::new(),
            pending_runs: Vec::new(),
            runners: HashMap::new(),
        }
    }

    fn run(act: &mut ExperimentServer, run_id: ModelId, ctx: &mut <Self as Actor>::Context) {
        let mut runner_id: Option<ModelId> = None;

        for (id, v) in &act.runners {
            if let None = v.1 {
                runner_id = Some(*id);
                break;
            }
        }

        // If there is an inactive runner
        if let Some(runner_id) = runner_id {
            let runner = act.runners.get_mut(&runner_id).unwrap();
            // mark this runner as active
            runner.1 = Some(run_id);

            act.active_runs.push(Run {
                run_id,
                runner_id,
                created_at: std::time::SystemTime::now(),
            });

            let addr = runner.0.clone();

            async move {
                if let Err(error) = addr.send(RunMessage { run_id })
                    .await {
                    error!("error occured while dispatching run: {:?}", error);
                } else {
                    // web::block(move || diesel::update(runs::table.find(run_id)).set(runs::status.eq(RunStatus::Running.value())).execute())
                }
            }
                .into_actor(act)
                .spawn(ctx)
        }
        // Otherwise push it into pending
        else {
            act.pending_runs.push(run_id);
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
        info!("Run with id {} received ", msg.run_id);

        Self::run(self, msg.run_id, ctx);
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
        if let Some(runner) = self.runners.get_mut(&msg.runner_id) {
            info!("got result {} id {}", msg.successful, msg.run_id);

            let correct_run = match runner.1 {
                Some(run_id) => run_id == msg.run_id,
                None => false
            };

            if correct_run {
                runner.1 = None;

                if let Some(run_id) = self.pending_runs.pop() {
                    Self::run(self, run_id, ctx);
                }
            }
        }
    }
}
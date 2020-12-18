use actix::prelude::*;
use log::info;

use core::types::ModelId;

#[derive(Message)]
#[rtype(result = "()")]
pub struct RunExperimentMessage {
    pub experiment_id: ModelId
}

pub struct ExperimentServer {
    queue: Vec<ModelId>,
    // if there is an active running experiment
    active: bool,
}

impl ExperimentServer {
    pub fn new() -> Self {
        ExperimentServer {
            queue: Vec::new(),
            active: false,
        }
    }

    fn run_experiment(&mut self) {}
}

impl Actor for ExperimentServer {
    type Context = Context<Self>;

    fn started(&mut self, _: &mut Self::Context) {}

    fn stopped(&mut self, _: &mut Self::Context) {}
}

impl Handler<RunExperimentMessage> for ExperimentServer {
    type Result = ();

    fn handle(&mut self, msg: RunExperimentMessage, ctx: &mut Self::Context) {
        info!("Experiment with id {} received ", msg.experiment_id);

        self.queue.push(msg.experiment_id);

        // If server was empty, start running experiment immediately
        if !self.active {
            self.run_experiment();
        }
    }
}
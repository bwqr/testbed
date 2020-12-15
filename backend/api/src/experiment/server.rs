use actix::{Actor, Context};

pub struct ExperimentServer {
}

impl ExperimentServer {
    pub fn new() -> Self {
        ExperimentServer {
        }
    }
}

impl Actor for ExperimentServer {
    type Context = Context<Self>;

    fn started(&mut self, _: &mut Self::Context) {}

    fn stopped(&mut self, _: &mut Self::Context) {}
}
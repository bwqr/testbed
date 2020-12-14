use actix::{Actor, Context};

pub struct ExperimentServer {
    backends: i64
}

impl Actor for ExperimentServer {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {}

    fn stopped(&mut self, ctx: &mut Self::Context) {
    }
}
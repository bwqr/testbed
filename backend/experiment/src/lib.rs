use actix_web::web;

pub use connection::server::ExperimentServer;
use core::middlewares::auth::Auth;

mod handlers;
mod connection;
pub mod models;
mod requests;

pub fn register(config: &mut web::ServiceConfig) {
    config
        .service(
            web::scope("/api/experiment")
                .service(handlers::join_server)
                .service(
                    web::scope("")
                        .wrap(Auth)
                        .service(handlers::fetch_experiments)
                        .service(handlers::fetch_experiment)
                        .service(handlers::create_new_experiment)
                        .service(handlers::update_experiment)
                        .service(handlers::run_experiment)
                        .service(handlers::delete_experiment)
                )
        );
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

use actix_web::http::StatusCode;
use actix_web::web;

pub use connection::server::ExperimentServer;
use core::error::{ErrorMessaging, HttpError};
use user::middlewares::auth::Auth;
use user::middlewares::role::AdminUser;

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
                        .service(handlers::fetch_runners)
                        .service(handlers::fetch_runner)
                        .service(handlers::fetch_experiments)
                        .service(handlers::fetch_experiment)
                        .service(handlers::fetch_experiment_jobs)
                        .service(handlers::fetch_job)
                        .service(handlers::create_new_experiment)
                        .service(handlers::update_experiment_name)
                        .service(handlers::update_experiment_code)
                        .service(handlers::run_experiment)
                        .service(handlers::delete_experiment)
                        .service(
                            web::scope("")
                                .wrap(AdminUser)
                                .service(handlers::runner_receiver_values)
                        )
                )
        );
}

#[derive(Debug)]
pub enum ErrorMessage {
    UnknownRunner
}

impl ErrorMessaging for ErrorMessage {
    fn value(&self) -> HttpError {
        match self {
            ErrorMessage::UnknownRunner => HttpError {
                code: StatusCode::NOT_FOUND,
                error_code: 100,
                message: String::from("unknown_runner"),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

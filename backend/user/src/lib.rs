#[macro_use]
extern crate diesel;
#[macro_use]
extern crate lazy_static;

use actix_web::web;

use crate::middlewares::auth::Auth;

mod handlers;
pub mod middlewares;
pub mod models;
mod requests;

pub fn register(config: &mut web::ServiceConfig) {
    config
        .service(
            web::scope("/api/user")
                .wrap(Auth)
                .service(handlers::fetch_profile)
                .service(handlers::update_profile)
                .service(handlers::update_password)
        );
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

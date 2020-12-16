#[macro_use]
extern crate diesel;

use actix_web::web;

use core::middlewares::auth::Auth;

mod handlers;
pub mod models;
mod requests;

pub fn register(config: &mut web::ServiceConfig) {
    config
        .service(
            web::scope("/api/user")
                .wrap(Auth)
                .service(handlers::fetch_profile)
                .service(handlers::update_profile)
        );
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

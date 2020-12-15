use actix_web::web;

use core::middlewares::auth::Auth;

pub mod handlers;
pub mod models;

pub fn register(config: &mut web::ServiceConfig) {
    config
        .service(
            web::scope("/api/user")
                .wrap(Auth)
                .service(handlers::fetch_profile)
        );
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

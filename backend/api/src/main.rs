#[macro_use]
extern crate lazy_static;

use actix::prelude::*;
use actix_cors::Cors;
use actix_web::{App, get, http::header, HttpRequest, HttpResponse, HttpServer, middleware, web};
use actix_web_actors::ws;
use diesel::{PgConnection, r2d2};
use diesel::r2d2::ConnectionManager;

use core::error::Algorithm;
use core::types::DBPool;
use core::utils::Hash;
use experiment::session::Session;

use crate::experiment::server::ExperimentServer;

mod experiment;

lazy_static! {
    static ref SECRET_KEY: String = std::env::var("SECRET_KEY").expect("SECRET_KEY is not provided in env");
}

fn setup_database() -> DBPool {
    let conn_info = std::env::var("DATABASE_URL").expect("DATABASE_URL is not provided in env");
    let manager = ConnectionManager::<PgConnection>::new(conn_info);
    let pool = r2d2::Pool::builder().build(manager).expect("Failed to create pool.");

    pool
}

#[get("/ws")]
pub async fn join_server(experiment_server: web::Data<Addr<ExperimentServer>>, req: HttpRequest, stream: web::Payload) -> actix_web::Result<HttpResponse> {
    ws::start(Session::new(experiment_server.get_ref().clone()), &req, stream)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load .env
    dotenv::dotenv().ok();

    // Enable logger
    env_logger::init();

    // Setup database
    let pool = setup_database();

    // Create utils
    let hash = Hash::new(&*SECRET_KEY, Algorithm::HS256);

    let experiment_server = ExperimentServer::new().start();

    let srv = HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin(std::env::var("ALLOWED_ORIGIN").expect("ALLOWED_ORIGIN is not provided in env").as_str())
            .allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT, header::CONTENT_TYPE])
            .allowed_header("enctype")
            .max_age(60);

        App::new()
            .wrap(cors)
            .wrap(middleware::Logger::default())
            .data(experiment_server.clone())
            .data(hash.clone())
            .data(pool.clone())
            .configure(user::register)
            .service(join_server)
    })
        .bind(std::env::var("APP_BIND_ADDRESS").expect("APP_BIND_ADDRESS is not provided in env").as_str())?;

    let srv = if let Ok(w) = std::env::var("NUM_WORKERS") {
        match w.parse::<usize>() {
            Ok(w) => srv.workers(w),
            Err(_) => panic!("Invalid NUM_WORKERS is provided, please give a positive integer")
        }
    } else {
        srv
    };

    srv
        .run()
        .await
}

#[macro_use]
extern crate lazy_static;

use std::sync::Arc;
use std::sync::mpsc::channel;

use actix::prelude::*;
use actix_cors::Cors;
use actix_web::{App, http::header, HttpServer, middleware};
use diesel::{PgConnection, r2d2};
use diesel::r2d2::ConnectionManager;

use core::Config;
use core::error::Algorithm;
use core::types::DBPool;
use core::utils::Hash;
use experiment::ExperimentServer;
use service::{ClientServices, MailClient, MailClientMock, MailService, SendMailMessage};

lazy_static! {
    static ref SECRET_KEY: String = std::env::var("SECRET_KEY").expect("SECRET_KEY is not provided in env");
}

fn setup_database() -> DBPool {
    let conn_info = std::env::var("DATABASE_URL").expect("DATABASE_URL is not provided in env");
    let manager = ConnectionManager::<PgConnection>::new(conn_info);
    let pool = r2d2::Pool::builder().build(manager).expect("Failed to create pool.");

    pool
}

fn setup_services() -> ClientServices {
    // mail service
    let send_mail_recipient: Recipient<SendMailMessage> = if std::env::var("ENV").expect("ENV is not provided in env") == "prod" {
        let (mail_tx, mail_rx) = channel::<Recipient<SendMailMessage>>();
        std::thread::Builder::new().name("mail_client".to_string()).spawn(move || {
            let sys = System::new("mail_client");
            let mail_client = MailClient::new(
                std::env::var("MAIL_ADDRESS").expect("MAIL_ADDRESS is not provided in env"),
                std::env::var("MAILGUN_ENDPOINT").expect("MAILGUN_ENDPOINT is not provided in env"),
                std::env::var("MAILGUN_KEY").expect("MAILGUN_KEY is not provided in env"),
            )
                .expect("MailClient failed to init").start().recipient();
            mail_tx.send(mail_client).expect("Failed to send MailClient from thread");
            sys.run()
        }).expect("Failed to initialize thread");

        mail_rx.recv().expect("Failed to receive MailClient from thread")
    } else {
        MailClientMock::new().start().recipient()
    };

    let mail_service = MailService::new(send_mail_recipient);

    ClientServices {
        mail: mail_service
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load .env
    dotenv::dotenv().ok();

    // Enable logger
    env_logger::init();

    // Setup database
    let pool = setup_database();

    // Setup services
    let client_services = setup_services();

    // Create utils
    let hash = Hash::new(&*SECRET_KEY, Algorithm::HS256);

    let experiment_server = ExperimentServer::new().start();

    let config = Arc::new(Config {
        web_app_url: std::env::var("WEB_APP_URL").expect("WEB_APP_URL is not provided in env"),
        app_url: std::env::var("APP_URL").expect("APP_URL is not provided in env"),
        storage_path: std::env::var("STORAGE_PATH").expect("STORAGE_PATH is not provided in env"),
    });

    let srv = HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin(std::env::var("ALLOWED_ORIGIN").expect("ALLOWED_ORIGIN is not provided in env").as_str())
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
            .allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT, header::CONTENT_TYPE])
            .allowed_header("enctype")
            .max_age(60);

        App::new()
            .wrap(cors)
            .wrap(middleware::Logger::default())
            .data(experiment_server.clone())
            .data(hash.clone())
            .data(pool.clone())
            .data(config.clone())
            .data(client_services.clone())
            .configure(user::register)
            .configure(auth::register)
            .configure(experiment::register)
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

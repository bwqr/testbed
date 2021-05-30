#[macro_use]
extern crate lazy_static;

use std::sync::Arc;
use std::sync::mpsc::channel;

use actix::prelude::*;
use actix_cors::Cors;
use actix_web::{App, http::header, HttpServer, middleware, web};
use diesel::{PgConnection, r2d2};

use core::Config;
use core::types::DBPool;
use core::utils::Algorithm;
use core::utils::Hash;
use experiment::ExperimentServer;
use service::{ClientServices, mail::{MailClient, MailClientMock, MailService, SendMailMessage}, NotificationServer, Servers, SessionManager};
use user::middlewares::auth::Auth;

mod handlers;

lazy_static! {
    static ref SECRET_KEY: String = std::env::var("SECRET_KEY").expect("SECRET_KEY is not provided in env");
}

fn setup_database() -> DBPool {
    let conn_info = std::env::var("DATABASE_URL").expect("DATABASE_URL is not provided in env");
    let manager = r2d2::ConnectionManager::<PgConnection>::new(conn_info);
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

fn setup_servers() -> Servers {
    let (notification_tx, notification_rx) = channel::<Addr<NotificationServer>>();
    std::thread::Builder::new().name("notification_server".to_string()).spawn(move || {
        let sys = System::new("notification_server");
        let notification = NotificationServer::new().start();
        notification_tx.send(notification).expect("Failed to send NotificationServer from thread");
        sys.run()
    }).expect("Failed to initialize thread");

    let notification = notification_rx.recv().expect("Failed to receive NotificationServer from thread");

    let (session_tx, session_rx) = channel::<Servers>();
    std::thread::Builder::new().name("session_manager".to_string()).spawn(move || {
        let sys = System::new("session_manager");
        let session_manager = SessionManager::new(notification.clone()).start();
        session_tx.send(Servers {
            notification,
            session_manager,
        }).expect("Failed to send Servers from thread");
        sys.run()
    }).expect("Failed to initialize thread");

    session_rx.recv().expect("Failed to receive Servers from thread")
}

fn setup_experiment_server(pool: DBPool, notification: Addr<NotificationServer>) -> Addr<ExperimentServer> {
    let (tx, rx) = channel::<Addr<ExperimentServer>>();
    std::thread::Builder::new().name("experiment_server".to_string()).spawn(move || {
        let sys = System::new("experiment_server");
        let experiment_server = ExperimentServer::new(pool, notification).start();
        tx.send(experiment_server).expect("Failed to send ExperimentServer from thread");
        sys.run()
    }).expect("Failed to initialize thread");

    rx.recv().expect("Failed to receive ExperimentServer from thread")
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

    // Setup servers
    let servers = setup_servers();

    let experiment_server = setup_experiment_server(pool.clone(), servers.notification.clone());


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
            .data(servers.clone())
            .configure(user::register)
            .configure(auth::register)
            .configure(experiment::register)
            .configure(slot::register)
            .service(
                web::scope("api")
                    .wrap(Auth)
                    .service(handlers::join_chat_server)
            )
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

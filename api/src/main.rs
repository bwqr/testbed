use actix::prelude::*;
use actix_cors::Cors;
use actix_web::{App, get, http::header, HttpRequest, HttpResponse, HttpServer, middleware, web};
use actix_web_actors::ws;
use actix_web_actors::ws::{Message, WebsocketContext, ProtocolError};

struct WSSession;

impl Actor for WSSession {
    type Context = WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {}

    fn stopping(&mut self, ctx: &mut Self::Context) -> Running {
        Running::Stop
    }
}

impl StreamHandler<Result<Message, ProtocolError>> for WSSession {
    fn handle(&mut self, msg: Result<Message, ProtocolError>, ctx: &mut Self::Context) {
        let msg = match msg {
            Ok(m) => m,
            Err(_) => {
                ctx.stop();
                return;
            }
        };

        println!("{:?}", msg);
    }
}

#[get("/ws")]
pub async fn join_server(req: HttpRequest, stream: web::Payload) -> actix_web::Result<HttpResponse> {
    ws::start(WSSession, &req, stream)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load .env
    dotenv::dotenv().ok();

    // Enable logger
    env_logger::init();

    let srv = HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin(std::env::var("ALLOWED_ORIGIN").expect("ALLOWED_ORIGIN is not provided in env").as_str())
            .allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT, header::CONTENT_TYPE])
            .allowed_header("enctype")
            .max_age(60);

        App::new()
            .wrap(cors)
            .wrap(middleware::Logger::default())
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

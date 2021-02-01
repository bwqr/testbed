use actix_web::{get, HttpRequest, HttpResponse, Result, web};
use actix_web::http::StatusCode;
use actix_web_actors::ws;

use core::error::HttpError;
use service::{CreateSession, Servers, Session};
use user::models::user::User;

#[get("ws")]
pub async fn join_chat_server(servers: web::Data<Servers>, req: HttpRequest, stream: web::Payload, user: User)
                              -> Result<HttpResponse> {
    let session: Session = servers.session_manager.send(CreateSession {
        user_id: user.id
    })
        .await
        .map_err(|_| HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR).json(HttpError {
            code: StatusCode::INTERNAL_SERVER_ERROR,
            error_code: 1,
            message: String::from("join_failed"),
        }))?;

    ws::start(session, &req, stream)
}
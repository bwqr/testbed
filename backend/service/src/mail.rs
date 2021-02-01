use std::rc::Rc;
use std::sync::Arc;

use actix::prelude::*;
use actix_web::client::{Client, Connector};
use log::{error, info};
use rustls::ClientConfig;
use serde::Serialize;

#[derive(Clone)]
pub struct MailService {
    send_mail_recipient: Recipient<SendMailMessage>
}

impl MailService {
    pub fn new(send_mail_recipient: Recipient<SendMailMessage>) -> Self {
        MailService {
            send_mail_recipient
        }
    }

    pub fn send_mail(&self, to: String, subject: String, text: String) -> () {
        self.send_mail_recipient.do_send(SendMailMessage {
            to,
            subject,
            text,
        }).unwrap()
    }
}

#[derive(Debug)]
pub struct SendMailMessage {
    pub to: String,
    pub subject: String,
    pub text: String,
}

impl Message for SendMailMessage {
    type Result = Result<(), Error>;
}

pub struct MailClientMock {
    mail_received: bool,
    mail_sent: bool,
}

impl MailClientMock {
    pub fn new() -> Self {
        MailClientMock {
            mail_received: false,
            mail_sent: false,
        }
    }

    pub fn is_mail_received(&self) -> bool {
        self.mail_received
    }

    pub fn is_mail_sent(&self) -> bool {
        self.mail_sent
    }
}


impl Actor for MailClientMock {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Context<Self>) {
        info!("MailClientMock is started!");
    }

    fn stopped(&mut self, _ctx: &mut Context<Self>) {
        info!("MailClientMock is stopped");
    }
}

impl Handler<SendMailMessage> for MailClientMock {
    type Result = Result<(), Error>;

    fn handle(&mut self, message: SendMailMessage, _: &mut Context<Self>) -> Self::Result {
        info!("MailClientMock message received, {:?}", message);
        self.mail_received = true;
        self.mail_sent = true;
        Ok(())
    }
}

#[derive(Serialize)]
struct MailRequest {
    from: String,
    to: String,
    subject: String,
    html: String,
}

impl MailRequest {
    pub fn from_message(message: SendMailMessage, from: String) -> Self {
        MailRequest {
            from,
            to: message.to,
            subject: message.subject,
            html: message.text,
        }
    }
}

pub struct MailClient {
    from: String,
    endpoint: Rc<String>,
    key: Rc<String>,
    client: Rc<Client>,
}

impl MailClient {
    pub fn new(from: String, endpoint: String, key: String) -> Result<Self, Error> {
        let mut config = ClientConfig::new();
        config.root_store.add_server_trust_anchors(&webpki_roots::TLS_SERVER_ROOTS);

        let client = Client::builder()
            .connector(Connector::new().rustls(Arc::new(config)).finish())
            .finish();

        Ok(MailClient {
            from,
            endpoint: Rc::new(endpoint),
            key: Rc::new(key),
            client: Rc::new(client),
        })
    }
}

impl Actor for MailClient {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Context<Self>) {
        info!("MailClient is started!");
    }

    fn stopped(&mut self, _ctx: &mut Context<Self>) {
        info!("MailClient is stopped");
    }
}

impl Handler<SendMailMessage> for MailClient {
    type Result = Result<(), Error>;

    fn handle(&mut self, message: SendMailMessage, ctx: &mut Context<Self>) -> Self::Result {
        deliver_mail(
            self.client.clone(),
            MailRequest::from_message(message, self.from.clone()),
            self.endpoint.clone(),
            self.key.clone(),
        )
            .into_actor(self)
            .spawn(ctx);

        Ok(())
    }
}

async fn deliver_mail(client: Rc<Client>, message: MailRequest, endpoint: Rc<String>, key: Rc<String>) -> () {
    let req = client
        .post(endpoint.as_str())
        .basic_auth("api", Some(key.as_str()))
        .send_form(&message)
        .await;

    if let Ok(mut req) = req {
        let body = req
            .body()
            .await;

        if let Ok(body) = body {
            info!("{:?}", body);
        } else {
            error!("Error while sending mail, {:?}", line!());
        }
    } else {
        error!("Error while sending mail, {:?}", line!());
    }
}

#[derive(Debug)]
pub enum Error {
    Client
}

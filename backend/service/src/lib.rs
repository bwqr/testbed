use actix::Addr;

use crate::mail::MailService;
pub use crate::notification::NotificationServer;
pub use crate::ws::{session::Session, session_manager::SessionManager};
pub use crate::ws::messages::{CreateSession, Notification, NotificationKind, NotificationMessage};

pub mod mail;
mod notification;
mod ws;

#[derive(Clone)]
pub struct ClientServices {
    pub mail: MailService
}

#[derive(Clone)]
pub struct Servers {
    pub session_manager: Addr<SessionManager>,
    pub notification: Addr<NotificationServer>,
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

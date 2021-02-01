use actix::{Addr, Message};
use serde::{Deserialize, Serialize};

use core::types::ModelId;

use super::session::{Session, SessionId};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WebSocketError {
    error_code: i32,
    message: String,
}

#[derive(Deserialize)]
pub enum IncomingMessageKind {
    KeepAlive,
}

#[derive(Serialize)]
pub enum OutgoingMessageKind {
    Timeout,
    Notification,
}

#[derive(Serialize, Clone)]
pub enum NotificationKind {
    JobUpdate
}

pub trait WebSocketMessaging: Message<Result=()> + Send {
    fn value(self) -> String;
}

#[derive(Serialize)]
pub struct ErrorMessage {
    pub error: WebSocketError,
}

#[derive(Message)]
#[rtype(result = "()")]
pub enum ErrorKind {
    InvalidMessage,
}

impl WebSocketMessaging for ErrorKind {
    fn value(self) -> String {
        serde_json::to_string(&self.as_message()).unwrap()
    }
}

#[derive(Serialize, Clone)]
pub struct NotificationMessage<T> {
    pub kind: NotificationKind,
    pub data: T,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Notification<T> {
    pub user_id: ModelId,
    pub message: NotificationMessage<T>,
}

#[derive(Message)]
#[rtype(result = "Session")]
pub struct CreateSession {
    pub user_id: ModelId
}

impl ErrorKind {
    pub fn as_message(&self) -> ErrorMessage {
        let error = match self {
            ErrorKind::InvalidMessage => WebSocketError {
                error_code: 100,
                message: "invalid_message".to_owned(),
            }
        };

        ErrorMessage {
            error
        }
    }
}

pub mod incoming {
    use super::{Deserialize, IncomingMessageKind};

    #[derive(Deserialize)]
    pub struct BaseMessage {
        pub kind: IncomingMessageKind,
    }

    #[derive(Deserialize)]
    pub struct IncomingMessage<T> {
        pub kind: IncomingMessageKind,
        pub data: T,
    }
}

pub mod outgoing {
    use super::{Message, ModelId, OutgoingMessageKind, Serialize, WebSocketMessaging};

    #[derive(Serialize)]
    pub struct OutgoingMessage<T> {
        pub kind: OutgoingMessageKind,
        pub data: T,
    }

    #[derive(Message, Serialize)]
    #[rtype(result = "()")]
    pub struct Timeout;

    impl WebSocketMessaging for Timeout {
        fn value(self) -> String {
            create_outgoing_message(self, OutgoingMessageKind::Timeout)
        }
    }

    #[derive(Message, Serialize)]
    #[rtype(result = "()")]
    #[serde(rename_all = "camelCase")]
    pub struct NotifyUser<T> {
        pub user_id: ModelId,
        pub message: T,
    }

    impl<T> WebSocketMessaging for NotifyUser<T> where T: Serialize + Send + Clone {
        fn value(self) -> String {
            create_outgoing_message(self, OutgoingMessageKind::Notification)
        }
    }

    fn create_outgoing_message<T: Serialize>(data: T, kind: OutgoingMessageKind) -> String {
        serde_json::to_string(&OutgoingMessage {
            kind,
            data,
        }).unwrap()
    }
}

pub mod internal {
    pub mod server {
        use super::super::{Addr, Message, ModelId, Session, SessionId};

        #[derive(Message)]
        #[rtype(result = "()")]
        pub struct ConnectServer {
            pub session_id: SessionId,
            pub user_id: ModelId,
            pub addr: Addr<Session>,
        }

        #[derive(Message)]
        #[rtype(result = "()")]
        pub struct DisconnectServer {
            pub session_id: SessionId,
            pub user_id: ModelId,
        }
    }
}
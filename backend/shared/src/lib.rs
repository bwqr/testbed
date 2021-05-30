use serde::{Deserialize, Serialize};

pub mod websocket_messages;


#[derive(Deserialize, Serialize, Clone)]
pub enum RunnerState {
    Idle,
    Running,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JoinServerRequest {
    pub token: String,
    pub runner_state: RunnerState,
}

#[derive(Debug)]
pub enum SocketErrorKind {
    InvalidMessage,
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

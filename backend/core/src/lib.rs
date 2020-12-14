pub mod messages;
pub mod models;
pub mod types;

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

pub use clients::mail::{MailClient, MailClientMock, MailService, SendMailMessage};

mod clients;

#[derive(Clone)]
pub struct ClientServices {
    pub mail: MailService
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

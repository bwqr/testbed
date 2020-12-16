use askama::Template;

#[derive(Template)]
#[template(path = "mails/verify-account.html")]
pub struct VerifyAccountMailTemplate<'a> {
    pub web_app_url: &'a str,
    pub full_name: &'a str,
    pub token: &'a str,
}
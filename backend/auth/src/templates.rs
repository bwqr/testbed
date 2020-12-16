use askama::Template;

#[derive(Template)]
#[template(path = "mails/verify-account.html")]
pub struct VerifyAccountMailTemplate<'a> {
    pub web_app_url: &'a str,
    pub full_name: &'a str,
    pub token: &'a str,
}

#[derive(Template)]
#[template(path = "mails/forgot-password.html")]
pub struct ForgotPasswordMailTemplate<'a> {
    pub web_app_url: &'a str,
    pub full_name: &'a str,
    pub token: &'a str,
}

#[derive(Template)]
#[template(path = "mails/reset-password.html")]
pub struct ResetPasswordMailTemplate<'a> {
    pub full_name: &'a str,
}
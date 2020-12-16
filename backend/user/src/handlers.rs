use actix_web::{get, HttpResponse, Result};

use crate::models::user::{User, UserStatus};

#[get("/profile")]
pub async fn fetch_profile() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(User {
        id: 1,
        first_name: String::from("Hola"),
        last_name: String::from("Herman"),
        email: String::from("hola@herman.com"),
        password: String::from("password"),
        status: UserStatus::Verified,
        role_id: 1,
    }))
}
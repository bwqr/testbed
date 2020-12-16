use actix_web::{get, HttpResponse, put, Result, web};
use diesel::prelude::*;

use core::responses::SuccessResponse;
use core::sanitized::SanitizedJson;
use core::schema::users;
use core::types::DBPool;
use core::utils::Hash;

use crate::models::user::User;
use crate::requests::{UpdatePasswordRequest, UpdateProfileRequest};

#[get("/profile")]
pub async fn fetch_profile(user: User) -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(user))
}

#[put("/profile")]
pub async fn update_profile(pool: web::Data<DBPool>, user: User, request: SanitizedJson<UpdateProfileRequest>) -> Result<HttpResponse> {
    let conn = pool.get().unwrap();

    web::block(move ||
        diesel::update(users::table.find(user.id))
            .set(&request.into_inner())
            .execute(&conn)
    )
        .await?;

    Ok(HttpResponse::Ok().json(SuccessResponse::default()))
}

#[put("/password")]
pub async fn update_password(pool: web::Data<DBPool>, hash: web::Data<Hash>, user: User, request: web::Json<UpdatePasswordRequest>) -> Result<HttpResponse> {
    let conn = pool.get().unwrap();

    web::block(move || {
        let password = hash.sign512(&request.0.password);

        diesel::update(users::table.find(user.id))
            .set(users::password.eq(password))
            .execute(&conn)
    })
        .await?;

    Ok(HttpResponse::Ok().json(SuccessResponse::default()))
}
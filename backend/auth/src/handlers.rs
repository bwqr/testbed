use std::sync::Arc;

use actix_web::{HttpResponse, post, put, web};
use askama::Template;
use diesel::prelude::*;
use diesel::result::{DatabaseErrorKind, Error};
use validator::Validate;

use core::Config;
use core::db::DieselEnum;
use core::error::{ErrorMessaging, ValidationError};
use core::ErrorMessage as CoreErrorMessage;
use core::models::token::{AuthToken, IdentityToken, IdentityTokenKind};
use core::responses::{SuccessResponse, TokenResponse};
use core::sanitized::SanitizedJson;
use core::schema::users;
use core::types::DBPool;
use core::utils::Hash;
use service::ClientServices;
use user::models::user::{User, UserStatus};

use crate::ErrorMessage;
use crate::requests::{ForgotPasswordRequest, LoginRequest, ResetPasswordRequest, SignUpRequest, VerifyAccountRequest};
use crate::templates::{ForgotPasswordMailTemplate, ResetPasswordMailTemplate, VerifyAccountMailTemplate};

const TIMEOUT: i64 = 60 * 60 * 24;

#[post("/login")]
pub async fn login(pool: web::Data<DBPool>, hash: web::Data<Hash>, request: SanitizedJson<LoginRequest>) -> Result<HttpResponse, Box<dyn ErrorMessaging>> {
    let conn = pool.get().unwrap();
    let hash = hash.into_inner();
    let request = request.into_inner();

    let password = hash.sign512(&request.password);
    let user = web::block(move || -> Result<User, Box<dyn ErrorMessaging>> {
        let result: QueryResult<User> = users::table
            .filter(users::email.eq(&request.email).and(users::password.eq(&password)))
            .first::<User>(&conn);

        match result {
            Ok(user) => {
                match user.status {
                    UserStatus::NotVerified => Err(Box::new(ErrorMessage::NotVerified)),
                    UserStatus::Banned => Err(Box::new(ErrorMessage::Banned)),
                    UserStatus::Verified => Ok(user)
                }
            }
            Err(err) => match err {
                diesel::result::Error::NotFound => Err(Box::new(ErrorMessage::InvalidCredentialsOrUser)),
                _ => Err(Box::new(err))
            }
        }
    })
        .await?;

    let token = hash.encode(&AuthToken::new(user.id, user.role_id, TIMEOUT))?;

    let token = TokenResponse { token };

    Ok(HttpResponse::Ok().json(token))
}

#[post("/sign-up")]
pub async fn sign_up(
    pool: web::Data<DBPool>,
    hash: web::Data<Hash>,
    config: web::Data<Arc<Config>>,
    client_services: web::Data<ClientServices>,
    request: SanitizedJson<SignUpRequest>,
)
    -> Result<HttpResponse, Box<dyn ErrorMessaging>> {
    let conn = pool.get().unwrap();
    let hash = hash.into_inner();
    let request = request.into_inner();

    request.validate()
        .map_err(|e| ValidationError::from(e))?;

    let insert_model = request.as_insert_model(&hash);

    let user = web::block(move || -> Result<User, Box<dyn ErrorMessaging>> {
        let result = diesel::insert_into(users::table)
            .values(&insert_model)
            .get_result::<User>(&conn);

        match result {
            Ok(user) => Ok(user),
            Err(err) => match err {
                Error::DatabaseError(kind, _) => match kind {
                    DatabaseErrorKind::UniqueViolation => Err(Box::new(ErrorMessage::UserExists)),
                    _ => Err(Box::new(err))
                },
                _ => Err(Box::new(err))
            }
        }
    })
        .await?;

    let token = hash.encode(
        &IdentityToken::new(user.id, IdentityTokenKind::VerifyAccount, TIMEOUT)
    )?;

    let text = VerifyAccountMailTemplate {
        web_app_url: config.web_app_url.as_str(),
        full_name: user.full_name().as_str(),
        token: token.as_str(),
    }
        .render()
        .map_err(|_| CoreErrorMessage::AskamaError)?;

    let full_name = user.full_name();

    client_services.mail.send_mail(user.email, full_name, text);

    Ok(HttpResponse::Ok().json(SuccessResponse::default()))
}

#[post("/forgot-password")]
pub async fn forgot_password(
    hash: web::Data<Hash>,
    pool: web::Data<DBPool>,
    config: web::Data<Arc<Config>>,
    client_services: web::Data<ClientServices>,
    request: SanitizedJson<ForgotPasswordRequest>,
) -> Result<HttpResponse, Box<dyn ErrorMessaging>> {
    let conn = pool.get().unwrap();
    let request = request.into_inner();

    let user = web::block(move || users::table
        .filter(users::email.eq(request.email))
        .select(users::all_columns)
        .first::<User>(&conn)
    )
        .await?;

    let token = hash.encode(&IdentityToken::new(user.id, IdentityTokenKind::ForgotPassword, TIMEOUT))?;

    let text = ForgotPasswordMailTemplate {
        web_app_url: config.web_app_url.as_str(),
        full_name: user.full_name().as_str(),
        token: token.as_str(),
    }
        .render()
        .map_err(|_| CoreErrorMessage::AskamaError)?;

    client_services.mail.send_mail(user.full_name(), user.email, text);

    Ok(HttpResponse::Ok().json(SuccessResponse::default()))
}

#[put("/reset-password")]
pub async fn reset_password(
    hash: web::Data<Hash>,
    pool: web::Data<DBPool>,
    client_services: web::Data<ClientServices>,
    request: SanitizedJson<ResetPasswordRequest>,
) -> Result<HttpResponse, Box<dyn ErrorMessaging>> {
    let conn = pool.get().unwrap();

    let request = request.into_inner();

    request.validate()
        .map_err(|e| ValidationError::from(e))?;

    let reset_password_token = hash.decode::<IdentityToken>(request.token.as_str())
        .map_err(|_| CoreErrorMessage::InvalidToken)?;

    if reset_password_token.kind != IdentityTokenKind::ForgotPassword {
        Err(CoreErrorMessage::InvalidToken)?;
    }

    let hash = hash.sign512(request.password.as_str());

    let user = web::block(move || -> Result<User, Error> {
        let user = users::table.find(reset_password_token.user_id).first::<User>(&conn)?;
        diesel::update(&user)
            .set(users::password.eq(hash))
            .execute(&conn)?;

        Ok(user)
    })
        .await?;

    let text = ResetPasswordMailTemplate {
        full_name: user.full_name().as_str(),
    }
        .render()
        .map_err(|_| CoreErrorMessage::AskamaError)?;

    client_services.mail.send_mail(user.full_name(), user.email, text);


    Ok(HttpResponse::Ok().json(SuccessResponse::default()))
}

#[post("verify-account")]
pub async fn verify_account(
    hash: web::Data<Hash>,
    pool: web::Data<DBPool>,
    request: SanitizedJson<VerifyAccountRequest>,
) -> Result<HttpResponse, Box<dyn ErrorMessaging>> {
    let conn = pool.get().unwrap();

    let verify_account_token = hash.decode::<IdentityToken>(request.0.token.as_str())
        .map_err(|_| CoreErrorMessage::InvalidToken)?;

    if verify_account_token.kind != IdentityTokenKind::VerifyAccount {
        Err(CoreErrorMessage::InvalidToken)?;
    }

    web::block(move || -> Result<(), Box<dyn ErrorMessaging>> {
        let user = users::table.find(verify_account_token.user_id).first::<User>(&conn)?;

        if user.status != UserStatus::NotVerified {
            Err(CoreErrorMessage::InvalidOperationForStatus)?;
        }

        diesel::update(&user)
            .set(users::status.eq(UserStatus::Verified.value()))
            .execute(&conn)?;

        Ok(())
    })
        .await?;

    Ok(HttpResponse::Ok().json(SuccessResponse::default()))
}
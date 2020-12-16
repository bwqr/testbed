use std::sync::Arc;

use actix_web::{HttpResponse, post, web};
use askama::Template;
use diesel::prelude::*;
use diesel::result::{DatabaseErrorKind, Error};
use validator::Validate;

use core::Config;
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
use crate::requests::{LoginRequest, SignUpRequest};
use crate::templates::VerifyAccountMailTemplate;

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
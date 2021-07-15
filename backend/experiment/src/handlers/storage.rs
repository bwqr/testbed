use std::sync::Arc;

use actix_files::NamedFile;
use actix_web::{get, post, web};
use actix_web::http::header::{DispositionParam, DispositionType};
use actix_web::web::Json;
use diesel::prelude::*;
use futures_util::AsyncWriteExt;
use futures_util::stream::StreamExt as _;

use core::{Config, ErrorMessage as CoreErrorMessage};
use core::error::ErrorMessaging;
use core::responses::{SuccessResponse, TokenResponse};
use core::schema::{experiments, jobs, runners};
use core::types::{DBPool, ModelId, Result};
use core::utils::Hash;
use user::models::user::User;

use crate::ErrorMessage;
use crate::models::runner::{Runner, RunnerToken};

#[get("job/{id}/output")]
pub async fn download_job_output(pool: web::Data<DBPool>, job_id: web::Path<ModelId>, user: User, config: web::Data<Arc<Config>>) -> Result<NamedFile> {
    let conn = pool.get().unwrap();

    let job_id = web::block(move ||
        jobs::table
            .filter(jobs::id.eq(job_id.into_inner()))
            .inner_join(experiments::table)
            .filter(experiments::user_id.eq(user.id))
            .select(jobs::id)
            .first::<ModelId>(&conn)
    )
        .await?;

    let named_file = NamedFile::open(format!("{}/{}/output.txt", config.storage_path, job_id))
        .map_err(|e| {
            match e.kind() {
                std::io::ErrorKind::NotFound => CoreErrorMessage::ItemNotFound,
                _ => CoreErrorMessage::IOError
            }
        })?
        .set_content_disposition(actix_web::http::header::ContentDisposition {
            disposition: DispositionType::Attachment,
            parameters: vec![DispositionParam::Filename(String::from("output.txt"))],
        });

    Ok(named_file)
}

#[post("job/{id}/output")]
pub async fn store_job_output(
    pool: web::Data<DBPool>,
    hash: web::Data<Hash>,
    config: web::Data<Arc<Config>>,
    runner_token: web::Query<TokenResponse>,
    mut stream: web::Payload,
    job_id: web::Path<ModelId>,
)
    -> Result<Json<SuccessResponse>> {
    let conn = pool.get().unwrap();
    let job_id = job_id.into_inner();
    let token = hash.decode::<RunnerToken>(runner_token.token.as_str())
        .map_err(|_| CoreErrorMessage::InvalidToken)?;

    web::block(move || -> Result<()> {
        let runner = runners::table
            .filter(runners::access_key.eq(token.access_key))
            .first::<Runner>(&conn)?;

        let job_exists = diesel::dsl::select(diesel::dsl::exists(
            jobs::table
                .filter(jobs::id.eq(job_id))
                .filter(jobs::runner_id.eq(runner.id))
        )).get_result(&conn)?;

        if job_exists {
            Ok(())
        } else {
            Err(Box::new(CoreErrorMessage::ItemNotFound))
        }
    })
        .await?;

    let path = format!("{}/{}", config.storage_path, job_id);

    async_std::fs::create_dir_all(&path)
        .await
        .map_err(|_| Box::new(CoreErrorMessage::IOError) as Box<dyn ErrorMessaging>)?;

    let mut file = async_std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(format!("{}/output.txt", path))
        .await
        .map_err(|e| match e.kind() {
            std::io::ErrorKind::AlreadyExists => Box::new(ErrorMessage::OutputAlreadyExist) as Box<dyn ErrorMessaging>,
            _ => Box::new(CoreErrorMessage::IOError)
        })?;

    while let Some(chunk) = stream.next().await {
        let bytes = chunk
            .map_err(|_| Box::new(CoreErrorMessage::IOError) as Box<dyn ErrorMessaging>)?;

        file.write_all(&bytes)
            .await
            .map_err(|_| Box::new(CoreErrorMessage::IOError) as Box<dyn ErrorMessaging>)?
    }

    Ok(Json(SuccessResponse::default()))
}

use actix::Addr;
use actix_web::{delete, get, post, put, web, web::Json, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use chrono::Utc;
use diesel::prelude::*;
use log::error;
use serde_json::json;

use core::db::DieselEnum;
use core::error::ErrorMessaging;
use core::models::paginate::{CountStarOver, Paginate, Pagination, PaginationRequest};
use core::responses::{SuccessResponse, TokenResponse};
use core::sanitized::SanitizedJson;
use core::schema::{experiments, jobs, controllers, slots};
use core::types::{DBPool, DefaultResponse, ModelId, Result};
use core::utils::Hash;
use core::ErrorMessage as CoreErrorMessage;
use shared::{JoinServerRequest, ControllerState};
use user::models::user::User;

use crate::connection::server::{ExperimentServer, RunExperiment, AbortRunningJob};
use crate::connection::session::Session;
use crate::connection::ReceiverValues;
use crate::models::experiment::{Experiment, SlimExperiment, SLIM_EXPERIMENT_COLUMNS};
use crate::models::job::{Job, JobStatus, SlimJob, SLIM_JOB_COLUMNS};
use crate::models::controller::{Controller, ControllerToken, SlimController, SLIM_CONTROLLER_COLUMNS};
use crate::requests::{ExperimentCodeRequest, ExperimentNameRequest};
use crate::ErrorMessage;

pub mod storage;

#[get("ws")]
pub async fn join_server(
    pool: web::Data<DBPool>,
    hash: web::Data<Hash>,
    experiment_server: web::Data<Addr<ExperimentServer>>,
    req: HttpRequest,
    stream: web::Payload,
    join_server_request: web::Query<JoinServerRequest>,
) -> DefaultResponse {
    let conn = pool.get().unwrap();

    let join_server_request = join_server_request.into_inner();

    let token = hash
        .decode::<ControllerToken>(join_server_request.token.as_str())
        .map_err(|_| CoreErrorMessage::InvalidToken)?;

    let controller = web::block(move || {
        controllers::table
            .filter(controllers::access_key.eq(token.access_key))
            .first::<Controller>(&conn)
    })
    .await?;

    let controller_state = if let Some(job_id) = join_server_request.running_job_id {
        ControllerState::Running(job_id)
    } else {
        ControllerState::Idle
    };

    ws::start(
        Session::new(
            experiment_server.get_ref().clone(),
            controller.id,
            controller_state,
        ),
        &req,
        stream,
    )
    .map_err(|_| Box::new(CoreErrorMessage::WebSocketConnectionError) as Box<dyn ErrorMessaging>)
}

#[get("controllers")]
pub async fn fetch_controllers(pool: web::Data<DBPool>) -> Result<Json<Vec<SlimController>>> {
    let conn = pool.get().unwrap();

    let controllers = web::block(move || {
        controllers::table
            .select(SLIM_CONTROLLER_COLUMNS)
            .load::<SlimController>(&conn)
    })
    .await?;

    Ok(Json(controllers))
}

#[get("controller/{id}")]
pub async fn fetch_controller(
    pool: web::Data<DBPool>,
    controller_id: web::Path<ModelId>,
) -> Result<Json<SlimController>> {
    let conn = pool.get().unwrap();

    let controller = web::block(move || {
        controllers::table
            .find(controller_id.into_inner())
            .select(SLIM_CONTROLLER_COLUMNS)
            .first::<SlimController>(&conn)
    })
    .await?;

    Ok(Json(controller))
}

#[get("controller/{id}/values")]
pub async fn controller_receiver_values(
    experiment_server: web::Data<Addr<ExperimentServer>>,
    controller_id: web::Path<ModelId>,
) -> DefaultResponse {
    let values = experiment_server
        .send(ReceiverValues {
            controller_id: controller_id.into_inner(),
        })
        .await
        .map_err(|_| {
            CoreErrorMessage::Custom("failed to receive receiver status from experiment server")
        })?
        .map_err(|_| ErrorMessage::UnknownController)?;

    Ok(HttpResponse::Ok().json(json!({ "values": values })))
}

#[get("experiments")]
pub async fn fetch_experiments(
    pool: web::Data<DBPool>,
    user: User,
    pagination: web::Query<PaginationRequest>,
) -> Result<Json<Pagination<SlimExperiment>>> {
    let conn = pool.get().unwrap();

    let experiments = web::block(move || {
        experiments::table
            .filter(experiments::user_id.eq(user.id))
            .order(experiments::created_at.desc())
            .select((SLIM_EXPERIMENT_COLUMNS, CountStarOver))
            .paginate(pagination.page)
            .per_page(pagination.per_page)
            .load_and_count_pages::<SlimExperiment>(&conn)
    })
    .await?;

    Ok(Json(experiments))
}

#[get("experiment/{id}")]
pub async fn fetch_experiment(
    pool: web::Data<DBPool>,
    experiment_id: web::Path<ModelId>,
    user: User,
) -> Result<Json<Experiment>> {
    let conn = pool.get().unwrap();

    let experiment = web::block(move || {
        experiments::table
            .filter(experiments::user_id.eq(user.id))
            .find(experiment_id.into_inner())
            .first::<Experiment>(&conn)
    })
    .await?;

    Ok(Json(experiment))
}

#[get("experiment/{id}/jobs")]
pub async fn fetch_experiment_jobs(
    pool: web::Data<DBPool>,
    experiment_id: web::Path<ModelId>,
    user: User,
    pagination: web::Query<PaginationRequest>,
) -> Result<Json<Pagination<(SlimJob, SlimController)>>> {
    let conn = pool.get().unwrap();

    let jobs = web::block(move || {
        let experiment = experiments::table
            .filter(experiments::user_id.eq(user.id))
            .find(experiment_id.into_inner())
            .first::<Experiment>(&conn)?;

        jobs::table
            .filter(jobs::experiment_id.eq(experiment.id))
            .inner_join(controllers::table)
            .order_by(jobs::id.desc())
            .select(((SLIM_JOB_COLUMNS, SLIM_CONTROLLER_COLUMNS), CountStarOver))
            .paginate(pagination.page)
            .per_page(pagination.per_page)
            .load_and_count_pages::<(SlimJob, SlimController)>(&conn)
    })
    .await?;

    Ok(Json(jobs))
}

#[get("job/{id}")]
pub async fn fetch_job(
    pool: web::Data<DBPool>,
    job_id: web::Path<ModelId>,
    user: User,
) -> Result<Json<(Job, SlimController)>> {
    let conn = pool.get().unwrap();

    let job = web::block(move || {
        jobs::table
            .filter(jobs::id.eq(job_id.into_inner()))
            .inner_join(controllers::table)
            .inner_join(experiments::table)
            .filter(experiments::user_id.eq(user.id))
            .select((jobs::all_columns, SLIM_CONTROLLER_COLUMNS))
            .first::<(Job, SlimController)>(&conn)
    })
    .await?;

    Ok(Json(job))
}

#[delete("job/{id}/abort")]
pub async fn abort_running_job(
    pool: web::Data<DBPool>,
    job_id: web::Path<ModelId>,
    user: User,
    experiment_server: web::Data<Addr<ExperimentServer>>
) -> Result<Json<SuccessResponse>> {
    let conn = pool.get().unwrap();

    let option = web::block(move || {
        conn.transaction(|| {
            let job = jobs::table
                .find(job_id.into_inner())
                .inner_join(experiments::table)
                .filter(experiments::user_id.eq(user.id))
                .for_update()
                .select(jobs::all_columns)
                .first::<Job>(&conn)?;

            match job.status {
                JobStatus::Running => Ok(Some((job.id, job.controller_id))),
                JobStatus::Pending => {
                    diesel::update(&job)
                        .set(jobs::status.eq(JobStatus::Failed.value()))
                        .execute(&conn)?;

                    Ok(None)
                }
                _ => Err(Box::new(CoreErrorMessage::InvalidOperationForStatus)
                    as Box<dyn ErrorMessaging>),
            }
        })
    })
    .await?;

    if let Some((job_id, controller_id)) = option {
        experiment_server.send(AbortRunningJob {job_id, controller_id})
            .await
            .map_err(|_| CoreErrorMessage::Custom("Failed to send AbortRunningJob message to experiment server"))?;
    }

    Ok(Json(SuccessResponse::default()))
}

#[post("experiment")]
pub async fn create_new_experiment(
    pool: web::Data<DBPool>,
    user: User,
    request: SanitizedJson<ExperimentNameRequest>,
) -> Result<Json<Experiment>> {
    let conn = pool.get().unwrap();
    let request = request.into_inner();

    let experiment = web::block(move || {
        diesel::insert_into(experiments::table)
            .values((
                experiments::user_id.eq(user.id),
                experiments::name.eq(request.name),
            ))
            .get_result::<Experiment>(&conn)
    })
    .await?;

    Ok(Json(experiment))
}

/// This will return a SuccessResponse even though update may not occur if experiment's user id is not
/// equal to user.id. Update endpoints will generally behave like this.
#[put("experiment/{id}")]
pub async fn update_experiment_name(
    pool: web::Data<DBPool>,
    experiment_id: web::Path<ModelId>,
    user: User,
    request: SanitizedJson<ExperimentNameRequest>,
) -> Result<Json<SuccessResponse>> {
    let conn = pool.get().unwrap();

    web::block(move || {
        diesel::update(
            experiments::table
                .filter(experiments::user_id.eq(user.id))
                .find(experiment_id.into_inner()),
        )
        .set(experiments::name.eq(request.into_inner().name))
        .execute(&conn)
    })
    .await?;

    Ok(Json(SuccessResponse::default()))
}

#[post("experiment/{experiment_id}/run/{controller_id}")]
pub async fn run_experiment(
    pool: web::Data<DBPool>,
    experiment_server: web::Data<Addr<ExperimentServer>>,
    ids: web::Path<(ModelId, ModelId)>,
    user: User,
) -> Result<Json<Job>> {
    let conn = pool.get().unwrap();
    let (experiment_id, controller_id) = ids.into_inner();
    let user_id = user.id;

    let mut job = web::block(move || -> Result<Job> {
        let experiment = experiments::table
            .filter(experiments::user_id.eq(user.id))
            .find(experiment_id)
            .first::<Experiment>(&conn)?;

        let controller = controllers::table.find(controller_id).first::<Controller>(&conn)?;

        let now = Utc::now().naive_utc();

        let slot_exist: bool = diesel::dsl::select(diesel::dsl::exists(
            slots::table
                .filter(slots::start_at.lt(&now).and(slots::end_at.gt(&now)))
                .filter(
                    slots::user_id
                        .eq(user.id)
                        .and(slots::controller_id.eq(controller.id)),
                ),
        ))
        .get_result(&conn)?;

        if !slot_exist {
            return Err(Box::new(ErrorMessage::NotAllowedToRunForSlot));
        }

        diesel::insert_into(jobs::table)
            .values((
                jobs::experiment_id.eq(experiment.id),
                jobs::controller_id.eq(controller.id),
                jobs::code.eq(experiment.code),
            ))
            .get_result::<Job>(&conn)
            .map_err(|e| e.into())
    })
    .await?;

    let job_id = job.id;

    if let Err(e) = experiment_server
        .send(RunExperiment {
            code: job.code.clone(),
            job_id,
            controller_id,
            user_id,
        })
        .await
    {
        error!("Error while sending run to ExperimentServer: {:?}", e);

        web::block(move || {
            diesel::update(jobs::table.find(job_id))
                .set(jobs::status.eq(JobStatus::Failed.value()))
                .execute(&pool.get().unwrap())
        })
        .await?;

        job.status = JobStatus::Failed;
    }

    Ok(Json(job))
}

#[put("experiment/{id}/code")]
pub async fn update_experiment_code(
    pool: web::Data<DBPool>,
    experiment_id: web::Path<ModelId>,
    user: User,
    request: SanitizedJson<ExperimentCodeRequest>,
) -> Result<Json<SuccessResponse>> {
    let conn = pool.get().unwrap();

    web::block(move || {
        diesel::update(
            experiments::table
                .filter(experiments::user_id.eq(user.id))
                .find(experiment_id.into_inner()),
        )
        .set(experiments::code.eq(request.into_inner().code))
        .execute(&conn)
    })
    .await?;

    Ok(Json(SuccessResponse::default()))
}

/// This will return a SuccessResponse even though delete may not occur if experiment's user id is not
/// equal to user.id. Delete endpoints will generally behave like this.
#[delete("experiment/{id}")]
pub async fn delete_experiment(
    pool: web::Data<DBPool>,
    experiment_id: web::Path<ModelId>,
    user: User,
) -> Result<Json<SuccessResponse>> {
    let conn = pool.get().unwrap();

    web::block(move || {
        diesel::delete(
            experiments::table
                .filter(experiments::user_id.eq(user.id))
                .find(experiment_id.into_inner()),
        )
        .execute(&conn)
    })
    .await?;

    Ok(Json(SuccessResponse::default()))
}

#[get("controller/{id}/token")]
pub async fn controller_token(pool: web::Data<DBPool>, hash: web::Data<Hash>, controller_id: web::Path<ModelId>) -> Result<Json<TokenResponse>> {
    let conn = pool.get().unwrap();

    let access_key = web::block(move ||
                                controllers::table.find(controller_id.into_inner())
                                .select(controllers::access_key)
                                .first(&conn)
        )
        .await?;

    let token = hash.encode::<ControllerToken>(&ControllerToken{
        access_key,
        exp: std::i64::MAX
    })
        .map_err(|_| CoreErrorMessage::HashFailed)?;

    Ok(Json(TokenResponse{token}))
}

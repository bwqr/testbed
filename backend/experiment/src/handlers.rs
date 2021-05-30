use actix::Addr;
use actix_web::{delete, get, HttpRequest, HttpResponse, post, put, web, web::Json};
use actix_web_actors::ws;
use diesel::prelude::*;
use log::error;
use serde_json::json;

use core::db::DieselEnum;
use core::error::ErrorMessaging;
use core::ErrorMessage as CoreErrorMessage;
use core::models::paginate::{CountStarOver, Paginate, Pagination, PaginationRequest};
use core::responses::SuccessResponse;
use core::sanitized::SanitizedJson;
use core::schema::{experiments, jobs, runners};
use core::types::{DBPool, DefaultResponse, ModelId, Result};
use core::utils::Hash;
use shared::JoinServerRequest;
use user::models::user::User;

use crate::connection::ReceiverValues;
use crate::connection::server::{ExperimentServer, RunExperiment};
use crate::connection::session::Session;
use crate::ErrorMessage;
use crate::models::experiment::{Experiment, SLIM_EXPERIMENT_COLUMNS, SlimExperiment};
use crate::models::job::{Job, JobStatus, SLIM_JOB_COLUMNS, SlimJob};
use crate::models::runner::{Runner, RunnerToken, SLIM_RUNNER_COLUMNS, SlimRunner};
use crate::requests::{ExperimentCodeRequest, ExperimentNameRequest};

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

    let token = hash.decode::<RunnerToken>(join_server_request.token.as_str())
        .map_err(|_| CoreErrorMessage::InvalidToken)?;

    let runner = web::block(move || runners::table
        .filter(runners::access_key.eq(token.access_key))
        .first::<Runner>(&conn)
    )
        .await?;

    ws::start(Session::new(experiment_server.get_ref().clone(), runner.id, join_server_request.runner_state), &req, stream)
        .map_err(|_| Box::new(CoreErrorMessage::WebSocketConnectionError) as Box<dyn ErrorMessaging>)
}

#[get("runners")]
pub async fn fetch_runners(pool: web::Data<DBPool>) -> Result<Json<Vec<SlimRunner>>> {
    let conn = pool.get().unwrap();

    let runners = web::block(move ||
        runners::table
            .select(SLIM_RUNNER_COLUMNS)
            .load::<SlimRunner>(&conn)
    )
        .await?;

    Ok(Json(runners))
}

#[get("runner/{id}")]
pub async fn fetch_runner(pool: web::Data<DBPool>, runner_id: web::Path<ModelId>) -> Result<Json<SlimRunner>> {
    let conn = pool.get().unwrap();

    let runner = web::block(move ||
        runners::table
            .find(runner_id.into_inner())
            .select(SLIM_RUNNER_COLUMNS)
            .first::<SlimRunner>(&conn)
    )
        .await?;

    Ok(Json(runner))
}

#[get("runner/{id}/values")]
pub async fn runner_receiver_values(experiment_server: web::Data<Addr<ExperimentServer>>, runner_id: web::Path<ModelId>) -> DefaultResponse {
    let values = experiment_server.send(ReceiverValues { runner_id: runner_id.into_inner() })
        .await
        .map_err(|_| CoreErrorMessage::Custom("failed to receive receiver status from experiment server"))?
        .map_err(|_| ErrorMessage::UnknownRunner)?;

    Ok(HttpResponse::Ok().json(json!({
        "values": values
    })))
}

#[get("experiments")]
pub async fn fetch_experiments(pool: web::Data<DBPool>, user: User, pagination: web::Query<PaginationRequest>) -> Result<Json<Pagination<SlimExperiment>>> {
    let conn = pool.get().unwrap();

    let experiments = web::block(move || experiments::table
        .filter(experiments::user_id.eq(user.id))
        .order(experiments::created_at.desc())
        .select((SLIM_EXPERIMENT_COLUMNS, CountStarOver))
        .paginate(pagination.page)
        .per_page(pagination.per_page)
        .load_and_count_pages::<SlimExperiment>(&conn)
    )
        .await?;

    Ok(Json(experiments))
}

#[get("experiment/{id}")]
pub async fn fetch_experiment(pool: web::Data<DBPool>, experiment_id: web::Path<ModelId>, user: User) -> Result<Json<Experiment>> {
    let conn = pool.get().unwrap();

    let experiment = web::block(move ||
        experiments::table
            .filter(experiments::user_id.eq(user.id))
            .find(experiment_id.into_inner())
            .first::<Experiment>(&conn)
    )
        .await?;

    Ok(Json(experiment))
}

#[get("experiment/{id}/jobs")]
pub async fn fetch_experiment_jobs(pool: web::Data<DBPool>, experiment_id: web::Path<ModelId>, user: User, pagination: web::Query<PaginationRequest>) -> Result<Json<Pagination<(SlimJob, SlimRunner)>>> {
    let conn = pool.get().unwrap();

    let jobs = web::block(move || {
        let experiment = experiments::table
            .filter(experiments::user_id.eq(user.id))
            .find(experiment_id.into_inner())
            .first::<Experiment>(&conn)?;

        jobs::table
            .filter(jobs::experiment_id.eq(experiment.id))
            .inner_join(runners::table)
            .order_by(jobs::id.desc())
            .select(((SLIM_JOB_COLUMNS, SLIM_RUNNER_COLUMNS), CountStarOver))
            .paginate(pagination.page)
            .per_page(pagination.per_page)
            .load_and_count_pages::<(SlimJob, SlimRunner)>(&conn)
    })
        .await?;

    Ok(Json(jobs))
}

#[get("job/{id}")]
pub async fn fetch_job(pool: web::Data<DBPool>, job_id: web::Path<ModelId>, user: User) -> Result<Json<(Job, SlimRunner)>> {
    let conn = pool.get().unwrap();

    let job = web::block(move ||
        jobs::table
            .filter(jobs::id.eq(job_id.into_inner()))
            .inner_join(runners::table)
            .inner_join(experiments::table)
            .filter(experiments::user_id.eq(user.id))
            .select((jobs::all_columns, SLIM_RUNNER_COLUMNS))
            .first::<(Job, SlimRunner)>(&conn)
    )
        .await?;

    Ok(Json(job))
}

#[post("experiment")]
pub async fn create_new_experiment(pool: web::Data<DBPool>, user: User, request: SanitizedJson<ExperimentNameRequest>) -> Result<Json<Experiment>> {
    let conn = pool.get().unwrap();
    let request = request.into_inner();

    let experiment = web::block(move || diesel::insert_into(experiments::table)
        .values(
            (experiments::user_id.eq(user.id), experiments::name.eq(request.name))
        )
        .get_result::<Experiment>(&conn)
    )
        .await?;

    Ok(Json(experiment))
}

/// This will return a SuccessResponse even though update may not occur if experiment's user id is not
/// equal to user.id. Update endpoints will generally behave like this.
#[put("experiment/{id}")]
pub async fn update_experiment_name(pool: web::Data<DBPool>, experiment_id: web::Path<ModelId>, user: User, request: SanitizedJson<ExperimentNameRequest>)
                                    -> Result<Json<SuccessResponse>> {
    let conn = pool.get().unwrap();

    web::block(move ||
        diesel::update(
            experiments::table
                .filter(experiments::user_id.eq(user.id))
                .find(experiment_id.into_inner())
        )
            .set(experiments::name.eq(request.into_inner().name))
            .execute(&conn)
    )
        .await?;

    Ok(Json(SuccessResponse::default()))
}

#[post("experiment/{experiment_id}/run/{runner_id}")]
pub async fn run_experiment(
    pool: web::Data<DBPool>,
    experiment_server: web::Data<Addr<ExperimentServer>>,
    ids: web::Path<(ModelId, ModelId)>,
    user: User,
) -> Result<Json<Job>> {
    let conn = pool.get().unwrap();
    let (experiment_id, runner_id) = ids.into_inner();
    let user_id = user.id;

    let mut job = web::block(move || {
        let experiment = experiments::table
            .filter(experiments::user_id.eq(user.id))
            .find(experiment_id)
            .first::<Experiment>(&conn)?;

        let runner = runners::table
            .find(runner_id)
            .first::<Runner>(&conn)?;

        diesel::insert_into(jobs::table)
            .values((jobs::experiment_id.eq(experiment.id), jobs::runner_id.eq(runner.id), jobs::code.eq(experiment.code)))
            .get_result::<Job>(&conn)
    })
        .await?;

    let job_id = job.id;

    if let Err(e) = experiment_server.send(RunExperiment {
        code: job.code.clone(),
        job_id,
        runner_id,
        user_id,
    })
        .await {
        error!("Error while sending run to ExperimentServer: {:?}", e);

        web::block(move || diesel::update(jobs::table.find(job_id))
            .set(jobs::status.eq(JobStatus::Failed.value()))
            .execute(&pool.get().unwrap())
        )
            .await?;

        job.status = JobStatus::Failed;
    }

    Ok(Json(job))
}

#[put("experiment/{id}/code")]
pub async fn update_experiment_code(pool: web::Data<DBPool>, experiment_id: web::Path<ModelId>, user: User, request: SanitizedJson<ExperimentCodeRequest>)
                                    -> Result<Json<SuccessResponse>> {
    let conn = pool.get().unwrap();

    web::block(move ||
        diesel::update(
            experiments::table
                .filter(experiments::user_id.eq(user.id))
                .find(experiment_id.into_inner())
        )
            .set(experiments::code.eq(request.into_inner().code))
            .execute(&conn)
    )
        .await?;

    Ok(Json(SuccessResponse::default()))
}

/// This will return a SuccessResponse even though delete may not occur if experiment's user id is not
/// equal to user.id. Delete endpoints will generally behave like this.
#[delete("experiment/{id}")]
pub async fn delete_experiment(pool: web::Data<DBPool>, experiment_id: web::Path<ModelId>, user: User) -> Result<Json<SuccessResponse>> {
    let conn = pool.get().unwrap();

    web::block(move ||
        diesel::delete(
            experiments::table
                .filter(experiments::user_id.eq(user.id))
                .find(experiment_id.into_inner())
        ).execute(&conn)
    )
        .await?;

    Ok(Json(SuccessResponse::default()))
}
use actix_web::{delete, get, post, web, web::Json};
use chrono::{NaiveDateTime, Utc};
use diesel::prelude::*;

use core::responses::SuccessResponse;
use core::schema::{runners, slots};
use core::types::{DBPool, ModelId, Result};
use experiment::models::runner::{SLIM_RUNNER_COLUMNS, SlimRunner};
use user::models::user::User;

use crate::ErrorMessage;
use crate::models::Slot;
use crate::requests::{ReservedQueryRequest, SlotReserveRequest};

// in seconds
const SLOT_RUN_TIME: i64 = 60 * 50;
const SLOT_IDLE_TIME: i64 = 60 * 10;

/// returns all slots belonging to a user and not ended yet
#[get("slots")]
pub async fn fetch_slots(pool: web::Data<DBPool>, user: User) -> Result<Json<Vec<(Slot, SlimRunner)>>> {
    let conn = pool.get().unwrap();
    let now = Utc::now().naive_utc();

    let slots = web::block(move ||
        slots::table
            .filter(slots::end_at.gt(now))
            .filter(slots::user_id.eq(user.id))
            .inner_join(runners::table)
            .select((slots::all_columns, SLIM_RUNNER_COLUMNS))
            .load::<(Slot, SlimRunner)>(&conn)
    )
        .await?;

    Ok(Json(slots))
}

#[get("slot/{id}")]
pub async fn fetch_slot(pool: web::Data<DBPool>, user: User, slot_id: web::Path<ModelId>) -> Result<Json<Slot>> {
    let conn = pool.get().unwrap();

    let slot = web::block(move ||
        slots::table
            .filter(slots::user_id.eq(user.id))
            .find(slot_id.into_inner())
            .first::<Slot>(&conn)
    )
        .await?;

    Ok(Json(slot))
}

#[get("slots/reserved")]
pub async fn fetch_resolved_slots(pool: web::Data<DBPool>, query: web::Query<ReservedQueryRequest>) -> Result<Json<Vec<NaiveDateTime>>> {
    let conn = pool.get().unwrap();
    let query = query.into_inner();

    let mut start_at_timestamp = query.start_at.timestamp();
    //truncate to hour
    start_at_timestamp -= start_at_timestamp % 3600;

    let start_at_beginning = NaiveDateTime::from_timestamp(start_at_timestamp, 0);
    let start_at_ending = NaiveDateTime::from_timestamp(start_at_timestamp + (SLOT_RUN_TIME + SLOT_IDLE_TIME) * query.count as i64, 0);

    let reserved_slots_start_ats = web::block(move ||
        slots::table
            .filter(slots::start_at.ge(start_at_beginning).and(slots::start_at.lt(start_at_ending)))
            .filter(slots::runner_id.eq(query.runner_id))
            .order_by(slots::start_at.asc())
            .select(slots::start_at)
            .load::<NaiveDateTime>(&conn)
    )
        .await?;

    Ok(Json(reserved_slots_start_ats))
}

#[post("slot")]
pub async fn reserve_slot(pool: web::Data<DBPool>, user: User, reserve_request: web::Json<SlotReserveRequest>) -> Result<Json<Slot>> {
    let reserve_request = reserve_request.into_inner();

    let mut now = Utc::now().timestamp();
    now -= now % 3600;

    let mut start_at_timestamp = reserve_request.start_at.timestamp();
    // truncate time to hour
    start_at_timestamp -= start_at_timestamp % 3600;
    if start_at_timestamp < now {
        return Err(Box::new(ErrorMessage::InvalidSlotInterval));
    }

    let start_at = NaiveDateTime::from_timestamp(start_at_timestamp, 0);
    let end_at = NaiveDateTime::from_timestamp(start_at_timestamp + SLOT_RUN_TIME, 0);
    let conn = pool.get().unwrap();

    let slot = web::block(move || -> Result<Slot> {
        let res = diesel::dsl::select(diesel::dsl::exists(slots::table
            .filter(
                (slots::end_at.ge(&start_at).and(slots::end_at.le(&end_at)))
                    .or(slots::start_at.ge(&start_at).and(slots::start_at.le(&end_at)))
            ).filter(slots::runner_id.eq(reserve_request.runner_id))
        ))
            .get_result(&conn)?;

        if res {
            return Err(Box::new(ErrorMessage::AlreadyReserved));
        }

        diesel::insert_into(slots::table)
            .values((
                slots::user_id.eq(user.id),
                slots::runner_id.eq(reserve_request.runner_id),
                slots::start_at.eq(start_at),
                slots::end_at.eq(end_at)
            ))
            .get_result(&conn)
            .map_err(|e| e.into())
    })
        .await?;

    Ok(Json(slot))
}

#[delete("slot/{id}")]
pub async fn delete_slot(pool: web::Data<DBPool>, user: User, slot_id: web::Path<ModelId>) -> Result<Json<SuccessResponse>> {
    let conn = pool.get().unwrap();

    web::block(move ||
        diesel::delete(
            slots::table
                .filter(slots::user_id.eq(user.id))
                .find(slot_id.into_inner())
        )
            .execute(&conn)
    )
        .await?;

    Ok(Json(SuccessResponse::default()))
}
use actix_web::{get, HttpResponse, post, web, web::Json};
use chrono::{NaiveDateTime, Utc};
use diesel::prelude::*;

use core::error::ErrorMessaging;
use core::ErrorMessage as CoreErrorMessage;
use core::responses::SuccessResponse;
use core::schema::slots;
use core::types::{DBPool, ModelId};
use user::models::user::User;

use crate::ErrorMessage;
use crate::models::Slot;
use crate::requests::SlotReserveRequest;

// in seconds
const SLOT_INTERVAL: i64 = 60 * 60;

/// returns all slots belonging to a user and not ended yet
#[get("slots")]
pub async fn fetch_slots(pool: web::Data<DBPool>, user: User) -> Result<Json<Vec<Slot>>, Box<dyn ErrorMessaging>> {
    let conn = pool.get().unwrap();
    let now = Utc::now().naive_utc();

    let slots = web::block(move ||
        slots::table
            .filter(slots::end_at.gt(now))
            .filter(slots::user_id.eq(user.id))
            .load::<Slot>(&conn)
    )
        .await?;

    Ok(Json(slots))
}

#[get("slot/{id}")]
pub async fn fetch_slot(pool: web::Data<DBPool>, user: User, slot_id: web::Path<ModelId>) -> Result<Json<Slot>, Box<dyn ErrorMessaging>> {
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

#[post("slot")]
pub async fn reserve_slot(pool: web::Data<DBPool>, user: User, reserve_request: web::Json<SlotReserveRequest>) -> Result<Json<Slot>, Box<dyn ErrorMessaging>> {
    let reserve_request = reserve_request.into_inner();

    let now = Utc::now().timestamp();
    let start_at_timestamp = reserve_request.start_at.timestamp();
    // truncate time to hour
    let start_at_timestamp = start_at_timestamp - (start_at_timestamp % 3600);
    if start_at_timestamp < now {
        return Err(Box::new(ErrorMessage::InvalidSlotInterval));
    }

    let end_at = NaiveDateTime::from_timestamp(start_at_timestamp + SLOT_INTERVAL, 0);
    let conn = pool.get().unwrap();

    let slot = web::block(move || -> Result<Slot, Box<dyn ErrorMessaging>> {
        let res = diesel::dsl::select(diesel::dsl::exists(slots::table
            .filter(
                (slots::end_at.gt(reserve_request.start_at).and(slots::end_at.lt(&end_at)))
                    .or(slots::start_at.gt(reserve_request.start_at).and(slots::start_at.lt(&end_at)))
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
                slots::start_at.eq(reserve_request.start_at),
                slots::end_at.eq(end_at)
            ))
            .get_result(&conn)
            .map_err(|e| e.into())
    })
        .await?;

    Ok(Json(slot))
}
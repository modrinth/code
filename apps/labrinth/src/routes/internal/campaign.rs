use actix_web::{post, web};
use chrono::{DateTime, Utc};
use eyre::eyre;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use tracing::warn;
use uuid::Uuid;

use crate::{
    database::{
        PgPool,
        models::{DBUser, generate_campaign_donation_id},
        redis::RedisPool,
    },
    routes::ApiError,
    util::error::Context,
};

pub fn config(cfg: &mut utoipa_actix_web::service_config::ServiceConfig) {
    cfg.service(tiltify_webhook);
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct TiltifyWebhook {
    pub data: TiltifyData,
    pub meta: TiltifyMeta,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct TiltifyData {
    pub amount_raised: AmountRaised,
    pub user: TiltifyUser,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct AmountRaised {
    pub currency: String,
    pub value: Decimal,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct TiltifyUser {
    pub id: Uuid,
    pub username: String,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct TiltifyMeta {
    pub attempted_at: DateTime<Utc>,
    pub event_type: String,
    pub generated_at: DateTime<Utc>,
    pub id: Uuid,
    pub subscription_source_id: Uuid,
    pub subscription_source_type: String,
}

#[utoipa::path]
#[post("/webhook")]
pub async fn tiltify_webhook(
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    web::Json(raw_payload): web::Json<serde_json::Value>,
) -> Result<(), ApiError> {
    // deserialize the JSON in the request handler, not in the params,
    // since if the JSON fails to deserialize then it's *our* fault,
    // not the caller's.
    let payload = TiltifyWebhook::deserialize(&raw_payload)
        .wrap_internal_err_with(|| {
            eyre!(
                "invalid Tiltify webhook payload schema\n{}",
                serde_json::to_string_pretty(&raw_payload)
                    .expect("serializing should not fail")
            )
        })?;

    // no matter what, we need to insert this donation record into the db
    let mut transaction = pool.begin().await?;
    let id = generate_campaign_donation_id(&mut transaction).await?;
    sqlx::query!(
        "
        insert into campaign_donations (id, raw_data, donated_at, user_id)
        values ($1, $2, $3, $4)
        ",
        id.0,
        raw_payload,
        payload.meta.generated_at,
        None::<i64>,
    )
    .execute(&**pool)
    .await
    .wrap_internal_err("inserting campaign donation")?;

    transaction
        .commit()
        .await
        .wrap_internal_err("committing donation transaction")?;

    // then we can attempt user lookups
    let username = &payload.data.user.username;
    let user = DBUser::get(username, &**pool, &redis)
        .await
        .wrap_internal_err("fetching user from database")?;

    if let Some(user) = user {
        sqlx::query!(
            "
            update campaign_donations set user_id = $1 where id = $2
            ",
            user.id.0,
            id.0,
        )
        .execute(&**pool)
        .await
        .wrap_internal_err("updating campaign donation user")?;
    } else {
        warn!("Got donation for user '{username}' which does not exist");
    }

    Ok(())
}

use actix_web::{HttpRequest, post, web};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use xredis::RedisPool;

use super::{Adjustment, PayoutRunPayload};
use crate::{
    auth::{
        AuthenticationError, get_user_from_headers, two_factor::verify_2fa_code,
    },
    database::{
        PgPool,
        models::{
            DBUserId, generate_payout_run_id,
            payout_run_item::{DBPayoutRun, PayoutRunStatus},
        },
    },
    models::{ids::PayoutRunId, pats::Scopes},
    queue::{payout_run::estimate, session::AuthQueue},
    routes::ApiError,
    util::{
        error::Context,
        time::{YearMonth, net_60_payout_available_at},
    },
};

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct StartPayoutRun {
    pub period: YearMonth,
    pub two_factor_code: Option<String>,
    /// Skip TOTP verification when testing this route locally.
    ///
    /// This field does not exist in release builds.
    #[cfg(debug_assertions)]
    #[serde(default)]
    pub ignore_totp: bool,
    #[serde(with = "rust_decimal::serde::float")]
    pub raw_actual_revenue_usd: Decimal,
    pub adjustments: Vec<Adjustment>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct StartPayoutRunResponse {
    pub id: PayoutRunId,
    pub execute_at: DateTime<Utc>,
}

/// Start a payout run.
///
/// Admin-only.
#[utoipa::path(
    tag = "payout runs",
    request_body = StartPayoutRun,
    responses(
        (status = OK, body = StartPayoutRunResponse),
        (status = BAD_REQUEST, description = "Invalid payout run input"),
        (status = UNAUTHORIZED, description = "Invalid authentication or TOTP code"),
        (status = CONFLICT, description = "Payout period is unavailable or another run is active"),
    ),
    security(("bearer_auth" = ["SESSION_ACCESS"])),
)]
#[post("/start")]
pub async fn start_run(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    aditude: web::Data<aditude::Client>,
    session_queue: web::Data<AuthQueue>,
    web::Json(body): web::Json<StartPayoutRun>,
) -> Result<web::Json<StartPayoutRunResponse>, ApiError> {
    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::SESSION_ACCESS,
    )
    .await
    .wrap_auth_err("authenticating API request")?
    .1;
    if !user.role.is_admin() {
        return Err(ApiError::Auth(eyre::eyre!(
            AuthenticationError::InvalidCredentials,
        )));
    }
    let user_id = DBUserId::from(user.id);

    #[cfg(debug_assertions)]
    let ignore_totp = body.ignore_totp;
    #[cfg(not(debug_assertions))]
    let ignore_totp = false;

    if !ignore_totp {
        let two_factor_code = body
            .two_factor_code
            .as_deref()
            .wrap_auth_err_with(|| AuthenticationError::InvalidCredentials)?;
        let secret = sqlx::query_scalar!(
            r#"
        SELECT totp_secret
        FROM users
        WHERE id = $1
        "#,
            user_id.0,
        )
        .fetch_one(&**pool)
        .await
        .wrap_internal_err("fetching user two-factor secret")?
        .wrap_auth_err_with(|| AuthenticationError::InvalidCredentials)?;
        let valid_totp =
            verify_2fa_code(two_factor_code, &secret, user_id, &redis)
                .await
                .wrap_auth_err("verifying two-factor code")?;
        if !valid_totp {
            return Err(ApiError::Auth(eyre::eyre!(
                AuthenticationError::InvalidCredentials,
            )));
        }
    }

    if body.raw_actual_revenue_usd.is_sign_negative() {
        return Err(ApiError::Request(eyre::eyre!(
            "`raw_actual_revenue_usd` cannot be negative",
        )));
    }

    let available_at = net_60_payout_available_at(body.period)
        .wrap_request_err("calculating payout period availability")?;
    if Utc::now() < available_at {
        return Err(ApiError::Conflict(eyre::eyre!(
            "payout period is still open",
        )));
    }

    let mut estimates =
        estimate(aditude.get_ref(), redis.get_ref(), &[body.period])
            .await
            .wrap_internal_err("fetching payout estimate")?;
    let estimate = estimates
        .pop()
        .wrap_internal_err("missing requested payout estimate")?;
    let payload = PayoutRunPayload {
        raw_actual_revenue_usd: body.raw_actual_revenue_usd,
        adjustments: body.adjustments.clone(),
    };

    let mut transaction = pool
        .begin()
        .await
        .wrap_internal_err("starting database transaction")?;

    sqlx::query!(
        r#"
        INSERT INTO payout_periods (
            period,
            raw_actual_aditude_revenue_usd,
            adjustments
        )
        VALUES ($1, $2, '[]'::jsonb)
        ON CONFLICT (period) DO UPDATE SET
            raw_actual_aditude_revenue_usd =
                EXCLUDED.raw_actual_aditude_revenue_usd
        "#,
        body.period.date(),
        body.raw_actual_revenue_usd,
    )
    .execute(&mut transaction)
    .await
    .wrap_internal_err("storing payout period")?;

    let run_state = sqlx::query!(
        r#"
        SELECT
            EXISTS (
                SELECT 1
                FROM payout_runs
                WHERE status IN ('scheduled', 'running')
            ) AS "has_active_run!",
            EXISTS (
                SELECT 1
                FROM payout_runs
                WHERE period = $1
                    AND status = 'succeeded'
            ) AS "has_succeeded_run!"
        "#,
        body.period.date(),
    )
    .fetch_one(&mut transaction)
    .await
    .wrap_internal_err("checking payout period run status")?;
    if run_state.has_active_run {
        return Err(ApiError::Conflict(eyre::eyre!(
            "another payout run is already scheduled or running",
        )));
    }
    if run_state.has_succeeded_run {
        return Err(ApiError::Conflict(eyre::eyre!(
            "payout period already has a succeeded run",
        )));
    }

    sqlx::query!(
        r#"
        DELETE FROM payout_period_days
        WHERE period = $1
        "#,
        body.period.date(),
    )
    .execute(&mut transaction)
    .await
    .wrap_internal_err("clearing stored payout period days")?;

    for day in estimate.days {
        let impressions = i64::try_from(day.impressions)
            .wrap_internal_err("converting Aditude impressions")?;
        sqlx::query!(
            r#"
            INSERT INTO payout_period_days (
                period,
                date,
                raw_estimated_aditude_revenue_usd,
                aditude_impressions
            )
            VALUES ($1, $2, $3, $4)
            "#,
            body.period.date(),
            day.date,
            day.raw_estimated_revenue_usd,
            impressions,
        )
        .execute(&mut transaction)
        .await
        .wrap_internal_err("storing payout period day")?;
    }

    let timing = sqlx::query!(
        r#"
        SELECT
            NOW() AS "started_at!",
            NOW() + INTERVAL '2 minutes' AS "execute_at!"
        "#,
    )
    .fetch_one(&mut transaction)
    .await
    .wrap_internal_err("calculating payout run schedule")?;
    let id = generate_payout_run_id(&mut transaction)
        .await
        .wrap_internal_err("generating payout run ID")?;
    let run = DBPayoutRun {
        id,
        period: body.period.date(),
        payload,
        status: PayoutRunStatus::Scheduled,
        started_at: timing.started_at,
        started_by: user_id,
        execute_at: timing.execute_at,
        processing_started_at: None,
        finished_at: None,
        cancelled_at: None,
        cancelled_by: None,
        error: None,
    };
    run.upsert(&mut transaction)
        .await
        .wrap_internal_err("creating scheduled payout run")?;

    transaction
        .commit()
        .await
        .wrap_internal_err("committing payout run")?;

    Ok(web::Json(StartPayoutRunResponse {
        id: id.into(),
        execute_at: run.execute_at,
    }))
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct CancelPayoutRunsResponse {
    pub cancelled: u64,
}

/// Cancel all scheduled payout runs.
///
/// Admin-only.
#[utoipa::path(
    tag = "payout runs",
    responses(
        (status = OK, body = CancelPayoutRunsResponse),
        (status = UNAUTHORIZED, description = "Invalid authentication"),
    ),
    security(("bearer_auth" = ["SESSION_ACCESS"])),
)]
#[post("/cancel")]
pub async fn cancel_runs(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<web::Json<CancelPayoutRunsResponse>, ApiError> {
    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::SESSION_ACCESS,
    )
    .await
    .wrap_auth_err("authenticating API request")?
    .1;
    if !user.role.is_admin() {
        return Err(ApiError::Auth(eyre::eyre!(
            AuthenticationError::InvalidCredentials,
        )));
    }
    let user_id = DBUserId::from(user.id);

    let result = sqlx::query!(
        r#"
        UPDATE payout_runs
        SET
            status = 'cancelled',
            cancelled_at = NOW(),
            cancelled_by = $1
        WHERE status = 'scheduled'
        "#,
        user_id.0,
    )
    .execute(&**pool)
    .await
    .wrap_internal_err("cancelling scheduled payout runs")?;

    Ok(web::Json(CancelPayoutRunsResponse {
        cancelled: result.rows_affected(),
    }))
}

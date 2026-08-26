use actix_http::StatusCode;
use actix_web::test;
use ariadne::ids::base62_impl::parse_base62;
use chrono::{DateTime, Duration, NaiveDate, TimeZone, Utc};
use rust_decimal::Decimal;
use serde::Deserialize;
use serde_json::json;

use common::{
    api_v3::ApiV3,
    database::{USER_USER_ID_PARSED, USER_USER_PAT},
    environment::{TestEnvironment, with_test_environment},
};

pub mod common;

#[derive(Debug, Deserialize)]
struct TestBalance {
    available: Decimal,
    withdrawn_lifetime: Decimal,
    withdrawn_ytd: Decimal,
    pending: Decimal,
    dates: std::collections::HashMap<DateTime<Utc>, Decimal>,
    estimated_dates: std::collections::HashMap<DateTime<Utc>, Decimal>,
    actual_dates: std::collections::HashMap<DateTime<Utc>, Decimal>,
}

async fn get_balance(test_env: &TestEnvironment<ApiV3>) -> TestBalance {
    let request = test::TestRequest::get()
        .uri("/v3/payout/balance")
        .append_header(("Authorization", USER_USER_PAT.unwrap()))
        .to_request();
    let response = test_env.call(request).await;
    assert_status!(&response, StatusCode::OK);
    test::read_body_json(response).await
}

fn amount_delta(
    current: &std::collections::HashMap<DateTime<Utc>, Decimal>,
    initial: &std::collections::HashMap<DateTime<Utc>, Decimal>,
    date: DateTime<Utc>,
) -> Decimal {
    current.get(&date).copied().unwrap_or_default()
        - initial.get(&date).copied().unwrap_or_default()
}

async fn insert_succeeded_period(
    pool: &sqlx::PgPool,
    period: NaiveDate,
    run_id: i64,
) {
    sqlx::query!(
        r#"
        INSERT INTO payout_periods (
            period,
            raw_actual_aditude_revenue_usd,
            adjustments
        )
        VALUES ($1, 100, '[]'::jsonb)
        "#,
        period,
    )
    .execute(pool)
    .await
    .unwrap();

    let started_at = Utc.with_ymd_and_hms(2001, 1, 1, 0, 0, 0).unwrap();
    sqlx::query!(
        r#"
        INSERT INTO payout_runs (
            id,
            period,
            payload,
            status,
            started_at,
            started_by,
            execute_at,
            processing_started_at,
            finished_at
        )
        VALUES (
            $1,
            $2,
            '{"raw_actual_revenue_usd":100,"adjustments":[]}'::jsonb,
            'succeeded',
            $3,
            $4,
            $5,
            $5,
            $6
        )
        "#,
        run_id,
        period,
        started_at,
        USER_USER_ID_PARSED,
        started_at + Duration::minutes(2),
        started_at + Duration::minutes(3),
    )
    .execute(pool)
    .await
    .unwrap();
}

#[actix_rt::test]
async fn estimate_only_period_uses_estimate_as_pending_revenue() {
    with_test_environment(
        None,
        |test_env: TestEnvironment<ApiV3>| async move {
            let pool = &test_env.db.pool;
            let project_id =
                parse_base62(&test_env.dummy.project_alpha.project_id).unwrap()
                    as i64;
            let now = Utc.timestamp_opt(Utc::now().timestamp(), 0).unwrap();
            let date_available = now - Duration::days(20);
            let initial_balance = get_balance(&test_env).await;

            sqlx::query!(
                r#"
                INSERT INTO payout_estimates (
                    period,
                    user_id,
                    mod_id,
                    amount,
                    created,
                    date_available
                )
                VALUES ($1, $2, $3, 30, $4, $5)
                "#,
                NaiveDate::from_ymd_opt(2001, 1, 1).unwrap(),
                USER_USER_ID_PARSED,
                project_id,
                now - Duration::days(50),
                date_available,
            )
            .execute(pool)
            .await
            .unwrap();

            let balance = get_balance(&test_env).await;
            assert_eq!(balance.available, initial_balance.available);
            assert_eq!(
                balance.pending,
                initial_balance.pending + Decimal::from(30),
            );
            assert_eq!(
                amount_delta(
                    &balance.dates,
                    &initial_balance.dates,
                    date_available,
                ),
                Decimal::from(30),
            );
            assert_eq!(
                amount_delta(
                    &balance.estimated_dates,
                    &initial_balance.estimated_dates,
                    date_available,
                ),
                Decimal::from(30),
            );
            assert_eq!(
                amount_delta(
                    &balance.actual_dates,
                    &initial_balance.actual_dates,
                    date_available,
                ),
                Decimal::ZERO,
            );
        },
    )
    .await;
}

#[actix_rt::test]
async fn finalized_only_period_uses_actual_as_pending_revenue() {
    with_test_environment(
        None,
        |test_env: TestEnvironment<ApiV3>| async move {
            let pool = &test_env.db.pool;
            let project_id =
                parse_base62(&test_env.dummy.project_alpha.project_id).unwrap()
                    as i64;
            let now = Utc.timestamp_opt(Utc::now().timestamp(), 0).unwrap();
            let date_available = now + Duration::days(20);
            let period = NaiveDate::from_ymd_opt(2001, 2, 1).unwrap();
            let initial_balance = get_balance(&test_env).await;
            insert_succeeded_period(pool, period, 20_001).await;

            sqlx::query!(
                r#"
                INSERT INTO payouts_values (
                    user_id,
                    mod_id,
                    amount,
                    created,
                    date_available,
                    payout_run_id
                )
                VALUES ($1, $2, 40, $3, $4, 20001)
                "#,
                USER_USER_ID_PARSED,
                project_id,
                now - Duration::days(50),
                date_available,
            )
            .execute(pool)
            .await
            .unwrap();

            let balance = get_balance(&test_env).await;
            assert_eq!(balance.available, initial_balance.available);
            assert_eq!(
                balance.pending,
                initial_balance.pending + Decimal::from(40),
            );
            assert_eq!(
                amount_delta(
                    &balance.dates,
                    &initial_balance.dates,
                    date_available,
                ),
                Decimal::from(40),
            );
            assert_eq!(
                amount_delta(
                    &balance.estimated_dates,
                    &initial_balance.estimated_dates,
                    date_available,
                ),
                Decimal::ZERO,
            );
            assert_eq!(
                amount_delta(
                    &balance.actual_dates,
                    &initial_balance.actual_dates,
                    date_available,
                ),
                Decimal::from(40),
            );
        },
    )
    .await;
}

#[actix_rt::test]
async fn finalized_period_retains_estimate_but_uses_actual_balance() {
    with_test_environment(
        None,
        |test_env: TestEnvironment<ApiV3>| async move {
            let pool = &test_env.db.pool;
            let project_id =
                parse_base62(&test_env.dummy.project_alpha.project_id).unwrap()
                    as i64;
            let now = Utc.timestamp_opt(Utc::now().timestamp(), 0).unwrap();
            let date_available = now - Duration::days(20);
            let period = NaiveDate::from_ymd_opt(2001, 3, 1).unwrap();
            let initial_balance = get_balance(&test_env).await;
            insert_succeeded_period(pool, period, 20_002).await;

            sqlx::query!(
                r#"
                INSERT INTO payout_estimates (
                    period,
                    user_id,
                    mod_id,
                    amount,
                    created,
                    date_available
                )
                VALUES ($1, $2, $3, 100, $4, $5)
                "#,
                period,
                USER_USER_ID_PARSED,
                project_id,
                now - Duration::days(50),
                date_available,
            )
            .execute(pool)
            .await
            .unwrap();
            sqlx::query!(
                r#"
                INSERT INTO payouts_values (
                    user_id,
                    mod_id,
                    amount,
                    created,
                    date_available,
                    payout_run_id
                )
                VALUES ($1, $2, 80, $3, $4, 20002)
                "#,
                USER_USER_ID_PARSED,
                project_id,
                now - Duration::days(50),
                date_available,
            )
            .execute(pool)
            .await
            .unwrap();

            let balance = get_balance(&test_env).await;
            assert_eq!(
                balance.available,
                initial_balance.available + Decimal::from(80),
            );
            assert_eq!(balance.pending, initial_balance.pending);
            assert_eq!(
                amount_delta(
                    &balance.dates,
                    &initial_balance.dates,
                    date_available,
                ),
                Decimal::from(80),
            );
            assert_eq!(
                amount_delta(
                    &balance.estimated_dates,
                    &initial_balance.estimated_dates,
                    date_available,
                ),
                Decimal::from(100),
            );
            assert_eq!(
                amount_delta(
                    &balance.actual_dates,
                    &initial_balance.actual_dates,
                    date_available,
                ),
                Decimal::from(80),
            );
        },
    )
    .await;
}

#[actix_rt::test]
async fn legacy_finalized_values_remain_available_and_pending() {
    with_test_environment(
        None,
        |test_env: TestEnvironment<ApiV3>| async move {
            let pool = &test_env.db.pool;
            let project_id =
                parse_base62(&test_env.dummy.project_alpha.project_id).unwrap()
                    as i64;
            let now = Utc.timestamp_opt(Utc::now().timestamp(), 0).unwrap();
            let available_at = now - Duration::days(20);
            let pending_at = now + Duration::days(20);
            let initial_balance = get_balance(&test_env).await;

            sqlx::query!(
                r#"
                INSERT INTO payouts_values (
                    user_id,
                    mod_id,
                    amount,
                    created,
                    date_available
                )
                VALUES
                    ($1, $2, 70, $3, $4),
                    ($1, $2, 30, $5, $6)
                "#,
                USER_USER_ID_PARSED,
                project_id,
                now - Duration::days(50),
                available_at,
                now - Duration::days(10),
                pending_at,
            )
            .execute(pool)
            .await
            .unwrap();

            let balance = get_balance(&test_env).await;
            assert_eq!(
                balance.available,
                initial_balance.available + Decimal::from(70),
            );
            assert_eq!(
                balance.pending,
                initial_balance.pending + Decimal::from(30),
            );
            for (date, amount) in [
                (available_at, Decimal::from(70)),
                (pending_at, Decimal::from(30)),
            ] {
                assert_eq!(
                    amount_delta(&balance.dates, &initial_balance.dates, date,),
                    amount,
                );
                assert_eq!(
                    amount_delta(
                        &balance.actual_dates,
                        &initial_balance.actual_dates,
                        date,
                    ),
                    amount,
                );
                assert_eq!(
                    amount_delta(
                        &balance.estimated_dates,
                        &initial_balance.estimated_dates,
                        date,
                    ),
                    Decimal::ZERO,
                );
            }
        },
    )
    .await;
}

#[actix_rt::test]
async fn provisional_estimates_only_affect_pending_balance() {
    with_test_environment(
        None,
        |test_env: TestEnvironment<ApiV3>| async move {
            let pool = &test_env.db.pool;
            let project_id =
                parse_base62(&test_env.dummy.project_alpha.project_id).unwrap()
                    as i64;
            let now = Utc.timestamp_opt(Utc::now().timestamp(), 0).unwrap();
            let finalized_available_at = now - Duration::days(30);
            let estimate_past_available_at = now - Duration::days(20);
            let shared_future_available_at = now + Duration::days(20);
            let initial_balance = get_balance(&test_env).await;

            sqlx::query!(
                r#"
                INSERT INTO payouts_values (
                    user_id,
                    mod_id,
                    amount,
                    created,
                    date_available
                )
                VALUES
                    ($1, $2, 100, $3, $4),
                    ($1, $2, 20, $5, $6)
                "#,
                USER_USER_ID_PARSED,
                project_id,
                now - Duration::days(60),
                finalized_available_at,
                now - Duration::days(10),
                shared_future_available_at,
            )
            .execute(pool)
            .await
            .unwrap();

            sqlx::query!(
                r#"
                INSERT INTO payout_estimates (
                    period,
                    user_id,
                    mod_id,
                    amount,
                    created,
                    date_available
                )
                VALUES
                    ($1, $2, $3, 30, $4, $5),
                    ($6, $2, $3, 40, $7, $8)
                "#,
                NaiveDate::from_ymd_opt(2000, 1, 1).unwrap(),
                USER_USER_ID_PARSED,
                project_id,
                now - Duration::days(50),
                estimate_past_available_at,
                NaiveDate::from_ymd_opt(2000, 2, 1).unwrap(),
                now - Duration::days(10),
                shared_future_available_at,
            )
            .execute(pool)
            .await
            .unwrap();

            let balance = get_balance(&test_env).await;
            assert_eq!(
                balance.available,
                initial_balance.available + Decimal::from(100),
            );
            assert_eq!(
                balance.pending,
                initial_balance.pending + Decimal::from(90),
            );
            assert_eq!(
                balance.withdrawn_lifetime,
                initial_balance.withdrawn_lifetime,
            );
            assert_eq!(balance.withdrawn_ytd, initial_balance.withdrawn_ytd);
            assert_eq!(
                balance.dates[&finalized_available_at]
                    - initial_balance
                        .dates
                        .get(&finalized_available_at)
                        .copied()
                        .unwrap_or_default(),
                Decimal::from(100),
            );
            assert_eq!(
                balance.dates[&estimate_past_available_at]
                    - initial_balance
                        .dates
                        .get(&estimate_past_available_at)
                        .copied()
                        .unwrap_or_default(),
                Decimal::from(30),
            );
            assert_eq!(
                balance.dates[&shared_future_available_at]
                    - initial_balance
                        .dates
                        .get(&shared_future_available_at)
                        .copied()
                        .unwrap_or_default(),
                Decimal::from(60),
            );

            let history_request = test::TestRequest::get()
                .uri("/v3/payout/history")
                .append_header(("Authorization", USER_USER_PAT.unwrap()))
                .to_request();
            let history_response = test_env.call(history_request).await;
            assert_status!(&history_response, StatusCode::OK);
            let history: Vec<serde_json::Value> =
                test::read_body_json(history_response).await;
            let available_entries = history
                .iter()
                .filter(|item| item["type"] == "payout_available")
                .collect::<Vec<_>>();
            assert!(available_entries.iter().any(|item| {
                serde_json::from_value::<DateTime<Utc>>(item["created"].clone())
                    .unwrap()
                    == finalized_available_at
                    && serde_json::from_value::<Decimal>(item["amount"].clone())
                        .unwrap()
                        == Decimal::from(100)
            }));
            assert!(available_entries.iter().all(|item| {
                serde_json::from_value::<DateTime<Utc>>(item["created"].clone())
                    .unwrap()
                    != estimate_past_available_at
            }));

            let affiliate_request = test::TestRequest::post()
                .uri("/v3/analytics")
                .append_header(("Authorization", USER_USER_PAT.unwrap()))
                .set_json(json!({
                    "time_range": {
                        "start": now - Duration::days(365),
                        "end": now + Duration::days(365),
                        "resolution": { "slices": 2 }
                    },
                    "return_metrics": {
                        "affiliate_code_revenue": {
                            "bucket_by": [],
                            "filter_by": {}
                        }
                    }
                }))
                .to_request();
            let affiliate_response = test_env.call(affiliate_request).await;
            assert_status!(&affiliate_response, StatusCode::OK);
            let affiliate_analytics: serde_json::Value =
                test::read_body_json(affiliate_response).await;
            assert!(
                affiliate_analytics["metrics"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .all(|slice| slice.as_array().unwrap().is_empty())
            );
        },
    )
    .await;
}

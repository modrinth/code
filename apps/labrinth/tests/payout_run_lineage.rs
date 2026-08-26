use ariadne::ids::base62_impl::parse_base62;
use chrono::NaiveDate;
use sqlx::postgres::PgQueryResult;

use common::{
    api_v3::ApiV3,
    database::{FRIEND_USER_ID_PARSED, USER_USER_ID_PARSED},
    environment::{TestEnvironment, with_test_environment},
};
use labrinth::database::PgPool;

pub mod common;

async fn insert_run(
    pool: &PgPool,
    id: i64,
    period: NaiveDate,
    status: &str,
) -> Result<PgQueryResult, sqlx::Error> {
    sqlx::query!(
        r#"
		INSERT INTO payout_runs (
			id,
			period,
			payload,
			status,
			started_at,
			started_by,
			execute_at
		)
		VALUES (
			$1,
			$2,
			'{"raw_actual_revenue_usd": 0, "revenue_adjustments": []}'::jsonb,
			$3,
			NOW(),
			$4,
			NOW()
		)
		"#,
        id,
        period,
        status,
        USER_USER_ID_PARSED,
    )
    .execute(pool)
    .await
}

async fn set_run_status(pool: &PgPool, id: i64, status: &str) {
    sqlx::query!(
        "UPDATE payout_runs SET status = $1 WHERE id = $2",
        status,
        id,
    )
    .execute(pool)
    .await
    .unwrap();
}

async fn insert_payout_value(
    pool: &PgPool,
    payout_run_id: Option<i64>,
    user_id: i64,
    mod_id: Option<i64>,
) -> Result<PgQueryResult, sqlx::Error> {
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
		VALUES ($1, $2, 1, NOW(), NOW(), $3)
		"#,
        user_id,
        mod_id,
        payout_run_id,
    )
    .execute(pool)
    .await
}

fn assert_constraint(error: sqlx::Error, expected: &str) {
    assert_eq!(
        error
            .as_database_error()
            .and_then(|error| error.constraint()),
        Some(expected),
    );
}

#[actix_rt::test]
async fn payout_run_lineage_and_active_run_constraints() {
    with_test_environment(
        None,
        |test_env: TestEnvironment<ApiV3>| async move {
            let pool = &test_env.db.pool;
            let alpha_project_id =
                parse_base62(&test_env.dummy.project_alpha.project_id).unwrap()
                    as i64;
            let beta_project_id =
                parse_base62(&test_env.dummy.project_beta.project_id).unwrap()
                    as i64;
            let periods = [
                NaiveDate::from_ymd_opt(2000, 1, 1).unwrap(),
                NaiveDate::from_ymd_opt(2000, 2, 1).unwrap(),
                NaiveDate::from_ymd_opt(2000, 3, 1).unwrap(),
                NaiveDate::from_ymd_opt(2000, 4, 1).unwrap(),
                NaiveDate::from_ymd_opt(2000, 5, 1).unwrap(),
            ];

            sqlx::query!(
                r#"
				INSERT INTO payout_periods (
					period,
					raw_actual_aditude_revenue_usd,
					revenue_adjustments
				)
				SELECT period, 0, '[]'::jsonb
				FROM UNNEST($1::date[]) AS period
				"#,
                &periods[..],
            )
            .execute(pool)
            .await
            .unwrap();

            insert_run(pool, 1001, periods[0], "succeeded")
                .await
                .unwrap();

            insert_payout_value(pool, None, USER_USER_ID_PARSED, None)
                .await
                .unwrap();
            insert_payout_value(
                pool,
                Some(1001),
                USER_USER_ID_PARSED,
                Some(alpha_project_id),
            )
            .await
            .unwrap();
            insert_payout_value(
                pool,
                Some(1001),
                FRIEND_USER_ID_PARSED,
                Some(alpha_project_id),
            )
            .await
            .unwrap();
            insert_payout_value(
                pool,
                Some(1001),
                USER_USER_ID_PARSED,
                Some(beta_project_id),
            )
            .await
            .unwrap();

            let payout_run_ids = sqlx::query_scalar!(
                r#"
				SELECT payout_run_id
				FROM payouts_values
				WHERE
					user_id = $1
					AND (mod_id IS NULL OR mod_id = $2)
				ORDER BY payout_run_id NULLS FIRST
				"#,
                USER_USER_ID_PARSED,
                alpha_project_id,
            )
            .fetch_all(pool)
            .await
            .unwrap();
            assert_eq!(payout_run_ids, vec![None, Some(1001)]);

            let duplicate = insert_payout_value(
                pool,
                Some(1001),
                USER_USER_ID_PARSED,
                Some(alpha_project_id),
            )
            .await
            .unwrap_err();
            assert_constraint(
                duplicate,
                "payouts_values_payout_run_distribution",
            );

            let projectless = insert_payout_value(
                pool,
                Some(1001),
                USER_USER_ID_PARSED,
                None,
            )
            .await
            .unwrap_err();
            assert_constraint(
                projectless,
                "payouts_values_payout_run_creator_only",
            );

            insert_run(pool, 1002, periods[1], "scheduled")
                .await
                .unwrap();
            let second_active = insert_run(pool, 1003, periods[2], "running")
                .await
                .unwrap_err();
            assert_constraint(second_active, "payout_runs_single_active");

            set_run_status(pool, 1002, "cancelled").await;
            insert_run(pool, 1003, periods[2], "scheduled")
                .await
                .unwrap();

            set_run_status(pool, 1003, "failed").await;
            insert_run(pool, 1004, periods[3], "scheduled")
                .await
                .unwrap();

            set_run_status(pool, 1004, "succeeded").await;
            insert_run(pool, 1005, periods[4], "scheduled")
                .await
                .unwrap();

            let succeeded_periods = sqlx::query_scalar!(
                r#"
				SELECT period
				FROM payout_runs
				WHERE status = 'succeeded'
				ORDER BY period
				"#,
            )
            .fetch_all(pool)
            .await
            .unwrap();
            assert_eq!(succeeded_periods, vec![periods[0], periods[3]]);
        },
    )
    .await;
}

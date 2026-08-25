use actix_http::StatusCode;
use actix_web::test;
use ariadne::ids::base62_impl::parse_base62;
use chrono::{DateTime, Duration, Utc};
use common::permissions::PermissionsTest;
use common::permissions::PermissionsTestContext;
use common::{
    api_common::Api,
    api_v3::ApiV3,
    database::*,
    environment::{TestEnvironment, with_test_environment},
};
use labrinth::models::teams::ProjectPermissions;
use labrinth::queue::payouts;
use rust_decimal::Decimal;
use serde_json::json;

pub mod common;

#[actix_rt::test]
pub async fn analytics_revenue() {
    with_test_environment(
        None,
        |test_env: TestEnvironment<ApiV3>| async move {
            let alpha_project_id =
                test_env.dummy.project_alpha.project_id.clone();

            let pool = test_env.db.pool.clone();

            // Generate sample revenue data- directly insert into sql
            let (
                mut insert_user_ids,
                mut insert_project_ids,
                mut insert_payouts,
                mut insert_starts,
                mut insert_availables,
            ) = (Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new());

            // Note: these go from most recent to least recent
            let money_time_pairs: [(f64, DateTime<Utc>); 10] = [
                (50.0, Utc::now() - Duration::minutes(5)),
                (50.1, Utc::now() - Duration::minutes(10)),
                (101.0, Utc::now() - Duration::days(1)),
                (200.0, Utc::now() - Duration::days(2)),
                (311.0, Utc::now() - Duration::days(3)),
                (400.0, Utc::now() - Duration::days(4)),
                (526.0, Utc::now() - Duration::days(5)),
                (633.0, Utc::now() - Duration::days(6)),
                (800.0, Utc::now() - Duration::days(14)),
                (800.0, Utc::now() - Duration::days(800)),
            ];

            let project_id = parse_base62(&alpha_project_id).unwrap() as i64;
            for (money, time) in &money_time_pairs {
                insert_user_ids.push(USER_USER_ID_PARSED);
                insert_project_ids.push(project_id);
                insert_payouts.push(Decimal::from_f64_retain(*money).unwrap());
                insert_starts.push(*time);
                insert_availables.push(*time);
            }

            let mut transaction = pool.begin().await.unwrap();
            payouts::insert_payouts(
                insert_user_ids,
                insert_project_ids,
                insert_payouts,
                insert_starts,
                insert_availables,
                &mut transaction,
            )
            .await
            .unwrap();
            transaction.commit().await.unwrap();

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
                VALUES ('2000-01-01', $1, $2, 75, $3, $4)
                "#,
                USER_USER_ID_PARSED,
                project_id,
                money_time_pairs[3].1,
                Utc::now() + Duration::days(60),
            )
            .execute(&pool)
            .await
            .unwrap();

            let modern_request = test::TestRequest::post()
                .uri("/v3/analytics")
                .append_header(("Authorization", USER_USER_PAT.unwrap()))
                .set_json(json!({
                    "time_range": {
                        "start": Utc::now() - Duration::days(801),
                        "end": Utc::now() + Duration::days(1),
                        "resolution": { "slices": 802 }
                    },
                    "return_metrics": {
                        "project_revenue": {
                            "bucket_by": [],
                            "filter_by": {}
                        }
                    },
                    "project_ids": [alpha_project_id]
                }))
                .to_request();
            let modern_response = test_env.call(modern_request).await;
            assert_status!(&modern_response, StatusCode::OK);
            let modern_analytics: serde_json::Value =
                test::read_body_json(modern_response).await;
            let revenue_metrics = modern_analytics["metrics"]
                .as_array()
                .unwrap()
                .iter()
                .flat_map(|slice| slice.as_array().unwrap())
                .filter(|metric| metric["metric_kind"] == "revenue")
                .collect::<Vec<_>>();

            let actual_revenue = revenue_metrics
                .iter()
                .filter(|metric| metric["revenue_kind"] == "actual")
                .map(|metric| metric["revenue"].as_f64().unwrap())
                .sum::<f64>();
            let estimated_revenue = revenue_metrics
                .iter()
                .filter(|metric| metric["revenue_kind"] == "estimated")
                .map(|metric| metric["revenue"].as_f64().unwrap())
                .sum::<f64>();

            assert!((actual_revenue - 3871.1).abs() < 1e-9);
            assert_eq!(estimated_revenue, 75.0);

            let shared_time_slice = modern_analytics["metrics"]
                .as_array()
                .unwrap()
                .iter()
                .map(|slice| slice.as_array().unwrap())
                .find(|slice| {
                    slice
                        .iter()
                        .any(|metric| metric["revenue_kind"] == "actual")
                        && slice
                            .iter()
                            .any(|metric| metric["revenue_kind"] == "estimated")
                });
            assert!(shared_time_slice.is_some());
        },
    )
    .await;
}

#[actix_rt::test]
pub async fn permissions_analytics_revenue() {
    with_test_environment(
        None,
        |test_env: TestEnvironment<ApiV3>| async move {
            let api = &test_env.api;
            let pool = &test_env.db.pool;

            let view_analytics = ProjectPermissions::VIEW_ANALYTICS;

            // first, do check with a project
            let req_gen = |ctx: PermissionsTestContext| async move {
                let project_id = ctx.project_id.unwrap();
                let project_id_i64 = parse_base62(&project_id).unwrap() as i64;
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
                    VALUES ('2000-01-01', $1, $2, 1, NOW(), NOW())
                    ON CONFLICT DO NOTHING
                    "#,
                    USER_USER_ID_PARSED,
                    project_id_i64,
                )
                .execute(pool)
                .await
                .unwrap();
                let mut request = test::TestRequest::post()
                    .uri("/v3/analytics")
                    .set_json(json!({
                        "time_range": {
                            "start": Utc::now() - Duration::days(1),
                            "end": Utc::now() + Duration::days(1),
                            "resolution": { "slices": 5 }
                        },
                        "return_metrics": {
                            "project_revenue": {
                                "bucket_by": [],
                                "filter_by": {}
                            }
                        },
                        "project_ids": [project_id]
                    }));
                if let Some(pat) = ctx.test_pat {
                    request = request.append_header(("Authorization", pat));
                }
                api.call(request.to_request()).await
            };

            PermissionsTest::new(&test_env)
                .with_failure_codes(vec![200, 401])
                .with_200_json_checks(
                    // On failure, should have 0 projects returned
                    |value: &serde_json::Value| {
                        let value = value["projects"].as_object().unwrap();
                        assert_eq!(value.len(), 0);
                    },
                    // On success, should have 1 project returned
                    |value: &serde_json::Value| {
                        let value = value["projects"].as_object().unwrap();
                        assert_eq!(value.len(), 1);
                    },
                )
                .simple_project_permissions_test(view_analytics, req_gen)
                .await
                .unwrap();

            // Cleanup test db
            test_env.cleanup().await;
        },
    )
    .await;
}

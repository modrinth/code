use actix_web::test;
use ariadne::ids::base62_impl::parse_base62;
use chrono::{DateTime, Duration, Utc};
use common::permissions::PermissionsTest;
use common::permissions::PermissionsTestContext;
use common::{
    api_common::{Api, AppendsOptionalPat},
    api_v3::ApiV3,
    database::*,
    environment::{TestEnvironment, with_test_environment},
};
use labrinth::models::teams::ProjectPermissions;
use labrinth::queue::payouts;

use labrinth::routes::v3::analytics_get::{
    AnalyticsData, GetRequest, Metrics, ReturnMetrics, TimeRange,
    TimeRangeResolution,
};
use rust_decimal::Decimal;
use std::num::NonZeroU64;

pub mod common;

#[actix_rt::test]
pub async fn analytics_revenue() {
    with_test_environment(
        None,
        |test_env: TestEnvironment<ApiV3>| async move {
            let api = &test_env.api;

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

            // Test analytics endpoint with default values
            // - all time points in the last 2 weeks
            // - 1 day resolution
            let time_range = TimeRange {
                start: Utc::now() - Duration::days(14),
                end: Utc::now(),
                resolution: TimeRangeResolution::Slices(
                    NonZeroU64::new(14).unwrap(),
                ),
            };

            let return_metrics = ReturnMetrics {
                project_revenue: Some(Metrics { bucket_by: vec![] }),
                ..Default::default()
            };

            let request = GetRequest {
                time_range,
                return_metrics: ReturnMetrics {
                    project_revenue: Some(Metrics { bucket_by: vec![] }),
                    ..Default::default()
                },
            };

            let response =
                api.get_analytics_revenue_new(request, USER_USER_PAT).await;

            // GetResponse is a Vec<TimeSlice>, each TimeSlice contains Vec<AnalyticsData>
            // For now, just check that we get some response
            assert!(!response.0.is_empty());

            // Find our project in the response
            for time_slice in &response.0 {
                if let Some(analytics_data) = time_slice.0.first() {
                    let AnalyticsData::Project(_project_analytics) =
                        analytics_data;
                    break;
                }
            }

            // GetResponse is a Vec<TimeSlice>, each TimeSlice contains Vec<AnalyticsData>
            // For now, just check that we get some response
            assert!(!response.0.is_empty());

            // Check that we have some project data (not specific to our project)
            let mut found_any_project = false;
            for time_slice in &response.0 {
                if let Some(analytics_data) = time_slice.0.first() {
                    let AnalyticsData::Project(_project_analytics) =
                        analytics_data;
                    found_any_project = true;
                    break;
                }
                if found_any_project {
                    break;
                }
            }
            assert!(
                found_any_project,
                "Should find some project in the analytics response"
            );

            // Test analytics with last 900 days to include all data
            // keep resolution at default
            let time_range = TimeRange {
                start: Utc::now() - Duration::days(801),
                end: Utc::now(),
                resolution: TimeRangeResolution::Slices(
                    NonZeroU64::new(900).unwrap(),
                ),
            };

            let request = GetRequest {
                time_range,
                return_metrics,
            };

            let response =
                api.get_analytics_revenue_new(request, USER_USER_PAT).await;

            // Again, just check that we get some response
            assert!(!response.0.is_empty());

            // Find our project in the response
            for time_slice in &response.0 {
                if let Some(analytics_data) = time_slice.0.first() {
                    let AnalyticsData::Project(_project_analytics) =
                        analytics_data;
                    break;
                }
            }

            // Again, just check that we get some response
            assert!(!response.0.is_empty());

            // Check that we have some project data (not specific to our project)
            let mut found_any_project = false;
            for time_slice in &response.0 {
                if let Some(analytics_data) = time_slice.0.first() {
                    let AnalyticsData::Project(_project_analytics) =
                        analytics_data;
                    found_any_project = true;
                    break;
                }
                if found_any_project {
                    break;
                }
            }
            assert!(
                found_any_project,
                "Should find some project in the analytics response"
            );
        },
    )
    .await;
}

#[actix_rt::test]
pub async fn permissions_analytics_revenue() {
    with_test_environment(
        None,
        |test_env: TestEnvironment<ApiV3>| async move {
            let alpha_project_id =
                test_env.dummy.project_alpha.project_id.clone();
            let alpha_version_id =
                test_env.dummy.project_alpha.version_id.clone();
            let alpha_team_id = test_env.dummy.project_alpha.team_id.clone();

            let api = &test_env.api;

            let view_analytics = ProjectPermissions::VIEW_ANALYTICS;

            // first, do check with a project
            let req_gen = |ctx: PermissionsTestContext| async move {
                // TODO: when we add filters, make sure this only returns the
                // projects with this ID
                let _project_id = ctx.project_id.unwrap();
                let time_range = TimeRange {
                    start: Utc::now() - Duration::days(14),
                    end: Utc::now(),
                    resolution: TimeRangeResolution::Slices(
                        NonZeroU64::new(14).unwrap(),
                    ),
                };
                let return_metrics = ReturnMetrics {
                    project_revenue: Some(Metrics { bucket_by: vec![] }),
                    ..Default::default()
                };
                let request = GetRequest {
                    time_range,
                    return_metrics,
                };
                // Return a ServiceResponse for the permissions test
                let req = test::TestRequest::get()
                    .uri("/v3/analytics")
                    .set_json(request)
                    .append_pat(ctx.test_pat.as_deref())
                    .to_request();
                api.call(req).await
            };

            PermissionsTest::new(&test_env)
                .with_failure_codes(vec![200, 401])
                .with_200_json_checks(
                    // On failure, should have 0 projects returned
                    |value: &serde_json::Value| {
                        let value = value.as_array().unwrap();
                        assert_eq!(value.len(), 0);
                    },
                    // On success, should have 1 project returned
                    |value: &serde_json::Value| {
                        let value = value.as_array().unwrap();
                        assert!(!value.is_empty());
                    },
                )
                .simple_project_permissions_test(view_analytics, req_gen)
                .await
                .unwrap();

            // Now with a version
            // Need to use alpha
            let req_gen = |ctx: PermissionsTestContext| {
                // TODO: when we add filters, make sure this only returns the
                // projects with this ID
                let _alpha_version_id = alpha_version_id.clone();
                async move {
                    let time_range = TimeRange {
                        start: Utc::now() - Duration::days(14),
                        end: Utc::now(),
                        resolution: TimeRangeResolution::Slices(
                            NonZeroU64::new(14).unwrap(),
                        ),
                    };
                    let return_metrics = ReturnMetrics {
                        project_revenue: Some(Metrics { bucket_by: vec![] }),
                        ..Default::default()
                    };
                    let request = GetRequest {
                        time_range,
                        return_metrics,
                    };
                    // Return a ServiceResponse for the permissions test
                    let req = test::TestRequest::get()
                        .uri("/v3/analytics")
                        .set_json(request)
                        .append_pat(ctx.test_pat.as_deref())
                        .to_request();
                    api.call(req).await
                }
            };

            PermissionsTest::new(&test_env)
                .with_failure_codes(vec![200, 401])
                .with_existing_project(&alpha_project_id, &alpha_team_id)
                .with_user(FRIEND_USER_ID, FRIEND_USER_PAT, true)
                .with_200_json_checks(
                    // On failure, should have 0 versions returned
                    |value: &serde_json::Value| {
                        let value = value.as_array().unwrap();
                        assert_eq!(value.len(), 0);
                    },
                    // On success, should have 1 versions returned
                    |value: &serde_json::Value| {
                        let value = value.as_array().unwrap();
                        assert_eq!(value.len(), 0);
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

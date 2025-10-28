use ariadne::ids::base62_impl::parse_base62;
use chrono::{DateTime, Duration, Utc};
use chrono::{Datelike, TimeZone};
use common::permissions::PermissionsTest;
use common::permissions::PermissionsTestContext;
use common::{
    api_v3::ApiV3,
    database::*,
    environment::{TestEnvironment, with_test_environment},
};
use itertools::Itertools;
use labrinth::database::models::charge_item::DBCharge;
use labrinth::database::models::{
    DBAffiliateCode, DBAffiliateCodeId, DBChargeId, DBProductPriceId, DBUserId,
    DBUserSubscriptionId,
};
use labrinth::models::billing::{ChargeStatus, ChargeType, PaymentPlatform};
use labrinth::models::teams::ProjectPermissions;
use labrinth::queue::payouts::{self, process_affiliate_payouts};
use rust_decimal::{Decimal, prelude::ToPrimitive};
use std::collections::HashMap;

pub mod common;

#[actix_rt::test]
pub async fn affiliate_payout_basic() {
    with_test_environment(
        None,
        |test_env: TestEnvironment<ApiV3>| async move {
            let pool = &test_env.db.pool;

            // Setup test data
            let affiliate_user_id = DBUserId(USER_USER_ID_PARSED);
            let customer_user_id = DBUserId(FRIEND_USER_ID_PARSED);
            let affiliate_code_id = DBAffiliateCodeId(1001);
            let subscription_id = DBUserSubscriptionId(2001);

            // Create affiliate code
            let affiliate_code = DBAffiliateCode {
                id: affiliate_code_id,
                created_at: Utc::now(),
                created_by: affiliate_user_id,
                affiliate: affiliate_user_id,
                source_name: "test_code".to_string(),
            };
            affiliate_code.insert(pool).await.unwrap();

            // Create subscription affiliation
            let mut affiliation = DBUsersSubscriptionsAffiliations {
                id: 0,
                subscription_id,
                affiliate_code: affiliate_code_id,
                deactivated_at: None,
            };
            affiliation.insert(pool).await.unwrap();

            // Create a successful charge with net amount
            let charge_id = DBChargeId(3001);
            let charge = DBCharge {
                id: charge_id,
                user_id: customer_user_id,
                price_id: DBProductPriceId(1001),
                amount: 1000, // $10.00
                currency_code: "USD".to_string(),
                status: ChargeStatus::Succeeded,
                due: Utc::now(),
                last_attempt: Some(Utc::now()),
                type_: ChargeType::Subscription,
                subscription_id: Some(subscription_id),
                subscription_interval: None,
                payment_platform: PaymentPlatform::Stripe,
                payment_platform_id: Some("ch_test123".to_string()),
                parent_charge_id: None,
                tax_amount: 50,
                tax_platform_id: None,
                tax_last_updated: None,
                tax_transaction_version: None,
                tax_platform_accounting_time: None,
                net: Some(1000), // $10.00 net
                tax_drift_loss: None,
            };

            let mut txn = pool.begin().await.unwrap();
            charge.upsert(&mut txn).await.unwrap();
            txn.commit().await.unwrap();

            // Process affiliate payouts
            process_affiliate_payouts(pool).await.unwrap();

            // Verify payout was created
            let payout_records = sqlx::query!(
                "SELECT user_id, amount, created, date_available, affiliate_code_source
                 FROM payouts_values WHERE affiliate_code_source IS NOT NULL"
            )
            .fetch_all(pool)
            .await
            .unwrap();

            assert_eq!(payout_records.len(), 1);
            let payout = &payout_records[0];
            assert_eq!(payout.user_id, affiliate_user_id.0);
            assert_eq!(payout.amount, Some(100)); // $10.00 * 0.1 = $1.00, but stored as cents
            assert_eq!(payout.affiliate_code_source, Some(affiliate_code_id.0));

            // Verify charge-payout association was created
            let association = sqlx::query!(
                "SELECT charge_id, subscription_id, affiliate_code, payout_value_id
                 FROM users_subscriptions_affiliations_payouts"
            )
            .fetch_one(pool)
            .await
            .unwrap();

            assert_eq!(association.charge_id, charge_id.0);
            assert_eq!(association.subscription_id, subscription_id.0);
            assert_eq!(association.affiliate_code, affiliate_code_id.0);
        },
    )
    .await;
}

fn to_f64_rounded_up(d: Decimal) -> f64 {
    d.round_dp_with_strategy(
        1,
        rust_decimal::RoundingStrategy::MidpointAwayFromZero,
    )
    .to_f64()
    .unwrap()
}

fn to_f64_vec_rounded_up(d: Vec<Decimal>) -> Vec<f64> {
    d.into_iter().map(to_f64_rounded_up).collect_vec()
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
                let project_id = ctx.project_id.unwrap();
                let ids_or_slugs = vec![project_id.as_str()];
                api.get_analytics_revenue(
                    ids_or_slugs,
                    false,
                    None,
                    None,
                    Some(5),
                    ctx.test_pat.as_deref(),
                )
                .await
            };

            PermissionsTest::new(&test_env)
                .with_failure_codes(vec![200, 401])
                .with_200_json_checks(
                    // On failure, should have 0 projects returned
                    |value: &serde_json::Value| {
                        let value = value.as_object().unwrap();
                        assert_eq!(value.len(), 0);
                    },
                    // On success, should have 1 project returned
                    |value: &serde_json::Value| {
                        let value = value.as_object().unwrap();
                        assert_eq!(value.len(), 1);
                    },
                )
                .simple_project_permissions_test(view_analytics, req_gen)
                .await
                .unwrap();

            // Now with a version
            // Need to use alpha
            let req_gen = |ctx: PermissionsTestContext| {
                let alpha_version_id = alpha_version_id.clone();
                async move {
                    let ids_or_slugs = vec![alpha_version_id.as_str()];
                    api.get_analytics_revenue(
                        ids_or_slugs,
                        true,
                        None,
                        None,
                        Some(5),
                        ctx.test_pat.as_deref(),
                    )
                    .await
                }
            };

            PermissionsTest::new(&test_env)
                .with_failure_codes(vec![200, 401])
                .with_existing_project(&alpha_project_id, &alpha_team_id)
                .with_user(FRIEND_USER_ID, FRIEND_USER_PAT, true)
                .with_200_json_checks(
                    // On failure, should have 0 versions returned
                    |value: &serde_json::Value| {
                        let value = value.as_object().unwrap();
                        assert_eq!(value.len(), 0);
                    },
                    // On success, should have 1 versions returned
                    |value: &serde_json::Value| {
                        let value = value.as_object().unwrap();
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

#[actix_rt::test]
pub async fn affiliate_payout_custom_revenue_split() {
    with_test_environment(
        None,
        |test_env: TestEnvironment<ApiV3>| async move {
            let pool = &test_env.db.pool;

            // Setup test data with custom revenue split (25%)
            let affiliate_user_id = DBUserId(USER_USER_ID_PARSED);
            let customer_user_id = DBUserId(FRIEND_USER_ID_PARSED);
            let affiliate_code_id = DBAffiliateCodeId(1002);
            let subscription_id = DBUserSubscriptionId(2002);

            // Create affiliate code with custom revenue split
            let affiliate_code = DBAffiliateCode {
                id: affiliate_code_id,
                created_at: Utc::now(),
                created_by: affiliate_user_id,
                affiliate: affiliate_user_id,
                source_name: "test_code_custom".to_string(),
            };
            affiliate_code.insert(pool).await.unwrap();

            // Update the affiliate code to have a custom revenue split
            sqlx::query!(
                "UPDATE affiliate_codes SET revenue_split = $1 WHERE id = $2",
                0.25f64,
                affiliate_code_id.0
            )
            .execute(pool)
            .await
            .unwrap();

            // Create subscription affiliation
            let mut affiliation = DBUsersSubscriptionsAffiliations {
                id: 0,
                subscription_id,
                affiliate_code: affiliate_code_id,
                deactivated_at: None,
            };
            affiliation.insert(pool).await.unwrap();

            // Create a successful charge
            let charge_id = DBChargeId(3002);
            let charge = DBCharge {
                id: charge_id,
                user_id: customer_user_id,
                price_id: 1002,
                amount: 2000, // $20.00
                currency_code: "USD".to_string(),
                status: ChargeStatus::Succeeded,
                due: Utc::now(),
                last_attempt: Some(Utc::now()),
                type_: ChargeType::Subscription,
                subscription_id: Some(subscription_id),
                subscription_interval: None,
                payment_platform: PaymentPlatform::Stripe,
                payment_platform_id: Some("ch_test456".to_string()),
                parent_charge_id: None,
                tax_amount: 100,
                tax_platform_id: None,
                tax_last_updated: None,
                tax_transaction_version: None,
                tax_platform_accounting_time: None,
                net: Some(2000), // $20.00 net
                tax_drift_loss: None,
            };

            let mut txn = pool.begin().await.unwrap();
            charge.upsert(&mut txn).await.unwrap();
            txn.commit().await.unwrap();

            // Process affiliate payouts
            process_affiliate_payouts(pool).await.unwrap();

            // Verify payout with custom split
            let payout = sqlx::query!(
                "SELECT amount FROM payouts_values WHERE affiliate_code_source = $1",
                affiliate_code_id.0
            )
            .fetch_one(pool)
            .await
            .unwrap();

            assert_eq!(payout.amount, Decimal::from(500)); // $20.00 * 0.25 = $5.00, stored as cents
        },
    )
    .await;
}

#[actix_rt::test]
pub async fn affiliate_payout_multiple_charges_same_code() {
    with_test_environment(
        None,
        |test_env: TestEnvironment<ApiV3>| async move {
            let pool = &test_env.db.pool;

            let affiliate_user_id = DBUserId(USER_USER_ID_PARSED);
            let customer_user_id = DBUserId(FRIEND_USER_ID_PARSED);
            let affiliate_code_id = DBAffiliateCodeId(1003);
            let subscription_id = DBUserSubscriptionId(2003);

            // Setup affiliate code and affiliation
            let affiliate_code = DBAffiliateCode {
                id: affiliate_code_id,
                created_at: Utc::now(),
                created_by: affiliate_user_id,
                affiliate: affiliate_user_id,
                source_name: "test_multi".to_string(),
            };
            affiliate_code.insert(pool).await.unwrap();

            let mut affiliation = DBUsersSubscriptionsAffiliations {
                id: 0,
                subscription_id,
                affiliate_code: affiliate_code_id,
                deactivated_at: None,
            };
            affiliation.insert(pool).await.unwrap();

            // Create multiple charges for the same subscription
            let charges = vec![
                (DBChargeId(3003), 1000), // $10.00
                (DBChargeId(3004), 1500), // $15.00
                (DBChargeId(3005), 2000), // $20.00
            ];

            let mut txn = pool.begin().await.unwrap();
            for (charge_id, net_amount) in &charges {
                let charge = DBCharge {
                    id: *charge_id,
                    user_id: customer_user_id,
                    price_id: 1003,
                    amount: *net_amount,
                    currency_code: "USD".to_string(),
                    status: ChargeStatus::Succeeded,
                    due: Utc::now(),
                    last_attempt: Some(Utc::now()),
                    type_: ChargeType::Subscription,
                    subscription_id: Some(subscription_id),
                    subscription_interval: None,
                    payment_platform: PaymentPlatform::Stripe,
                    payment_platform_id: Some(format!("ch_test{}", charge_id.0)),
                    parent_charge_id: None,
                    tax_amount: 50,
                    tax_platform_id: None,
                    tax_last_updated: None,
                    tax_transaction_version: None,
                    tax_platform_accounting_time: None,
                    net: Some(*net_amount),
                    tax_drift_loss: None,
                };
                charge.upsert(&mut txn).await.unwrap();
            }
            txn.commit().await.unwrap();

            // Process affiliate payouts
            process_affiliate_payouts(pool).await.unwrap();

            // Should create a single payout aggregating all charges
            let payout_records = sqlx::query!(
                "SELECT user_id, amount FROM payouts_values WHERE affiliate_code_source = $1",
                affiliate_code_id.0
            )
            .fetch_all(pool)
            .await
            .unwrap();

            assert_eq!(payout_records.len(), 1);
            let total_expected = Decimal::from(1000 + 1500 + 2000) * Decimal::from_f64_retain(0.1).unwrap() / Decimal::new(1, 0);
            assert_eq!(payout_records[0].amount, Some(total_expected.to_i64().unwrap()));

            // Should create 3 charge-payout associations
            let associations = sqlx::query!(
                "SELECT COUNT(*) as count FROM users_subscriptions_affiliations_payouts WHERE affiliate_code = $1",
                affiliate_code_id.0
            )
            .fetch_one(pool)
            .await
            .unwrap();

            assert_eq!(associations.count.unwrap(), 3);
        },
    )
    .await;
}

#[actix_rt::test]
pub async fn affiliate_payout_deactivated_affiliation() {
    with_test_environment(
        None,
        |test_env: TestEnvironment<ApiV3>| async move {
            let pool = &test_env.db.pool;

            let affiliate_user_id = DBUserId(USER_USER_ID_PARSED);
            let customer_user_id = DBUserId(FRIEND_USER_ID_PARSED);
            let affiliate_code_id = DBAffiliateCodeId(1004);
            let subscription_id = DBUserSubscriptionId(2004);

            // Setup affiliate code
            let affiliate_code = DBAffiliateCode {
                id: affiliate_code_id,
                created_at: Utc::now(),
                created_by: affiliate_user_id,
                affiliate: affiliate_user_id,
                source_name: "test_deactivated".to_string(),
            };
            affiliate_code.insert(pool).await.unwrap();

            // Create DEACTIVATED subscription affiliation
            let mut affiliation = DBUsersSubscriptionsAffiliations {
                id: 0,
                subscription_id,
                affiliate_code: affiliate_code_id,
                deactivated_at: Some(Utc::now() - Duration::days(1)), // Deactivated yesterday
            };
            affiliation.insert(pool).await.unwrap();

            // Create a successful charge
            let charge_id = DBChargeId(3006);
            let charge = DBCharge {
                id: charge_id,
                user_id: customer_user_id,
                price_id: 1004,
                amount: 1000,
                currency_code: "USD".to_string(),
                status: ChargeStatus::Succeeded,
                due: Utc::now(),
                last_attempt: Some(Utc::now()),
                type_: ChargeType::Subscription,
                subscription_id: Some(subscription_id),
                subscription_interval: None,
                payment_platform: PaymentPlatform::Stripe,
                payment_platform_id: Some("ch_test789".to_string()),
                parent_charge_id: None,
                tax_amount: 50,
                tax_platform_id: None,
                tax_last_updated: None,
                tax_transaction_version: None,
                tax_platform_accounting_time: None,
                net: Some(1000),
                tax_drift_loss: None,
            };

            let mut txn = pool.begin().await.unwrap();
            charge.upsert(&mut txn).await.unwrap();
            txn.commit().await.unwrap();

            // Process affiliate payouts
            process_affiliate_payouts(pool).await.unwrap();

            // Should NOT create any payouts for deactivated affiliations
            let payout_records = sqlx::query!(
                "SELECT COUNT(*) as count FROM payouts_values WHERE affiliate_code_source = $1",
                affiliate_code_id.0
            )
            .fetch_one(pool)
            .await
            .unwrap();

            assert_eq!(payout_records.count.unwrap(), 0);
        },
    )
    .await;
}

#[actix_rt::test]
pub async fn affiliate_payout_edge_cases() {
    with_test_environment(
        None,
        |test_env: TestEnvironment<ApiV3>| async move {
            let pool = &test_env.db.pool;

            let affiliate_user_id = DBUserId(USER_USER_ID_PARSED);
            let customer_user_id = DBUserId(FRIEND_USER_ID_PARSED);
            let affiliate_code_id = DBAffiliateCodeId(1005);
            let subscription_id = DBUserSubscriptionId(2005);

            // Setup affiliate code
            let affiliate_code = DBAffiliateCode {
                id: affiliate_code_id,
                created_at: Utc::now(),
                created_by: affiliate_user_id,
                affiliate: affiliate_user_id,
                source_name: "test_edge_cases".to_string(),
            };
            affiliate_code.insert(pool).await.unwrap();

            // Set an invalid revenue split (out of range)
            sqlx::query!(
                "UPDATE affiliate_codes SET revenue_split = $1 WHERE id = $2",
                1.5f64, // Invalid: > 1.0
                affiliate_code_id.0
            )
            .execute(pool)
            .await
            .unwrap();

            let mut affiliation = DBUsersSubscriptionsAffiliations {
                id: 0,
                subscription_id,
                affiliate_code: affiliate_code_id,
                deactivated_at: None,
            };
            affiliation.insert(pool).await.unwrap();

            // Create charges with edge case scenarios
            let charges = vec![
                (DBChargeId(3007), Some(1000), ChargeStatus::Succeeded),    // Normal case
                (DBChargeId(3008), Some(0), ChargeStatus::Succeeded),       // Zero net
                (DBChargeId(3009), Some(1000), ChargeStatus::Failed),       // Failed charge
                (DBChargeId(3010), None, ChargeStatus::Succeeded),         // No net amount
            ];

            let mut txn = pool.begin().await.unwrap();
            for (charge_id, net_amount, status) in charges {
                let charge = DBCharge {
                    id: charge_id,
                    user_id: customer_user_id,
                    price_id: 1005,
                    amount: net_amount.unwrap_or(0),
                    currency_code: "USD".to_string(),
                    status,
                    due: Utc::now(),
                    last_attempt: Some(Utc::now()),
                    type_: ChargeType::Subscription,
                    subscription_id: Some(subscription_id),
                    subscription_interval: None,
                    payment_platform: PaymentPlatform::Stripe,
                    payment_platform_id: Some(format!("ch_test{}", charge_id.0)),
                    parent_charge_id: None,
                    tax_amount: 0,
                    tax_platform_id: None,
                    tax_last_updated: None,
                    tax_transaction_version: None,
                    tax_platform_accounting_time: None,
                    net: net_amount,
                    tax_drift_loss: None,
                };
                charge.upsert(&mut txn).await.unwrap();
            }
            txn.commit().await.unwrap();

            // Process affiliate payouts
            process_affiliate_payouts(pool).await.unwrap();

            // Should only create payout for the valid charge (3007)
            // But since the revenue split is invalid, even that should be skipped
            let payout_records = sqlx::query!(
                "SELECT COUNT(*) as count FROM payouts_values WHERE affiliate_code_source = $1",
                affiliate_code_id.0
            )
            .fetch_one(pool)
            .await
            .unwrap();

            assert_eq!(payout_records.count.unwrap(), 0);
        },
    )
    .await;
}

#[actix_rt::test]
pub async fn affiliate_payout_idempotent() {
    with_test_environment(
        None,
        |test_env: TestEnvironment<ApiV3>| async move {
            let pool = &test_env.db.pool;

            let affiliate_user_id = DBUserId(USER_USER_ID_PARSED);
            let customer_user_id = DBUserId(FRIEND_USER_ID_PARSED);
            let affiliate_code_id = DBAffiliateCodeId(1006);
            let subscription_id = DBUserSubscriptionId(2006);

            // Setup affiliate code and affiliation
            let affiliate_code = DBAffiliateCode {
                id: affiliate_code_id,
                created_at: Utc::now(),
                created_by: affiliate_user_id,
                affiliate: affiliate_user_id,
                source_name: "test_idempotent".to_string(),
            };
            affiliate_code.insert(pool).await.unwrap();

            let mut affiliation = DBUsersSubscriptionsAffiliations {
                id: 0,
                subscription_id,
                affiliate_code: affiliate_code_id,
                deactivated_at: None,
            };
            affiliation.insert(pool).await.unwrap();

            // Create a successful charge
            let charge_id = DBChargeId(3011);
            let charge = DBCharge {
                id: charge_id,
                user_id: customer_user_id,
                price_id: 1006,
                amount: 1000,
                currency_code: "USD".to_string(),
                status: ChargeStatus::Succeeded,
                due: Utc::now(),
                last_attempt: Some(Utc::now()),
                type_: ChargeType::Subscription,
                subscription_id: Some(subscription_id),
                subscription_interval: None,
                payment_platform: PaymentPlatform::Stripe,
                payment_platform_id: Some("ch_test_idempotent".to_string()),
                parent_charge_id: None,
                tax_amount: 50,
                tax_platform_id: None,
                tax_last_updated: None,
                tax_transaction_version: None,
                tax_platform_accounting_time: None,
                net: Some(1000),
                tax_drift_loss: None,
            };

            let mut txn = pool.begin().await.unwrap();
            charge.upsert(&mut txn).await.unwrap();
            txn.commit().await.unwrap();

            // Process affiliate payouts first time
            process_affiliate_payouts(pool).await.unwrap();

            let first_payout_count = sqlx::query!(
                "SELECT COUNT(*) as count FROM payouts_values WHERE affiliate_code_source = $1",
                affiliate_code_id.0
            )
            .fetch_one(pool)
            .await
            .unwrap()
            .count
            .unwrap();

            let first_association_count = sqlx::query!(
                "SELECT COUNT(*) as count FROM users_subscriptions_affiliations_payouts WHERE charge_id = $1",
                charge_id.0
            )
            .fetch_one(pool)
            .await
            .unwrap()
            .count
            .unwrap();

            assert_eq!(first_payout_count, 1);
            assert_eq!(first_association_count, 1);

            // Process affiliate payouts second time (should be idempotent)
            process_affiliate_payouts(pool).await.unwrap();

            let second_payout_count = sqlx::query!(
                "SELECT COUNT(*) as count FROM payouts_values WHERE affiliate_code_source = $1",
                affiliate_code_id.0
            )
            .fetch_one(pool)
            .await
            .unwrap()
            .count
            .unwrap();

            let second_association_count = sqlx::query!(
                "SELECT COUNT(*) as count FROM users_subscriptions_affiliations_payouts WHERE charge_id = $1",
                charge_id.0
            )
            .fetch_one(pool)
            .await
            .unwrap()
            .count
            .unwrap();

            // Should not create duplicate payouts
            assert_eq!(second_payout_count, 1);
            assert_eq!(second_association_count, 1);
        },
    )
    .await;
}

#[actix_rt::test]
pub async fn affiliate_payout_availability_date() {
    with_test_environment(
        None,
        |test_env: TestEnvironment<ApiV3>| async move {
            let pool = &test_env.db.pool;

            let affiliate_user_id = DBUserId(USER_USER_ID_PARSED);
            let customer_user_id = DBUserId(FRIEND_USER_ID_PARSED);
            let affiliate_code_id = DBAffiliateCodeId(1007);
            let subscription_id = DBUserSubscriptionId(2007);

            // Setup affiliate code and affiliation
            let affiliate_code = DBAffiliateCode {
                id: affiliate_code_id,
                created_at: Utc::now(),
                created_by: affiliate_user_id,
                affiliate: affiliate_user_id,
                source_name: "test_availability".to_string(),
            };
            affiliate_code.insert(pool).await.unwrap();

            let mut affiliation = DBUsersSubscriptionsAffiliations {
                id: 0,
                subscription_id,
                affiliate_code: affiliate_code_id,
                deactivated_at: None,
            };
            affiliation.insert(pool).await.unwrap();

            // Create a successful charge
            let charge_id = DBChargeId(3012);
            let charge = DBCharge {
                id: charge_id,
                user_id: customer_user_id,
                price_id: 1007,
                amount: 1000,
                currency_code: "USD".to_string(),
                status: ChargeStatus::Succeeded,
                due: Utc::now(),
                last_attempt: Some(Utc::now()),
                type_: ChargeType::Subscription,
                subscription_id: Some(subscription_id),
                subscription_interval: None,
                payment_platform: PaymentPlatform::Stripe,
                payment_platform_id: Some("ch_test_availability".to_string()),
                parent_charge_id: None,
                tax_amount: 50,
                tax_platform_id: None,
                tax_last_updated: None,
                tax_transaction_version: None,
                tax_platform_accounting_time: None,
                net: Some(1000),
                tax_drift_loss: None,
            };

            let mut txn = pool.begin().await.unwrap();
            charge.upsert(&mut txn).await.unwrap();
            txn.commit().await.unwrap();

            // Process affiliate payouts
            process_affiliate_payouts(pool).await.unwrap();

            // Verify availability date is set correctly (Net 60)
            let payout = sqlx::query!(
                "SELECT created, date_available FROM payouts_values WHERE affiliate_code_source = $1",
                affiliate_code_id.0
            )
            .fetch_one(pool)
            .await
            .unwrap();

            let now = Utc::now();
            let expected_available = {
                let year = now.year();
                let month = now.month();

                // First day of next month + 59 days (Net 60)
                let first_day_next_month = if month == 12 {
                    Utc.with_ymd_and_hms(year + 1, 1, 1, 0, 0, 0).unwrap()
                } else {
                    Utc.with_ymd_and_hms(year, month + 1, 1, 0, 0, 0).unwrap()
                };

                first_day_next_month + Duration::days(59)
            };

            // Check that availability date is approximately correct (within 1 minute)
            let availability_diff = (payout.date_available - expected_available).abs();
            assert!(availability_diff.num_minutes() < 1);

            // Check that created date is yesterday (start of the day)
            let expected_created = (now - Duration::days(1)).date_naive().and_hms_opt(0, 0, 0).unwrap();
            let actual_created = payout.created.naive_utc();
            let created_diff = (actual_created - expected_created).abs();
            assert!(created_diff.num_seconds() < 1);
        },
    )
    .await;
}

#[actix_rt::test]
pub async fn affiliate_payout_multiple_codes_same_affiliate() {
    with_test_environment(
        None,
        |test_env: TestEnvironment<ApiV3>| async move {
            let pool = &test_env.db.pool;

            let affiliate_user_id = DBUserId(USER_USER_ID_PARSED);
            let customer_user_id = DBUserId(FRIEND_USER_ID_PARSED);
            let affiliate_code_id_1 = DBAffiliateCodeId(1008);
            let affiliate_code_id_2 = DBAffiliateCodeId(1009);
            let subscription_id_1 = DBUserSubscriptionId(2008);
            let subscription_id_2 = DBUserSubscriptionId(2009);

            // Create two affiliate codes for the same affiliate user
            let affiliate_code_1 = DBAffiliateCode {
                id: affiliate_code_id_1,
                created_at: Utc::now(),
                created_by: affiliate_user_id,
                affiliate: affiliate_user_id,
                source_name: "test_code_1".to_string(),
            };
            affiliate_code_1.insert(pool).await.unwrap();

            let affiliate_code_2 = DBAffiliateCode {
                id: affiliate_code_id_2,
                created_at: Utc::now(),
                created_by: affiliate_user_id,
                affiliate: affiliate_user_id,
                source_name: "test_code_2".to_string(),
            };
            affiliate_code_2.insert(pool).await.unwrap();

            // Create affiliations for both codes
            let mut affiliation_1 = DBUsersSubscriptionsAffiliations {
                id: 0,
                subscription_id: subscription_id_1,
                affiliate_code: affiliate_code_id_1,
                deactivated_at: None,
            };
            affiliation_1.insert(pool).await.unwrap();

            let mut affiliation_2 = DBUsersSubscriptionsAffiliations {
                id: 0,
                subscription_id: subscription_id_2,
                affiliate_code: affiliate_code_id_2,
                deactivated_at: None,
            };
            affiliation_2.insert(pool).await.unwrap();

            // Create charges for both subscriptions
            let charge_1_id = DBChargeId(3013);
            let charge_1 = DBCharge {
                id: charge_1_id,
                user_id: customer_user_id,
                price_id: 1008,
                amount: 1000, // $10.00
                currency_code: "USD".to_string(),
                status: ChargeStatus::Succeeded,
                due: Utc::now(),
                last_attempt: Some(Utc::now()),
                type_: ChargeType::Subscription,
                subscription_id: Some(subscription_id_1),
                subscription_interval: None,
                payment_platform: PaymentPlatform::Stripe,
                payment_platform_id: Some("ch_test_multi1".to_string()),
                parent_charge_id: None,
                tax_amount: 50,
                tax_platform_id: None,
                tax_last_updated: None,
                tax_transaction_version: None,
                tax_platform_accounting_time: None,
                net: Some(1000),
                tax_drift_loss: None,
            };

            let charge_2_id = DBChargeId(3014);
            let charge_2 = DBCharge {
                id: charge_2_id,
                user_id: customer_user_id,
                price_id: 1009,
                amount: 2000, // $20.00
                currency_code: "USD".to_string(),
                status: ChargeStatus::Succeeded,
                due: Utc::now(),
                last_attempt: Some(Utc::now()),
                type_: ChargeType::Subscription,
                subscription_id: Some(subscription_id_2),
                subscription_interval: None,
                payment_platform: PaymentPlatform::Stripe,
                payment_platform_id: Some("ch_test_multi2".to_string()),
                parent_charge_id: None,
                tax_amount: 100,
                tax_platform_id: None,
                tax_last_updated: None,
                tax_transaction_version: None,
                tax_platform_accounting_time: None,
                net: Some(2000),
                tax_drift_loss: None,
            };

            let mut txn = pool.begin().await.unwrap();
            charge_1.upsert(&mut txn).await.unwrap();
            charge_2.upsert(&mut txn).await.unwrap();
            txn.commit().await.unwrap();

            // Process affiliate payouts
            process_affiliate_payouts(pool).await.unwrap();

            // Should create separate payouts for each code
            let payout_records = sqlx::query!(
                "SELECT affiliate_code_source, amount FROM payouts_values WHERE user_id = $1 ORDER BY affiliate_code_source",
                affiliate_user_id.0
            )
            .fetch_all(pool)
            .await
            .unwrap();

            assert_eq!(payout_records.len(), 2);

            // Verify amounts for each code
            let payouts_by_code: HashMap<i64, Decimal> = payout_records
                .into_iter()
                .map(|r| (r.affiliate_code_source.unwrap(), r.amount))
                .collect();

            assert_eq!(payouts_by_code.get(&1008).unwrap().unwrap(), 100); // $10.00 * 0.1 = $1.00
            assert_eq!(payouts_by_code.get(&1009).unwrap().unwrap(), 200); // $20.00 * 0.1 = $2.00
        },
    )
    .await;
}

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

            let day = 86400;

            // Test analytics endpoint with default values
            // - all time points in the last 2 weeks
            // - 1 day resolution
            let analytics = api
                .get_analytics_revenue_deserialized(
                    vec![&alpha_project_id],
                    false,
                    None,
                    None,
                    None,
                    USER_USER_PAT,
                )
                .await;
            assert_eq!(analytics.len(), 1); // 1 project
            let project_analytics = &analytics[&alpha_project_id];
            assert_eq!(project_analytics.len(), 8); // 1 days cut off, and 2 points take place on the same day. note that the day exactly 14 days ago is included
            // sorted_by_key, values in the order of smallest to largest key
            let (sorted_keys, sorted_by_key): (Vec<i64>, Vec<Decimal>) =
                project_analytics
                    .iter()
                    .sorted_by_key(|(k, _)| *k)
                    .rev()
                    .unzip();
            assert_eq!(
                vec![100.1, 101.0, 200.0, 311.0, 400.0, 526.0, 633.0, 800.0],
                to_f64_vec_rounded_up(sorted_by_key)
            );
            // Ensure that the keys are in multiples of 1 day
            for k in sorted_keys {
                assert_eq!(k % day, 0);
            }

            // Test analytics with last 900 days to include all data
            // keep resolution at default
            let analytics = api
                .get_analytics_revenue_deserialized(
                    vec![&alpha_project_id],
                    false,
                    Some(Utc::now() - Duration::days(801)),
                    None,
                    None,
                    USER_USER_PAT,
                )
                .await;
            let project_analytics = &analytics[&alpha_project_id];
            assert_eq!(project_analytics.len(), 9); // and 2 points take place on the same day
            let (sorted_keys, sorted_by_key): (Vec<i64>, Vec<Decimal>) =
                project_analytics
                    .iter()
                    .sorted_by_key(|(k, _)| *k)
                    .rev()
                    .unzip();
            assert_eq!(
                vec![
                    100.1, 101.0, 200.0, 311.0, 400.0, 526.0, 633.0, 800.0,
                    800.0
                ],
                to_f64_vec_rounded_up(sorted_by_key)
            );
            for k in sorted_keys {
                assert_eq!(k % day, 0);
            }
        },
    )
    .await;
}

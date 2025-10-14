use chrono::{Datelike, Duration, TimeZone, Utc};
use common::{
    api_v3::ApiV3,
    environment::{TestEnvironment, with_test_environment},
};
use labrinth::database::models::{DBAffiliateCodeId, DBUserId};
use labrinth::queue::affiliate_codes::process_affiliate_code_revenue;
use rust_decimal::dec;

pub mod common;

#[actix_rt::test]
pub async fn test_affiliate_code_revenue_processing_default_split() {
    with_test_environment(
        None,
        |test_env: TestEnvironment<ApiV3>| async move {
            let pool = test_env.db.pool.clone();

            // Create test users
            let affiliate_user_id = DBUserId(1000);
            let buyer_user_id = DBUserId(1001);

            // Create test users in the database
            sqlx::query!(
                r#"
                INSERT INTO users (id, username, email, role)
                VALUES
                ($1, 'affiliate_user', 'affiliate@test.com', 'developer'),
                ($2, 'buyer_user', 'buyer@test.com', 'developer')
                "#,
                affiliate_user_id.0,
                buyer_user_id.0
            )
            .execute(&pool)
            .await
            .unwrap();

            // Create a dummy product and price for charges to reference
            sqlx::query!(
                r#"
                INSERT INTO products (id, metadata, unitary, name)
                VALUES (9000, '{}', false, 'Test Product')
                "#
            )
            .execute(&pool)
            .await
            .unwrap();

            sqlx::query!(
                r#"
                INSERT INTO products_prices (id, product_id, currency_code, prices, public)
                VALUES (9000, 9000, 'USD', '[{"amount": 1000}]', true)
                "#
            )
            .execute(&pool)
            .await
            .unwrap();

            // Create affiliate code with default split (NULL)
            let affiliate_code_id = DBAffiliateCodeId(2000);
            sqlx::query!(
                r#"
                INSERT INTO affiliate_codes (id, created_at, created_by, affiliate, revenue_split)
                VALUES ($1, NOW(), $2, $2, NULL)
                "#,
                affiliate_code_id.0,
                affiliate_user_id.0
            )
            .execute(&pool)
            .await
            .unwrap();

            // Create test charges within the date range
            let now = Utc::now();
            let charge_date = now - Duration::days(45); // Within the 30-day window (between 30-60 days ago)

            // Create multiple charges
            let charges = [
                1000, // $10.00
                2000, // $20.00
                500,  // $5.00
            ];

            for (i, amount_cents) in charges.iter().enumerate() {
                sqlx::query!(
                    r#"
                    INSERT INTO charges (id, user_id, price_id, amount, currency_code, net, status, due, affiliate_code, charge_type, payment_platform, tax_amount)
                    VALUES ($1, $2, 9000, $3, 'USD', $3, 'succeeded', $4, $5, 'recurring', 'stripe', 0)
                    "#,
                    3000 + i as i64, // unique ID
                    buyer_user_id.0,
                    amount_cents,
                    charge_date,
                    affiliate_code_id.0
                )
                .execute(&pool)
                .await
                .unwrap();
            }

            // Process affiliate code revenue
            process_affiliate_code_revenue(&pool).await.unwrap();


            // Verify payouts were created
            let payouts = sqlx::query!(
                r#"
                SELECT user_id, amount, created, date_available, affiliate_code_id
                FROM payouts_values
                WHERE user_id = $1 AND affiliate_code_id = $2
                ORDER BY created DESC
                "#,
                affiliate_user_id.0,
                affiliate_code_id.0
            )
            .fetch_all(&pool)
            .await
            .unwrap();

            // Should have one payout entry with 10% of total ($3.50)
            assert_eq!(payouts.len(), 1);
            assert_eq!(payouts[0].user_id, affiliate_user_id.0);
            assert_eq!(payouts[0].affiliate_code_id, Some(affiliate_code_id.0));

            // Expected payout: $35.00 * 10% = $3.50
            let payout_amount = payouts[0].amount;
            assert_eq!(payout_amount, dec!(3.50));

            // Verify availability date is Net 30
            let expected_available = {
                let processing_month = (now - Duration::days(30)).date_naive();
                let year = processing_month.year();
                let month = processing_month.month();

                let first_of_next_month = if month == 12 {
                    Utc.with_ymd_and_hms(year + 1, 1, 1, 0, 0, 0).unwrap()
                } else {
                    Utc.with_ymd_and_hms(year, month + 1, 1, 0, 0, 0).unwrap()
                };

                first_of_next_month + Duration::days(29)
            };

            // Allow small time differences due to test execution
            let time_diff = (payouts[0].date_available - expected_available).num_seconds();
            assert!(time_diff.abs() < 60);
        },
    )
    .await;
}

#[actix_rt::test]
pub async fn test_affiliate_code_revenue_processing_custom_split() {
    with_test_environment(
        None,
        |test_env: TestEnvironment<ApiV3>| async move {
            let pool = test_env.db.pool.clone();

            // Create test users
            let affiliate_user_id = DBUserId(2000);
            let buyer_user_id = DBUserId(2001);

            // Create test users in the database
            sqlx::query!(
                r#"
                INSERT INTO users (id, username, email, role)
                VALUES
                ($1, 'affiliate_user2', 'affiliate2@test.com', 'developer'),
                ($2, 'buyer_user2', 'buyer2@test.com', 'developer')
                "#,
                affiliate_user_id.0,
                buyer_user_id.0
            )
            .execute(&pool)
            .await
            .unwrap();

            // Create a dummy product and price for charges to reference
            sqlx::query!(
                r#"
                INSERT INTO products (id, metadata, unitary, name)
                VALUES (9001, '{}', false, 'Test Product')
                "#
            )
            .execute(&pool)
            .await
            .unwrap();

            sqlx::query!(
                r#"
                INSERT INTO products_prices (id, product_id, currency_code, prices, public)
                VALUES (9001, 9001, 'USD', '[{"amount": 1000}]', true)
                "#
            )
            .execute(&pool)
            .await
            .unwrap();

            // Create affiliate code with custom 25% split
            let affiliate_code_id = DBAffiliateCodeId(2001);
            sqlx::query!(
                r#"
                INSERT INTO affiliate_codes (id, created_at, created_by, affiliate, revenue_split)
                VALUES ($1, NOW(), $2, $2, 0.25)
                "#,
                affiliate_code_id.0,
                affiliate_user_id.0
            )
            .execute(&pool)
            .await
            .unwrap();

            // Create test charges
            let now = Utc::now();
            let charge_date = now - Duration::days(45);

            let charge_amount = 1000; // $10.00

            sqlx::query!(
                r#"
                INSERT INTO charges (id, user_id, price_id, amount, currency_code, net, status, due, affiliate_code, charge_type, payment_platform, tax_amount)
                VALUES ($1, $2, 9001, $3, 'USD', $3, 'succeeded', $4, $5, 'recurring', 'stripe', 0)
                "#,
                4000, // unique ID
                buyer_user_id.0,
                charge_amount,
                charge_date,
                affiliate_code_id.0
            )
            .execute(&pool)
            .await
            .unwrap();

            // Process affiliate code revenue
            process_affiliate_code_revenue(&pool).await.unwrap();

            // Verify payout was created with custom split
            let payouts = sqlx::query!(
                r#"
                SELECT user_id, amount, created, date_available, affiliate_code_id
                FROM payouts_values
                WHERE user_id = $1 AND affiliate_code_id = $2
                "#,
                affiliate_user_id.0,
                affiliate_code_id.0
            )
            .fetch_all(&pool)
            .await
            .unwrap();

            assert_eq!(payouts.len(), 1);

            // Expected payout: $10.00 * 25% = $2.50
            let payout_amount = payouts[0].amount;
            assert_eq!(payout_amount, dec!(2.50));
        },
    )
    .await;
}

#[actix_rt::test]
pub async fn test_affiliate_code_revenue_processing_invalid_split() {
    with_test_environment(
        None,
        |test_env: TestEnvironment<ApiV3>| async move {
            let pool = test_env.db.pool.clone();

            // Create test users
            let affiliate_user_id = DBUserId(3000);
            let buyer_user_id = DBUserId(3001);

            // Create test users in the database
            sqlx::query!(
                r#"
                INSERT INTO users (id, username, email, role)
                VALUES
                ($1, 'affiliate_user3', 'affiliate3@test.com', 'developer'),
                ($2, 'buyer_user3', 'buyer3@test.com', 'developer')
                "#,
                affiliate_user_id.0,
                buyer_user_id.0
            )
            .execute(&pool)
            .await
            .unwrap();

            // Create a dummy product and price for charges to reference
            sqlx::query!(
                r#"
                INSERT INTO products (id, metadata, unitary, name)
                VALUES (9002, '{}', false, 'Test Product')
                "#
            )
            .execute(&pool)
            .await
            .unwrap();

            sqlx::query!(
                r#"
                INSERT INTO products_prices (id, product_id, currency_code, prices, public)
                VALUES (9002, 9002, 'USD', '[{"amount": 1000}]', true)
                "#
            )
            .execute(&pool)
            .await
            .unwrap();

            // Create affiliate code with invalid split (150%)
            let affiliate_code_id = DBAffiliateCodeId(2002);
            sqlx::query!(
                r#"
                INSERT INTO affiliate_codes (id, created_at, created_by, affiliate, revenue_split)
                VALUES ($1, NOW(), $2, $2, 1.5)
                "#,
                affiliate_code_id.0,
                affiliate_user_id.0
            )
            .execute(&pool)
            .await
            .unwrap();

            // Create test charge
            let now = Utc::now();
            let charge_date = now - Duration::days(45);

            sqlx::query!(
                r#"
                INSERT INTO charges (id, user_id, price_id, amount, currency_code, net, status, due, affiliate_code, charge_type, payment_platform, tax_amount)
                VALUES ($1, $2, 9002, $3, 'USD', $3, 'succeeded', $4, $5, 'recurring', 'stripe', 0)
                "#,
                5000, // unique ID
                buyer_user_id.0,
                1000, // $10.00
                charge_date,
                affiliate_code_id.0
            )
            .execute(&pool)
            .await
            .unwrap();

            // Process affiliate code revenue - should not create payout due to invalid split
            process_affiliate_code_revenue(&pool).await.unwrap();

            // Verify no payout was created
            let payouts = sqlx::query!(
                r#"
                SELECT user_id, amount, created, date_available, affiliate_code_id
                FROM payouts_values
                WHERE user_id = $1 AND affiliate_code_id = $2
                "#,
                affiliate_user_id.0,
                affiliate_code_id.0
            )
            .fetch_all(&pool)
            .await
            .unwrap();

            assert_eq!(payouts.len(), 0);
        },
    )
    .await;
}

#[actix_rt::test]
pub async fn test_affiliate_code_revenue_processing_outside_date_range() {
    with_test_environment(
        None,
        |test_env: TestEnvironment<ApiV3>| async move {
            let pool = test_env.db.pool.clone();

            // Create test users
            let affiliate_user_id = DBUserId(4000);
            let buyer_user_id = DBUserId(4001);

            // Create test users in the database
            sqlx::query!(
                r#"
                INSERT INTO users (id, username, email, role)
                VALUES
                ($1, 'affiliate_user4', 'affiliate4@test.com', 'developer'),
                ($2, 'buyer_user4', 'buyer4@test.com', 'developer')
                "#,
                affiliate_user_id.0,
                buyer_user_id.0
            )
            .execute(&pool)
            .await
            .unwrap();

            // Create a dummy product and price for charges to reference
            sqlx::query!(
                r#"
                INSERT INTO products (id, metadata, unitary, name)
                VALUES (9003, '{}', false, 'Test Product')
                "#
            )
            .execute(&pool)
            .await
            .unwrap();

            sqlx::query!(
                r#"
                INSERT INTO products_prices (id, product_id, currency_code, prices, public)
                VALUES (9003, 9003, 'USD', '[{"amount": 1000}]', true)
                "#
            )
            .execute(&pool)
            .await
            .unwrap();

            // Create affiliate code
            let affiliate_code_id = DBAffiliateCodeId(2003);
            sqlx::query!(
                r#"
                INSERT INTO affiliate_codes (id, created_at, created_by, affiliate, revenue_split)
                VALUES ($1, NOW(), $2, $2, NULL)
                "#,
                affiliate_code_id.0,
                affiliate_user_id.0
            )
            .execute(&pool)
            .await
            .unwrap();

            // Create test charge outside the date range (more than 60 days ago)
            let old_charge_date = Utc::now() - Duration::days(90);

            sqlx::query!(
                r#"
                INSERT INTO charges (id, user_id, price_id, amount, currency_code, net, status, due, affiliate_code, charge_type, payment_platform, tax_amount)
                VALUES ($1, $2, 9003, $3, 'USD', $3, 'succeeded', $4, $5, 'recurring', 'stripe', 0)
                "#,
                6000, // unique ID
                buyer_user_id.0,
                1000, // $10.00
                old_charge_date,
                affiliate_code_id.0
            )
            .execute(&pool)
            .await
            .unwrap();

            // Process affiliate code revenue
            process_affiliate_code_revenue(&pool).await.unwrap();

            // Verify no payout was created (charge is outside date range)
            let payouts = sqlx::query!(
                r#"
                SELECT user_id, amount, created, date_available, affiliate_code_id
                FROM payouts_values
                WHERE user_id = $1 AND affiliate_code_id = $2
                "#,
                affiliate_user_id.0,
                affiliate_code_id.0
            )
            .fetch_all(&pool)
            .await
            .unwrap();

            assert_eq!(payouts.len(), 0);
        },
    )
    .await;
}

#[actix_rt::test]
pub async fn test_affiliate_code_revenue_processing_failed_charges() {
    with_test_environment(
        None,
        |test_env: TestEnvironment<ApiV3>| async move {
            let pool = test_env.db.pool.clone();

            // Create test users
            let affiliate_user_id = DBUserId(5000);
            let buyer_user_id = DBUserId(5001);

            // Create test users in the database
            sqlx::query!(
                r#"
                INSERT INTO users (id, username, email, role)
                VALUES
                ($1, 'affiliate_user5', 'affiliate5@test.com', 'developer'),
                ($2, 'buyer_user5', 'buyer5@test.com', 'developer')
                "#,
                affiliate_user_id.0,
                buyer_user_id.0
            )
            .execute(&pool)
            .await
            .unwrap();

            // Create a dummy product and price for charges to reference
            sqlx::query!(
                r#"
                INSERT INTO products (id, metadata, unitary, name)
                VALUES (9004, '{}', false, 'Test Product')
                "#
            )
            .execute(&pool)
            .await
            .unwrap();

            sqlx::query!(
                r#"
                INSERT INTO products_prices (id, product_id, currency_code, prices, public)
                VALUES (9004, 9004, 'USD', '[{"amount": 1000}]', true)
                "#
            )
            .execute(&pool)
            .await
            .unwrap();

            // Create affiliate code
            let affiliate_code_id = DBAffiliateCodeId(2004);
            sqlx::query!(
                r#"
                INSERT INTO affiliate_codes (id, created_at, created_by, affiliate, revenue_split)
                VALUES ($1, NOW(), $2, $2, NULL)
                "#,
                affiliate_code_id.0,
                affiliate_user_id.0
            )
            .execute(&pool)
            .await
            .unwrap();

            // Create test charge with failed status
            let now = Utc::now();
            let charge_date = now - Duration::days(45);

            sqlx::query!(
                r#"
                INSERT INTO charges (id, user_id, price_id, amount, currency_code, net, status, due, affiliate_code, charge_type, payment_platform, tax_amount)
                VALUES ($1, $2, 9004, $3, 'USD', $3, 'failed', $4, $5, 'recurring', 'stripe', 0)
                "#,
                7000, // unique ID
                buyer_user_id.0,
                1000, // $10.00
                charge_date,
                affiliate_code_id.0
            )
            .execute(&pool)
            .await
            .unwrap();

            // Process affiliate code revenue
            process_affiliate_code_revenue(&pool).await.unwrap();

            // Verify no payout was created (charge failed)
            let payouts = sqlx::query!(
                r#"
                SELECT user_id, amount, created, date_available, affiliate_code_id
                FROM payouts_values
                WHERE user_id = $1 AND affiliate_code_id = $2
                "#,
                affiliate_user_id.0,
                affiliate_code_id.0
            )
            .fetch_all(&pool)
            .await
            .unwrap();

            assert_eq!(payouts.len(), 0);
        },
    )
    .await;
}

#[actix_rt::test]
pub async fn test_affiliate_code_revenue_processing_multiple_affiliate_codes() {
    with_test_environment(
        None,
        |test_env: TestEnvironment<ApiV3>| async move {
            let pool = test_env.db.pool.clone();

            // Create test users
            let affiliate_user_id_1 = DBUserId(6000);
            let affiliate_user_id_2 = DBUserId(6001);
            let buyer_user_id = DBUserId(6002);

            // Create test users in the database
            sqlx::query!(
                r#"
                INSERT INTO users (id, username, email, role)
                VALUES
                ($1, 'affiliate_user6', 'affiliate6@test.com', 'developer'),
                ($2, 'affiliate_user7', 'affiliate7@test.com', 'developer'),
                ($3, 'buyer_user6', 'buyer6@test.com', 'developer')
                "#,
                affiliate_user_id_1.0,
                affiliate_user_id_2.0,
                buyer_user_id.0
            )
            .execute(&pool)
            .await
            .unwrap();

            // Create dummy products and prices for charges to reference
            sqlx::query!(
                r#"
                INSERT INTO products (id, metadata, unitary, name)
                VALUES
                (9005, '{}', false, 'Test Product'),
                (9006, '{}', false, 'Test Product')
                "#
            )
            .execute(&pool)
            .await
            .unwrap();

            sqlx::query!(
                r#"
                INSERT INTO products_prices (id, product_id, currency_code, prices, public)
                VALUES
                (9005, 9005, 'USD', '[{"amount": 1000}]', true),
                (9006, 9006, 'USD', '[{"amount": 1000}]', true)
                "#
            )
            .execute(&pool)
            .await
            .unwrap();

            // Create two affiliate codes for different users
            let affiliate_code_id_1 = DBAffiliateCodeId(2005);
            let affiliate_code_id_2 = DBAffiliateCodeId(2006);

            sqlx::query!(
                r#"
                INSERT INTO affiliate_codes (id, created_at, created_by, affiliate, revenue_split)
                VALUES
                ($1, NOW(), $2, $2, NULL),
                ($3, NOW(), $4, $4, 0.2)
                "#,
                affiliate_code_id_1.0,
                affiliate_user_id_1.0,
                affiliate_code_id_2.0,
                affiliate_user_id_2.0
            )
            .execute(&pool)
            .await
            .unwrap();

            // Create test charges for each affiliate code
            let now = Utc::now();
            let charge_date = now - Duration::days(45);

            // Charge for affiliate 1: $20.00
            sqlx::query!(
                r#"
                INSERT INTO charges (id, user_id, price_id, amount, currency_code, net, status, due, affiliate_code, charge_type, payment_platform, tax_amount)
                VALUES ($1, $2, 9005, $3, 'USD', $3, 'succeeded', $4, $5, 'recurring', 'stripe', 0)
                "#,
                8000, // unique ID
                buyer_user_id.0,
                2000, // $20.00
                charge_date,
                affiliate_code_id_1.0
            )
            .execute(&pool)
            .await
            .unwrap();

            // Charge for affiliate 2: $15.00
            sqlx::query!(
                r#"
                INSERT INTO charges (id, user_id, price_id, amount, currency_code, net, status, due, affiliate_code, charge_type, payment_platform, tax_amount)
                VALUES ($1, $2, 9006, $3, 'USD', $3, 'succeeded', $4, $5, 'recurring', 'stripe', 0)
                "#,
                9000, // unique ID
                buyer_user_id.0,
                1500, // $15.00
                charge_date,
                affiliate_code_id_2.0
            )
            .execute(&pool)
            .await
            .unwrap();

            // Process affiliate code revenue
            process_affiliate_code_revenue(&pool).await.unwrap();

            // Verify payouts were created for both affiliates
            let payouts_1 = sqlx::query!(
                r#"
                SELECT user_id, amount, created, date_available, affiliate_code_id
                FROM payouts_values
                WHERE user_id = $1 AND affiliate_code_id = $2
                "#,
                affiliate_user_id_1.0,
                affiliate_code_id_1.0
            )
            .fetch_all(&pool)
            .await
            .unwrap();

            let payouts_2 = sqlx::query!(
                r#"
                SELECT user_id, amount, created, date_available, affiliate_code_id
                FROM payouts_values
                WHERE user_id = $1 AND affiliate_code_id = $2
                "#,
                affiliate_user_id_2.0,
                affiliate_code_id_2.0
            )
            .fetch_all(&pool)
            .await
            .unwrap();

            // Affiliate 1: $20.00 * 10% = $2.00
            assert_eq!(payouts_1.len(), 1);
            let payout_amount_1 = payouts_1[0].amount;
            assert_eq!(payout_amount_1, dec!(2.00));

            // Affiliate 2: $15.00 * 20% = $3.00 (with potential floating point precision)
            assert_eq!(payouts_2.len(), 1);
            let payout_amount_2 = payouts_2[0].amount;
            // The actual calculation may have tiny precision differences due to f64 -> Decimal conversion
            assert!(payout_amount_2 > dec!(2.99) && payout_amount_2 < dec!(3.01));
        },
    )
    .await;
}

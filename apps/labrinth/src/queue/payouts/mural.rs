use ariadne::ids::UserId;
use chrono::Utc;
use eyre::{Result, eyre};
use futures::{StreamExt, TryFutureExt, stream::FuturesUnordered};
use modrinth_util::decimal::Decimal2dp;
use muralpay::{MuralError, TokenFeeRequest};
use rust_decimal::{Decimal, prelude::ToPrimitive};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tracing::{info, trace, warn};

use crate::{
    database::models::DBPayoutId,
    models::payouts::{PayoutMethodType, PayoutStatus},
    queue::payouts::{AccountBalance, PayoutFees, PayoutsQueue},
    routes::ApiError,
    util::{
        error::Context,
        gotenberg::{GotenbergClient, PaymentStatement},
    },
};

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MuralPayoutRequest {
    Fiat {
        bank_name: String,
        bank_account_owner: String,
        fiat_and_rail_details: muralpay::FiatAndRailDetails,
    },
    Blockchain {
        wallet_address: String,
    },
}

impl PayoutsQueue {
    pub async fn compute_muralpay_fees(
        &self,
        amount: Decimal2dp,
        fiat_and_rail_code: muralpay::FiatAndRailCode,
    ) -> Result<muralpay::TokenPayoutFee, ApiError> {
        let muralpay = self.muralpay.load();
        let muralpay = muralpay
            .as_ref()
            .wrap_internal_err("Mural Pay client not available")?;

        let fees = muralpay
            .client
            .get_fees_for_token_amount(&[TokenFeeRequest {
                amount: muralpay::TokenAmount {
                    token_symbol: muralpay::USDC.into(),
                    token_amount: amount.get(),
                },
                fiat_and_rail_code,
            }])
            .await
            .wrap_internal_err("failed to request fees")?;
        let fee = fees
            .into_iter()
            .next()
            .wrap_internal_err("no fees returned")?;
        Ok(fee)
    }

    pub async fn create_muralpay_payout_request(
        &self,
        payout_id: DBPayoutId,
        user_id: UserId,
        gross_amount: Decimal2dp,
        fees: PayoutFees,
        payout_details: MuralPayoutRequest,
        recipient_info: muralpay::CreatePayoutRecipientInfo,
        gotenberg: &GotenbergClient,
    ) -> Result<muralpay::PayoutRequest, ApiError> {
        let muralpay = self.muralpay.load();
        let muralpay = muralpay
            .as_ref()
            .wrap_internal_err("Mural Pay client not available")?;

        let payout_details = match payout_details {
            crate::queue::payouts::mural::MuralPayoutRequest::Fiat {
                bank_name,
                bank_account_owner,
                fiat_and_rail_details,
            } => muralpay::CreatePayoutDetails::Fiat {
                bank_name,
                bank_account_owner,
                developer_fee: None,
                fiat_and_rail_details,
            },
            crate::queue::payouts::mural::MuralPayoutRequest::Blockchain {
                wallet_address,
            } => {
                muralpay::CreatePayoutDetails::Blockchain {
                    wallet_details: muralpay::WalletDetails {
                        // only Polygon chain is currently supported
                        blockchain: muralpay::Blockchain::Polygon,
                        wallet_address,
                    },
                }
            }
        };

        // Mural takes `fees.method_fee` off the top of the amount we tell them to send
        let sent_to_method = gross_amount - fees.platform_fee;
        // ..so the net is `gross - platform_fee - method_fee`
        let net_amount = gross_amount - fees.total_fee();

        let recipient_address = recipient_info.physical_address();
        let recipient_email = recipient_info.email().to_string();
        let gross_amount_cents = gross_amount.get() * Decimal::from(100);
        let net_amount_cents = net_amount.get() * Decimal::from(100);
        let fees_cents = fees.total_fee().get() * Decimal::from(100);
        let address_line_3 = format!(
            "{}, {}, {}",
            recipient_address.city,
            recipient_address.state,
            recipient_address.zip
        );

        let payment_statement = PaymentStatement {
            payment_id: payout_id.into(),
            recipient_address_line_1: Some(recipient_address.address1.clone()),
            recipient_address_line_2: recipient_address.address2.clone(),
            recipient_address_line_3: Some(address_line_3),
            recipient_email,
            payment_date: Utc::now(),
            gross_amount_cents: gross_amount_cents
                .to_i64()
                .wrap_internal_err_with(|| eyre!("gross amount of cents `{gross_amount_cents}` cannot be expressed as an `i64`"))?,
            net_amount_cents: net_amount_cents
                .to_i64()
                .wrap_internal_err_with(|| eyre!("net amount of cents `{net_amount_cents}` cannot be expressed as an `i64`"))?,
            fees_cents: fees_cents
                .to_i64()
                .wrap_internal_err_with(|| eyre!("fees amount of cents `{fees_cents}` cannot be expressed as an `i64`"))?,
            currency_code: "USD".into(),
        };
        let payment_statement_doc = gotenberg
            .wait_for_payment_statement(&payment_statement)
            .await
            .wrap_internal_err("failed to generate payment statement")?;

        // TODO
        // std::fs::write(
        //     "/tmp/modrinth-payout-statement.pdf",
        //     base64::Engine::decode(
        //         &base64::engine::general_purpose::STANDARD,
        //         &payment_statement_doc.body,
        //     )
        //     .unwrap(),
        // )
        // .unwrap();

        let payout = muralpay::CreatePayout {
            amount: muralpay::TokenAmount {
                token_amount: sent_to_method.get(),
                token_symbol: muralpay::USDC.into(),
            },
            payout_details,
            recipient_info,
            supporting_details: Some(muralpay::SupportingDetails {
                supporting_document: Some(format!(
                    "data:application/pdf;base64,{}",
                    payment_statement_doc.body
                )),
                payout_purpose: Some(muralpay::PayoutPurpose::VendorPayment),
            }),
        };

        let payout_request = muralpay
            .client
            .create_payout_request(
                muralpay.source_account_id,
                Some(format!("User {user_id}")),
                &[payout],
            )
            .await
            .map_err(|err| match err {
                MuralError::Api(err) => ApiError::Mural(Box::new(err)),
                err => ApiError::Internal(
                    eyre!(err).wrap_err("failed to create payout request"),
                ),
            })?;

        Ok(payout_request)
    }

    pub async fn execute_mural_payout_request(
        &self,
        id: muralpay::PayoutRequestId,
    ) -> Result<(), ApiError> {
        let muralpay = self.muralpay.load();
        let muralpay = muralpay
            .as_ref()
            .wrap_internal_err("Mural Pay client not available")?;

        muralpay
            .client
            .execute_payout_request(id)
            .await
            .wrap_internal_err("failed to execute payout request")?;
        Ok(())
    }

    pub async fn cancel_mural_payout_request(
        &self,
        id: muralpay::PayoutRequestId,
    ) -> Result<()> {
        let muralpay = self.muralpay.load();
        let muralpay = muralpay
            .as_ref()
            .wrap_err("Mural Pay client not available")?;

        muralpay.client.cancel_payout_request(id).await?;
        Ok(())
    }

    pub async fn get_mural_balance(
        &self,
    ) -> Result<Option<AccountBalance>, ApiError> {
        let muralpay = self.muralpay.load();
        let muralpay = muralpay
            .as_ref()
            .wrap_internal_err("Mural Pay client not available")?;

        let account = muralpay
            .client
            .get_account(muralpay.source_account_id)
            .await
            .wrap_internal_err("failed to get source account")?;
        let details = account
            .account_details
            .wrap_internal_err("source account does not have details")?;
        let available = details
            .balances
            .iter()
            .map(|balance| {
                if balance.token_symbol == muralpay::USDC {
                    balance.token_amount
                } else {
                    Decimal::ZERO
                }
            })
            .sum::<Decimal>();
        Ok(Some(AccountBalance {
            available,
            pending: Decimal::ZERO,
        }))
    }
}

/// Finds Labrinth payouts which are not complete, fetches their corresponding
/// Mural state, and updates the payout status.
pub async fn sync_pending_payouts_from_mural(
    db: &PgPool,
    mural: &muralpay::Client,
    limit: u32,
) -> eyre::Result<()> {
    #[derive(Debug)]
    struct UpdatePayoutOp {
        payout_id: i64,
        status: PayoutStatus,
    }

    info!("Syncing pending payouts from Mural");

    let mut txn = db
        .begin()
        .await
        .wrap_internal_err("failed to begin transaction")?;

    let rows = sqlx::query!(
        "
        SELECT id, platform_id FROM payouts
        WHERE
            method = $1
            AND status = ANY($2::text[])
        LIMIT $3
        ",
        &PayoutMethodType::MuralPay.to_string(),
        &[
            PayoutStatus::InTransit,
            PayoutStatus::Unknown,
            PayoutStatus::Cancelling
        ]
        .iter()
        .map(|s| s.to_string())
        .collect::<Vec<String>>(),
        i64::from(limit),
    )
    .fetch_all(&mut *txn)
    .await
    .wrap_internal_err("failed to fetch incomplete Mural payouts")?;

    info!("Found {} incomplete Mural payouts", rows.len());

    let futs = rows.into_iter().map(|row| async move {
        let platform_id = row.platform_id.wrap_err("no platform ID")?;
        let payout_request_id = platform_id.parse::<muralpay::PayoutRequestId>()
            .wrap_err_with(|| eyre!("platform ID '{platform_id:?}' is not a valid payout request ID"))?;
        let payout_request = mural.get_payout_request(payout_request_id).await
            .wrap_err_with(|| eyre!("failed to fetch payout request {payout_request_id}"))?;

        let new_payout_status = match payout_request.status {
            muralpay::PayoutStatus::Canceled => Some(PayoutStatus::Cancelled),
            muralpay::PayoutStatus::Executed => Some(PayoutStatus::Success),
            muralpay::PayoutStatus::Failed => Some(PayoutStatus::Failed),
            _ => None,
        };

        if let Some(status) = new_payout_status {
            eyre::Ok(Some(UpdatePayoutOp {
                payout_id: row.id,
                status
            }))
        } else {
            eyre::Ok(None)
        }
    }.map_err(move |err| eyre!(err).wrap_err(eyre!("failed to update payout with ID '{}'", row.id))));
    let mut futs = futs.collect::<FuturesUnordered<_>>();

    let mut payout_ids = Vec::<i64>::new();
    let mut payout_statuses = Vec::<String>::new();

    while let Some(result) = futs.next().await {
        let op = match result {
            Ok(Some(op)) => op,
            Ok(None) => continue,
            Err(err) => {
                warn!("Failed to update payout: {err:#?}");
                continue;
            }
        };

        payout_ids.push(op.payout_id);
        payout_statuses.push(op.status.to_string());
    }

    sqlx::query!(
        "
        UPDATE payouts
        SET status = u.status
        FROM UNNEST($1::bigint[], $2::varchar[]) AS u(id, status)
        WHERE payouts.id = u.id
        ",
        &payout_ids,
        &payout_statuses,
    )
    .execute(&mut *txn)
    .await
    .wrap_internal_err("failed to update payout statuses")?;

    txn.commit()
        .await
        .wrap_internal_err("failed to commit transaction")?;

    Ok(())
}

/// Queries Mural for canceled or failed payouts, and updates the corresponding
/// Labrinth payouts' statuses.
///
/// This will update:
/// - Mural payout requests which are failed or canceled
/// - Mural payout requests where all of the payouts are failed or canceled
pub async fn sync_failed_mural_payouts_to_labrinth(
    db: &PgPool,
    mural: &muralpay::Client,
    limit: u32,
) -> eyre::Result<()> {
    info!("Syncing failed Mural payouts to Labrinth");

    let mut next_id = None;
    loop {
        let search_resp = mural
            .search_payout_requests(
                None,
                Some(muralpay::SearchParams {
                    limit: Some(u64::from(limit)),
                    next_id,
                }),
            )
            .await
            .wrap_internal_err(
                "failed to fetch failed payout requests from Mural",
            )?;
        if search_resp.results.is_empty() {
            break;
        }
        next_id = search_resp.next_id;

        let mut payout_platform_ids = Vec::<String>::new();
        let mut payout_new_statuses = Vec::<String>::new();

        for payout_request in search_resp.results {
            let payout_platform_id = payout_request.id;

            let new_payout_status = match payout_request.status {
                muralpay::PayoutStatus::Canceled => {
                    trace!(
                        "- Payout request {payout_platform_id} set to {} because it is cancelled in Mural",
                        PayoutStatus::Cancelled
                    );
                    Some(PayoutStatus::Cancelled)
                }
                muralpay::PayoutStatus::Failed => {
                    trace!(
                        "- Payout request {payout_platform_id} set to {} because it is failed in Mural",
                        PayoutStatus::Failed
                    );
                    Some(PayoutStatus::Failed)
                }
                // this will also fail any payout request which has no payouts
                _ if payout_request
                    .payouts
                    .iter()
                    .all(payout_should_be_failed) =>
                {
                    trace!(
                        "- Payout request {payout_platform_id} set to {} because all of its payouts are failed",
                        PayoutStatus::Failed
                    );
                    Some(PayoutStatus::Failed)
                }
                _ => None,
            };

            if let Some(new_payout_status) = new_payout_status {
                payout_platform_ids.push(payout_platform_id.to_string());
                payout_new_statuses.push(new_payout_status.to_string());
            }
        }

        let result = sqlx::query!(
            "
            UPDATE payouts
            SET status = u.status
            FROM UNNEST($1::text[], $2::text[]) AS u(platform_id, status)
            WHERE
                payouts.method = $3
                AND payouts.platform_id = u.platform_id
            ",
            &payout_platform_ids,
            &payout_new_statuses,
            PayoutMethodType::MuralPay.as_str(),
        )
        .execute(db)
        .await
        .wrap_internal_err("failed to update payout statuses")?;

        info!(
            "Attempted to update {} payouts in database from Mural info, {} rows affected",
            payout_platform_ids.len(),
            result.rows_affected()
        );

        if next_id.is_none() {
            break;
        }
    }

    Ok(())
}

fn payout_should_be_failed(payout: &muralpay::Payout) -> bool {
    matches!(
        payout.details,
        muralpay::PayoutDetails::Fiat(muralpay::FiatPayoutDetails {
            fiat_payout_status: muralpay::FiatPayoutStatus::Failed { .. }
                | muralpay::FiatPayoutStatus::Refunded { .. },
            ..
        })
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::{
        api_v3::ApiV3,
        environment::{TestEnvironment, with_test_environment},
    };
    use muralpay::MuralPayMock;
    use rust_decimal::dec;

    fn create_mock_payout_request(
        id: &str,
        status: muralpay::PayoutStatus,
    ) -> muralpay::PayoutRequest {
        use muralpay::*;

        PayoutRequest {
            id: PayoutRequestId(id.parse().unwrap()),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            source_account_id: AccountId(uuid::Uuid::new_v4()),
            transaction_hash: None,
            memo: None,
            status,
            payouts: vec![Payout {
                id: PayoutId(uuid::Uuid::new_v4()),
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
                amount: TokenAmount {
                    token_amount: dec!(10.00),
                    token_symbol: "USDC".into(),
                },
                details: PayoutDetails::Fiat(FiatPayoutDetails {
                    fiat_and_rail_code: FiatAndRailCode::Usd,
                    fiat_payout_status: FiatPayoutStatus::Pending {
                        initiated_at: chrono::Utc::now(),
                    },
                    fiat_amount: FiatAmount {
                        fiat_amount: dec!(10.00),
                        fiat_currency_code: CurrencyCode::Usd,
                    },
                    transaction_fee: TokenAmount {
                        token_amount: dec!(1.00),
                        token_symbol: "USDC".into(),
                    },
                    exchange_fee_percentage: dec!(0.0),
                    exchange_rate: dec!(1.0),
                    fee_total: TokenAmount {
                        token_amount: dec!(1.00),
                        token_symbol: "USDC".into(),
                    },
                    developer_fee: None,
                }),
                recipient_info: PayoutRecipientInfo::Inline {
                    name: "John Smith".into(),
                    details: InlineRecipientDetails::Fiat {
                        details: InlineFiatRecipientDetails {
                            fiat_currency_code: CurrencyCode::Usd,
                            bank_name: "Foo Bank".into(),
                            truncated_bank_account_number: "1234".into(),
                        },
                    },
                },
            }],
        }
    }

    fn create_mock_muralpay() -> muralpay::Client {
        muralpay::Client::from_mock(MuralPayMock {
            get_payout_request: Box::new(|_id| {
                Err(muralpay::MuralError::Api(muralpay::ApiError {
                    error_instance_id: uuid::Uuid::new_v4(),
                    name: "Not found".to_string(),
                    message: "Payout request not found".to_string(),
                    details: vec![],
                    params: std::collections::HashMap::new(),
                }))
            }),
            search_payout_requests: Box::new(|_filter, _params| {
                Ok(muralpay::SearchResponse {
                    total: 0,
                    next_id: None,
                    results: vec![],
                })
            }),
            ..Default::default()
        })
    }

    async fn setup_test_db_with_payouts(
        db: &sqlx::PgPool,
        payouts: Vec<(i64, String, PayoutStatus)>,
    ) -> Result<(), eyre::Error> {
        for (id, platform_id, status) in payouts {
            sqlx::query!(
                "
                INSERT INTO payouts (id, method, platform_id, status, user_id, amount, created)
                VALUES ($1, $2, $3, $4, $5, 10.0, NOW())
                ",
                id,
                PayoutMethodType::MuralPay.as_str(),
                platform_id,
                status.as_str(),
                1i64, // user_id
            )
            .execute(db)
            .await?;
        }
        Ok(())
    }

    #[actix_rt::test]
    async fn test_sync_pending_payouts_from_mural_success() {
        with_test_environment(None, |env: TestEnvironment<ApiV3>| async move {
            let db = &env.db.pool;

            // Setup test data
            let uuid1 = uuid::Uuid::new_v4().to_string();
            let uuid2 = uuid::Uuid::new_v4().to_string();
            let uuid3 = uuid::Uuid::new_v4().to_string();
            let uuid4 = uuid::Uuid::new_v4().to_string();

            setup_test_db_with_payouts(
                db,
                vec![
                    (1, uuid1.clone(), PayoutStatus::InTransit),
                    (2, uuid2.clone(), PayoutStatus::Unknown),
                    (3, uuid3.clone(), PayoutStatus::Cancelling),
                    (4, uuid4.clone(), PayoutStatus::InTransit), // This one won't change
                ],
            )
            .await
            .unwrap();

            // Verify setup
            let updated_payouts = sqlx::query!(
                r#"
                SELECT
                    id,
                    status AS "status: PayoutStatus"
                FROM payouts
                ORDER BY id
                "#
            )
            .fetch_all(db)
            .await
            .unwrap();
            assert_eq!(updated_payouts.len(), 4);
            assert_eq!(updated_payouts[0].status, PayoutStatus::InTransit);
            assert_eq!(updated_payouts[1].status, PayoutStatus::Unknown);
            assert_eq!(updated_payouts[2].status, PayoutStatus::Cancelling);
            assert_eq!(updated_payouts[3].status, PayoutStatus::InTransit);

            // Setup mock client with proper responses
            let mut mock = MuralPayMock::default();

            // Create mock payout requests
            let payout1 = create_mock_payout_request(
                &uuid1,
                muralpay::PayoutStatus::Executed,
            );
            let payout2 = create_mock_payout_request(
                &uuid2,
                muralpay::PayoutStatus::Canceled,
            );
            let payout3 = create_mock_payout_request(
                &uuid3,
                muralpay::PayoutStatus::Failed,
            );
            let payout4 = create_mock_payout_request(
                &uuid4,
                muralpay::PayoutStatus::Pending,
            );

            // Mock get_payout_request
            let payout_requests = std::collections::HashMap::from([
                (uuid1.clone(), payout1.clone()),
                (uuid2.clone(), payout2.clone()),
                (uuid3.clone(), payout3.clone()),
                (uuid4.clone(), payout4.clone()),
            ]);

            mock.get_payout_request = Box::new(move |id| {
                let id_str = id.to_string();
                match payout_requests.get(&id_str) {
                    Some(request) => Ok(request.clone()),
                    None => {
                        Err(muralpay::MuralError::Api(muralpay::ApiError {
                            error_instance_id: uuid::Uuid::new_v4(),
                            name: "Not found".to_string(),
                            message: "Payout request not found".to_string(),
                            details: vec![],
                            params: std::collections::HashMap::new(),
                        }))
                    }
                }
            });

            // Mock search_payout_requests
            mock.search_payout_requests = Box::new(move |_filter, _params| {
                Ok(muralpay::SearchResponse {
                    total: 4,
                    results: vec![
                        payout1.clone(),
                        payout2.clone(),
                        payout3.clone(),
                        payout4.clone(),
                    ],
                    next_id: None,
                })
            });

            let mock_client = muralpay::Client::from_mock(mock);

            // Run the function
            let result =
                sync_pending_payouts_from_mural(db, &mock_client, 10).await;
            assert!(result.is_ok());

            // Verify results
            let updated_payouts = sqlx::query!(
                r#"
                SELECT
                    id,
                    status AS "status: PayoutStatus"
                FROM payouts
                ORDER BY id
                "#
            )
            .fetch_all(db)
            .await
            .unwrap();
            assert_eq!(updated_payouts.len(), 4);
            assert_eq!(updated_payouts[0].status, PayoutStatus::Success);
            assert_eq!(updated_payouts[1].status, PayoutStatus::Cancelled);
            assert_eq!(updated_payouts[2].status, PayoutStatus::Failed);
            assert_eq!(updated_payouts[3].status, PayoutStatus::InTransit);
        })
        .await;
    }

    #[actix_rt::test]
    async fn test_sync_pending_payouts_from_mural_handles_missing_platform_id()
    {
        with_test_environment(None, |env: TestEnvironment<ApiV3>| async move {
            let db = &env.db.pool;

            // Setup test data with null platform_id
            sqlx::query!(
                    "
                    INSERT INTO payouts (id, method, platform_id, status, user_id, amount, created)
                    VALUES ($1, $2, NULL, $3, $4, 10.00, NOW())
                    ",
                    1,
                    PayoutMethodType::MuralPay.as_str(),
                    PayoutStatus::InTransit.as_str(),
                    1i64, // user_id
                )
                .execute(db)
                .await
                .unwrap();

            let mock_client = create_mock_muralpay();

            // Run the function - should not fail even with null platform_id
            sync_pending_payouts_from_mural(
                db,
                &mock_client,
                10,
            )
            .await.unwrap();
        }).await;
    }

    #[actix_rt::test]
    async fn test_sync_failed_mural_payouts_to_labrinth_success() {
        with_test_environment(None, |env: TestEnvironment<ApiV3>| async move {
            let db = &env.db.pool;

            // Setup test data
            let uuid1 = uuid::Uuid::new_v4().to_string();
            let uuid2 = uuid::Uuid::new_v4().to_string();
            let uuid3 = uuid::Uuid::new_v4().to_string();

            setup_test_db_with_payouts(
                db,
                vec![
                    (1, uuid1.clone(), PayoutStatus::InTransit), // Will be updated to cancelled
                    (2, uuid2.clone(), PayoutStatus::Success), // Will be updated to failed
                    (3, uuid3.clone(), PayoutStatus::Success), // Will remain unchanged
                ],
            )
            .await
            .unwrap();

            // Setup mock client
            let mut mock = MuralPayMock::default();

            // Create mock payout requests
            let payout1 = create_mock_payout_request(
                &uuid1,
                muralpay::PayoutStatus::Canceled,
            );
            let payout2 = create_mock_payout_request(
                &uuid2,
                muralpay::PayoutStatus::Failed,
            );
            let payout3 = create_mock_payout_request(
                &uuid::Uuid::new_v4().to_string(),
                muralpay::PayoutStatus::Failed,
            ); // No matching DB record

            // Mock search_payout_requests
            mock.search_payout_requests = Box::new(move |_filter, _params| {
                Ok(muralpay::SearchResponse {
                    total: 3,
                    results: vec![
                        payout1.clone(),
                        payout2.clone(),
                        payout3.clone(),
                    ],
                    next_id: None,
                })
            });

            let mock_client = muralpay::Client::from_mock(mock);

            // Run the function
            let result =
                sync_failed_mural_payouts_to_labrinth(db, &mock_client, 10)
                    .await;
            assert!(result.is_ok());

            // Verify results
            let updated_payouts = sqlx::query!(
                r#"
                SELECT
                    id,
                    status AS "status: PayoutStatus"
                FROM payouts
                ORDER BY id
                "#
            )
            .fetch_all(db)
            .await
            .unwrap();

            assert_eq!(updated_payouts.len(), 3);
            assert_eq!(updated_payouts[0].status, PayoutStatus::Cancelled); // search_req_1 -> canceled
            assert_eq!(updated_payouts[1].status, PayoutStatus::Failed); // search_req_2 -> failed
            assert_eq!(updated_payouts[2].status, PayoutStatus::Success); // search_req_3 unchanged
        })
        .await;
    }

    #[actix_rt::test]
    async fn test_sync_failed_mural_payouts_to_labrinth_handles_wrong_status() {
        with_test_environment(None, |env: TestEnvironment<ApiV3>| async move {
            let db = &env.db.pool;

            // Setup test data
            let uuid1 = uuid::Uuid::new_v4().to_string();

            setup_test_db_with_payouts(
                db,
                vec![(1, uuid1.clone(), PayoutStatus::InTransit)],
            )
            .await
            .unwrap();

            // Setup mock client with a payout that has unexpected status
            let mut mock = MuralPayMock::default();

            let payout1 = create_mock_payout_request(
                &uuid1,
                muralpay::PayoutStatus::Pending,
            ); // Should be filtered out

            // Mock search_payout_requests
            mock.search_payout_requests = Box::new(move |_filter, _params| {
                Ok(muralpay::SearchResponse {
                    total: 1,
                    results: vec![payout1.clone()],
                    next_id: None,
                })
            });

            let mock_client = muralpay::Client::from_mock(mock);

            // Run the function - should handle this gracefully
            sync_failed_mural_payouts_to_labrinth(
                db,
                &mock_client,
                10,
            )
            .await
            .unwrap();

            // Verify status remains unchanged
            let payout =
                sqlx::query!(r#"SELECT status AS "status: PayoutStatus" FROM payouts WHERE id = 1"#)
                    .fetch_one(db)
                    .await
                    .unwrap();

            assert_eq!(payout.status, PayoutStatus::InTransit); // Unchanged
        })
        .await;
    }
}

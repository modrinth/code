use crate::{queue::payouts::flow::ExecuteContext, routes::ApiError};

#[derive(Debug)]
pub(super) struct TremendousFlow {}

pub(super) async fn create(
    queue: &PayoutsQueue,
    amount: Decimal,
    details: TremendousDetails,
) -> Result<PayoutFlow, ApiError> {
    let forex: TremendousForexResponse = queue
        .make_tremendous_request(Method::GET, "forex", None::<()>)
        .await
        .wrap_internal_err("failed to fetch Tremendous forex data")?;

    let currency = details.currency.unwrap_or(TremendousCurrency::Usd);
    let currency_code = currency.to_string();
    let usd_to_currency = forex
        .forex
        .get(&currency_code)
        .copied()
        .wrap_internal_err_with(|| {
            eyre!("no Tremendous forex rate for '{currency}'")
        })?;

    Ok(PayoutFlow { total_fee })
}

pub(super) async fn execute(
    ExecuteContext {
        queue,
        user,
        payout_id,
        db: _,
        mut transaction,
        gotenberg: _,
    }: ExecuteContext<'_>,
    TremendousFlow {}: TremendousFlow,
) -> Result<(), ApiError> {
}

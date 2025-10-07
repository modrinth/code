use std::env;

use eyre::{Result, WrapErr};
use muralpay::{MuralPay, organization::SearchRequest};

#[tokio::main]
async fn main() -> Result<()> {
    let api_url = env::var("MURALPAY_API_URL")
        .unwrap_or_else(|_| muralpay::SANDBOX_API_URL.to_string());
    let api_key = env::var("MURALPAY_API_KEY").wrap_err("no API key")?;

    let mural_pay = MuralPay::new(api_url, api_key);
    let orgs = mural_pay
        .search_organizations(SearchRequest::default())
        .await?;
    println!("{orgs:#?}");

    let accounts = mural_pay.get_all_accounts().await?;
    println!("{accounts:#?}");

    Ok(())
}

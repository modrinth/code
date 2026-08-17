//! Example Aditude client.
#![expect(clippy::print_stdout, reason = "this is an example")]

use aditude::{Client, v1, v2};
use eyre::{Context, Result};

#[tokio::main]
async fn main() -> Result<()> {
    _ = dotenvy::dotenv();
    let aditude = Client::new(
        dotenvy::var("ADITUDE_API_URL").wrap_err("no API URL")?,
        dotenvy::var("ADITUDE_API_KEY").wrap_err("no API key")?,
    );

    let resp = aditude
        .get_metrics_v1(v1::GetMetrics {
            metrics: &[v1::MetricKind::Impressions, v1::MetricKind::Revenue],
            range: v1::Range::Yesterday,
            interval: v1::Interval::OneDay,
        })
        .await
        .wrap_err("failed to get metrics")?;
    println!("{resp:#?}");

    println!("\n---\n");

    let resp = aditude
        .get_metrics_v2(v2::GetMetrics {
            metrics: &[v2::MetricKind::Impressions, v2::MetricKind::Revenue],
            range: v2::Range::Yesterday,
            interval: v2::Interval::OneDay,
        })
        .await
        .wrap_err("failed to get metrics")?;
    println!("{resp:#?}");

    Ok(())
}

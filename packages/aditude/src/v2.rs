pub use crate::v1::{Interval, Range};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use secrecy::ExposeSecret;
use serde::{Deserialize, Serialize};

use crate::Client;

impl Client {
    /// Fetches insights metrics from the API.
    ///
    /// # Errors
    ///
    /// Errors if the request could not be completed.
    pub async fn get_metrics_v2(
        &self,
        req: GetMetrics<'_>,
    ) -> reqwest::Result<Metrics> {
        #[derive(Debug, Clone, Serialize)]
        #[serde(rename_all = "camelCase")]
        pub struct Body<'a> {
            pub metrics: &'a [MetricKind],
            pub range: &'static str,
            pub interval: Interval,
            #[serde(with = "chrono::serde::ts_milliseconds_option")]
            pub start_time: Option<DateTime<Utc>>,
            #[serde(with = "chrono::serde::ts_milliseconds_option")]
            pub end_time: Option<DateTime<Utc>>,
        }

        #[cfg(feature = "mock")]
        if let Some(mock) = &*self.mock.load() {
            return Ok(mock.get_metrics_v2());
        }

        let body = Body {
            metrics: req.metrics,
            range: match req.range {
                Range::Yesterday => "Yesterday",
                Range::Custom { .. } => "custom",
            },
            interval: req.interval,
            start_time: if let Range::Custom { start, .. } = req.range {
                Some(start)
            } else {
                None
            },
            end_time: if let Range::Custom { end, .. } = req.range {
                Some(end)
            } else {
                None
            },
        };

        self.http
            .post(format!("{}/public/insights/metrics/v2", self.api_url))
            .bearer_auth(self.api_key.expose_secret())
            .json(&body)
            .send()
            .await?
            .error_for_status()?
            .json::<Metrics>()
            .await
    }
}

#[derive(Debug, Clone)]
pub struct GetMetrics<'a> {
    pub metrics: &'a [MetricKind],
    pub range: Range,
    pub interval: Interval,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MetricKind {
    #[serde(rename = "IMPRESSIONS")]
    Impressions,
    #[serde(rename = "REVENUE")]
    Revenue,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Metrics {
    pub responses: Vec<Response>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Response {
    pub rows: Vec<Row>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Row {
    #[serde(rename = "IMPRESSIONS")]
    pub impressions: Option<u128>,
    #[serde(rename = "REVENUE")]
    pub revenue: Option<Decimal>,
    #[serde(rename = "_TIME", with = "chrono::serde::ts_milliseconds")]
    pub time: DateTime<Utc>,
}

#[cfg(feature = "mock")]
const _: () = {
    use crate::{mock::AditudeMock, v2};

    impl AditudeMock {
        pub(crate) fn get_metrics_v2(&self) -> v2::Metrics {
            self.metrics_v2_response
                .as_ref()
                .expect("missing mock `metrics_v2_response`")
                .clone()
        }
    }
};

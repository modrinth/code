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
    pub async fn get_metrics_v1(
        &self,
        req: GetMetrics<'_>,
    ) -> reqwest::Result<Vec<MetricsResponse>> {
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
            return Ok(mock.get_metrics_v1());
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
            .post(format!("{}/public/insights/metrics", self.api_url))
            .bearer_auth(self.api_key.expose_secret())
            .json(&body)
            .send()
            .await?
            .error_for_status()?
            .json::<Vec<MetricsResponse>>()
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
pub enum Range {
    #[serde(rename = "Yesterday")]
    Yesterday,
    #[serde(rename = "custom", rename_all = "camelCase")]
    Custom {
        #[serde(with = "chrono::serde::ts_milliseconds")]
        start: DateTime<Utc>,
        #[serde(with = "chrono::serde::ts_milliseconds")]
        end: DateTime<Utc>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Interval {
    #[serde(rename = "1d")]
    OneDay,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MetricKind {
    #[serde(rename = "METRIC_IMPRESSIONS")]
    Impressions,
    #[serde(rename = "METRIC_REVENUE")]
    Revenue,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MetricsResponse {
    pub points_list: Vec<Point>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Point {
    pub metric: Metric,
    pub time: Time,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Metric {
    #[serde(with = "rust_decimal::serde::float_option", default)]
    pub revenue: Option<Decimal>,
    pub impressions: Option<u128>,
    #[serde(with = "rust_decimal::serde::float_option", default)]
    pub cpm: Option<Decimal>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Time {
    pub seconds: u64,
    pub nanos: u64,
}

#[cfg(feature = "mock")]
const _: () = {
    use crate::{mock::AditudeMock, v1};

    impl AditudeMock {
        pub(crate) fn get_metrics_v1(&self) -> Vec<v1::MetricsResponse> {
            self.metrics_v1_response
                .as_ref()
                .expect("missing mock `metrics_v1_response`")
                .clone()
        }
    }
};

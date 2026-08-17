#![expect(missing_docs, reason = "test crate")]

use aditude::{Client, mock::AditudeMock, v1, v2};
use chrono::{DateTime, Utc};
use rust_decimal::dec;

#[tokio::test]
async fn mock_metrics() {
    let metrics_v1_response = vec![v1::MetricsResponse {
        points_list: vec![v1::Point {
            metric: v1::Metric {
                revenue: Some(dec!(1.23)),
                impressions: Some(456),
                cpm: Some(dec!(7.89)),
            },
            time: v1::Time {
                seconds: 123,
                nanos: 0,
            },
        }],
    }];

    let epoch = DateTime::UNIX_EPOCH;
    let metrics_v2_response = v2::Metrics {
        responses: vec![
            v2::Response {
                rows: vec![v2::Row {
                    impressions: Some(123),
                    revenue: None,
                    time: epoch,
                }],
            },
            v2::Response {
                rows: vec![v2::Row {
                    impressions: None,
                    revenue: Some(dec!(45.6)),
                    time: epoch,
                }],
            },
        ],
    };

    let aditude = Client::from_mock(AditudeMock {
        metrics_v1_response: Some(metrics_v1_response.clone()),
        metrics_v2_response: Some(metrics_v2_response.clone()),
    });
    assert_eq!(
        metrics_v1_response,
        aditude
            .get_metrics_v1(v1::GetMetrics {
                metrics: &[],
                range: v1::Range::Yesterday,
                interval: v1::Interval::OneDay,
            })
            .await
            .unwrap()
    );
    assert_eq!(
        metrics_v2_response,
        aditude
            .get_metrics_v2(v2::GetMetrics {
                metrics: &[],
                range: v2::Range::Yesterday,
                interval: v2::Interval::OneDay,
            })
            .await
            .unwrap()
    );
}

#[tokio::test]
async fn test_yesterday() {
    _ = dotenvy::dotenv();
    let (Ok(url), Ok(key)) = (
        dotenvy::var("ADITUDE_API_URL"),
        dotenvy::var("ADITUDE_API_KEY"),
    ) else {
        return;
    };
    if url.trim().is_empty()
        || key.trim().is_empty()
        || url == "none"
        || key == "none"
    {
        return;
    }

    let aditude = Client::new(url, key);

    let real_yesterday = aditude
        .get_metrics_v2(v2::GetMetrics {
            metrics: &[v2::MetricKind::Impressions, v2::MetricKind::Revenue],
            range: v2::Range::Yesterday,
            interval: v2::Interval::OneDay,
        })
        .await
        .unwrap();

    // Aditude defines the time range "Yesterday" according to this logic, I think.
    // We need to make sure it stays defined like this.
    let now = Utc::now();
    let (start_of_yesterday, end_of_yesterday) = aditude::yesterday(now);

    let our_yesterday = aditude
        .get_metrics_v2(v2::GetMetrics {
            metrics: &[v2::MetricKind::Impressions, v2::MetricKind::Revenue],
            range: v2::Range::Custom {
                start: start_of_yesterday,
                end: end_of_yesterday,
            },
            interval: v2::Interval::OneDay,
        })
        .await
        .unwrap();

    assert_eq!(real_yesterday, our_yesterday);
}

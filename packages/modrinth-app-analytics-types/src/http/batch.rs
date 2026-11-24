use std::time::{SystemTime, UNIX_EPOCH};

use chrono::{DateTime, TimeDelta, Utc};
use serde::{Deserialize, Serialize};
use smol_str::SmolStr;
use uuid::Uuid;

use crate::enums::{AppArch, AppOs};
use crate::event::AppEventType;

const EVENT_AGE_THRESHOLD_HOURS: i64 = 48; // Reject events older than 2 days

/// App event batch request batch
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppEventBatch<E = AppEvent> {
    /// Sent time from the client clock
    pub sent_at: DateTime<Utc>,
    /// A unique installation ID for the client. Should be a UUIDv4
    pub installation_id: Uuid,
    /// Client OS
    pub os: AppOs,
    /// Client CPU arch
    pub arch: AppArch,
    /// App version
    pub app_version: SmolStr,
    /// List of events to be processed
    pub events: Vec<E>,
}

/// A single event in an [`AppEventBatch`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppEvent {
    /// Event ID (UUIDv7)
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    /// Where in the app the action was initiated from, as a PascalCase
    /// identifier such as `BrowseTab`. See
    /// [`is_valid_source`](crate::http::extract::is_valid_source).
    pub source: SmolStr,
    pub event: AppEventType,
}

impl AppEvent {
    #[inline]
    pub fn new(
        ctx: &uuid::ContextV7,
        source: SmolStr,
        event: AppEventType,
    ) -> Self {
        // this code is adapted from Utc::now()
        // these panics should not happen

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time before Unix epoch");

        let secs = now.as_secs();
        let subsec_nanos = now.subsec_nanos();

        let uuid_ts = uuid::Timestamp::from_unix(ctx, secs, subsec_nanos);
        let timestamp = DateTime::<Utc>::from_timestamp(
            now.as_secs() as i64,
            now.subsec_nanos(),
        )
        .unwrap();

        let id = Uuid::new_v7(uuid_ts);

        AppEvent {
            id,
            timestamp,
            source,
            event,
        }
    }
}

pub struct SkewCorrectionParameters {
    pub client_sent_at: DateTime<Utc>,
    pub server_received_at: DateTime<Utc>,
}

#[derive(Debug, thiserror::Error)]
pub enum SkewCorrectionError {
    #[error("the event is too old")]
    EventTooOld { num_seconds: i64 },
    #[error("timestamp underflowed, this should not happen")]
    TimestampUnderflow,
}

impl AppEvent {
    pub fn corrected_timestamp_skew(
        &self,
        params: &SkewCorrectionParameters,
    ) -> Result<DateTime<Utc>, SkewCorrectionError> {
        // Both timestamps come from the client clock, so its skew cancels out
        let age = params.client_sent_at.signed_duration_since(self.timestamp);

        // Negative age can happen if the client's clock went backwards before the event was sent. In this case, clamp
        // the TimeDelta to zero, the event_ts will be equal to params.server_received_at
        let age = age.max(TimeDelta::zero());

        if age.num_hours() >= EVENT_AGE_THRESHOLD_HOURS {
            return Err(SkewCorrectionError::EventTooOld {
                num_seconds: age.num_seconds(),
            });
        }

        let event_ts = params
            .server_received_at
            .checked_sub_signed(age)
            .ok_or(SkewCorrectionError::TimestampUnderflow)?;

        Ok(event_ts)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn time(s: &str) -> DateTime<Utc> {
        s.parse().unwrap()
    }

    fn correct(
        timestamp: &str,
        sent_at: &str,
        received_at: &str,
    ) -> Result<DateTime<Utc>, SkewCorrectionError> {
        AppEvent {
            id: Uuid::nil(),
            timestamp: time(timestamp),
            source: "app".into(),
            event: AppEventType::MicrosoftSignIn {},
        }
        .corrected_timestamp_skew(&SkewCorrectionParameters {
            client_sent_at: time(sent_at),
            server_received_at: time(received_at),
        })
    }

    #[test]
    fn cancels_client_clock_skew() {
        let event_ts = correct(
            "2026-11-23T09:59:00Z",
            "2026-11-23T10:00:00Z",
            "2026-11-23T12:00:00Z",
        )
        .unwrap();

        assert_eq!(event_ts, time("2026-11-23T11:59:00Z"));
    }

    #[test]
    fn accepts_recent_event_from_client_clock_days_behind() {
        let event_ts = correct(
            "2026-11-20T11:59:00Z",
            "2026-11-20T12:00:00Z",
            "2026-11-23T12:00:00Z",
        )
        .unwrap();

        assert_eq!(event_ts, time("2026-11-23T11:59:00Z"));
    }

    #[test]
    fn clamps_event_after_sent_at_to_received_at() {
        let event_ts = correct(
            "2026-11-23T12:05:00Z",
            "2026-11-23T12:00:00Z",
            "2026-11-23T12:00:01Z",
        )
        .unwrap();

        assert_eq!(event_ts, time("2026-11-23T12:00:01Z"));
    }

    #[test]
    fn rejects_event_older_than_threshold_when_sent() {
        let result = correct(
            "2026-11-21T11:00:00Z",
            "2026-11-23T12:00:00Z",
            "2026-11-23T12:00:00Z",
        );

        assert!(matches!(
            result,
            Err(SkewCorrectionError::EventTooOld { .. })
        ));
    }
}

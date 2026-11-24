use chrono::{DateTime, Utc};
use smol_str::SmolStr;
use uuid::Uuid;

use crate::enums::{AppArch, AppOs, Country};
use crate::event::AppEventType;

#[derive(Debug, Clone)]
pub struct NormalizedAppEvent {
    /// The event ID. Should be UUIDv7.
    pub event_id: Uuid,
    /// Sent time from the client clock
    pub sent_at: DateTime<Utc>,
    /// The timestamp from the client clock when the event was sent
    pub client_timestamp: DateTime<Utc>,
    /// The corrected event timestamp
    pub event_ts: DateTime<Utc>,
    /// The country the client is located in
    pub ip_country: Country,
    /// A unique installation ID for the client. Should be a UUIDv4
    pub installation_id: Uuid,
    /// Client OS
    pub os: AppOs,
    /// Client CPU arch
    pub arch: AppArch,
    /// App version
    pub app_version: SmolStr,
    /// Where in the app the action was initiated from, such as `browse`
    pub source: SmolStr,
    /// Event data
    pub data: AppEventType,
}

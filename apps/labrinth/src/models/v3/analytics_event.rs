use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::models::ids::AnalyticsEventId;

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct AnalyticsEvent {
    pub id: AnalyticsEventId,
    #[serde(flatten)]
    pub meta: AnalyticsEventMeta,
    pub starts: DateTime<Utc>,
    pub ends: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct AnalyticsEventMeta {
    pub title: String,
    pub announcement_url: Option<String>,
}

impl From<crate::database::models::DBAnalyticsEvent> for AnalyticsEvent {
    fn from(data: crate::database::models::DBAnalyticsEvent) -> Self {
        Self {
            id: data.id.into(),
            meta: data.meta,
            starts: data.starts,
            ends: data.ends,
        }
    }
}

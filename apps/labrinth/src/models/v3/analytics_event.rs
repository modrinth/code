use std::collections::HashSet;

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
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub announcement_url: Option<String>,
    #[serde(default)]
    pub for_metric_kind: HashSet<MetricKind>,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
    utoipa::ToSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum MetricKind {
    Views,
    Downloads,
    Playtime,
    Revenue,
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

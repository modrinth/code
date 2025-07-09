use crate::models::ids::{ReportId, ThreadId};
use crate::models::reports::{ItemType, Report};
use ariadne::ids::UserId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct LegacyReport {
    pub id: ReportId,
    pub report_type: String,
    pub item_id: String,
    pub item_type: LegacyItemType,
    pub reporter: UserId,
    pub body: String,
    pub created: DateTime<Utc>,
    pub closed: bool,
    pub thread_id: ThreadId,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub enum LegacyItemType {
    Project,
    Version,
    User,
    Unknown,
}
impl From<ItemType> for LegacyItemType {
    fn from(x: ItemType) -> Self {
        match x {
            ItemType::Project => LegacyItemType::Project,
            ItemType::Version => LegacyItemType::Version,
            ItemType::User => LegacyItemType::User,
            ItemType::Unknown => LegacyItemType::Unknown,
        }
    }
}

impl From<Report> for LegacyReport {
    fn from(x: Report) -> Self {
        LegacyReport {
            id: x.id,
            report_type: x.report_type,
            item_id: x.item_id,
            item_type: x.item_type.into(),
            reporter: x.reporter,
            body: x.body,
            created: x.created,
            closed: x.closed,
            thread_id: x.thread_id,
        }
    }
}

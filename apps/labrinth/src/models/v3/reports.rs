use crate::database::models::report_item::ReportQueryResult as DBReport;
use crate::models::ids::{ProjectId, ReportId, ThreadId, VersionId};
use ariadne::ids::UserId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Report {
    pub id: ReportId,
    pub report_type: String,
    pub item_id: String,
    pub item_type: ItemType,
    pub reporter: UserId,
    pub body: String,
    pub created: DateTime<Utc>,
    pub closed: bool,
    pub thread_id: ThreadId,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub enum ItemType {
    Project,
    Version,
    User,
    Unknown,
}

impl ItemType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ItemType::Project => "project",
            ItemType::Version => "version",
            ItemType::User => "user",
            ItemType::Unknown => "unknown",
        }
    }
}

impl From<DBReport> for Report {
    fn from(x: DBReport) -> Self {
        let mut item_id = "".to_string();
        let mut item_type = ItemType::Unknown;

        if let Some(project_id) = x.project_id {
            item_id = ProjectId::from(project_id).to_string();
            item_type = ItemType::Project;
        } else if let Some(version_id) = x.version_id {
            item_id = VersionId::from(version_id).to_string();
            item_type = ItemType::Version;
        } else if let Some(user_id) = x.user_id {
            item_id = UserId::from(user_id).to_string();
            item_type = ItemType::User;
        }

        Report {
            id: x.id.into(),
            report_type: x.report_type,
            item_id,
            item_type,
            reporter: x.reporter.into(),
            body: x.body,
            created: x.created,
            closed: x.closed,
            thread_id: x.thread_id.into(),
        }
    }
}

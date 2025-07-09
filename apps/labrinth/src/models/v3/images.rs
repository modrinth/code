use super::{
    ids::{ProjectId, ThreadMessageId, VersionId},
    pats::Scopes,
};
use crate::database::models::image_item::DBImage;
use crate::models::ids::{ImageId, ReportId};
use ariadne::ids::UserId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Image {
    pub id: ImageId,
    pub url: String,
    pub size: u64,
    pub created: DateTime<Utc>,
    pub owner_id: UserId,

    // context it is associated with
    #[serde(flatten)]
    pub context: ImageContext,
}

impl From<DBImage> for Image {
    fn from(x: DBImage) -> Self {
        let mut context = ImageContext::from_str(&x.context, None);
        match &mut context {
            ImageContext::Project { project_id } => {
                *project_id = x.project_id.map(|x| x.into());
            }
            ImageContext::Version { version_id } => {
                *version_id = x.version_id.map(|x| x.into());
            }
            ImageContext::ThreadMessage { thread_message_id } => {
                *thread_message_id = x.thread_message_id.map(|x| x.into());
            }
            ImageContext::Report { report_id } => {
                *report_id = x.report_id.map(|x| x.into());
            }
            ImageContext::Unknown => {}
        }

        Image {
            id: x.id.into(),
            url: x.url,
            size: x.size,
            created: x.created,
            owner_id: x.owner_id.into(),
            context,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
#[serde(tag = "context")]
#[serde(rename_all = "snake_case")]
pub enum ImageContext {
    Project {
        project_id: Option<ProjectId>,
    },
    Version {
        // version changelogs
        version_id: Option<VersionId>,
    },
    ThreadMessage {
        thread_message_id: Option<ThreadMessageId>,
    },
    Report {
        report_id: Option<ReportId>,
    },
    Unknown,
}

impl ImageContext {
    pub fn context_as_str(&self) -> &'static str {
        match self {
            ImageContext::Project { .. } => "project",
            ImageContext::Version { .. } => "version",
            ImageContext::ThreadMessage { .. } => "thread_message",
            ImageContext::Report { .. } => "report",
            ImageContext::Unknown => "unknown",
        }
    }
    pub fn inner_id(&self) -> Option<u64> {
        match self {
            ImageContext::Project { project_id } => project_id.map(|x| x.0),
            ImageContext::Version { version_id } => version_id.map(|x| x.0),
            ImageContext::ThreadMessage { thread_message_id } => {
                thread_message_id.map(|x| x.0)
            }
            ImageContext::Report { report_id } => report_id.map(|x| x.0),
            ImageContext::Unknown => None,
        }
    }
    pub fn relevant_scope(&self) -> Scopes {
        match self {
            ImageContext::Project { .. } => Scopes::PROJECT_WRITE,
            ImageContext::Version { .. } => Scopes::VERSION_WRITE,
            ImageContext::ThreadMessage { .. } => Scopes::THREAD_WRITE,
            ImageContext::Report { .. } => Scopes::REPORT_WRITE,
            ImageContext::Unknown => Scopes::NONE,
        }
    }
    pub fn from_str(context: &str, id: Option<u64>) -> Self {
        match context {
            "project" => ImageContext::Project {
                project_id: id.map(ProjectId),
            },
            "version" => ImageContext::Version {
                version_id: id.map(VersionId),
            },
            "thread_message" => ImageContext::ThreadMessage {
                thread_message_id: id.map(ThreadMessageId),
            },
            "report" => ImageContext::Report {
                report_id: id.map(ReportId),
            },
            _ => ImageContext::Unknown,
        }
    }
}

use crate::models::ids::{
    ImageId, ProjectId, ReportId, ThreadId, ThreadMessageId,
};
use crate::models::projects::ProjectStatus;
use crate::models::users::User;
use ariadne::ids::UserId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct LegacyThread {
    pub id: ThreadId,
    #[serde(rename = "type")]
    pub type_: LegacyThreadType,
    pub project_id: Option<ProjectId>,
    pub report_id: Option<ReportId>,
    pub messages: Vec<LegacyThreadMessage>,
    pub members: Vec<User>,
}

#[derive(Serialize, Deserialize)]
pub struct LegacyThreadMessage {
    pub id: ThreadMessageId,
    pub author_id: Option<UserId>,
    pub body: LegacyMessageBody,
    pub created: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum LegacyMessageBody {
    Text {
        body: String,
        #[serde(default)]
        private: bool,
        replying_to: Option<ThreadMessageId>,
        #[serde(default)]
        associated_images: Vec<ImageId>,
    },
    StatusChange {
        new_status: ProjectStatus,
        old_status: ProjectStatus,
    },
    ThreadClosure,
    ThreadReopen,
    Deleted {
        #[serde(default)]
        private: bool,
    },
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Copy, Clone)]
#[serde(rename_all = "snake_case")]
pub enum LegacyThreadType {
    Report,
    Project,
    DirectMessage,
}

impl From<crate::models::v3::threads::ThreadType> for LegacyThreadType {
    fn from(t: crate::models::v3::threads::ThreadType) -> Self {
        match t {
            crate::models::v3::threads::ThreadType::Report => {
                LegacyThreadType::Report
            }
            crate::models::v3::threads::ThreadType::Project => {
                LegacyThreadType::Project
            }
            crate::models::v3::threads::ThreadType::DirectMessage => {
                LegacyThreadType::DirectMessage
            }
        }
    }
}

impl From<crate::models::v3::threads::MessageBody> for LegacyMessageBody {
    fn from(b: crate::models::v3::threads::MessageBody) -> Self {
        match b {
            crate::models::v3::threads::MessageBody::Text {
                body,
                private,
                replying_to,
                associated_images,
            } => LegacyMessageBody::Text {
                body,
                private,
                replying_to,
                associated_images,
            },
            crate::models::v3::threads::MessageBody::StatusChange {
                new_status,
                old_status,
            } => LegacyMessageBody::StatusChange {
                new_status,
                old_status,
            },
            crate::models::v3::threads::MessageBody::ThreadClosure => {
                LegacyMessageBody::ThreadClosure
            }
            crate::models::v3::threads::MessageBody::ThreadReopen => {
                LegacyMessageBody::ThreadReopen
            }
            crate::models::v3::threads::MessageBody::Deleted { private } => {
                LegacyMessageBody::Deleted { private }
            }
        }
    }
}

impl From<crate::models::v3::threads::ThreadMessage> for LegacyThreadMessage {
    fn from(m: crate::models::v3::threads::ThreadMessage) -> Self {
        LegacyThreadMessage {
            id: m.id,
            author_id: m.author_id,
            body: m.body.into(),
            created: m.created,
        }
    }
}

impl From<crate::models::v3::threads::Thread> for LegacyThread {
    fn from(t: crate::models::v3::threads::Thread) -> Self {
        LegacyThread {
            id: t.id,
            type_: t.type_.into(),
            project_id: t.project_id,
            report_id: t.report_id,
            messages: t.messages.into_iter().map(|m| m.into()).collect(),
            members: t.members,
        }
    }
}

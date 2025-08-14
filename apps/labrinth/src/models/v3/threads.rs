use crate::models::ids::{
    ImageId, ProjectId, ReportId, ThreadId, ThreadMessageId,
};
use crate::models::projects::ProjectStatus;
use crate::models::users::User;
use ariadne::ids::UserId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Thread {
    pub id: ThreadId,
    #[serde(rename = "type")]
    pub type_: ThreadType,
    pub project_id: Option<ProjectId>,
    pub report_id: Option<ReportId>,
    pub messages: Vec<ThreadMessage>,
    pub members: Vec<User>,
}

#[derive(Serialize, Deserialize)]
pub struct ThreadMessage {
    pub id: ThreadMessageId,
    pub author_id: Option<UserId>,
    pub body: MessageBody,
    pub created: DateTime<Utc>,
    pub hide_identity: bool,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MessageBody {
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
pub enum ThreadType {
    Report,
    Project,
    DirectMessage,
}

impl std::fmt::Display for ThreadType {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(fmt, "{}", self.as_str())
    }
}

impl ThreadType {
    // These are constant, so this can remove unneccessary allocations (`to_string`)
    pub fn as_str(&self) -> &'static str {
        match self {
            ThreadType::Report => "report",
            ThreadType::Project => "project",
            ThreadType::DirectMessage => "direct_message",
        }
    }

    pub fn from_string(string: &str) -> ThreadType {
        match string {
            "report" => ThreadType::Report,
            "project" => ThreadType::Project,
            "direct_message" => ThreadType::DirectMessage,
            _ => ThreadType::DirectMessage,
        }
    }
}

impl Thread {
    pub fn from(
        data: crate::database::models::DBThread,
        users: Vec<User>,
        user: &User,
    ) -> Self {
        let thread_type = data.type_;

        Thread {
            id: data.id.into(),
            type_: thread_type,
            project_id: data.project_id.map(|x| x.into()),
            report_id: data.report_id.map(|x| x.into()),
            messages: data
                .messages
                .into_iter()
                .filter(|x| {
                    if let MessageBody::Text { private, .. } = x.body {
                        !private || user.role.is_mod()
                    } else if let MessageBody::Deleted { private, .. } = x.body
                    {
                        !private || user.role.is_mod()
                    } else {
                        true
                    }
                })
                .map(|x| ThreadMessage::from(x, user))
                .collect(),
            members: users,
        }
    }
}

impl ThreadMessage {
    pub fn from(
        data: crate::database::models::DBThreadMessage,
        user: &User,
    ) -> Self {
        Self {
            id: data.id.into(),
            author_id: if data.hide_identity && !user.role.is_mod() {
                None
            } else {
                data.author_id.map(|x| x.into())
            },
            body: data.body,
            created: data.created,
            hide_identity: data.hide_identity,
        }
    }
}

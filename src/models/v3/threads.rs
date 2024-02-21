use super::ids::{Base62Id, ImageId};
use crate::models::ids::{ProjectId, ReportId};
use crate::models::projects::ProjectStatus;
use crate::models::users::{User, UserId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "Base62Id")]
#[serde(into = "Base62Id")]
pub struct ThreadId(pub u64);

#[derive(Copy, Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
#[serde(from = "Base62Id")]
#[serde(into = "Base62Id")]
pub struct ThreadMessageId(pub u64);

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
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MessageBody {
    Text {
        body: String,
        #[serde(default)]
        private: bool,
        #[serde(default)]
        hide_identity: bool,
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
    pub fn from(data: crate::database::models::Thread, users: Vec<User>, user: &User) -> Self {
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
                    } else if let MessageBody::Deleted { private, .. } = x.body {
                        !private || user.role.is_mod()
                    } else {
                        true
                    }
                })
                .map(|x| ThreadMessage {
                    id: x.id.into(),
                    author_id: if users
                        .iter()
                        .find(|y| x.author_id == Some(y.id.into()))
                        .map(|x| x.role.is_mod() && !user.role.is_mod())
                        .unwrap_or(false)
                    {
                        None
                    } else {
                        x.author_id.map(|x| x.into())
                    },
                    body: x.body,
                    created: x.created,
                })
                .collect(),
            members: users
                .into_iter()
                .filter(|x| !x.role.is_mod() || user.role.is_mod())
                .collect(),
        }
    }
}

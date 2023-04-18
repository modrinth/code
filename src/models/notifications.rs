use super::ids::Base62Id;
use super::users::UserId;
use crate::database::models::notification_item::Notification as DBNotification;
use crate::database::models::notification_item::NotificationAction as DBNotificationAction;
use crate::models::ids::{
    ProjectId, ReportId, TeamId, ThreadId, ThreadMessageId, VersionId,
};
use crate::models::projects::ProjectStatus;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "Base62Id")]
#[serde(into = "Base62Id")]
pub struct NotificationId(pub u64);

#[derive(Serialize, Deserialize)]
pub struct Notification {
    pub id: NotificationId,
    pub user_id: UserId,
    pub read: bool,
    pub created: DateTime<Utc>,
    pub body: NotificationBody,

    // DEPRECATED: use body field instead
    #[serde(rename = "type")]
    pub type_: Option<String>,
    pub title: String,
    pub text: String,
    pub link: String,
    pub actions: Vec<NotificationAction>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum NotificationBody {
    ProjectUpdate {
        project_id: ProjectId,
        version_id: VersionId,
    },
    TeamInvite {
        project_id: ProjectId,
        team_id: TeamId,
        invited_by: UserId,
        role: String,
    },
    StatusChange {
        project_id: ProjectId,
        old_status: ProjectStatus,
        new_status: ProjectStatus,
    },
    ModeratorMessage {
        thread_id: ThreadId,
        message_id: ThreadMessageId,

        project_id: Option<ProjectId>,
        report_id: Option<ReportId>,
    },
    LegacyMarkdown {
        notification_type: Option<String>,
        title: String,
        text: String,
        link: String,
        actions: Vec<NotificationAction>,
    },
    Unknown,
}

impl From<DBNotification> for Notification {
    fn from(notif: DBNotification) -> Self {
        let (type_, title, text, link, actions) = {
            match &notif.body {
                NotificationBody::ProjectUpdate {
                    project_id,
                    version_id,
                } => (
                    Some("project_update".to_string()),
                    "A project you follow has been updated!".to_string(),
                    format!(
                        "The project {} has released a new version: {}",
                        project_id, version_id
                    ),
                    format!("/project/{}/version/{}", project_id, version_id),
                    vec![],
                ),
                NotificationBody::TeamInvite {
                    project_id,
                    role,
                    team_id,
                    ..
                } => (
                    Some("team_invite".to_string()),
                    "You have been invited to join a team!".to_string(),
                    format!(
                        "An invite has been sent for you to be {} of a team",
                        role
                    ),
                    format!("/project/{}", project_id),
                    vec![
                        NotificationAction {
                            title: "Accept".to_string(),
                            action_route: (
                                "POST".to_string(),
                                format!("team/{team_id}/join"),
                            ),
                        },
                        NotificationAction {
                            title: "Deny".to_string(),
                            action_route: (
                                "DELETE".to_string(),
                                format!(
                                    "team/{team_id}/members/{}",
                                    UserId::from(notif.user_id)
                                ),
                            ),
                        },
                    ],
                ),
                NotificationBody::StatusChange {
                    old_status,
                    new_status,
                    project_id,
                } => (
                    Some("status_change".to_string()),
                    "Project status has changed".to_string(),
                    format!(
                        "Status has changed from {} to {}",
                        old_status.as_friendly_str(),
                        new_status.as_friendly_str()
                    ),
                    format!("/project/{}", project_id),
                    vec![],
                ),
                NotificationBody::ModeratorMessage {
                    project_id,
                    report_id,
                    ..
                } => (
                    Some("moderator_message".to_string()),
                    "A moderator has sent you a message!".to_string(),
                    "Click on the link to read more.".to_string(),
                    if let Some(project_id) = project_id {
                        format!("/project/{}", project_id)
                    } else if let Some(report_id) = report_id {
                        format!("/project/{}", report_id)
                    } else {
                        "#".to_string()
                    },
                    vec![],
                ),
                NotificationBody::LegacyMarkdown {
                    notification_type,
                    title,
                    text,
                    link,
                    actions,
                } => (
                    notification_type.clone(),
                    title.clone(),
                    text.clone(),
                    link.clone(),
                    actions.clone().into_iter().map(Into::into).collect(),
                ),
                NotificationBody::Unknown => (
                    None,
                    "".to_string(),
                    "".to_string(),
                    "#".to_string(),
                    vec![],
                ),
            }
        };

        Self {
            id: notif.id.into(),
            user_id: notif.user_id.into(),
            body: notif.body,
            read: notif.read,
            created: notif.created,

            // DEPRECATED
            type_,
            title,
            text,
            link,
            actions,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct NotificationAction {
    pub title: String,
    /// The route to call when this notification action is called. Formatted HTTP Method, route
    pub action_route: (String, String),
}

impl From<DBNotificationAction> for NotificationAction {
    fn from(act: DBNotificationAction) -> Self {
        Self {
            title: act.title,
            action_route: (act.action_route_method, act.action_route),
        }
    }
}

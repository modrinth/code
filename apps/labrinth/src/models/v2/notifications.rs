use crate::models::ids::{ThreadMessageId, VersionId};
use crate::models::{
    ids::{
        NotificationId, OrganizationId, ProjectId, ReportId, TeamId, ThreadId,
    },
    notifications::{Notification, NotificationAction, NotificationBody},
    projects::ProjectStatus,
};
use ariadne::ids::UserId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct LegacyNotification {
    pub id: NotificationId,
    pub user_id: UserId,
    pub read: bool,
    pub created: DateTime<Utc>,
    pub body: LegacyNotificationBody,

    // DEPRECATED: use body field instead
    #[serde(rename = "type")]
    pub type_: Option<String>,
    pub title: String,
    pub text: String,
    pub link: String,
    pub actions: Vec<LegacyNotificationAction>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct LegacyNotificationAction {
    pub title: String,
    /// The route to call when this notification action is called. Formatted HTTP Method, route
    pub action_route: (String, String),
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum LegacyNotificationBody {
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
    OrganizationInvite {
        organization_id: OrganizationId,
        invited_by: UserId,
        team_id: TeamId,
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

impl LegacyNotification {
    pub fn from(notification: Notification) -> Self {
        let type_ = match &notification.body {
            NotificationBody::ProjectUpdate { .. } => {
                Some("project_update".to_string())
            }
            NotificationBody::TeamInvite { .. } => {
                Some("team_invite".to_string())
            }
            NotificationBody::OrganizationInvite { .. } => {
                Some("organization_invite".to_string())
            }
            NotificationBody::StatusChange { .. } => {
                Some("status_change".to_string())
            }
            NotificationBody::ModeratorMessage { .. } => {
                Some("moderator_message".to_string())
            }
            NotificationBody::LegacyMarkdown {
                notification_type, ..
            } => notification_type.clone(),
            NotificationBody::Unknown => None,
        };

        let legacy_body = match notification.body {
            NotificationBody::ProjectUpdate {
                project_id,
                version_id,
            } => LegacyNotificationBody::ProjectUpdate {
                project_id,
                version_id,
            },
            NotificationBody::TeamInvite {
                project_id,
                team_id,
                invited_by,
                role,
            } => LegacyNotificationBody::TeamInvite {
                project_id,
                team_id,
                invited_by,
                role,
            },
            NotificationBody::OrganizationInvite {
                organization_id,
                invited_by,
                team_id,
                role,
            } => LegacyNotificationBody::OrganizationInvite {
                organization_id,
                invited_by,
                team_id,
                role,
            },
            NotificationBody::StatusChange {
                project_id,
                old_status,
                new_status,
            } => LegacyNotificationBody::StatusChange {
                project_id,
                old_status,
                new_status,
            },
            NotificationBody::ModeratorMessage {
                thread_id,
                message_id,
                project_id,
                report_id,
            } => LegacyNotificationBody::ModeratorMessage {
                thread_id,
                message_id,
                project_id,
                report_id,
            },
            NotificationBody::LegacyMarkdown {
                notification_type,
                name,
                text,
                link,
                actions,
            } => LegacyNotificationBody::LegacyMarkdown {
                notification_type,
                title: name,
                text,
                link,
                actions,
            },
            NotificationBody::Unknown => LegacyNotificationBody::Unknown,
        };

        Self {
            id: notification.id,
            user_id: notification.user_id,
            read: notification.read,
            created: notification.created,
            body: legacy_body,
            type_,
            title: notification.name,
            text: notification.text,
            link: notification.link,
            actions: notification
                .actions
                .into_iter()
                .map(LegacyNotificationAction::from)
                .collect(),
        }
    }
}

impl LegacyNotificationAction {
    pub fn from(notification_action: NotificationAction) -> Self {
        Self {
            title: notification_action.name,
            action_route: notification_action.action_route,
        }
    }
}

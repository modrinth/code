use super::ids::*;
use crate::database::models::notification_item::DBNotification;
use crate::database::models::notification_item::DBNotificationAction;
use crate::database::models::notifications_deliveries_item::DBNotificationDelivery;
use crate::models::ids::{
    NotificationId, ProjectId, ReportId, TeamId, ThreadId, ThreadMessageId,
    VersionId,
};
use crate::models::projects::ProjectStatus;
use ariadne::ids::UserId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Notification {
    pub id: NotificationId,
    pub user_id: UserId,
    pub read: bool,
    pub created: DateTime<Utc>,
    pub body: NotificationBody,

    pub name: String,
    pub text: String,
    pub link: String,
    pub actions: Vec<NotificationAction>,
}

#[derive(Serialize, Deserialize, Clone, Copy, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum NotificationType {
    // If adding a notification type, add a variant in `NotificationBody` of the same name!
    ProjectUpdate,
    TeamInvite,
    OrganizationInvite,
    StatusChange,
    ModeratorMessage,
    LegacyMarkdown,
    ResetPassword,
    Unknown,
}

impl NotificationType {
    pub fn as_str(self) -> &'static str {
        match self {
            NotificationType::ProjectUpdate => "project_update",
            NotificationType::TeamInvite => "team_invite",
            NotificationType::OrganizationInvite => "organization_invite",
            NotificationType::StatusChange => "status_change",
            NotificationType::ModeratorMessage => "moderator_message",
            NotificationType::LegacyMarkdown => "legacy_markdown",
            NotificationType::ResetPassword => "reset_password",
            NotificationType::Unknown => "unknown",
        }
    }

    pub fn from_str_or_default(s: &str) -> Self {
        match s {
            "project_update" => NotificationType::ProjectUpdate,
            "team_invite" => NotificationType::TeamInvite,
            "organization_invite" => NotificationType::OrganizationInvite,
            "status_change" => NotificationType::StatusChange,
            "moderator_message" => NotificationType::ModeratorMessage,
            "legacy_markdown" => NotificationType::LegacyMarkdown,
            "reset_password" => NotificationType::ResetPassword,
            "unknown" => NotificationType::Unknown,
            _ => NotificationType::Unknown,
        }
    }
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
        name: String,
        text: String,
        link: String,
        actions: Vec<NotificationAction>,
    },
    ResetPassword {
        flow: String,
    },
    Unknown,
}

impl NotificationBody {
    pub fn notification_type(&self) -> NotificationType {
        match &self {
            NotificationBody::ProjectUpdate { .. } => {
                NotificationType::ProjectUpdate
            }
            NotificationBody::TeamInvite { .. } => NotificationType::TeamInvite,
            NotificationBody::OrganizationInvite { .. } => {
                NotificationType::OrganizationInvite
            }
            NotificationBody::StatusChange { .. } => {
                NotificationType::StatusChange
            }
            NotificationBody::ModeratorMessage { .. } => {
                NotificationType::ModeratorMessage
            }
            NotificationBody::LegacyMarkdown { .. } => {
                NotificationType::LegacyMarkdown
            }
            NotificationBody::ResetPassword { .. } => {
                NotificationType::ResetPassword
            }
            NotificationBody::Unknown => NotificationType::Unknown,
        }
    }
}

impl From<DBNotification> for Notification {
    fn from(notif: DBNotification) -> Self {
        let (name, text, link, actions) = {
            match &notif.body {
                NotificationBody::ProjectUpdate {
                    project_id,
                    version_id,
                } => (
                    "A project you follow has been updated!".to_string(),
                    format!(
                        "The project {project_id} has released a new version: {version_id}"
                    ),
                    format!("/project/{project_id}/version/{version_id}"),
                    vec![],
                ),
                NotificationBody::TeamInvite {
                    project_id,
                    role,
                    team_id,
                    ..
                } => (
                    "You have been invited to join a team!".to_string(),
                    format!(
                        "An invite has been sent for you to be {role} of a team"
                    ),
                    format!("/project/{project_id}"),
                    vec![
                        NotificationAction {
                            name: "Accept".to_string(),
                            action_route: (
                                "POST".to_string(),
                                format!("team/{team_id}/join"),
                            ),
                        },
                        NotificationAction {
                            name: "Deny".to_string(),
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
                NotificationBody::OrganizationInvite {
                    organization_id,
                    role,
                    team_id,
                    ..
                } => (
                    "You have been invited to join an organization!"
                        .to_string(),
                    format!(
                        "An invite has been sent for you to be {role} of an organization"
                    ),
                    format!("/organization/{organization_id}"),
                    vec![
                        NotificationAction {
                            name: "Accept".to_string(),
                            action_route: (
                                "POST".to_string(),
                                format!("team/{team_id}/join"),
                            ),
                        },
                        NotificationAction {
                            name: "Deny".to_string(),
                            action_route: (
                                "DELETE".to_string(),
                                format!(
                                    "organization/{organization_id}/members/{}",
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
                    "Project status has changed".to_string(),
                    format!(
                        "Status has changed from {} to {}",
                        old_status.as_friendly_str(),
                        new_status.as_friendly_str()
                    ),
                    format!("/project/{project_id}"),
                    vec![],
                ),
                NotificationBody::ModeratorMessage {
                    project_id,
                    report_id,
                    ..
                } => (
                    "A moderator has sent you a message!".to_string(),
                    "Click on the link to read more.".to_string(),
                    if let Some(project_id) = project_id {
                        format!("/project/{project_id}")
                    } else if let Some(report_id) = report_id {
                        format!("/project/{report_id}")
                    } else {
                        "#".to_string()
                    },
                    vec![],
                ),
                // Don't expose the `flow` field
                NotificationBody::ResetPassword { .. } => (
                    "Password reset requested".to_string(),
                    "You've requested to reset your password. Please check your email for a reset link.".to_string(),
                    "#".to_string(),
                    vec![],
                ),
                NotificationBody::LegacyMarkdown {
                    name,
                    text,
                    link,
                    actions,
                    ..
                } => (
                    name.clone(),
                    text.clone(),
                    link.clone(),
                    actions.clone().into_iter().collect(),
                ),
                NotificationBody::Unknown => {
                    ("".to_string(), "".to_string(), "#".to_string(), vec![])
                }
            }
        };

        Self {
            id: notif.id.into(),
            user_id: notif.user_id.into(),
            body: notif.body,
            read: notif.read,
            created: notif.created,

            name,
            text,
            link,
            actions,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct NotificationAction {
    pub name: String,
    /// The route to call when this notification action is called. Formatted HTTP Method, route
    pub action_route: (String, String),
}

impl From<DBNotificationAction> for NotificationAction {
    fn from(act: DBNotificationAction) -> Self {
        Self {
            name: act.name,
            action_route: (act.action_route_method, act.action_route),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotificationChannel {
    Email,
}

impl NotificationChannel {
    pub fn as_str(self) -> &'static str {
        match self {
            NotificationChannel::Email => "email",
        }
    }

    pub fn from_str_or_default(s: &str) -> Self {
        match s {
            "email" => NotificationChannel::Email,
            _ => NotificationChannel::Email,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotificationDeliveryStatus {
    Pending,
    SkippedPreferences,
    SkippedDefault,
    Delivered,
    PermanentlyFailed,
}

impl NotificationDeliveryStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            NotificationDeliveryStatus::Pending => "pending",
            NotificationDeliveryStatus::SkippedPreferences => {
                "skipped_preferences"
            }
            NotificationDeliveryStatus::SkippedDefault => "skipped_default",
            NotificationDeliveryStatus::Delivered => "delivered",
            NotificationDeliveryStatus::PermanentlyFailed => {
                "permanently_failed"
            }
        }
    }

    pub fn from_str_or_default(s: &str) -> Self {
        match s {
            "pending" => NotificationDeliveryStatus::Pending,
            "skipped_preferences" => {
                NotificationDeliveryStatus::SkippedPreferences
            }
            "skipped_default" => NotificationDeliveryStatus::SkippedDefault,
            "delivered" => NotificationDeliveryStatus::Delivered,
            "permanently_failed" => {
                NotificationDeliveryStatus::PermanentlyFailed
            }
            _ => NotificationDeliveryStatus::Pending,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct NotificationDelivery {
    pub notification_id: NotificationId,
    pub user_id: UserId,
    pub channel: NotificationChannel,
    pub delivery_priority: i32,
    pub status: NotificationDeliveryStatus,
    pub next_attempt: DateTime<Utc>,
    pub attempt_count: i32,
}

impl From<DBNotificationDelivery> for NotificationDelivery {
    fn from(delivery: DBNotificationDelivery) -> Self {
        Self {
            notification_id: delivery.notification_id.into(),
            user_id: delivery.user_id.into(),
            channel: delivery.channel,
            delivery_priority: delivery.delivery_priority,
            status: delivery.status,
            next_attempt: delivery.next_attempt,
            attempt_count: delivery.attempt_count,
        }
    }
}

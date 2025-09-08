use super::ids::OrganizationId;
use crate::database::models::notification_item::DBNotification;
use crate::database::models::notification_item::DBNotificationAction;
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
    VerifyEmail {
        flow: String,
    },
    AuthProviderAdded {
        provider: String,
    },
    AuthProviderRemoved {
        provider: String,
    },
    TwoFactorEnabled,
    TwoFactorRemoved,
    PasswordChanged,
    PasswordRemoved,
    EmailChanged {
        new_email: String,
        to_email: String,
    },
    PaymentFailed {
        amount: String,
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
            NotificationBody::VerifyEmail { .. } => {
                NotificationType::VerifyEmail
            }
            NotificationBody::AuthProviderAdded { .. } => {
                NotificationType::AuthProviderAdded
            }
            NotificationBody::AuthProviderRemoved { .. } => {
                NotificationType::AuthProviderRemoved
            }
            NotificationBody::TwoFactorEnabled => {
                NotificationType::TwoFactorEnabled
            }
            NotificationBody::TwoFactorRemoved => {
                NotificationType::TwoFactorRemoved
            }
            NotificationBody::PasswordChanged => {
                NotificationType::PasswordChanged
            }
            NotificationBody::PasswordRemoved => {
                NotificationType::PasswordRemoved
            }
            NotificationBody::EmailChanged { .. } => {
                NotificationType::EmailChanged
            }
            NotificationBody::PaymentFailed { .. } => {
                NotificationType::PaymentFailed
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
                // The notifications from here to down below are listed with messages for completeness' sake,
                // though they should never be sent via site notifications. This should be disabled via database
                // options. Messages should be reviewed and worded better if we want to distribute these notifications
                // via the site.
                NotificationBody::PaymentFailed { .. } => (
                    "Payment failed".to_string(),
                    "A payment on your account failed. Please update your billing information.".to_string(),
                    "/settings/billing".to_string(),
                    vec![],
                ),
                NotificationBody::VerifyEmail { .. } => (
                    "Verify your email".to_string(),
                    "You've requested to verify your email. Please check your email for a verification link.".to_string(),
                    "#".to_string(),
                    vec![],
                ),
                NotificationBody::AuthProviderAdded { .. } => (
                    "Auth provider added".to_string(),
                    "You've added a new authentication provider to your account.".to_string(),
                    "#".to_string(),
                    vec![],
                ),
                NotificationBody::AuthProviderRemoved { .. } => (
                    "Auth provider removed".to_string(),
                    "You've removed a authentication provider from your account.".to_string(),
                    "#".to_string(),
                    vec![],
                ),
                NotificationBody::TwoFactorEnabled => (
                    "Two-factor authentication enabled".to_string(),
                    "You've enabled two-factor authentication on your account.".to_string(),
                    "#".to_string(),
                    vec![],
                ),
                NotificationBody::TwoFactorRemoved => (
                    "Two-factor authentication removed".to_string(),
                    "You've removed two-factor authentication from your account.".to_string(),
                    "#".to_string(),
                    vec![],
                ),
                NotificationBody::PasswordChanged => (
                    "Password changed".to_string(),
                    "You've changed your account password.".to_string(),
                    "#".to_string(),
                    vec![],
                ),
                NotificationBody::PasswordRemoved => (
                    "Password removed".to_string(),
                    "You've removed your account password.".to_string(),
                    "#".to_string(),
                    vec![],
                ),
                NotificationBody::EmailChanged { .. } => (
                    "Email changed".to_string(),
                    "Your account email was changed.".to_string(),
                    "#".to_string(),
                    vec![],
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

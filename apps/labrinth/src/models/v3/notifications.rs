use super::ids::*;
use crate::database::models::notification_item::DBNotification;
use crate::database::models::notification_item::DBNotificationAction;
use crate::database::models::notifications_deliveries_item::DBNotificationDelivery;
use crate::models::ids::{
    NotificationId, ProjectId, ReportId, TeamId, ThreadId, ThreadMessageId,
    VersionId,
};
use crate::models::projects::ProjectStatus;
use crate::routes::error::ApiError;
use ariadne::i18n::{I18nEnum, TranslationData};
use ariadne::i18n_enum;
use ariadne::ids::UserId;
use chrono::{DateTime, Utc};
use derive_more::Display;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Notification {
    pub id: NotificationId,
    pub user_id: UserId,
    pub read: bool,
    pub created: DateTime<Utc>,
    pub body: NotificationBody,

    pub name: String,
    pub translatable_name: TranslationData,
    pub text: String,
    pub translatable_text: TranslationData,
    pub link: String,
    pub actions: Vec<NotificationAction>,
}

#[derive(Serialize, Deserialize, Clone, Copy, Eq, PartialEq, Display)]
#[serde(rename_all = "snake_case")]
pub enum NotificationType {
    // If adding a notification type, add a variant in `NotificationBody` of the same name!
    #[display("A project you follow has been updated!")]
    ProjectUpdate,
    #[display("You have been invited to join a team!")]
    TeamInvite,
    #[display("You have been invited to join an organization!")]
    OrganizationInvite,
    #[display("Project status has changed")]
    StatusChange,
    #[display("A moderator has sent you a message!")]
    ModeratorMessage,
    // The notifications from here to down below are listed with messages for completeness' sake,
    // though they should never be sent via site notifications. This should be disabled via database
    // options. Messages should be reviewed and worded better if we want to distribute these notifications
    // via the site.
    #[display("LEGACY MARKDOWN NOTIFICATION")]
    LegacyMarkdown,
    #[display("Password reset requested")]
    ResetPassword,
    #[display("Verify your email")]
    VerifyEmail,
    #[display("Auth provider added")]
    AuthProviderAdded,
    #[display("Auth provider removed")]
    AuthProviderRemoved,
    #[display("Two-factor authentication enabled")]
    TwoFactorEnabled,
    #[display("Two-factor authentication removed")]
    TwoFactorRemoved,
    #[display("Password changed")]
    PasswordChanged,
    #[display("Password removed")]
    PasswordRemoved,
    #[display("Email changed")]
    EmailChanged,
    #[display("Payment failed")]
    PaymentFailed,
    #[display("New personal access token created")]
    PatCreated,
    #[display("New message in moderation thread")]
    ModerationMessageReceived,
    #[display("Report status updated")]
    ReportStatusUpdated,
    #[display("Report submitted")]
    ReportSubmitted,
    #[display("Project approved")]
    ProjectStatusApproved,
    #[display("Project status updated")]
    ProjectStatusNeutral,
    #[display("Project ownership transferred")]
    ProjectTransferred,
    #[display("Payout available")]
    PayoutAvailable,
    #[display("")]
    Unknown,
}

i18n_enum!(
    NotificationType,
    root_key: "labrinth.notification.type",
    ProjectUpdate! => "project_update",
    TeamInvite! => "team_invite",
    OrganizationInvite! => "organization_invite",
    StatusChange! => "status_change",
    ModeratorMessage! => "moderator_message",
    LegacyMarkdown! => "legacy_markdown",
    ResetPassword! => "reset_password",
    VerifyEmail! => "verify_email",
    AuthProviderAdded! => "auth_provider_added",
    AuthProviderRemoved! => "auth_provider_removed",
    TwoFactorEnabled! => "two_factor_enabled",
    TwoFactorRemoved! => "two_factor_removed",
    PasswordChanged! => "password_changed",
    PasswordRemoved! => "password_removed",
    EmailChanged! => "email_changed",
    PaymentFailed! => "payment_failed",
    PatCreated! => "pat_created",
    ModerationMessageReceived! => "moderation_message_received",
    ReportStatusUpdated! => "report_status_updated",
    ReportSubmitted! => "report_submitted",
    ProjectStatusApproved! => "project_status_approved",
    ProjectStatusNeutral! => "project_status_neutral",
    ProjectTransferred! => "project_transferred",
    PayoutAvailable! => "payout_available",
    Unknown! => "unknown",
);

impl NotificationType {
    pub fn as_str(self) -> &'static str {
        self.translation_id()
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
            "verify_email" => NotificationType::VerifyEmail,
            "auth_provider_added" => NotificationType::AuthProviderAdded,
            "auth_provider_removed" => NotificationType::AuthProviderRemoved,
            "two_factor_enabled" => NotificationType::TwoFactorEnabled,
            "two_factor_removed" => NotificationType::TwoFactorRemoved,
            "password_changed" => NotificationType::PasswordChanged,
            "password_removed" => NotificationType::PasswordRemoved,
            "email_changed" => NotificationType::EmailChanged,
            "payment_failed" => NotificationType::PaymentFailed,
            "pat_created" => NotificationType::PatCreated,
            "moderation_message_received" => {
                NotificationType::ModerationMessageReceived
            }
            "report_status_updated" => NotificationType::ReportStatusUpdated,
            "report_submitted" => NotificationType::ReportSubmitted,
            "project_status_approved" => {
                NotificationType::ProjectStatusApproved
            }
            "project_status_neutral" => NotificationType::ProjectStatusNeutral,
            "project_transferred" => NotificationType::ProjectTransferred,
            "payout_available" => NotificationType::PayoutAvailable,
            "unknown" => NotificationType::Unknown,
            _ => NotificationType::Unknown,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Display)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum NotificationBody {
    #[display(
        "The project {project_id} has released a new version: {version_id}"
    )]
    ProjectUpdate {
        project_id: ProjectId,
        version_id: VersionId,
    },
    #[display("An invite has been sent for you to be {role} of a team")]
    TeamInvite {
        project_id: ProjectId,
        team_id: TeamId,
        invited_by: UserId,
        role: String,
    },
    #[display(
        "An invite has been sent for you to be {role} of an organization"
    )]
    OrganizationInvite {
        organization_id: OrganizationId,
        invited_by: UserId,
        team_id: TeamId,
        role: String,
    },
    #[display("Status has changed from {old_status} to {new_status}")]
    StatusChange {
        project_id: ProjectId,
        old_status: ProjectStatus,
        new_status: ProjectStatus,
    },
    /// This is for website notifications only. Email notifications have `ModerationMessageReceived`.
    #[display("Click on the link to read more.")]
    ModeratorMessage {
        thread_id: ThreadId,
        message_id: ThreadMessageId,

        project_id: Option<ProjectId>,
        report_id: Option<ReportId>,
    },
    #[display("Your personal access token '{token_name}' was created.")]
    PatCreated { token_name: String },
    /// This differs from ModeratorMessage as this notification is only for project threads and
    /// email notifications, not for site notifications.
    #[display("You have a new message in a moderation thread.")]
    ModerationMessageReceived { project_id: ProjectId },
    #[display("A report you are involved in has been updated.")]
    ReportStatusUpdated { report_id: ReportId },
    #[display("Your report was submitted successfully.")]
    ReportSubmitted { report_id: ReportId },
    #[display("Your project has been approved.")]
    ProjectStatusApproved { project_id: ProjectId },
    #[display("Your project status has been updated.")]
    ProjectStatusNeutral {
        project_id: ProjectId,
        old_status: ProjectStatus,
        new_status: ProjectStatus,
    },
    #[display("A project's ownership has been transferred.")]
    ProjectTransferred {
        project_id: ProjectId,
        new_owner_user_id: Option<UserId>,
        new_owner_organization_id: Option<OrganizationId>,
    },
    #[display("{text}")]
    LegacyMarkdown {
        notification_type: Option<String>,
        name: String,
        text: String,
        link: String,
        actions: Vec<NotificationAction>,
    },
    #[display(
        "You've requested to reset your password. Please check your email for a reset link."
    )]
    ResetPassword { flow: String },
    // See comment above VerifyEmail in NotificationType
    #[display(
        "You've requested to verify your email. Please check your email for a verification link."
    )]
    VerifyEmail { flow: String },
    #[display("You've added a new authentication provider to your account.")]
    AuthProviderAdded { provider: String },
    #[display("You've removed a authentication provider from your account.")]
    AuthProviderRemoved { provider: String },
    #[display("You've enabled two-factor authentication on your account.")]
    TwoFactorEnabled,
    #[display("You've removed two-factor authentication from your account.")]
    TwoFactorRemoved,
    #[display("You've changed your account password.")]
    PasswordChanged,
    #[display("You've removed your account password.")]
    PasswordRemoved,
    #[display("Your account email was changed.")]
    EmailChanged { new_email: String, to_email: String },
    #[display(
        "A payment on your account failed. Please update your billing information."
    )]
    PaymentFailed { amount: String, service: String },
    #[display("A payout is available!")]
    PayoutAvailable {
        date_available: DateTime<Utc>,
        amount: f64,
    },
    #[display("")]
    Unknown,
}

i18n_enum!(
    NotificationBody,
    root_key: "labrinth.notification.body",
    ProjectUpdate { project_id, version_id } => "project_update",
    TeamInvite { role, .. } => "team_invite",
    OrganizationInvite { role, .. } => "organization_invite",
    StatusChange { old_status, new_status, .. } => "status_change",
    ModeratorMessage { .. } => "moderator_message",
    PatCreated { token_name } => "pat_created",
    ModerationMessageReceived { .. } => "moderation_message_received",
    ReportStatusUpdated { .. } => "report_status_updated",
    ReportSubmitted { .. } => "report_submitted",
    ProjectStatusApproved { .. } => "project_status_approved",
    ProjectStatusNeutral { .. } => "project_status_neutral",
    ProjectTransferred { .. } => "project_transferred",
    LegacyMarkdown { transparent text } => "legacy_markdown",
    ResetPassword { .. } => "reset_password",
    VerifyEmail { .. } => "verify_email",
    AuthProviderAdded { .. } => "auth_provider_added",
    AuthProviderRemoved { .. } => "auth_provider_removed",
    TwoFactorEnabled! => "two_factor_enabled",
    TwoFactorRemoved! => "two_factor_removed",
    PasswordChanged! => "password_changed",
    PasswordRemoved! => "password_removed",
    EmailChanged { .. } => "email_changed",
    PaymentFailed { .. } => "payment_failed",
    PayoutAvailable { .. } => "payout_available",
    Unknown! => "unknown",
);

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
            NotificationBody::PatCreated { .. } => NotificationType::PatCreated,
            NotificationBody::ModerationMessageReceived { .. } => {
                NotificationType::ModerationMessageReceived
            }
            NotificationBody::ReportStatusUpdated { .. } => {
                NotificationType::ReportStatusUpdated
            }
            NotificationBody::ReportSubmitted { .. } => {
                NotificationType::ReportSubmitted
            }
            NotificationBody::ProjectStatusApproved { .. } => {
                NotificationType::ProjectStatusApproved
            }
            NotificationBody::ProjectStatusNeutral { .. } => {
                NotificationType::ProjectStatusNeutral
            }
            NotificationBody::ProjectTransferred { .. } => {
                NotificationType::ProjectTransferred
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
            NotificationBody::PayoutAvailable { .. } => {
                NotificationType::PayoutAvailable
            }
            NotificationBody::Unknown => NotificationType::Unknown,
        }
    }
}

impl From<DBNotification> for Notification {
    fn from(notif: DBNotification) -> Self {
        let (link, actions) = {
            match &notif.body {
                NotificationBody::ProjectUpdate {
                    project_id,
                    version_id,
                } => (
                    format!("/project/{project_id}/version/{version_id}"),
                    vec![],
                ),
                NotificationBody::TeamInvite {
                    project_id,
                    team_id,
                    ..
                } => (
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
                    team_id,
                    ..
                } => (
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
                NotificationBody::StatusChange { project_id, .. } => {
                    (format!("/project/{project_id}"), vec![])
                }
                NotificationBody::ModeratorMessage {
                    project_id,
                    report_id,
                    ..
                } => (
                    if let Some(project_id) = project_id {
                        format!("/project/{project_id}")
                    } else if let Some(report_id) = report_id {
                        format!("/project/{report_id}")
                    } else {
                        "#".to_string()
                    },
                    vec![],
                ),
                NotificationBody::PatCreated { .. } => {
                    ("#".to_string(), vec![])
                }
                NotificationBody::ReportStatusUpdated { .. } => {
                    ("#".to_string(), vec![])
                }
                NotificationBody::ReportSubmitted { .. } => {
                    ("#".to_string(), vec![])
                }
                NotificationBody::ProjectStatusApproved { .. } => {
                    ("#".to_string(), vec![])
                }
                NotificationBody::ProjectStatusNeutral { .. } => {
                    ("#".to_string(), vec![])
                }
                NotificationBody::ProjectTransferred { .. } => {
                    ("#".to_string(), vec![])
                }
                // Don't expose the `flow` field
                NotificationBody::ResetPassword { .. } => {
                    ("#".to_string(), vec![])
                }
                NotificationBody::LegacyMarkdown { link, actions, .. } => {
                    (link.clone(), actions.clone().into_iter().collect())
                }
                NotificationBody::PaymentFailed { .. } => {
                    ("/settings/billing".to_string(), vec![])
                }
                NotificationBody::VerifyEmail { .. } => {
                    ("#".to_string(), vec![])
                }
                NotificationBody::AuthProviderAdded { .. } => {
                    ("#".to_string(), vec![])
                }
                NotificationBody::AuthProviderRemoved { .. } => {
                    ("#".to_string(), vec![])
                }
                NotificationBody::TwoFactorEnabled => ("#".to_string(), vec![]),
                NotificationBody::TwoFactorRemoved => ("#".to_string(), vec![]),
                NotificationBody::PasswordChanged => ("#".to_string(), vec![]),
                NotificationBody::PasswordRemoved => ("#".to_string(), vec![]),
                NotificationBody::EmailChanged { .. } => {
                    ("#".to_string(), vec![])
                }
                NotificationBody::PayoutAvailable { .. } => {
                    ("#".to_string(), vec![])
                }
                NotificationBody::ModerationMessageReceived { .. } => {
                    ("#".to_string(), vec![])
                }
                NotificationBody::Unknown => ("#".to_string(), vec![]),
            }
        };
        let (name, translatable_name) = match &notif.body {
            NotificationBody::LegacyMarkdown { name, .. } => {
                (name.clone(), TranslationData::Literal(name.clone()))
            }
            _ => (
                notif.body.notification_type().to_string(),
                notif.body.notification_type().translation_data(),
            ),
        };
        let text = notif.body.to_string();
        let translatable_text = notif.body.translation_data();

        Self {
            id: notif.id.into(),
            user_id: notif.user_id.into(),
            body: notif.body,
            read: notif.read,
            created: notif.created,

            name,
            translatable_name,
            text,
            translatable_text,
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
    pub fn list() -> &'static [Self] {
        &[NotificationChannel::Email]
    }

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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NotificationDeliveryStatus {
    Pending,
    SkippedPreferences,
    SkippedDefault,
    Delivered,
    PermanentlyFailed,
}

impl NotificationDeliveryStatus {
    pub fn as_user_error(self) -> Result<(), ApiError> {
        match self {
            NotificationDeliveryStatus::Delivered => Ok(()),
            NotificationDeliveryStatus::SkippedPreferences |
            NotificationDeliveryStatus::SkippedDefault |
            NotificationDeliveryStatus::Pending => Err(ApiError::InvalidInput("An error occured while sending an email to your email address. Please try again later.".to_owned())),
            NotificationDeliveryStatus::PermanentlyFailed => Err(ApiError::InvalidInput("This email address doesn't exist! Please try another one.".to_owned())),
        }
    }

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

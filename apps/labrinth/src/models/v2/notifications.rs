use crate::models::ids::{ThreadMessageId, VersionId};
use crate::models::v3::billing::PriceDuration;
use crate::models::{
    ids::{
        NotificationId, OrganizationId, ProjectId, ReportId, TeamId, ThreadId,
        UserSubscriptionId,
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
    TaxNotification {
        subscription_id: UserSubscriptionId,
        old_amount: i64,
        old_tax_amount: i64,
        new_amount: i64,
        new_tax_amount: i64,
        billing_interval: PriceDuration,
        currency: String,
        due: DateTime<Utc>,
        service: String,
    },
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
    // In `NotificationBody`, this has the `flow` field, however, don't
    // include it here, to be 100% certain we don't end up leaking it
    // in site notifications.
    ResetPassword,
    // Idem as ResetPassword
    VerifyEmail,
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
        service: String,
    },
    SubscriptionCredited {
        subscription_id: UserSubscriptionId,
        days: i32,
        previous_due: DateTime<Utc>,
        next_due: DateTime<Utc>,
        header_message: Option<String>,
    },
    PatCreated {
        token_name: String,
    },
    ModerationMessageReceived {
        project_id: ProjectId,
    },
    ReportStatusUpdated {
        report_id: ReportId,
    },
    ReportSubmitted {
        report_id: ReportId,
    },
    ProjectStatusApproved {
        project_id: ProjectId,
    },
    ProjectStatusNeutral {
        project_id: ProjectId,
        old_status: ProjectStatus,
        new_status: ProjectStatus,
    },
    ProjectTransferred {
        project_id: ProjectId,
        // Store only the raw identifiers in legacy body
        new_owner_user_id: Option<UserId>,
        new_owner_organization_id: Option<OrganizationId>,
    },
    PayoutAvailable {
        amount: u64,
        date_available: DateTime<Utc>,
    },
    Custom {
        key: String,
        title: String,
        body_md: String,
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
            NotificationBody::PatCreated { .. } => {
                Some("pat_created".to_string())
            }
            NotificationBody::ModerationMessageReceived { .. } => {
                Some("moderation_message_received".to_string())
            }
            NotificationBody::ReportStatusUpdated { .. } => {
                Some("report_status_updated".to_string())
            }
            NotificationBody::ReportSubmitted { .. } => {
                Some("report_submitted".to_string())
            }
            NotificationBody::ProjectStatusApproved { .. } => {
                Some("project_status_approved".to_string())
            }
            NotificationBody::ProjectStatusNeutral { .. } => {
                Some("project_status_neutral".to_string())
            }
            NotificationBody::ProjectTransferred { .. } => {
                Some("project_transferred".to_string())
            }
            NotificationBody::ResetPassword { .. } => {
                Some("reset_password".to_string())
            }
            NotificationBody::VerifyEmail { .. } => {
                Some("verify_email".to_string())
            }
            NotificationBody::AuthProviderAdded { .. } => {
                Some("auth_provider_added".to_string())
            }
            NotificationBody::AuthProviderRemoved { .. } => {
                Some("auth_provider_removed".to_string())
            }
            NotificationBody::TwoFactorEnabled => {
                Some("two_factor_enabled".to_string())
            }
            NotificationBody::TwoFactorRemoved => {
                Some("two_factor_removed".to_string())
            }
            NotificationBody::PasswordChanged => {
                Some("password_changed".to_string())
            }
            NotificationBody::PasswordRemoved => {
                Some("password_removed".to_string())
            }
            NotificationBody::EmailChanged { .. } => {
                Some("email_changed".to_string())
            }
            NotificationBody::PaymentFailed { .. } => {
                Some("payment_failed".to_string())
            }
            NotificationBody::TaxNotification { .. } => {
                Some("tax_notification".to_string())
            }
            NotificationBody::SubscriptionCredited { .. } => {
                Some("subscription_credited".to_string())
            }
            NotificationBody::PayoutAvailable { .. } => {
                Some("payout_available".to_string())
            }
            NotificationBody::Custom { .. } => Some("custom".to_string()),
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
            NotificationBody::PatCreated { token_name } => {
                LegacyNotificationBody::PatCreated { token_name }
            }
            NotificationBody::ModerationMessageReceived { project_id } => {
                LegacyNotificationBody::ModerationMessageReceived { project_id }
            }
            NotificationBody::ReportStatusUpdated { report_id } => {
                LegacyNotificationBody::ReportStatusUpdated { report_id }
            }
            NotificationBody::ReportSubmitted { report_id } => {
                LegacyNotificationBody::ReportSubmitted { report_id }
            }
            NotificationBody::ProjectStatusApproved { project_id } => {
                LegacyNotificationBody::ProjectStatusApproved { project_id }
            }
            NotificationBody::ProjectStatusNeutral {
                project_id,
                old_status,
                new_status,
            } => LegacyNotificationBody::ProjectStatusNeutral {
                project_id,
                old_status,
                new_status,
            },
            NotificationBody::ProjectTransferred {
                project_id,
                new_owner_user_id,
                new_owner_organization_id,
            } => LegacyNotificationBody::ProjectTransferred {
                project_id,
                new_owner_user_id,
                new_owner_organization_id,
            },
            NotificationBody::PayoutAvailable {
                amount,
                date_available,
            } => LegacyNotificationBody::PayoutAvailable {
                amount,
                date_available,
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
            NotificationBody::ResetPassword { .. } => {
                LegacyNotificationBody::ResetPassword
            }
            NotificationBody::VerifyEmail { .. } => {
                LegacyNotificationBody::VerifyEmail
            }
            NotificationBody::AuthProviderAdded { provider } => {
                LegacyNotificationBody::AuthProviderAdded { provider }
            }
            NotificationBody::AuthProviderRemoved { provider } => {
                LegacyNotificationBody::AuthProviderRemoved { provider }
            }
            NotificationBody::TwoFactorEnabled => {
                LegacyNotificationBody::TwoFactorEnabled
            }
            NotificationBody::TwoFactorRemoved => {
                LegacyNotificationBody::TwoFactorRemoved
            }
            NotificationBody::PasswordChanged => {
                LegacyNotificationBody::PasswordChanged
            }
            NotificationBody::PasswordRemoved => {
                LegacyNotificationBody::PasswordRemoved
            }
            NotificationBody::EmailChanged {
                new_email,
                to_email,
            } => LegacyNotificationBody::EmailChanged {
                new_email,
                to_email,
            },
            NotificationBody::TaxNotification {
                subscription_id,
                old_amount,
                old_tax_amount,
                new_amount,
                new_tax_amount,
                billing_interval,
                currency,
                due,
                service,
            } => LegacyNotificationBody::TaxNotification {
                subscription_id,
                old_amount,
                old_tax_amount,
                new_amount,
                new_tax_amount,
                billing_interval,
                due,
                service,
                currency,
            },
            NotificationBody::Custom {
                title,
                body_md,
                key,
            } => LegacyNotificationBody::Custom {
                title,
                body_md,
                key,
            },
            NotificationBody::PaymentFailed { amount, service } => {
                LegacyNotificationBody::PaymentFailed { amount, service }
            }
            NotificationBody::SubscriptionCredited {
                subscription_id,
                days,
                previous_due,
                next_due,
                header_message,
            } => LegacyNotificationBody::SubscriptionCredited {
                subscription_id,
                days,
                previous_due,
                next_due,
                header_message,
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

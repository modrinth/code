use super::ids::Base62Id;
use super::ids::OrganizationId;
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
    Unknown,
}

impl From<DBNotification> for Notification {
    fn from(notif: DBNotification) -> Self {
        let (name, text, link, actions) = {
            match &notif.body {
                NotificationBody::ProjectUpdate {
                    project_id,
                    version_id,
                } => (
                    "您关注的项目已更新！".to_string(),
                    format!("项目 {} 发布了新版本：{}", project_id, version_id),
                    format!("/project/{}/version/{}", project_id, version_id),
                    vec![],
                ),
                NotificationBody::TeamInvite {
                    project_id,
                    role,
                    team_id,
                    ..
                } => (
                    "您已被邀请加入团队！".to_string(),
                    format!("已向您发送邀请，成为团队的 {}", role),
                    format!("/project/{}", project_id),
                    vec![
                        NotificationAction {
                            name: "接受".to_string(),
                            action_route: (
                                "POST".to_string(),
                                format!("team/{team_id}/join"),
                            ),
                        },
                        NotificationAction {
                            name: "拒绝".to_string(),
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
                    "您已被邀请加入组织！".to_string(),
                    format!("已向您发送邀请，成为组织的 {}", role),
                    format!("/organization/{}", organization_id),
                    vec![
                        NotificationAction {
                            name: "接受".to_string(),
                            action_route: (
                                "POST".to_string(),
                                format!("team/{team_id}/join"),
                            ),
                        },
                        NotificationAction {
                            name: "拒绝".to_string(),
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
                    "项目状态已更改".to_string(),
                    format!(
                        "状态已从 {} 更改为 {}",
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
                    "管理员已向您发送消息！".to_string(),
                    "点击链接查看更多信息。".to_string(),
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
                    name,
                    text,
                    link,
                    actions,
                    ..
                } => (
                    name.clone(),
                    text.clone(),
                    link.clone(),
                    actions.clone().into_iter().map(Into::into).collect(),
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

use super::ids::Base62Id;
use super::users::UserId;
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
    #[serde(rename = "type")]
    pub type_: Option<String>,
    pub title: String,
    pub text: String,
    pub link: String,
    pub read: bool,
    pub created: DateTime<Utc>,
    pub actions: Vec<NotificationAction>,
}

use crate::database::models::notification_item::Notification as DBNotification;
use crate::database::models::notification_item::NotificationAction as DBNotificationAction;

impl From<DBNotification> for Notification {
    fn from(notif: DBNotification) -> Self {
        Self {
            id: notif.id.into(),
            user_id: notif.user_id.into(),
            type_: notif.notification_type,
            title: notif.title,
            text: notif.text,
            link: notif.link,
            read: notif.read,
            created: notif.created,
            actions: notif.actions.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Serialize, Deserialize)]
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

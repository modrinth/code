use crate::models::ids::SessionId;
use ariadne::ids::UserId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Session {
    pub id: SessionId,
    pub session: Option<String>,
    pub user_id: UserId,

    pub created: DateTime<Utc>,
    pub last_login: DateTime<Utc>,
    pub expires: DateTime<Utc>,
    pub refresh_expires: DateTime<Utc>,

    pub os: Option<String>,
    pub platform: Option<String>,
    pub user_agent: String,

    pub city: Option<String>,
    pub country: Option<String>,
    pub ip: String,

    pub current: bool,
}

impl Session {
    pub fn from(
        data: crate::database::models::session_item::DBSession,
        include_session: bool,
        current_session: Option<&str>,
    ) -> Self {
        Session {
            id: data.id.into(),
            current: Some(&*data.session) == current_session,
            session: if include_session {
                Some(data.session)
            } else {
                None
            },
            user_id: data.user_id.into(),
            created: data.created,
            last_login: data.last_login,
            expires: data.expires,
            refresh_expires: data.refresh_expires,
            os: data.os,
            platform: data.platform,
            user_agent: data.user_agent,
            city: data.city,
            country: data.country,
            ip: data.ip,
        }
    }
}

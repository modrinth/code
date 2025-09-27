use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::{database::models::DBUserId, models::users::User};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserLimits {
    pub current: UserLimitCount,
    pub max: UserLimitCount,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserLimitCount {
    pub projects: u64,
    pub organizations: u64,
    pub collections: u64,
}

impl UserLimits {
    pub async fn get(user: &User, pool: &PgPool) -> Result<Self, sqlx::Error> {
        let current = sqlx::query!(
            "SELECT
                (SELECT COUNT(*) FROM mods m
                 JOIN teams t ON m.team_id = t.id
                 JOIN team_members tm ON t.id = tm.team_id
                 WHERE tm.user_id = $1) as projects,

                (SELECT COUNT(*) FROM organizations o
                 JOIN teams t ON o.team_id = t.id
                 JOIN team_members tm ON t.id = tm.team_id
                 WHERE tm.user_id = $1) as organizations,

                (SELECT COUNT(*) FROM collections
                 WHERE user_id = $1) as collections",
            DBUserId::from(user.id) as DBUserId,
        )
        .fetch_one(pool)
        .await?;

        let current = UserLimitCount {
            projects: current.projects.map_or(0, |x| x as u64),
            organizations: current.organizations.map_or(0, |x| x as u64),
            collections: current.collections.map_or(0, |x| x as u64),
        };

        if user.role.is_admin() {
            Ok(Self {
                current,
                max: UserLimitCount {
                    projects: u64::MAX,
                    organizations: u64::MAX,
                    collections: u64::MAX,
                },
            })
        } else {
            // TODO: global config for max
            Ok(Self {
                current,
                max: UserLimitCount {
                    projects: user.max_projects.unwrap_or(256),
                    organizations: user.max_organizations.unwrap_or(16),
                    collections: user.max_collections.unwrap_or(64),
                },
            })
        }
    }
}

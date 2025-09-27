use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::{
    database::models::{DBUserId, user_limits::DBUserLimits},
    models::users::User,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserLimits {
    pub current: u64,
    pub max: u64,
}

impl UserLimits {
    fn adjust_for_user(self, user: &User) -> Self {
        if user.role.is_admin() {
            Self {
                current: self.current,
                max: u64::MAX,
            }
        } else {
            self
        }
    }

    pub async fn get_for_projects(
        user: &User,
        pool: &PgPool,
    ) -> Result<Self, sqlx::Error> {
        let user_id = DBUserId::from(user.id);
        let db_limits =
            DBUserLimits::get(DBUserId::from(user.id), pool).await?;
        let current = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM mods m
            JOIN teams t ON m.team_id = t.id
            JOIN team_members tm ON t.id = tm.team_id
            WHERE tm.user_id = $1",
            user_id as DBUserId,
        )
        .fetch_one(pool)
        .await?
        .map_or(0, |x| x as u64);

        Ok(Self {
            current,
            max: db_limits.projects,
        }
        .adjust_for_user(user))
    }

    pub async fn get_for_organizations(
        user: &User,
        pool: &PgPool,
    ) -> Result<Self, sqlx::Error> {
        let user_id = DBUserId::from(user.id);
        let db_limits =
            DBUserLimits::get(DBUserId::from(user.id), pool).await?;
        let current = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM organizations o
            JOIN teams t ON o.team_id = t.id
            JOIN team_members tm ON t.id = tm.team_id
            WHERE tm.user_id = $1",
            user_id as DBUserId,
        )
        .fetch_one(pool)
        .await?
        .map_or(0, |x| x as u64);

        Ok(Self {
            current,
            max: db_limits.organizations,
        }
        .adjust_for_user(user))
    }

    pub async fn get_for_collections(
        user: &User,
        pool: &PgPool,
    ) -> Result<Self, sqlx::Error> {
        let user_id = DBUserId::from(user.id);
        let db_limits =
            DBUserLimits::get(DBUserId::from(user.id), pool).await?;
        let current = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM collections
            WHERE user_id = $1",
            user_id as DBUserId,
        )
        .fetch_one(pool)
        .await?
        .map_or(0, |x| x as u64);

        Ok(Self {
            current,
            max: db_limits.collections,
        }
        .adjust_for_user(user))
    }
}

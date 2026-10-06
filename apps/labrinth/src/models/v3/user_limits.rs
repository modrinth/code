use crate::database::PgPool;
use chrono::{DateTime, TimeDelta, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    database::models::{DBProjectId, DBUserId, user_limits::DBUserLimits},
    models::users::User,
};

fn utc_day_bounds(now: DateTime<Utc>) -> (DateTime<Utc>, DateTime<Utc>) {
    let day_start = now
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .expect("midnight is always a valid time")
        .and_utc();
    (day_start, day_start + TimeDelta::days(1))
}

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

    pub async fn get_for_projects_per_day(
        user: &User,
        now: DateTime<Utc>,
        pool: &PgPool,
    ) -> Result<Self, sqlx::Error> {
        let user_id = DBUserId::from(user.id);
        let db_limits = DBUserLimits::get(user_id, pool).await?;
        let (day_start, day_end) = utc_day_bounds(now);
        let current = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM mods m
            JOIN teams t ON m.team_id = t.id
            JOIN team_members tm ON t.id = tm.team_id
            WHERE tm.user_id = $1
                AND m.published >= $2
                AND m.published < $3",
            user_id as DBUserId,
            day_start,
            day_end,
        )
        .fetch_one(pool)
        .await?
        .map_or(0, |x| x as u64);

        Ok(Self {
            current,
            max: db_limits.projects_per_day,
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

    pub async fn get_for_versions_per_project(
        user: &User,
        project_id: DBProjectId,
        pool: &PgPool,
    ) -> Result<Self, sqlx::Error> {
        let db_limits =
            DBUserLimits::get(DBUserId::from(user.id), pool).await?;
        let current = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM versions WHERE mod_id = $1",
            project_id as DBProjectId,
        )
        .fetch_one(pool)
        .await?
        .map_or(0, |x| x as u64);

        Ok(Self {
            current,
            max: db_limits.versions_per_project,
        }
        .adjust_for_user(user))
    }

    pub async fn get_for_versions_per_day(
        user: &User,
        now: DateTime<Utc>,
        pool: &PgPool,
    ) -> Result<Self, sqlx::Error> {
        let user_id = DBUserId::from(user.id);
        let db_limits = DBUserLimits::get(user_id, pool).await?;
        let (day_start, day_end) = utc_day_bounds(now);
        let current = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM versions
            WHERE author_id = $1
                AND date_published >= $2
                AND date_published < $3",
            user_id as DBUserId,
            day_start,
            day_end,
        )
        .fetch_one(pool)
        .await?
        .map_or(0, |x| x as u64);

        Ok(Self {
            current,
            max: db_limits.versions_per_day,
        }
        .adjust_for_user(user))
    }
}

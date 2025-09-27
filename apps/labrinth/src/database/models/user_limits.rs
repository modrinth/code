use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::database::models::DBUserId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DBUserLimits {
    pub user_id: Option<DBUserId>,
    pub projects: u64,
    pub organizations: u64,
    pub collections: u64,
}

impl DBUserLimits {
    pub async fn upsert(&self, pool: &PgPool) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "INSERT INTO user_limits (user_id, projects, organizations, collections)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (user_id) DO UPDATE
            SET projects = EXCLUDED.projects,
                organizations = EXCLUDED.organizations,
                collections = EXCLUDED.collections",
            self.user_id.map(|id| id.0),
            self.projects as i64,
            self.organizations as i64,
            self.collections as i64
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn get_defaults(pool: &PgPool) -> Result<Self, sqlx::Error> {
        let row = sqlx::query!(
            "SELECT projects, organizations, collections
            FROM user_limits
            WHERE user_id IS NULL"
        )
        .fetch_one(pool)
        .await?;

        Ok(Self {
            user_id: None,
            projects: row.projects as u64,
            organizations: row.organizations as u64,
            collections: row.collections as u64,
        })
    }

    pub async fn get(
        user_id: DBUserId,
        pool: &PgPool,
    ) -> Result<Self, sqlx::Error> {
        let row = sqlx::query!(
            "SELECT projects, organizations, collections
            FROM user_limits
            WHERE user_id = $1",
            user_id as DBUserId
        )
        .fetch_optional(pool)
        .await?;

        if let Some(row) = row {
            Ok(Self {
                user_id: Some(user_id),
                projects: row.projects as u64,
                organizations: row.organizations as u64,
                collections: row.collections as u64,
            })
        } else {
            Self::get_defaults(pool).await
        }
    }
}

// impl UserLimits {
//     pub async fn get(user: &User, pool: &PgPool) -> Result<Self, sqlx::Error> {
//         let current = sqlx::query!(
//             "SELECT
//                 (SELECT COUNT(*) FROM mods m
//                  JOIN teams t ON m.team_id = t.id
//                  JOIN team_members tm ON t.id = tm.team_id
//                  WHERE tm.user_id = $1) as projects,

//                 (SELECT COUNT(*) FROM organizations o
//                  JOIN teams t ON o.team_id = t.id
//                  JOIN team_members tm ON t.id = tm.team_id
//                  WHERE tm.user_id = $1) as organizations,

//                 (SELECT COUNT(*) FROM collections
//                  WHERE user_id = $1) as collections",
//             DBUserId::from(user.id) as DBUserId,
//         )
//         .fetch_one(pool)
//         .await?;

//         let current = UserLimitCount {
//             projects: current.projects.map_or(0, |x| x as u64),
//             organizations: current.organizations.map_or(0, |x| x as u64),
//             collections: current.collections.map_or(0, |x| x as u64),
//         };

//         if user.role.is_admin() {
//             Ok(Self {
//                 current,
//                 max: UserLimitCount {
//                     projects: u64::MAX,
//                     organizations: u64::MAX,
//                     collections: u64::MAX,
//                 },
//             })
//         } else {
//             // TODO: global config for max
//             Ok(Self {
//                 current,
//                 max: UserLimitCount {
//                     projects: user.max_projects.unwrap_or(256),
//                     organizations: user.max_organizations.unwrap_or(16),
//                     collections: user.max_collections.unwrap_or(64),
//                 },
//             })
//         }
//     }
// }

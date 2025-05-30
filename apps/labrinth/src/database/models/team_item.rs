use super::{DBOrganization, DBProject, ids::*};
use crate::{
    database::redis::RedisPool,
    models::teams::{OrganizationPermissions, ProjectPermissions},
};
use dashmap::DashMap;
use futures::TryStreamExt;
use itertools::Itertools;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

const TEAMS_NAMESPACE: &str = "teams";

pub struct TeamBuilder {
    pub members: Vec<TeamMemberBuilder>,
}
pub struct TeamMemberBuilder {
    pub user_id: DBUserId,
    pub role: String,
    pub is_owner: bool,
    pub permissions: ProjectPermissions,
    pub organization_permissions: Option<OrganizationPermissions>,
    pub accepted: bool,
    pub payouts_split: Decimal,
    pub ordering: i64,
}

impl TeamBuilder {
    pub async fn insert(
        self,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<DBTeamId, super::DatabaseError> {
        let team_id = generate_team_id(transaction).await?;

        let team = DBTeam { id: team_id };

        sqlx::query!(
            "
            INSERT INTO teams (id)
            VALUES ($1)
            ",
            team.id as DBTeamId,
        )
        .execute(&mut **transaction)
        .await?;

        let mut team_member_ids = Vec::new();
        for _ in self.members.iter() {
            team_member_ids.push(generate_team_member_id(transaction).await?.0);
        }
        let TeamBuilder { members } = self;
        let (
            team_ids,
            user_ids,
            roles,
            is_owners,
            permissions,
            organization_permissions,
            accepteds,
            payouts_splits,
            orderings,
        ): (
            Vec<_>,
            Vec<_>,
            Vec<_>,
            Vec<_>,
            Vec<_>,
            Vec<_>,
            Vec<_>,
            Vec<_>,
            Vec<_>,
        ) = members
            .into_iter()
            .map(|m| {
                (
                    team.id.0,
                    m.user_id.0,
                    m.role,
                    m.is_owner,
                    m.permissions.bits() as i64,
                    m.organization_permissions.map(|p| p.bits() as i64),
                    m.accepted,
                    m.payouts_split,
                    m.ordering,
                )
            })
            .multiunzip();
        sqlx::query!(
            "
            INSERT INTO team_members (id, team_id, user_id, role, is_owner, permissions, organization_permissions, accepted, payouts_split, ordering)
            SELECT * FROM UNNEST ($1::int8[], $2::int8[], $3::int8[], $4::varchar[], $5::bool[], $6::int8[], $7::int8[], $8::bool[], $9::numeric[], $10::int8[])
            ",
            &team_member_ids[..],
            &team_ids[..],
            &user_ids[..],
            &roles[..],
            &is_owners[..],
            &permissions[..],
            &organization_permissions[..] as &[Option<i64>],
            &accepteds[..],
            &payouts_splits[..],
            &orderings[..],
        )
        .execute(&mut **transaction)
        .await?;

        Ok(team_id)
    }
}

/// A team of users who control a project
pub struct DBTeam {
    /// The id of the team
    pub id: DBTeamId,
}

#[derive(Deserialize, Serialize, Clone, Debug, Copy)]
pub enum TeamAssociationId {
    Project(DBProjectId),
    Organization(DBOrganizationId),
}

impl DBTeam {
    pub async fn get_association<'a, 'b, E>(
        id: DBTeamId,
        executor: E,
    ) -> Result<Option<TeamAssociationId>, super::DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        let result = sqlx::query!(
            "
            SELECT m.id AS pid, NULL AS oid
            FROM mods m
            WHERE m.team_id = $1

            UNION ALL

            SELECT NULL AS pid, o.id AS oid
            FROM organizations o
            WHERE o.team_id = $1
            ",
            id as DBTeamId
        )
        .fetch_optional(executor)
        .await?;

        if let Some(t) = result {
            // Only one of project_id or organization_id will be set
            let mut team_association_id = None;
            if let Some(pid) = t.pid {
                team_association_id =
                    Some(TeamAssociationId::Project(DBProjectId(pid)));
            }
            if let Some(oid) = t.oid {
                team_association_id = Some(TeamAssociationId::Organization(
                    DBOrganizationId(oid),
                ));
            }
            return Ok(team_association_id);
        }
        Ok(None)
    }
}

/// A member of a team
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct DBTeamMember {
    pub id: DBTeamMemberId,
    pub team_id: DBTeamId,

    /// The ID of the user associated with the member
    pub user_id: DBUserId,
    pub role: String,
    pub is_owner: bool,

    // The permissions of the user in this project team
    // For an organization team, these are the fallback permissions for any project in the organization
    pub permissions: ProjectPermissions,

    // The permissions of the user in this organization team
    // For a project team, this is None
    pub organization_permissions: Option<OrganizationPermissions>,

    pub accepted: bool,
    pub payouts_split: Decimal,
    pub ordering: i64,
}

impl DBTeamMember {
    // Lists the full members of a team
    pub async fn get_from_team_full<'a, 'b, E>(
        id: DBTeamId,
        executor: E,
        redis: &RedisPool,
    ) -> Result<Vec<DBTeamMember>, super::DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres> + Copy,
    {
        Self::get_from_team_full_many(&[id], executor, redis).await
    }

    pub async fn get_from_team_full_many<'a, E>(
        team_ids: &[DBTeamId],
        exec: E,
        redis: &RedisPool,
    ) -> Result<Vec<DBTeamMember>, super::DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres> + Copy,
    {
        if team_ids.is_empty() {
            return Ok(Vec::new());
        }

        let val = redis.get_cached_keys(
            TEAMS_NAMESPACE,
            &team_ids.iter().map(|x| x.0).collect::<Vec<_>>(),
            |team_ids| async move {
                let teams = sqlx::query!(
                    "
                    SELECT id, team_id, role AS member_role, is_owner, permissions, organization_permissions,
                    accepted, payouts_split,
                    ordering, user_id
                    FROM team_members
                    WHERE team_id = ANY($1)
                    ORDER BY team_id, ordering;
                    ",
                    &team_ids
                )
                    .fetch(exec)
                    .try_fold(DashMap::new(), |acc: DashMap<i64, Vec<DBTeamMember>>, m| {
                        let member = DBTeamMember {
                            id: DBTeamMemberId(m.id),
                            team_id: DBTeamId(m.team_id),
                            role: m.member_role,
                            is_owner: m.is_owner,
                            permissions: ProjectPermissions::from_bits(m.permissions as u64)
                                .unwrap_or_default(),
                            organization_permissions: m
                                .organization_permissions
                                .map(|p| OrganizationPermissions::from_bits(p as u64).unwrap_or_default()),
                            accepted: m.accepted,
                            user_id: DBUserId(m.user_id),
                            payouts_split: m.payouts_split,
                            ordering: m.ordering,
                        };

                        acc.entry(m.team_id)
                            .or_default()
                            .push(member);

                        async move { Ok(acc) }
                    })
                    .await?;

                Ok(teams)
            },
        ).await?;

        Ok(val.into_iter().flatten().collect())
    }

    pub async fn clear_cache(
        id: DBTeamId,
        redis: &RedisPool,
    ) -> Result<(), super::DatabaseError> {
        let mut redis = redis.connect().await?;
        redis.delete(TEAMS_NAMESPACE, id.0).await?;
        Ok(())
    }

    /// Gets a team member from a user id and team id.  Does not return pending members.
    pub async fn get_from_user_id<'a, 'b, E>(
        id: DBTeamId,
        user_id: DBUserId,
        executor: E,
    ) -> Result<Option<Self>, super::DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        Self::get_from_user_id_many(&[id], user_id, executor)
            .await
            .map(|x| x.into_iter().next())
    }

    /// Gets team members from user ids and team ids.  Does not return pending members.
    pub async fn get_from_user_id_many<'a, 'b, E>(
        team_ids: &[DBTeamId],
        user_id: DBUserId,
        executor: E,
    ) -> Result<Vec<Self>, super::DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        let team_ids_parsed: Vec<i64> = team_ids.iter().map(|x| x.0).collect();

        let team_members = sqlx::query!(
            "
            SELECT id, team_id, role AS member_role, is_owner, permissions, organization_permissions,
            accepted, payouts_split, role,
            ordering, user_id
            FROM team_members
            WHERE (team_id = ANY($1) AND user_id = $2 AND accepted = TRUE)
            ORDER BY ordering
            ",
            &team_ids_parsed,
            user_id as DBUserId
        )
        .fetch(executor)
        .map_ok(|m| DBTeamMember {
            id: DBTeamMemberId(m.id),
            team_id: DBTeamId(m.team_id),
            user_id,
            role: m.role,
            is_owner: m.is_owner,
            permissions: ProjectPermissions::from_bits(m.permissions as u64)
                .unwrap_or_default(),
            organization_permissions: m
                .organization_permissions
                .map(|p| OrganizationPermissions::from_bits(p as u64).unwrap_or_default()),
            accepted: m.accepted,
            payouts_split: m.payouts_split,
            ordering: m.ordering,
        })
        .try_collect::<Vec<DBTeamMember>>()
        .await?;

        Ok(team_members)
    }

    /// Gets a team member from a user id and team id, including pending members.
    pub async fn get_from_user_id_pending<'a, 'b, E>(
        id: DBTeamId,
        user_id: DBUserId,
        executor: E,
    ) -> Result<Option<Self>, super::DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        let result = sqlx::query!(
            "
            SELECT id, team_id, role AS member_role, is_owner, permissions, organization_permissions,
                accepted, payouts_split, role,
                ordering, user_id

            FROM team_members
            WHERE (team_id = $1 AND user_id = $2)
            ORDER BY ordering
            ",
            id as DBTeamId,
            user_id as DBUserId
        )
        .fetch_optional(executor)
        .await?;

        if let Some(m) = result {
            Ok(Some(DBTeamMember {
                id: DBTeamMemberId(m.id),
                team_id: id,
                user_id,
                role: m.role,
                is_owner: m.is_owner,
                permissions: ProjectPermissions::from_bits(
                    m.permissions as u64,
                )
                .unwrap_or_default(),
                organization_permissions: m.organization_permissions.map(|p| {
                    OrganizationPermissions::from_bits(p as u64)
                        .unwrap_or_default()
                }),
                accepted: m.accepted,
                payouts_split: m.payouts_split,
                ordering: m.ordering,
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn insert(
        &self,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<(), sqlx::error::Error> {
        sqlx::query!(
            "
            INSERT INTO team_members (
                id, team_id, user_id, role, permissions, organization_permissions, is_owner, accepted, payouts_split
            )
            VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9
            )
            ",
            self.id as DBTeamMemberId,
            self.team_id as DBTeamId,
            self.user_id as DBUserId,
            self.role,
            self.permissions.bits() as i64,
            self.organization_permissions.map(|p| p.bits() as i64),
            self.is_owner,
            self.accepted,
            self.payouts_split
        )
        .execute(&mut **transaction)
        .await?;

        Ok(())
    }

    pub async fn delete(
        id: DBTeamId,
        user_id: DBUserId,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<(), super::DatabaseError> {
        sqlx::query!(
            "
            DELETE FROM team_members
            WHERE (team_id = $1 AND user_id = $2 AND NOT is_owner = TRUE)
            ",
            id as DBTeamId,
            user_id as DBUserId,
        )
        .execute(&mut **transaction)
        .await?;

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn edit_team_member(
        id: DBTeamId,
        user_id: DBUserId,
        new_permissions: Option<ProjectPermissions>,
        new_organization_permissions: Option<OrganizationPermissions>,
        new_role: Option<String>,
        new_accepted: Option<bool>,
        new_payouts_split: Option<Decimal>,
        new_ordering: Option<i64>,
        new_is_owner: Option<bool>,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<(), super::DatabaseError> {
        if let Some(permissions) = new_permissions {
            sqlx::query!(
                "
                UPDATE team_members
                SET permissions = $1
                WHERE (team_id = $2 AND user_id = $3)
                ",
                permissions.bits() as i64,
                id as DBTeamId,
                user_id as DBUserId,
            )
            .execute(&mut **transaction)
            .await?;
        }

        if let Some(organization_permissions) = new_organization_permissions {
            sqlx::query!(
                "
                UPDATE team_members
                SET organization_permissions = $1
                WHERE (team_id = $2 AND user_id = $3)
                ",
                organization_permissions.bits() as i64,
                id as DBTeamId,
                user_id as DBUserId,
            )
            .execute(&mut **transaction)
            .await?;
        }

        if let Some(role) = new_role {
            sqlx::query!(
                "
                UPDATE team_members
                SET role = $1
                WHERE (team_id = $2 AND user_id = $3)
                ",
                role,
                id as DBTeamId,
                user_id as DBUserId,
            )
            .execute(&mut **transaction)
            .await?;
        }

        if let Some(accepted) = new_accepted {
            if accepted {
                sqlx::query!(
                    "
                    UPDATE team_members
                    SET accepted = TRUE
                    WHERE (team_id = $1 AND user_id = $2)
                    ",
                    id as DBTeamId,
                    user_id as DBUserId,
                )
                .execute(&mut **transaction)
                .await?;
            }
        }

        if let Some(payouts_split) = new_payouts_split {
            sqlx::query!(
                "
                UPDATE team_members
                SET payouts_split = $1
                WHERE (team_id = $2 AND user_id = $3)
                ",
                payouts_split,
                id as DBTeamId,
                user_id as DBUserId,
            )
            .execute(&mut **transaction)
            .await?;
        }

        if let Some(ordering) = new_ordering {
            sqlx::query!(
                "
                UPDATE team_members
                SET ordering = $1
                WHERE (team_id = $2 AND user_id = $3)
                ",
                ordering,
                id as DBTeamId,
                user_id as DBUserId,
            )
            .execute(&mut **transaction)
            .await?;
        }

        if let Some(is_owner) = new_is_owner {
            sqlx::query!(
                "
                UPDATE team_members
                SET is_owner = $1
                WHERE (team_id = $2 AND user_id = $3)
                ",
                is_owner,
                id as DBTeamId,
                user_id as DBUserId,
            )
            .execute(&mut **transaction)
            .await?;
        }

        Ok(())
    }

    pub async fn get_from_user_id_project<'a, 'b, E>(
        id: DBProjectId,
        user_id: DBUserId,
        allow_pending: bool,
        executor: E,
    ) -> Result<Option<Self>, super::DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        let accepted = if allow_pending {
            vec![true, false]
        } else {
            vec![true]
        };

        let result = sqlx::query!(
            "
            SELECT tm.id, tm.team_id, tm.user_id, tm.role, tm.is_owner, tm.permissions, tm.organization_permissions, tm.accepted, tm.payouts_split, tm.ordering
            FROM mods m
            INNER JOIN team_members tm ON tm.team_id = m.team_id AND user_id = $2 AND accepted = ANY($3)
            WHERE m.id = $1
            ",
            id as DBProjectId,
            user_id as DBUserId,
            &accepted
        )
            .fetch_optional(executor)
            .await?;

        if let Some(m) = result {
            Ok(Some(DBTeamMember {
                id: DBTeamMemberId(m.id),
                team_id: DBTeamId(m.team_id),
                user_id,
                role: m.role,
                is_owner: m.is_owner,
                permissions: ProjectPermissions::from_bits(
                    m.permissions as u64,
                )
                .unwrap_or_default(),
                organization_permissions: m.organization_permissions.map(|p| {
                    OrganizationPermissions::from_bits(p as u64)
                        .unwrap_or_default()
                }),
                accepted: m.accepted,
                payouts_split: m.payouts_split,
                ordering: m.ordering,
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn get_from_user_id_organization<'a, 'b, E>(
        id: DBOrganizationId,
        user_id: DBUserId,
        allow_pending: bool,
        executor: E,
    ) -> Result<Option<Self>, super::DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        let accepted = if allow_pending {
            vec![true, false]
        } else {
            vec![true]
        };
        let result = sqlx::query!(
            "
            SELECT tm.id, tm.team_id, tm.user_id, tm.role, tm.is_owner, tm.permissions, tm.organization_permissions, tm.accepted, tm.payouts_split, tm.ordering
            FROM organizations o
            INNER JOIN team_members tm ON tm.team_id = o.team_id AND user_id = $2 AND accepted = ANY($3)
            WHERE o.id = $1
            ",
            id as DBOrganizationId,
            user_id as DBUserId,
            &accepted
        )
            .fetch_optional(executor)
            .await?;

        if let Some(m) = result {
            Ok(Some(DBTeamMember {
                id: DBTeamMemberId(m.id),
                team_id: DBTeamId(m.team_id),
                user_id,
                role: m.role,
                is_owner: m.is_owner,
                permissions: ProjectPermissions::from_bits(
                    m.permissions as u64,
                )
                .unwrap_or_default(),
                organization_permissions: m.organization_permissions.map(|p| {
                    OrganizationPermissions::from_bits(p as u64)
                        .unwrap_or_default()
                }),
                accepted: m.accepted,
                payouts_split: m.payouts_split,
                ordering: m.ordering,
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn get_from_user_id_version<'a, 'b, E>(
        id: DBVersionId,
        user_id: DBUserId,
        executor: E,
    ) -> Result<Option<Self>, super::DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        let result = sqlx::query!(
            "
            SELECT tm.id, tm.team_id, tm.user_id, tm.role, tm.is_owner, tm.permissions, tm.organization_permissions, tm.accepted, tm.payouts_split, tm.ordering, v.mod_id
            FROM versions v
            INNER JOIN mods m ON m.id = v.mod_id
            INNER JOIN team_members tm ON tm.team_id = m.team_id AND tm.user_id = $2 AND tm.accepted = TRUE
            WHERE v.id = $1
            ",
            id as DBVersionId,
            user_id as DBUserId
        )
            .fetch_optional(executor)
            .await?;

        if let Some(m) = result {
            Ok(Some(DBTeamMember {
                id: DBTeamMemberId(m.id),
                team_id: DBTeamId(m.team_id),
                user_id,
                role: m.role,
                is_owner: m.is_owner,
                permissions: ProjectPermissions::from_bits(
                    m.permissions as u64,
                )
                .unwrap_or_default(),
                organization_permissions: m.organization_permissions.map(|p| {
                    OrganizationPermissions::from_bits(p as u64)
                        .unwrap_or_default()
                }),
                accepted: m.accepted,
                payouts_split: m.payouts_split,
                ordering: m.ordering,
            }))
        } else {
            Ok(None)
        }
    }

    // Gets both required members for checking permissions of an action on a project
    // - project team member (a user's membership to a given project)
    // - organization team member (a user's membership to a given organization that owns a given project)
    pub async fn get_for_project_permissions<'a, 'b, E>(
        project: &DBProject,
        user_id: DBUserId,
        executor: E,
    ) -> Result<(Option<Self>, Option<Self>), super::DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres> + Copy,
    {
        let project_team_member =
            Self::get_from_user_id(project.team_id, user_id, executor).await?;

        let organization =
            DBOrganization::get_associated_organization_project_id(
                project.id, executor,
            )
            .await?;

        let organization_team_member = if let Some(organization) = &organization
        {
            Self::get_from_user_id(organization.team_id, user_id, executor)
                .await?
        } else {
            None
        };

        Ok((project_team_member, organization_team_member))
    }
}

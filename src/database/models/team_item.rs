use super::{ids::*, Organization, Project};
use crate::{
    database::redis::RedisPool,
    models::teams::{OrganizationPermissions, ProjectPermissions},
};
use itertools::Itertools;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

const TEAMS_NAMESPACE: &str = "teams";

pub struct TeamBuilder {
    pub members: Vec<TeamMemberBuilder>,
}
pub struct TeamMemberBuilder {
    pub user_id: UserId,
    pub role: String,
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
    ) -> Result<TeamId, super::DatabaseError> {
        let team_id = generate_team_id(transaction).await?;

        let team = Team { id: team_id };

        sqlx::query!(
            "
            INSERT INTO teams (id)
            VALUES ($1)
            ",
            team.id as TeamId,
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
        ) = members
            .into_iter()
            .map(|m| {
                (
                    team.id.0,
                    m.user_id.0,
                    m.role,
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
            INSERT INTO team_members (id, team_id, user_id, role, permissions, organization_permissions, accepted, payouts_split, ordering)
            SELECT * FROM UNNEST ($1::int8[], $2::int8[], $3::int8[], $4::varchar[], $5::int8[], $6::int8[], $7::bool[], $8::numeric[], $9::int8[])
            ",
            &team_member_ids[..],
            &team_ids[..],
            &user_ids[..],
            &roles[..],
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
pub struct Team {
    /// The id of the team
    pub id: TeamId,
}

#[derive(Deserialize, Serialize, Clone, Debug, Copy)]
pub enum TeamAssociationId {
    Project(ProjectId),
    Organization(OrganizationId),
}

impl Team {
    pub async fn get_association<'a, 'b, E>(
        id: TeamId,
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
            id as TeamId
        )
        .fetch_optional(executor)
        .await?;

        if let Some(t) = result {
            // Only one of project_id or organization_id will be set
            let mut team_association_id = None;
            if let Some(pid) = t.pid {
                team_association_id = Some(TeamAssociationId::Project(ProjectId(pid)));
            }
            if let Some(oid) = t.oid {
                team_association_id = Some(TeamAssociationId::Organization(OrganizationId(oid)));
            }
            return Ok(team_association_id);
        }
        Ok(None)
    }
}

/// A member of a team
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct TeamMember {
    pub id: TeamMemberId,
    pub team_id: TeamId,

    /// The ID of the user associated with the member
    pub user_id: UserId,
    pub role: String,

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

impl TeamMember {
    // Lists the full members of a team
    pub async fn get_from_team_full<'a, 'b, E>(
        id: TeamId,
        executor: E,
        redis: &RedisPool,
    ) -> Result<Vec<TeamMember>, super::DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres> + Copy,
    {
        Self::get_from_team_full_many(&[id], executor, redis).await
    }

    pub async fn get_from_team_full_many<'a, E>(
        team_ids: &[TeamId],
        exec: E,
        redis: &RedisPool,
    ) -> Result<Vec<TeamMember>, super::DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres> + Copy,
    {
        if team_ids.is_empty() {
            return Ok(Vec::new());
        }

        use futures::stream::TryStreamExt;

        let mut team_ids_parsed: Vec<i64> = team_ids.iter().map(|x| x.0).collect();

        let mut found_teams = Vec::new();

        let teams = redis
            .multi_get::<String, _>(TEAMS_NAMESPACE, team_ids_parsed.clone())
            .await?;

        for team_raw in teams {
            if let Some(mut team) = team_raw
                .clone()
                .and_then(|x| serde_json::from_str::<Vec<TeamMember>>(&x).ok())
            {
                if let Some(team_id) = team.first().map(|x| x.team_id) {
                    team_ids_parsed.retain(|x| &team_id.0 != x);
                }

                found_teams.append(&mut team);
                continue;
            }
        }

        if !team_ids_parsed.is_empty() {
            let teams: Vec<TeamMember> = sqlx::query!(
                "
                SELECT id, team_id, role AS member_role, permissions, organization_permissions,
                accepted, payouts_split, 
                ordering, user_id
                FROM team_members
                WHERE team_id = ANY($1)
                ORDER BY team_id, ordering;
                ",
                &team_ids_parsed
            )
            .fetch_many(exec)
            .try_filter_map(|e| async {
                Ok(e.right().map(|m| TeamMember {
                    id: TeamMemberId(m.id),
                    team_id: TeamId(m.team_id),
                    role: m.member_role,
                    permissions: ProjectPermissions::from_bits(m.permissions as u64)
                        .unwrap_or_default(),
                    organization_permissions: m
                        .organization_permissions
                        .map(|p| OrganizationPermissions::from_bits(p as u64).unwrap_or_default()),
                    accepted: m.accepted,
                    user_id: UserId(m.user_id),
                    payouts_split: m.payouts_split,
                    ordering: m.ordering,
                }))
            })
            .try_collect::<Vec<TeamMember>>()
            .await?;

            for (id, members) in &teams.into_iter().group_by(|x| x.team_id) {
                let mut members = members.collect::<Vec<_>>();

                redis
                    .set_serialized_to_json(TEAMS_NAMESPACE, id.0, &members, None)
                    .await?;
                found_teams.append(&mut members);
            }
        }

        Ok(found_teams)
    }

    pub async fn clear_cache(id: TeamId, redis: &RedisPool) -> Result<(), super::DatabaseError> {
        redis.delete(TEAMS_NAMESPACE, id.0).await?;
        Ok(())
    }

    /// Gets a team member from a user id and team id.  Does not return pending members.
    pub async fn get_from_user_id<'a, 'b, E>(
        id: TeamId,
        user_id: UserId,
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
        team_ids: &[TeamId],
        user_id: UserId,
        executor: E,
    ) -> Result<Vec<Self>, super::DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        use futures::stream::TryStreamExt;

        let team_ids_parsed: Vec<i64> = team_ids.iter().map(|x| x.0).collect();

        let team_members = sqlx::query!(
            "
            SELECT id, team_id, role AS member_role, permissions, organization_permissions,
            accepted, payouts_split, role,
            ordering, user_id
            FROM team_members
            WHERE (team_id = ANY($1) AND user_id = $2 AND accepted = TRUE)
            ORDER BY ordering
            ",
            &team_ids_parsed,
            user_id as UserId
        )
        .fetch_many(executor)
        .try_filter_map(|e| async {
            if let Some(m) = e.right() {
                Ok(Some(Ok(TeamMember {
                    id: TeamMemberId(m.id),
                    team_id: TeamId(m.team_id),
                    user_id,
                    role: m.role,
                    permissions: ProjectPermissions::from_bits(m.permissions as u64)
                        .unwrap_or_default(),
                    organization_permissions: m
                        .organization_permissions
                        .map(|p| OrganizationPermissions::from_bits(p as u64).unwrap_or_default()),
                    accepted: m.accepted,
                    payouts_split: m.payouts_split,
                    ordering: m.ordering,
                })))
            } else {
                Ok(None)
            }
        })
        .try_collect::<Vec<Result<TeamMember, super::DatabaseError>>>()
        .await?;

        let team_members = team_members
            .into_iter()
            .collect::<Result<Vec<TeamMember>, super::DatabaseError>>()?;

        Ok(team_members)
    }

    /// Gets a team member from a user id and team id, including pending members.
    pub async fn get_from_user_id_pending<'a, 'b, E>(
        id: TeamId,
        user_id: UserId,
        executor: E,
    ) -> Result<Option<Self>, super::DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        let result = sqlx::query!(
            "
            SELECT id, team_id, role AS member_role, permissions, organization_permissions,
                accepted, payouts_split, role,
                ordering, user_id
                
            FROM team_members
            WHERE (team_id = $1 AND user_id = $2)
            ORDER BY ordering
            ",
            id as TeamId,
            user_id as UserId
        )
        .fetch_optional(executor)
        .await?;

        if let Some(m) = result {
            Ok(Some(TeamMember {
                id: TeamMemberId(m.id),
                team_id: id,
                user_id,
                role: m.role,
                permissions: ProjectPermissions::from_bits(m.permissions as u64)
                    .unwrap_or_default(),
                organization_permissions: m
                    .organization_permissions
                    .map(|p| OrganizationPermissions::from_bits(p as u64).unwrap_or_default()),
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
                id, team_id, user_id, role, permissions, organization_permissions, accepted
            )
            VALUES (
                $1, $2, $3, $4, $5, $6, $7
            )
            ",
            self.id as TeamMemberId,
            self.team_id as TeamId,
            self.user_id as UserId,
            self.role,
            self.permissions.bits() as i64,
            self.organization_permissions.map(|p| p.bits() as i64),
            self.accepted,
        )
        .execute(&mut **transaction)
        .await?;

        Ok(())
    }

    pub async fn delete<'a, 'b>(
        id: TeamId,
        user_id: UserId,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<(), super::DatabaseError> {
        sqlx::query!(
            "
            DELETE FROM team_members
            WHERE (team_id = $1 AND user_id = $2 AND NOT role = $3)
            ",
            id as TeamId,
            user_id as UserId,
            crate::models::teams::OWNER_ROLE,
        )
        .execute(&mut **transaction)
        .await?;

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn edit_team_member(
        id: TeamId,
        user_id: UserId,
        new_permissions: Option<ProjectPermissions>,
        new_organization_permissions: Option<OrganizationPermissions>,
        new_role: Option<String>,
        new_accepted: Option<bool>,
        new_payouts_split: Option<Decimal>,
        new_ordering: Option<i64>,
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
                id as TeamId,
                user_id as UserId,
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
                id as TeamId,
                user_id as UserId,
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
                id as TeamId,
                user_id as UserId,
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
                    id as TeamId,
                    user_id as UserId,
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
                id as TeamId,
                user_id as UserId,
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
                id as TeamId,
                user_id as UserId,
            )
            .execute(&mut **transaction)
            .await?;
        }

        Ok(())
    }

    pub async fn get_from_user_id_project<'a, 'b, E>(
        id: ProjectId,
        user_id: UserId,
        executor: E,
    ) -> Result<Option<Self>, super::DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        let result = sqlx::query!(
            "
            SELECT tm.id, tm.team_id, tm.user_id, tm.role, tm.permissions, tm.organization_permissions, tm.accepted, tm.payouts_split, tm.ordering
            FROM mods m
            INNER JOIN team_members tm ON tm.team_id = m.team_id AND user_id = $2 AND accepted = TRUE
            WHERE m.id = $1
            ",
            id as ProjectId,
            user_id as UserId
        )
            .fetch_optional(executor)
            .await?;

        if let Some(m) = result {
            Ok(Some(TeamMember {
                id: TeamMemberId(m.id),
                team_id: TeamId(m.team_id),
                user_id,
                role: m.role,
                permissions: ProjectPermissions::from_bits(m.permissions as u64)
                    .unwrap_or_default(),
                organization_permissions: m
                    .organization_permissions
                    .map(|p| OrganizationPermissions::from_bits(p as u64).unwrap_or_default()),
                accepted: m.accepted,
                payouts_split: m.payouts_split,
                ordering: m.ordering,
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn get_from_user_id_organization<'a, 'b, E>(
        id: OrganizationId,
        user_id: UserId,
        executor: E,
    ) -> Result<Option<Self>, super::DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        let result = sqlx::query!(
            "
            SELECT tm.id, tm.team_id, tm.user_id, tm.role, tm.permissions, tm.organization_permissions, tm.accepted, tm.payouts_split, tm.ordering
            FROM organizations o
            INNER JOIN team_members tm ON tm.team_id = o.team_id AND user_id = $2 AND accepted = TRUE
            WHERE o.id = $1
            ",
            id as OrganizationId,
            user_id as UserId
        )
            .fetch_optional(executor)
            .await?;

        if let Some(m) = result {
            Ok(Some(TeamMember {
                id: TeamMemberId(m.id),
                team_id: TeamId(m.team_id),
                user_id,
                role: m.role,
                permissions: ProjectPermissions::from_bits(m.permissions as u64)
                    .unwrap_or_default(),
                organization_permissions: m
                    .organization_permissions
                    .map(|p| OrganizationPermissions::from_bits(p as u64).unwrap_or_default()),
                accepted: m.accepted,
                payouts_split: m.payouts_split,
                ordering: m.ordering,
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn get_from_user_id_version<'a, 'b, E>(
        id: VersionId,
        user_id: UserId,
        executor: E,
    ) -> Result<Option<Self>, super::DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        let result = sqlx::query!(
            "
            SELECT tm.id, tm.team_id, tm.user_id, tm.role, tm.permissions, tm.organization_permissions, tm.accepted, tm.payouts_split, tm.ordering, v.mod_id 
            FROM versions v
            INNER JOIN mods m ON m.id = v.mod_id
            INNER JOIN team_members tm ON tm.team_id = m.team_id AND tm.user_id = $2 AND tm.accepted = TRUE
            WHERE v.id = $1
            ",
            id as VersionId,
            user_id as UserId
        )
            .fetch_optional(executor)
            .await?;

        if let Some(m) = result {
            Ok(Some(TeamMember {
                id: TeamMemberId(m.id),
                team_id: TeamId(m.team_id),
                user_id,
                role: m.role,
                permissions: ProjectPermissions::from_bits(m.permissions as u64)
                    .unwrap_or_default(),
                organization_permissions: m
                    .organization_permissions
                    .map(|p| OrganizationPermissions::from_bits(p as u64).unwrap_or_default()),
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
        project: &Project,
        user_id: UserId,
        executor: E,
    ) -> Result<(Option<Self>, Option<Self>), super::DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres> + Copy,
    {
        let project_team_member =
            Self::get_from_user_id(project.team_id, user_id, executor).await?;

        let organization =
            Organization::get_associated_organization_project_id(project.id, executor).await?;

        let organization_team_member = if let Some(organization) = &organization {
            Self::get_from_user_id(organization.team_id, user_id, executor).await?
        } else {
            None
        };

        Ok((project_team_member, organization_team_member))
    }
}

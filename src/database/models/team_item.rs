use super::ids::*;
use crate::models::teams::Permissions;

pub struct TeamBuilder {
    pub members: Vec<TeamMemberBuilder>,
}
pub struct TeamMemberBuilder {
    pub user_id: UserId,
    pub name: String,
    pub role: String,
    pub permissions: Permissions,
    pub accepted: bool,
}

impl TeamBuilder {
    pub async fn insert(
        self,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<TeamId, super::DatabaseError> {
        let team_id = generate_team_id(&mut *transaction).await?;

        let team = Team { id: team_id };

        sqlx::query!(
            "
            INSERT INTO teams (id)
            VALUES ($1)
            ",
            team.id as TeamId,
        )
        .execute(&mut *transaction)
        .await?;

        for member in self.members {
            let team_member_id = generate_team_member_id(&mut *transaction).await?;
            let team_member = TeamMember {
                id: team_member_id,
                team_id,
                user_id: member.user_id,
                name: member.name,
                role: member.role,
                permissions: member.permissions,
                accepted: member.accepted,
            };

            sqlx::query!(
                "
                INSERT INTO team_members (id, team_id, user_id, member_name, role, permissions, accepted)
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                ",
                team_member.id as TeamMemberId,
                team_member.team_id as TeamId,
                team_member.user_id as UserId,
                team_member.name,
                team_member.role,
                team_member.permissions.bits() as i64,
                team_member.accepted,
            )
            .execute(&mut *transaction)
            .await?;
        }

        Ok(team_id)
    }
}

/// A team of users who control a mod
pub struct Team {
    /// The id of the team
    pub id: TeamId,
}

/// A member of a team
pub struct TeamMember {
    pub id: TeamMemberId,
    pub team_id: TeamId,
    /// The ID of the user associated with the member
    pub user_id: UserId,
    /// The name of the user
    pub name: String,
    pub role: String,
    pub permissions: Permissions,
    pub accepted: bool,
}

impl TeamMember {
    /// Lists the members of a team
    pub async fn get_from_team<'a, 'b, E>(
        id: TeamId,
        executor: E,
    ) -> Result<Vec<TeamMember>, super::DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        use futures::stream::TryStreamExt;

        let team_members = sqlx::query!(
            "
            SELECT id, user_id, member_name, role, permissions, accepted
            FROM team_members
            WHERE (team_id = $1 AND accepted = TRUE)
            ",
            id as TeamId,
        )
        .fetch_many(executor)
        .try_filter_map(|e| async {
            if let Some(m) = e.right() {
                let permissions = Permissions::from_bits(m.permissions as u64);
                if let Some(perms) = permissions {
                    Ok(Some(Ok(TeamMember {
                        id: TeamMemberId(m.id),
                        team_id: id,
                        user_id: UserId(m.user_id),
                        name: m.member_name,
                        role: m.role,
                        permissions: perms,
                        accepted: m.accepted,
                    })))
                } else {
                    Ok(Some(Err(super::DatabaseError::BitflagError)))
                }
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

    /// Lists the team members for a user.  Does not list pending requests.
    pub async fn get_from_user_public<'a, 'b, E>(
        id: UserId,
        executor: E,
    ) -> Result<Vec<TeamMember>, super::DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        use futures::stream::TryStreamExt;

        let team_members = sqlx::query!(
            "
            SELECT id, team_id, member_name, role, permissions, accepted
            FROM team_members
            WHERE (user_id = $1 AND accepted = TRUE)
            ",
            id as UserId,
        )
        .fetch_many(executor)
        .try_filter_map(|e| async {
            if let Some(m) = e.right() {
                let permissions = Permissions::from_bits(m.permissions as u64);
                if let Some(perms) = permissions {
                    Ok(Some(Ok(TeamMember {
                        id: TeamMemberId(m.id),
                        team_id: TeamId(m.team_id),
                        user_id: id,
                        name: m.member_name,
                        role: m.role,
                        permissions: perms,
                        accepted: m.accepted,
                    })))
                } else {
                    Ok(Some(Err(super::DatabaseError::BitflagError)))
                }
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

    /// Lists the team members for a user. Includes pending requests.
    pub async fn get_from_user_private<'a, 'b, E>(
        id: UserId,
        executor: E,
    ) -> Result<Vec<TeamMember>, super::DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        use futures::stream::TryStreamExt;

        let team_members = sqlx::query!(
            "
            SELECT id, team_id, member_name, role, permissions, accepted
            FROM team_members
            WHERE user_id = $1
            ",
            id as UserId,
        )
        .fetch_many(executor)
        .try_filter_map(|e| async {
            if let Some(m) = e.right() {
                let permissions = Permissions::from_bits(m.permissions as u64);
                if let Some(perms) = permissions {
                    Ok(Some(Ok(TeamMember {
                        id: TeamMemberId(m.id),
                        team_id: TeamId(m.team_id),
                        user_id: id,
                        name: m.member_name,
                        role: m.role,
                        permissions: perms,
                        accepted: m.accepted,
                    })))
                } else {
                    Ok(Some(Err(super::DatabaseError::BitflagError)))
                }
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

    /// Gets a team member from a user id and team id.  Does not return pending members.
    pub async fn get_from_user_id<'a, 'b, E>(
        id: TeamId,
        user_id: UserId,
        executor: E,
    ) -> Result<Option<Self>, super::DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        let result = sqlx::query!(
            "
            SELECT id, user_id, member_name, role, permissions, accepted
            FROM team_members
            WHERE (team_id = $1 AND user_id = $2 AND accepted = TRUE)
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
                name: m.member_name,
                role: m.role,
                permissions: Permissions::from_bits(m.permissions as u64)
                    .ok_or(super::DatabaseError::BitflagError)?,
                accepted: m.accepted,
            }))
        } else {
            Ok(None)
        }
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
            SELECT id, user_id, member_name, role, permissions, accepted
            FROM team_members
            WHERE (team_id = $1 AND user_id = $2)
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
                name: m.member_name,
                role: m.role,
                permissions: Permissions::from_bits(m.permissions as u64)
                    .ok_or(super::DatabaseError::BitflagError)?,
                accepted: m.accepted,
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
                id, user_id, member_name, role, permissions, accepted
            )
            VALUES (
                $1, $2, $3, $4, $5,
                $6
            )
            ",
            self.id as TeamMemberId,
            self.user_id as UserId,
            self.name,
            self.role,
            self.permissions.bits() as i64,
            self.accepted,
        )
        .execute(&mut *transaction)
        .await?;

        Ok(())
    }

    pub async fn delete<'a, 'b, E>(
        id: TeamId,
        user_id: UserId,
        executor: E,
    ) -> Result<(), super::DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        use sqlx::Done;
        let result = sqlx::query!(
            "
            DELETE FROM team_members
            WHERE (team_id = $1 AND user_id = $2 AND NOT role = $3)
            ",
            id as TeamId,
            user_id as UserId,
            crate::models::teams::OWNER_ROLE,
        )
        .execute(executor)
        .await?;

        if result.rows_affected() != 1 {
            return Err(super::DatabaseError::Other(format!(
                "Deleting a member failed; {} rows deleted",
                result.rows_affected()
            )));
        }

        Ok(())
    }

    pub async fn edit_team_member(
        id: TeamId,
        user_id: UserId,
        new_permissions: Option<Permissions>,
        new_role: Option<String>,
        new_accepted: Option<bool>,
        new_name: Option<String>,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<(), super::DatabaseError> {
        if let Some(permissions) = new_permissions {
            sqlx::query!(
                "
                UPDATE team_members
                SET permissions = $1
                WHERE (team_id = $2 AND user_id = $3 AND NOT role = $4)
                ",
                permissions.bits() as i64,
                id as TeamId,
                user_id as UserId,
                crate::models::teams::OWNER_ROLE,
            )
            .execute(&mut *transaction)
            .await?;
        }

        if let Some(role) = new_role {
            sqlx::query!(
                "
                UPDATE team_members
                SET role = $1
                WHERE (team_id = $2 AND user_id = $3 AND NOT role = $4)
                ",
                role,
                id as TeamId,
                user_id as UserId,
                crate::models::teams::OWNER_ROLE,
            )
            .execute(&mut *transaction)
            .await?;
        }

        if let Some(accepted) = new_accepted {
            if accepted {
                sqlx::query!(
                    "
                    UPDATE team_members
                    SET accepted = TRUE
                    WHERE (team_id = $1 AND user_id = $2 AND NOT role = $3)
                    ",
                    id as TeamId,
                    user_id as UserId,
                    crate::models::teams::OWNER_ROLE,
                )
                .execute(&mut *transaction)
                .await?;
            }
        }

        if let Some(name) = new_name {
            sqlx::query!(
                "
                UPDATE team_members
                SET member_name = $1
                WHERE (team_id = $2 AND user_id = $3 AND NOT role = $4)
                ",
                name,
                id as TeamId,
                user_id as UserId,
                crate::models::teams::OWNER_ROLE,
            )
            .execute(&mut *transaction)
            .await?;
        }

        Ok(())
    }
}

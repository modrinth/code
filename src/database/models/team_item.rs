use super::ids::*;
use crate::database::models::User;
use crate::models::teams::Permissions;
use crate::models::users::{Badges, RecipientType, RecipientWallet};
use rust_decimal::Decimal;

pub struct TeamBuilder {
    pub members: Vec<TeamMemberBuilder>,
}
pub struct TeamMemberBuilder {
    pub user_id: UserId,
    pub role: String,
    pub permissions: Permissions,
    pub accepted: bool,
    pub payouts_split: Decimal,
    pub ordering: i64,
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
            let team_member_id =
                generate_team_member_id(&mut *transaction).await?;
            let team_member = TeamMember {
                id: team_member_id,
                team_id,
                user_id: member.user_id,
                role: member.role,
                permissions: member.permissions,
                accepted: member.accepted,
                payouts_split: member.payouts_split,
                ordering: member.ordering,
            };

            sqlx::query!(
                "
                INSERT INTO team_members (id, team_id, user_id, role, permissions, accepted, payouts_split, ordering)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                ",
                team_member.id as TeamMemberId,
                team_member.team_id as TeamId,
                team_member.user_id as UserId,
                team_member.role,
                team_member.permissions.bits() as i64,
                team_member.accepted,
                team_member.payouts_split,
                team_member.ordering,
            )
            .execute(&mut *transaction)
            .await?;
        }

        Ok(team_id)
    }
}

/// A team of users who control a project
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
    pub role: String,
    pub permissions: Permissions,
    pub accepted: bool,
    pub payouts_split: Decimal,
    pub ordering: i64,
}

/// A member of a team
pub struct QueryTeamMember {
    pub id: TeamMemberId,
    pub team_id: TeamId,
    /// The user associated with the member
    pub user: User,
    pub role: String,
    pub permissions: Permissions,
    pub accepted: bool,
    pub payouts_split: Decimal,
    pub ordering: i64,
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
            SELECT id, user_id, role, permissions, accepted, payouts_split, ordering
            FROM team_members
            WHERE team_id = $1
            ORDER BY ordering
            ",
            id as TeamId,
        )
        .fetch_many(executor)
        .try_filter_map(|e| async {
            if let Some(m) = e.right() {
                Ok(Some(Ok(TeamMember {
                    id: TeamMemberId(m.id),
                    team_id: id,
                    user_id: UserId(m.user_id),
                    role: m.role,
                    permissions: Permissions::from_bits(m.permissions as u64)
                        .unwrap_or_default(),
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

    // Lists the full members of a team
    pub async fn get_from_team_full<'a, 'b, E>(
        id: TeamId,
        executor: E,
    ) -> Result<Vec<QueryTeamMember>, super::DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        use futures::stream::TryStreamExt;

        let team_members = sqlx::query!(
            "
            SELECT tm.id id, tm.role member_role, tm.permissions permissions, tm.accepted accepted, tm.payouts_split payouts_split, tm.ordering ordering,
            u.id user_id, u.github_id github_id, u.name user_name, u.email email,
            u.avatar_url avatar_url, u.username username, u.bio bio,
            u.created created, u.role user_role, u.badges badges, u.balance balance,
            u.payout_wallet payout_wallet, u.payout_wallet_type payout_wallet_type,
            u.payout_address payout_address, u.flame_anvil_key flame_anvil_key
            FROM team_members tm
            INNER JOIN users u ON u.id = tm.user_id
            WHERE tm.team_id = $1
            ORDER BY tm.ordering
            ",
            id as TeamId,
        )
        .fetch_many(executor)
        .try_filter_map(|e| async {
            if let Some(m) = e.right() {

                    Ok(Some(Ok(QueryTeamMember {
                        id: TeamMemberId(m.id),
                        team_id: id,
                        role: m.member_role,
                        permissions: Permissions::from_bits(m.permissions as u64).unwrap_or_default(),
                        accepted: m.accepted,
                        user: User {
                            id: UserId(m.user_id),
                            github_id: m.github_id,
                            name: m.user_name,
                            email: m.email,
                            avatar_url: m.avatar_url,
                            username: m.username,
                            bio: m.bio,
                            created: m.created,
                            role: m.user_role,
                            badges: Badges::from_bits(m.badges as u64).unwrap_or_default(),
                            balance: m.balance,
                            payout_wallet: m.payout_wallet.map(|x| RecipientWallet::from_string(&x)),
                            payout_wallet_type: m.payout_wallet_type.map(|x| RecipientType::from_string(&x)),
                            payout_address: m.payout_address,
                            flame_anvil_key: m.flame_anvil_key,
                        },
                        payouts_split: m.payouts_split,
                        ordering: m.ordering,
                    })))
            } else {
                Ok(None)
            }
        })
        .try_collect::<Vec<Result<QueryTeamMember, super::DatabaseError>>>()
        .await?;

        let team_members = team_members
            .into_iter()
            .collect::<Result<Vec<QueryTeamMember>, super::DatabaseError>>()?;

        Ok(team_members)
    }

    pub async fn get_from_team_full_many<'a, E>(
        team_ids: Vec<TeamId>,
        exec: E,
    ) -> Result<Vec<QueryTeamMember>, super::DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres> + Copy,
    {
        use futures::stream::TryStreamExt;

        let team_ids_parsed: Vec<i64> =
            team_ids.into_iter().map(|x| x.0).collect();

        let teams = sqlx::query!(
            "
            SELECT tm.id id, tm.team_id team_id, tm.role member_role, tm.permissions permissions, tm.accepted accepted, tm.payouts_split payouts_split, tm.ordering,
            u.id user_id, u.github_id github_id, u.name user_name, u.email email,
            u.avatar_url avatar_url, u.username username, u.bio bio,
            u.created created, u.role user_role, u.badges badges, u.balance balance,
            u.payout_wallet payout_wallet, u.payout_wallet_type payout_wallet_type,
            u.payout_address payout_address, u.flame_anvil_key flame_anvil_key
            FROM team_members tm
            INNER JOIN users u ON u.id = tm.user_id
            WHERE tm.team_id = ANY($1)
            ORDER BY tm.team_id, tm.ordering
            ",
            &team_ids_parsed
        )
          .fetch_many(exec)
          .try_filter_map(|e| async {
              if let Some(m) = e.right() {

                      Ok(Some(Ok(QueryTeamMember {
                          id: TeamMemberId(m.id),
                          team_id: TeamId(m.team_id),
                          role: m.member_role,
                          permissions: Permissions::from_bits(m.permissions as u64).unwrap_or_default(),
                          accepted: m.accepted,
                          user: User {
                              id: UserId(m.user_id),
                              github_id: m.github_id,
                              name: m.user_name,
                              email: m.email,
                              avatar_url: m.avatar_url,
                              username: m.username,
                              bio: m.bio,
                              created: m.created,
                              role: m.user_role,
                              badges: Badges::from_bits(m.badges as u64).unwrap_or_default(),
                              balance: m.balance,
                              payout_wallet: m.payout_wallet.map(|x| RecipientWallet::from_string(&x)),
                              payout_wallet_type: m.payout_wallet_type.map(|x| RecipientType::from_string(&x)),
                              payout_address: m.payout_address,
                              flame_anvil_key: m.flame_anvil_key,
                          },
                          payouts_split: m.payouts_split,
                          ordering: m.ordering,
                      })))
              } else {
                  Ok(None)
              }
          })
          .try_collect::<Vec<Result<QueryTeamMember, super::DatabaseError>>>()
          .await?;

        let team_members = teams
            .into_iter()
            .collect::<Result<Vec<QueryTeamMember>, super::DatabaseError>>()?;

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
            SELECT id, team_id, role, permissions, accepted, payouts_split, ordering
            FROM team_members
            WHERE (user_id = $1 AND accepted = TRUE)
            ORDER BY ordering
            ",
            id as UserId,
        )
        .fetch_many(executor)
        .try_filter_map(|e| async {
            if let Some(m) = e.right() {
                Ok(Some(Ok(TeamMember {
                    id: TeamMemberId(m.id),
                    team_id: TeamId(m.team_id),
                    user_id: id,
                    role: m.role,
                    permissions: Permissions::from_bits(m.permissions as u64)
                        .unwrap_or_default(),
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
            SELECT id, team_id, role, permissions, accepted, payouts_split, ordering
            FROM team_members
            WHERE user_id = $1
            ORDER BY ordering
            ",
            id as UserId,
        )
        .fetch_many(executor)
        .try_filter_map(|e| async {
            if let Some(m) = e.right() {
                Ok(Some(Ok(TeamMember {
                    id: TeamMemberId(m.id),
                    team_id: TeamId(m.team_id),
                    user_id: id,
                    role: m.role,
                    permissions: Permissions::from_bits(m.permissions as u64)
                        .unwrap_or_default(),
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
            SELECT id, user_id, role, permissions, accepted, payouts_split, ordering
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
                role: m.role,
                permissions: Permissions::from_bits(m.permissions as u64)
                    .unwrap_or_default(),
                accepted: m.accepted,
                payouts_split: m.payouts_split,
                ordering: m.ordering,
            }))
        } else {
            Ok(None)
        }
    }

    /// Gets team members from user ids and team ids.  Does not return pending members.
    pub async fn get_from_user_id_many<'a, 'b, E>(
        team_ids: Vec<TeamId>,
        user_id: UserId,
        executor: E,
    ) -> Result<Vec<Self>, super::DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        use futures::stream::TryStreamExt;

        let team_ids_parsed: Vec<i64> =
            team_ids.into_iter().map(|x| x.0).collect();

        let team_members = sqlx::query!(
            "
            SELECT id, team_id, user_id, role, permissions, accepted, payouts_split, ordering
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
                        permissions: Permissions::from_bits(m.permissions as u64).unwrap_or_default(),
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
            SELECT id, user_id, role, permissions, accepted, payouts_split, ordering
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
                role: m.role,
                permissions: Permissions::from_bits(m.permissions as u64)
                    .unwrap_or_default(),
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
                id, team_id, user_id, role, permissions, accepted
            )
            VALUES (
                $1, $2, $3, $4, $5, $6
            )
            ",
            self.id as TeamMemberId,
            self.team_id as TeamId,
            self.user_id as UserId,
            self.role,
            self.permissions.bits() as i64,
            self.accepted,
        )
        .execute(&mut *transaction)
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
            UPDATE mods
            SET flame_anvil_user = NULL
            WHERE (team_id = $1 AND flame_anvil_user = $2 )
            ",
            id as TeamId,
            user_id as UserId,
        )
        .execute(&mut *transaction)
        .await?;

        sqlx::query!(
            "
            DELETE FROM team_members
            WHERE (team_id = $1 AND user_id = $2 AND NOT role = $3)
            ",
            id as TeamId,
            user_id as UserId,
            crate::models::teams::OWNER_ROLE,
        )
        .execute(&mut *transaction)
        .await?;

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn edit_team_member(
        id: TeamId,
        user_id: UserId,
        new_permissions: Option<Permissions>,
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
            .execute(&mut *transaction)
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
            .execute(&mut *transaction)
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
                .execute(&mut *transaction)
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
            .execute(&mut *transaction)
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
            .execute(&mut *transaction)
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
            SELECT tm.id, tm.team_id, tm.user_id, tm.role, tm.permissions, tm.accepted, tm.payouts_split, tm.ordering FROM mods m
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
                permissions: Permissions::from_bits(m.permissions as u64)
                    .unwrap_or_default(),
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
            SELECT tm.id, tm.team_id, tm.user_id, tm.role, tm.permissions, tm.accepted, tm.payouts_split, tm.ordering FROM versions v
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
                permissions: Permissions::from_bits(m.permissions as u64)
                    .unwrap_or_default(),
                accepted: m.accepted,
                payouts_split: m.payouts_split,
                ordering: m.ordering,
            }))
        } else {
            Ok(None)
        }
    }
}

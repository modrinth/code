use super::ids::*;

pub struct TeamBuilder {
    pub members: Vec<TeamMemberBuilder>,
}
pub struct TeamMemberBuilder {
    pub user_id: UserId,
    pub name: String,
    pub role: String,
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
            };

            sqlx::query!(
                "
                INSERT INTO team_members (id, team_id, user_id, member_name, role)
                VALUES ($1, $2, $3, $4, $5)
                ",
                team_member.id as TeamMemberId,
                team_member.team_id as TeamId,
                team_member.user_id as UserId,
                team_member.name,
                team_member.role,
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
}

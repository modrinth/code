use crate::database::models::{DBOrganization, DBTeamId, DBTeamMember, DBUser};
use crate::database::redis::RedisPool;
use crate::models::ids::OrganizationId;
use crate::routes::internal::moderation::Ownership;
use crate::util::error::Context;
use ariadne::ids::UserId;
use eyre::eyre;
use sqlx::PgPool;

/// Fetches ownership information for multiple projects efficiently
pub async fn get_projects_ownership(
    projects: &[crate::models::projects::Project],
    pool: &PgPool,
    redis: &RedisPool,
) -> Result<Vec<Ownership>, crate::routes::ApiError> {
    let team_ids = projects
        .iter()
        .map(|project| project.team_id)
        .map(DBTeamId::from)
        .collect::<Vec<_>>();
    let org_ids = projects
        .iter()
        .filter_map(|project| project.organization)
        .collect::<Vec<_>>();

    let team_members =
        DBTeamMember::get_from_team_full_many(&team_ids, pool, redis)
            .await
            .wrap_internal_err("failed to fetch team members")?;
    let users = DBUser::get_many_ids(
        &team_members
            .iter()
            .map(|member| member.user_id)
            .collect::<Vec<_>>(),
        pool,
        redis,
    )
    .await
    .wrap_internal_err("failed to fetch user data of team members")?;
    let orgs = DBOrganization::get_many(&org_ids, pool, redis)
        .await
        .wrap_internal_err("failed to fetch organizations")?;

    let mut ownerships = Vec::with_capacity(projects.len());

    for project in projects {
        let project_id = project.id;
        let ownership = if let Some(org_id) = project.organization {
            let org = orgs
                    .iter()
                    .find(|org| OrganizationId::from(org.id) == org_id)
                    .wrap_internal_err_with(|| {
                        eyre!(
                            "project {project_id} is owned by an invalid organization {org_id}"
                        )
                    })?;

            Ownership::Organization {
                id: OrganizationId::from(org.id),
                name: org.name.clone(),
                icon_url: org.icon_url.clone(),
            }
        } else {
            let team_id = project.team_id;
            let team_owner = team_members.iter().find(|member| {
                crate::models::ids::TeamId::from(member.team_id) == team_id && member.is_owner
            })
                .wrap_internal_err_with(|| eyre!("project {project_id} is owned by a team {team_id} which has no valid owner"))?;
            let team_owner_id = team_owner.user_id;
            let user = users.iter().find(|user| user.id == team_owner_id)
                .wrap_internal_err_with(|| eyre!("project {project_id} is owned by a team {team_id} which has owner {} which does not exist", UserId::from(team_owner_id)))?;

            Ownership::User {
                id: ariadne::ids::UserId::from(user.id),
                name: user.username.clone(),
                icon_url: user.avatar_url.clone(),
            }
        };

        ownerships.push(ownership);
    }

    Ok(ownerships)
}

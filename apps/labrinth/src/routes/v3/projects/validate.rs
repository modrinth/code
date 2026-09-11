use actix_web::{HttpRequest, get, web};
use eyre::eyre;
use serde::Serialize;
use xredis::RedisPool;

use crate::auth::get_user_from_headers;
use crate::database::models::DBProjectId;
use crate::database::models::project_item::ProjectQueryResult;
use crate::database::{
    PgPool, PgTransaction, ReadOnlyPgPool, models as db_models,
};
use crate::models::ids::ProjectId;
use crate::models::pats::Scopes;
use crate::models::projects::{Project, Version};
use crate::models::teams::ProjectPermissions;
use crate::queue::session::AuthQueue;
use crate::routes::ApiError;
use crate::util::error::Context as _;
use crate::validate::project::{
    ProjectNag, ProjectSaveValidation,
    validate_with_context as validate_project,
};

#[derive(Debug, thiserror::Error)]
#[error("resolve required project validation messages before saving")]
pub(crate) struct ProjectValidationError(pub Vec<ProjectNag>);

pub(crate) fn require_valid_project(
    nags: Vec<ProjectNag>,
) -> Result<(), ApiError> {
    if nags.iter().any(|nag| {
        nag.severity == crate::validate::project::ProjectNagSeverity::Required
    }) {
        return Err(ApiError::Request(eyre!(ProjectValidationError(nags))));
    }
    Ok(())
}

pub(crate) async fn validate_link_changes(
    project: &Project,
    changes: &super::EditProject,
    force: bool,
) -> Result<(), ApiError> {
    let links_changed = changes.link_urls.is_some();
    let license_changed =
        changes.license_id.is_some() || changes.license_url.is_some();
    let description_changed = changes.description.is_some();
    if !force && !links_changed && !license_changed && !description_changed {
        return Ok(());
    }
    let mut candidate = project.clone();
    if let Some(links) = &changes.link_urls {
        apply_link_changes(&mut candidate, links);
    }
    if let Some(license) = &changes.license_id {
        candidate.license.id = license.clone();
    }
    if let Some(url) = &changes.license_url {
        candidate.license.url = url.clone();
    }
    if let Some(description) = &changes.description {
        candidate.description = description.clone();
    }
    let scope = crate::validate::project::LinkValidationScope {
        external: force || links_changed,
        license: force || license_changed,
        description: force || description_changed,
    };
    let nags =
        crate::validate::project::validate_link_fields(&candidate, scope).await;
    require_valid_project(nags)
}

pub(crate) fn save_validation_scope(
    project: &Project,
    changes: &super::EditProject,
) -> ProjectSaveValidation {
    let server = changes.minecraft_server.as_ref().and_then(Option::as_ref);
    let java = changes
        .minecraft_java_server
        .as_ref()
        .and_then(Option::as_ref);
    ProjectSaveValidation {
        name: changes.name.is_some(),
        summary: changes.summary.is_some(),
        description: changes.description.is_some(),
        license: changes.license_id.is_some() || changes.license_url.is_some(),
        links: changes.link_urls.is_some(),
        tags: changes.categories.is_some()
            || changes.additional_categories.is_some(),
        region: server.is_some_and(|server| server.region.is_some()),
        languages: server.is_some_and(|server| server.languages.is_some()),
        java_address: changes.minecraft_java_server.as_ref().is_some_and(
            |java| java.as_ref().is_none_or(|java| java.address.is_some()),
        ),
        compatibility: java.is_some_and(|java| java.content.is_some()),
        language_requirements: changes
            .minecraft_java_server
            .as_ref()
            .is_some_and(|java| {
                java.is_some()
                    != project.components.minecraft_java_server.is_some()
            }),
        ..Default::default()
    }
}

pub(crate) fn apply_link_changes(
    project: &mut Project,
    links: &std::collections::HashMap<String, Option<String>>,
) {
    for (field, url) in links {
        if let Some(url) = url {
            project.link_urls.insert(
                field.clone(),
                crate::models::projects::Link {
                    platform: field.clone(),
                    url: url.clone(),
                    donation: !matches!(
                        field.as_str(),
                        "source"
                            | "issues"
                            | "wiki"
                            | "discord"
                            | "site"
                            | "store"
                    ),
                },
            );
        } else {
            project.link_urls.remove(field);
        }
    }
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct ProjectValidationResponse {
    pub nags: Vec<ProjectNag>,
}

pub(crate) async fn ensure_project_is_valid_for_review(
    project_id: DBProjectId,
    pool: &PgPool,
    transaction: &mut PgTransaction<'_>,
    redis: &RedisPool,
) -> Result<ProjectQueryResult, ApiError> {
    ensure_project_is_valid_for_save(
        project_id,
        pool,
        transaction,
        redis,
        ProjectSaveValidation {
            all: true,
            ..Default::default()
        },
    )
    .await
}

pub(crate) async fn ensure_project_is_valid_for_save(
    project_id: DBProjectId,
    pool: &PgPool,
    transaction: &mut PgTransaction<'_>,
    redis: &RedisPool,
    scope: ProjectSaveValidation,
) -> Result<ProjectQueryResult, ApiError> {
    let mut projects = db_models::DBProject::get_many_uncached(
        &[ProjectId::from(project_id)],
        &mut *transaction,
        redis,
    )
    .await
    .wrap_internal_err("reloading project for save validation")?;
    let reloaded_project =
        projects.pop().wrap_not_found_err("resource not found")?;
    let versions = if scope.all || scope.license || scope.links {
        db_models::DBVersion::get_many_uncached(
            &reloaded_project.versions,
            &mut *transaction,
            redis,
        )
        .await
        .wrap_internal_err("reloading project versions for save validation")?
        .into_iter()
        .map(Version::from)
        .collect::<Vec<_>>()
    } else {
        Vec::new()
    };
    let available_categories = if scope.all || scope.tags {
        db_models::categories::Category::list(&**pool, redis)
            .await
            .wrap_internal_err("fetching project categories")?
    } else {
        Vec::new()
    };
    let disclosures = if scope.all || scope.disclosures {
        db_models::DBProjectDisclosure::get_many_for_project(
            reloaded_project.inner.id,
            false,
            &mut *transaction,
        )
        .await
        .wrap_internal_err("fetching project disclosures")?
        .into_iter()
        .map(|disclosure| disclosure.disclosure)
        .collect::<Vec<_>>()
    } else {
        Vec::new()
    };
    let project = Project::from(reloaded_project.clone());

    let nags = web::block(move || {
        let gallery_urls = project
            .gallery
            .iter()
            .map(|item| item.url.as_str())
            .collect::<Vec<_>>();
        validate_project(
            &project,
            &versions,
            &available_categories,
            &disclosures,
        )
        .into_iter()
        .filter(|nag| scope.includes(nag, &gallery_urls))
        .collect()
    })
    .await
    .wrap_internal_err("validating project save")?;
    require_valid_project(nags)?;

    Ok(reloaded_project)
}

/// Validate that a project is ready to be submitted for review.
#[utoipa::path(
	context_path = "/project",
	tag = "projects",
	responses((status = OK, body = ProjectValidationResponse))
)]
#[get("/{id}/validate")]
pub async fn validate(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    ro_pool: web::Data<ReadOnlyPgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<web::Json<ProjectValidationResponse>, ApiError> {
    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::PROJECT_READ,
    )
    .await
    .wrap_auth_err("authenticating API request")?
    .1;

    let project =
        db_models::DBProject::get(&info.into_inner().0, &***ro_pool, &redis)
            .await
            .wrap_internal_err("fetching project from database")?
            .wrap_not_found_err("resource not found")?;

    let (team_member, organization_team_member) =
        db_models::DBTeamMember::get_for_project_permissions(
            &project.inner,
            user.id.into(),
            &***ro_pool,
        )
        .await
        .wrap_internal_err("fetching project permissions")?;

    if ProjectPermissions::get_permissions_by_role(
        &user.role,
        &team_member,
        &organization_team_member,
    )
    .is_none()
    {
        return Err(ApiError::Auth(eyre!(
            "you do not have permission to validate this project"
        )));
    }

    let versions =
        db_models::DBVersion::get_many(&project.versions, &***ro_pool, &redis)
            .await
            .wrap_internal_err("fetching project versions from database")?
            .into_iter()
            .map(Version::from)
            .collect::<Vec<_>>();
    let available_categories =
        db_models::categories::Category::list(&**pool, &redis)
            .await
            .wrap_internal_err("fetching project categories")?;
    let disclosures = db_models::DBProjectDisclosure::get_many_for_project(
        project.inner.id,
        false,
        &***ro_pool,
    )
    .await
    .wrap_internal_err("fetching project disclosures")?
    .into_iter()
    .map(|disclosure| disclosure.disclosure)
    .collect::<Vec<_>>();
    let project = Project::from(project);

    let network_nags =
        crate::validate::project::validate_link_network(&project).await;
    let mut nags = web::block(move || {
        validate_project(
            &project,
            &versions,
            &available_categories,
            &disclosures,
        )
    })
    .await
    .wrap_internal_err("validating project")?;
    nags.extend(network_nags);
    Ok(web::Json(ProjectValidationResponse { nags }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validate::project::{ProjectNagKind, ProjectNagSeverity};

    #[test]
    fn only_required_nags_reject_saves_with_structured_details() {
        let nag = |severity| ProjectNag {
            kind: ProjectNagKind::DescriptionTooShort,
            severity,
            details: serde_json::json!({ "length": 9, "min_chars": 125 }),
        };
        assert!(
            require_valid_project(vec![
                nag(ProjectNagSeverity::Warning),
                nag(ProjectNagSeverity::Suggestion)
            ])
            .is_ok()
        );
        let error =
            require_valid_project(vec![nag(ProjectNagSeverity::Required)])
                .unwrap_err();
        let response = error.as_api_error();
        assert_eq!(response.error, "request_error");
        assert_eq!(
            response.details.unwrap()["nags"][0]["kind"],
            "description_too_short"
        );
    }
}

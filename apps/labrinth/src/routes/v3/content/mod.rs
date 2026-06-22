use super::ApiError;
use crate::auth::checks::{
    filter_visible_versions, is_visible_project, is_visible_version,
};
use crate::auth::get_user_from_headers;
use crate::database::models::ids::DBVersionId;
use crate::database::models::version_item::VersionQueryResult;
use crate::database::models::{DBProject, DBVersion};
use crate::database::{PgPool, redis::RedisPool};
use crate::models::pats::Scopes;
use crate::models::projects::{DependencyType, Version};
use crate::models::users::User;
use crate::queue::session::AuthQueue;
use actix_web::{HttpRequest, post, web};
use ariadne::ids::base62_impl::parse_base62;
use async_trait::async_trait;
use modrinth_content_management::{
    ContentMetadataProvider, Error as ResolveError, ResolveContentPlan,
    ResolveContentRequest,
};

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(resolve_content);
}

#[post("content/resolve")]
async fn resolve_content(
    req: HttpRequest,
    request: web::Json<ResolveContentRequest>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<web::Json<ResolveContentPlan>, ApiError> {
    let user_option = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::PROJECT_READ | Scopes::VERSION_READ,
    )
    .await
    .map(|x| x.1)
    .ok();
    let provider = LabrinthContentProvider {
        pool,
        redis,
        user_option,
    };
    let plan =
        modrinth_content_management::resolve_content(&provider, request.into_inner())
            .await
            .map_err(resolve_error_to_api)?;

    Ok(web::Json(plan))
}

struct LabrinthContentProvider {
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    user_option: Option<User>,
}

#[async_trait]
impl ContentMetadataProvider for LabrinthContentProvider {
    async fn get_version(
        &self,
        version_id: &str,
    ) -> Result<Option<modrinth_content_management::Version>, ResolveError>
    {
        let Some(version_id) = parse_version_id(version_id) else {
            return Ok(None);
        };
        let version = DBVersion::get(version_id, &**self.pool, &self.redis)
            .await
            .map_err(resolve_provider_error)?;

        let Some(version) = version else {
            return Ok(None);
        };

        if !is_visible_version(
            &version.inner,
            &self.user_option,
            &self.pool,
            &self.redis,
        )
        .await
        .map_err(resolve_provider_error)?
        {
            return Ok(None);
        }

        Ok(Some(version_to_resolver(Version::from(version))))
    }

    async fn get_project_versions(
        &self,
        project_id: &str,
    ) -> Result<Vec<modrinth_content_management::Version>, ResolveError>
    {
        let project = DBProject::get(project_id, &**self.pool, &self.redis)
            .await
            .map_err(resolve_provider_error)?;
        let Some(project) = project else {
            return Ok(Vec::new());
        };

        if !is_visible_project(&project.inner, &self.user_option, &self.pool, false)
            .await
            .map_err(resolve_provider_error)?
        {
            return Ok(Vec::new());
        }

        let versions = DBVersion::get_many(
            &project.versions,
            &**self.pool,
            &self.redis,
        )
        .await
        .map_err(resolve_provider_error)?;
        let versions =
            visible_versions(versions, &self.user_option, &self.pool, &self.redis)
                .await
                .map_err(resolve_provider_error)?;

        Ok(versions.into_iter().map(version_to_resolver).collect())
    }
}

fn parse_version_id(version_id: &str) -> Option<DBVersionId> {
    parse_base62(version_id).ok().map(|id| DBVersionId(id as i64))
}

async fn visible_versions(
    versions: Vec<VersionQueryResult>,
    user_option: &Option<User>,
    pool: &PgPool,
    redis: &RedisPool,
) -> Result<Vec<Version>, ApiError> {
    filter_visible_versions(versions, user_option, pool, redis).await
}

fn version_to_resolver(version: Version) -> modrinth_content_management::Version {
    modrinth_content_management::Version {
        id: version.id.to_string(),
        project_id: version.project_id.to_string(),
        date_published: version.date_published,
        dependencies: version
            .dependencies
            .into_iter()
            .map(|dependency| modrinth_content_management::Dependency {
                version_id: dependency.version_id.map(|id| id.to_string()),
                project_id: dependency.project_id.map(|id| id.to_string()),
                file_name: dependency.file_name,
                dependency_type: dependency_type_to_resolver(
                    dependency.dependency_type,
                ),
            })
            .collect(),
        game_versions: version.game_versions,
        loaders: version.loaders,
    }
}

fn dependency_type_to_resolver(
    dependency_type: DependencyType,
) -> modrinth_content_management::DependencyType {
    match dependency_type {
        DependencyType::Required => {
            modrinth_content_management::DependencyType::Required
        }
        DependencyType::Optional => {
            modrinth_content_management::DependencyType::Optional
        }
        DependencyType::Incompatible => {
            modrinth_content_management::DependencyType::Incompatible
        }
        DependencyType::Embedded => {
            modrinth_content_management::DependencyType::Embedded
        }
    }
}

fn resolve_provider_error(error: impl std::fmt::Display) -> ResolveError {
    ResolveError::Provider(error.to_string())
}

fn resolve_error_to_api(error: ResolveError) -> ApiError {
    match error {
        ResolveError::Provider(message) => {
            ApiError::Internal(eyre::eyre!(message))
        }
        ResolveError::ProjectNotFound(project_id) => {
            ApiError::Request(eyre::eyre!("project `{project_id}` was not found"))
        }
        ResolveError::VersionNotFound(version_id) => {
            ApiError::Request(eyre::eyre!("version `{version_id}` was not found"))
        }
        ResolveError::VersionProjectMismatch {
            version_id,
            project_id,
        } => ApiError::Request(eyre::eyre!(
            "version `{version_id}` does not belong to project `{project_id}`"
        )),
        ResolveError::NoCompatibleVersion(project_id) => ApiError::Request(
            eyre::eyre!("no compatible version was found for project `{project_id}`"),
        ),
    }
}

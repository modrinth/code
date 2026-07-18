use super::ApiError;
use crate::auth::checks::{
    filter_visible_versions, is_visible_project, is_visible_version,
};
use crate::auth::get_user_from_headers;
use crate::database::models::ids::DBVersionId;
use crate::database::models::version_item::VersionQueryResult;
use crate::database::models::{DBProject, DBVersion};
use crate::database::{PgPool, ReadOnlyPgPool, redis::RedisPool};
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
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

const CONTENT_RESOLVE_CACHE_NAMESPACE: &str = "content_resolve:v1";
const CONTENT_RESOLVE_CACHE_HEAT_NAMESPACE: &str = "content_resolve_heat:v1";
const CONTENT_RESOLVE_CACHE_SCHEMA_VERSION: &str = "v1";
const CONTENT_RESOLVE_CACHE_HEAT_WINDOW_SECONDS: i64 = 60 * 60 * 24;

pub fn config(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(resolve_content);
}

/// Resolve content.
#[utoipa::path(
	tag = "content",
	request_body = serde_json::Value,
	responses((status = OK, body = serde_json::Value)),
)]
#[post("/content/resolve")]
pub async fn resolve_content(
    req: HttpRequest,
    request: web::Json<ResolveContentRequest>,
    pool: web::Data<PgPool>,
    ro_pool: web::Data<ReadOnlyPgPool>,
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
    let cache_public_result = user_option.is_none();
    let mut provider = LabrinthContentProvider {
        pool: pool.get_ref(),
        ro_pool: ro_pool.get_ref(),
        redis: redis.get_ref(),
        user_option: &user_option,
        trace: ResolveContentTrace::default(),
    };
    let request = request.into_inner();
    let plan = if cache_public_result {
        resolve_content_with_cache(&mut provider, request).await
    } else {
        modrinth_content_management::resolve_content(&mut provider, request)
            .await
    }
    .map_err(resolve_error_to_api)?;

    Ok(web::Json(plan))
}

struct LabrinthContentProvider<'a> {
    pool: &'a PgPool,
    ro_pool: &'a ReadOnlyPgPool,
    redis: &'a RedisPool,
    user_option: &'a Option<User>,
    trace: ResolveContentTrace,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
struct ResolveContentTrace {
    versions: BTreeMap<String, String>,
    project_versions: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct CachedResolveContentPlan {
    trace: ResolveContentTrace,
    plan: ResolveContentPlan,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize)]
struct DependencyState {
    version_id: Option<String>,
    project_id: Option<String>,
    file_name: Option<String>,
    dependency_type: &'static str,
}

#[derive(Serialize)]
struct VersionState<'a> {
    id: &'a str,
    project_id: &'a str,
    date_published: String,
    dependencies: Vec<DependencyState>,
    game_versions: Vec<&'a str>,
    loaders: Vec<&'a str>,
}

#[derive(Serialize)]
struct ResolveContentHeatKey<'a> {
    project_id: &'a str,
    version_id: Option<&'a str>,
    content_type: modrinth_content_management::ContentType,
}

#[async_trait]
impl ContentMetadataProvider for &mut LabrinthContentProvider<'_> {
    async fn get_version(
        &mut self,
        version_id: &str,
    ) -> Result<Option<modrinth_content_management::Version>, ResolveError>
    {
        let Some(db_version_id) = parse_version_id(version_id) else {
            return Ok(None);
        };
        let version = DBVersion::get(db_version_id, self.pool, self.redis)
            .await
            .map_err(resolve_provider_error)?;

        let Some(version) = version else {
            self.record_version(version_id, None);
            return Ok(None);
        };

        if !is_visible_version(
            &version.inner,
            self.user_option,
            self.pool,
            self.redis,
        )
        .await
        .map_err(resolve_provider_error)?
        {
            self.record_version(version_id, None);
            return Ok(None);
        }

        let version = version_to_resolver(Version::from(version));
        self.record_version(version_id, Some(&version));

        Ok(Some(version))
    }

    async fn get_project_versions(
        &mut self,
        project_id: &str,
    ) -> Result<Vec<modrinth_content_management::Version>, ResolveError> {
        let project = DBProject::get(project_id, self.pool, self.redis)
            .await
            .map_err(resolve_provider_error)?;
        let Some(project) = project else {
            self.record_project_versions(project_id, &[]);
            return Ok(Vec::new());
        };

        if !is_visible_project(
            &project.inner,
            self.user_option,
            self.pool,
            false,
        )
        .await
        .map_err(resolve_provider_error)?
        {
            self.record_project_versions(project_id, &[]);
            return Ok(Vec::new());
        }

        let versions =
            DBVersion::get_many(&project.versions, self.pool, self.redis)
                .await
                .map_err(resolve_provider_error)?;
        let versions = visible_versions(
            versions,
            self.user_option,
            self.pool,
            self.ro_pool,
            self.redis,
        )
        .await
        .map_err(resolve_provider_error)?;

        let versions = versions
            .into_iter()
            .map(version_to_resolver)
            .collect::<Vec<_>>();
        self.record_project_versions(project_id, &versions);

        Ok(versions)
    }
}

impl LabrinthContentProvider<'_> {
    fn record_version(
        &mut self,
        version_id: &str,
        version: Option<&modrinth_content_management::Version>,
    ) {
        self.trace
            .versions
            .insert(version_id.to_string(), hash_optional_version(version));
    }

    fn record_project_versions(
        &mut self,
        project_id: &str,
        versions: &[modrinth_content_management::Version],
    ) {
        self.trace
            .project_versions
            .insert(project_id.to_string(), hash_project_versions(versions));
    }

    fn reset_trace(&mut self) {
        self.trace = ResolveContentTrace::default();
    }

    fn trace(&self) -> ResolveContentTrace {
        self.trace.clone()
    }
}

async fn resolve_content_with_cache(
    provider: &mut LabrinthContentProvider<'_>,
    request: ResolveContentRequest,
) -> Result<ResolveContentPlan, ResolveError> {
    let cache_key = content_resolve_cache_key(&request);
    let heat_key = content_resolve_heat_key(&request);
    let heat = increment_content_resolve_cache_heat(provider.redis, &heat_key)
        .await
        .unwrap_or(1);
    let cache_expiry = content_resolve_cache_expiry_seconds(heat);

    if let Some(cached) =
        get_cached_resolve_content_plan(provider.redis, &cache_key).await
        && validate_cached_trace(provider, &cached.trace).await?
    {
        set_cached_resolve_content_plan(
            provider.redis,
            &cache_key,
            &cached,
            cache_expiry,
        )
        .await;
        return Ok(cached.plan);
    }

    provider.reset_trace();
    let plan =
        modrinth_content_management::resolve_content(&mut *provider, request)
            .await?;
    let trace = provider.trace();
    set_cached_resolve_content_plan(
        provider.redis,
        &cache_key,
        &CachedResolveContentPlan {
            trace,
            plan: plan.clone(),
        },
        cache_expiry,
    )
    .await;

    Ok(plan)
}

async fn increment_content_resolve_cache_heat(
    redis: &RedisPool,
    heat_key: &str,
) -> Option<u64> {
    let mut redis = match redis.connect().await {
        Ok(redis) => redis,
        Err(error) => {
            tracing::warn!(
                "failed to connect to redis for content resolve cache heat: {error}"
            );
            return None;
        }
    };

    let count = match redis
        .incr(CONTENT_RESOLVE_CACHE_HEAT_NAMESPACE, heat_key)
        .await
    {
        Ok(Some(count)) => count,
        Ok(None) => 1,
        Err(error) => {
            tracing::warn!(
                "failed to increment content resolve cache heat: {error}"
            );
            return None;
        }
    };

    if let Err(error) = redis
        .set(
            CONTENT_RESOLVE_CACHE_HEAT_NAMESPACE,
            heat_key,
            &count.to_string(),
            Some(CONTENT_RESOLVE_CACHE_HEAT_WINDOW_SECONDS),
        )
        .await
    {
        tracing::warn!("failed to refresh content resolve cache heat: {error}");
    }

    Some(count)
}

async fn get_cached_resolve_content_plan(
    redis: &RedisPool,
    cache_key: &str,
) -> Option<CachedResolveContentPlan> {
    let mut redis = match redis.connect().await {
        Ok(redis) => redis,
        Err(error) => {
            tracing::warn!(
                "failed to connect to redis for content resolve cache: {error}"
            );
            return None;
        }
    };

    match redis
        .get_deserialized(CONTENT_RESOLVE_CACHE_NAMESPACE, cache_key)
        .await
    {
        Ok(cached) => cached,
        Err(error) => {
            tracing::warn!("failed to read content resolve cache: {error}");
            None
        }
    }
}

async fn set_cached_resolve_content_plan(
    redis: &RedisPool,
    cache_key: &str,
    cached: &CachedResolveContentPlan,
    expiry_seconds: i64,
) {
    let mut redis = match redis.connect().await {
        Ok(redis) => redis,
        Err(error) => {
            tracing::warn!(
                "failed to connect to redis for content resolve cache: {error}"
            );
            return;
        }
    };

    if let Err(error) = redis
        .set_serialized(
            CONTENT_RESOLVE_CACHE_NAMESPACE,
            cache_key,
            cached,
            Some(expiry_seconds),
        )
        .await
    {
        tracing::warn!("failed to write content resolve cache: {error}");
    }
}

async fn validate_cached_trace(
    provider: &mut LabrinthContentProvider<'_>,
    trace: &ResolveContentTrace,
) -> Result<bool, ResolveError> {
    provider.reset_trace();

    for (version_id, expected_hash) in &trace.versions {
        let Some(db_version_id) = parse_version_id(version_id) else {
            return Ok(false);
        };
        let version =
            DBVersion::get(db_version_id, provider.pool, provider.redis)
                .await
                .map_err(resolve_provider_error)?;

        let version = if let Some(version) = version {
            if is_visible_version(
                &version.inner,
                provider.user_option,
                provider.pool,
                provider.redis,
            )
            .await
            .map_err(resolve_provider_error)?
            {
                Some(version_to_resolver(Version::from(version)))
            } else {
                None
            }
        } else {
            None
        };

        if &hash_optional_version(version.as_ref()) != expected_hash {
            return Ok(false);
        }
    }

    for (project_id, expected_hash) in &trace.project_versions {
        let versions =
            (&mut *provider).get_project_versions(project_id).await?;

        if &hash_project_versions(&versions) != expected_hash {
            return Ok(false);
        }
    }

    Ok(true)
}

fn content_resolve_cache_key(request: &ResolveContentRequest) -> String {
    format!(
        "{CONTENT_RESOLVE_CACHE_SCHEMA_VERSION}:{}",
        hash_serializable(&normalized_resolve_content_request(request))
    )
}

fn content_resolve_heat_key(request: &ResolveContentRequest) -> String {
    hash_serializable(&ResolveContentHeatKey {
        project_id: &request.project_id,
        version_id: request.version_id.as_deref(),
        content_type: request.content_type,
    })
}

fn content_resolve_cache_expiry_seconds(heat: u64) -> i64 {
    match heat {
        0..=1 => 60 * 5,
        2..=9 => 60 * 30,
        10..=99 => 60 * 60 * 6,
        _ => 60 * 60 * 24,
    }
}

fn normalized_resolve_content_request(
    request: &ResolveContentRequest,
) -> ResolveContentRequest {
    let mut request = request.clone();

    request.selected.game_versions.sort();
    request.selected.game_versions.dedup();
    request.selected.loaders.sort();
    request.selected.loaders.dedup();
    request.target.game_versions.sort();
    request.target.game_versions.dedup();
    request.target.loaders.sort();
    request.target.loaders.dedup();
    request.existing_project_ids.sort();
    request.existing_project_ids.dedup();

    request
}

fn hash_optional_version(
    version: Option<&modrinth_content_management::Version>,
) -> String {
    match version {
        Some(version) => format!("some:{}", hash_version(version)),
        None => "none".to_string(),
    }
}

fn hash_project_versions(
    versions: &[modrinth_content_management::Version],
) -> String {
    let mut versions = versions
        .iter()
        .map(|version| (version.id.as_str(), hash_version(version)))
        .collect::<Vec<_>>();
    versions.sort_by(|a, b| a.0.cmp(b.0));

    hash_serializable(&versions)
}

fn hash_version(version: &modrinth_content_management::Version) -> String {
    let mut dependencies = version
        .dependencies
        .iter()
        .map(|dependency| DependencyState {
            version_id: dependency.version_id.clone(),
            project_id: dependency.project_id.clone(),
            file_name: dependency.file_name.clone(),
            dependency_type: dependency_type_cache_key(
                dependency.dependency_type,
            ),
        })
        .collect::<Vec<_>>();
    dependencies.sort();

    let mut game_versions = version
        .game_versions
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();
    game_versions.sort();

    let mut loaders = version
        .loaders
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();
    loaders.sort();

    hash_serializable(&VersionState {
        id: &version.id,
        project_id: &version.project_id,
        date_published: version.date_published.to_rfc3339(),
        dependencies,
        game_versions,
        loaders,
    })
}

fn dependency_type_cache_key(
    dependency_type: modrinth_content_management::DependencyType,
) -> &'static str {
    match dependency_type {
        modrinth_content_management::DependencyType::Required => "required",
        modrinth_content_management::DependencyType::Optional => "optional",
        modrinth_content_management::DependencyType::Incompatible => {
            "incompatible"
        }
        modrinth_content_management::DependencyType::Embedded => "embedded",
    }
}

fn hash_serializable(value: &impl Serialize) -> String {
    let bytes = serde_json::to_vec(value)
        .expect("serializing cache key should not fail");
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

fn parse_version_id(version_id: &str) -> Option<DBVersionId> {
    parse_base62(version_id)
        .ok()
        .map(|id| DBVersionId(id as i64))
}

async fn visible_versions(
    versions: Vec<VersionQueryResult>,
    user_option: &Option<User>,
    pool: &PgPool,
    ro_pool: &ReadOnlyPgPool,
    redis: &RedisPool,
) -> Result<Vec<Version>, ApiError> {
    filter_visible_versions(versions, user_option, pool, ro_pool, redis).await
}

fn version_to_resolver(
    version: Version,
) -> modrinth_content_management::Version {
    let game_versions = version
        .fields
        .get("game_versions")
        .cloned()
        .and_then(|value| serde_json::from_value(value).ok())
        .unwrap_or_default();

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
        game_versions,
        loaders: version.loaders.into_iter().map(|loader| loader.0).collect(),
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
        ResolveError::ProjectNotFound(project_id) => ApiError::Request(
            eyre::eyre!("project `{project_id}` was not found"),
        ),
        ResolveError::VersionNotFound(version_id) => ApiError::Request(
            eyre::eyre!("version `{version_id}` was not found"),
        ),
        ResolveError::VersionProjectMismatch {
            version_id,
            project_id,
        } => ApiError::Request(eyre::eyre!(
            "version `{version_id}` does not belong to project `{project_id}`"
        )),
        ResolveError::NoCompatibleVersion(project_id) => {
            ApiError::Request(eyre::eyre!(
                "no compatible version was found for project `{project_id}`"
            ))
        }
    }
}

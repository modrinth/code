use crate::database::PgPool;
use crate::database::models::DBProjectId;
use crate::env::ENV;
use crate::models::ids::ProjectId;
use crate::util::cors::default_cors;
use crate::util::error::Context;
use actix_cors::Cors;
use actix_files::Files;
use actix_web::http::{StatusCode, header};
use actix_web::{HttpRequest, HttpResponse, web};
use futures::FutureExt;
use std::collections::{HashMap, HashSet};
use utoipa::openapi::extensions::ExtensionsBuilder;
use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};
use xredis::RedisPool;

pub mod debug;
pub mod internal;
pub mod v2;
pub mod v2_reroute;
pub mod v3;

pub mod analytics;
mod index;
mod maven;
mod not_found;
mod updates;

pub use self::not_found::not_found;

const PROJECT_REDIRECTS_NAMESPACE: &str = "project_redirects:v1";
const PROJECT_REDIRECT_CACHE_TTL_SECONDS: i64 = 300;

pub async fn resolve_ref(
    project_ref: &str,
    pool: &PgPool,
    redis: &RedisPool,
) -> Result<Option<ProjectId>, ApiError> {
    let project_refs = [project_ref.to_string()];
    Ok(resolve_refs(&project_refs, pool, redis)
        .await?
        .into_iter()
        .next()
        .flatten())
}

pub async fn resolve_refs(
    project_refs: &[String],
    pool: &PgPool,
    redis: &RedisPool,
) -> Result<Vec<Option<ProjectId>>, ApiError> {
    if project_refs.is_empty() {
        return Ok(Vec::new());
    }

    let mut redis = redis
        .connect()
        .await
        .wrap_internal_err("connecting to Redis for project redirects")?;
    let keys = project_refs
        .iter()
        .map(|project_ref| {
            redis.key().entity(PROJECT_REDIRECTS_NAMESPACE, project_ref)
        })
        .collect::<Vec<_>>();
    let cached_targets = redis
        .get_many_deserialized::<Option<i64>>(&keys)
        .await
        .wrap_internal_err("reading cached project redirects")?;

    let missing_refs = project_refs
        .iter()
        .zip(&cached_targets)
        .filter_map(|(project_ref, cached_target)| {
            cached_target.is_none().then_some(project_ref.clone())
        })
        .collect::<HashSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let normalized_missing_refs = missing_refs
        .iter()
        .map(|project_ref| project_ref.to_lowercase())
        .collect::<Vec<_>>();
    let redirects = if missing_refs.is_empty() {
        HashMap::new()
    } else {
        sqlx::query!(
            r#"
            SELECT
                identifier,
                target_project_id AS "target_project_id: DBProjectId"
            FROM project_redirects
            WHERE identifier = ANY($1) OR identifier = ANY($2)
            "#,
            &missing_refs,
            &normalized_missing_refs,
        )
        .fetch_all(pool)
        .await
        .wrap_internal_err("looking up project redirects")?
        .into_iter()
        .map(|redirect| (redirect.identifier, redirect.target_project_id))
        .collect::<HashMap<_, _>>()
    };

    let mut resolved = Vec::with_capacity(project_refs.len());
    for ((project_ref, key), cached_target) in
        project_refs.iter().zip(&keys).zip(cached_targets)
    {
        let target_project_id = if let Some(cached_target) = cached_target {
            cached_target.map(DBProjectId)
        } else {
            let target_project_id = redirects
                .get(project_ref)
                .or_else(|| redirects.get(&project_ref.to_lowercase()))
                .copied();
            redis
                .set_serialized(
                    key,
                    &target_project_id.map(|project_id| project_id.0),
                    Some(PROJECT_REDIRECT_CACHE_TTL_SECONDS),
                )
                .await
                .wrap_internal_err("caching project redirect")?;
            target_project_id
        };
        resolved.push(target_project_id.map(ProjectId::from));
    }

    Ok(resolved)
}

pub async fn redirect_query_refs(
    req: &HttpRequest,
    parameter_name: &str,
    project_refs: &[String],
    pool: &PgPool,
    redis: &RedisPool,
) -> Result<Option<HttpResponse>, ApiError> {
    let resolved_refs = resolve_refs(project_refs, pool, redis).await?;
    if resolved_refs.iter().all(Option::is_none) {
        return Ok(None);
    }

    let canonical_refs = project_refs
        .iter()
        .zip(resolved_refs)
        .map(|(project_ref, resolved_ref)| {
            resolved_ref.map_or_else(
                || project_ref.clone(),
                |project_id| project_id.to_string(),
            )
        })
        .collect::<Vec<_>>();
    let canonical_refs = serde_json::to_string(&canonical_refs)
        .wrap_internal_err("serializing redirected project references")?;
    let mut query = url::form_urlencoded::Serializer::new(String::new());
    for (name, value) in
        url::form_urlencoded::parse(req.query_string().as_bytes())
    {
        if name == parameter_name {
            query.append_pair(&name, &canonical_refs);
        } else {
            query.append_pair(&name, &value);
        }
    }
    let location = format!("{}?{}", req.path(), query.finish());

    Ok(Some(
        HttpResponse::PermanentRedirect()
            .append_header((header::LOCATION, location))
            .finish(),
    ))
}

pub async fn redirect_ref(
    req: &HttpRequest,
    parameter_name: &str,
    pool: &PgPool,
    redis: &RedisPool,
) -> Result<Option<HttpResponse>, ApiError> {
    let Some(project_ref) = req.match_info().get(parameter_name) else {
        return Ok(None);
    };
    let Some(target_project_id) = resolve_ref(project_ref, pool, redis).await?
    else {
        return Ok(None);
    };

    let Some(route_pattern) = req.match_pattern() else {
        return Ok(None);
    };
    let parameter = format!("{{{parameter_name}}}");
    let constrained_parameter = format!("{{{parameter_name}:");
    let Some(parameter_index) = route_pattern.split('/').position(|segment| {
        segment == parameter || segment.starts_with(&constrained_parameter)
    }) else {
        return Ok(None);
    };
    let mut path_segments = req
        .uri()
        .path()
        .split('/')
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();
    let Some(path_segment) = path_segments.get_mut(parameter_index) else {
        return Ok(None);
    };
    *path_segment = target_project_id.to_string();

    let mut location = path_segments.join("/");
    if let Some(query) = req.uri().query() {
        location.push('?');
        location.push_str(query);
    }

    Ok(Some(
        HttpResponse::PermanentRedirect()
            .append_header((header::LOCATION, location))
            .finish(),
    ))
}

// utoipa-specific struct to use a value_type for docs.
/// A sha1 or sha512 hash.
pub struct FileHash;

impl utoipa::PartialSchema for FileHash {
    fn schema() -> utoipa::openapi::RefOr<utoipa::openapi::schema::Schema> {
        utoipa::openapi::ObjectBuilder::new()
            .schema_type(utoipa::openapi::schema::Type::String)
            .min_length(Some(40))
            .max_length(Some(128))
            .pattern(Some("^([A-Fa-f0-9]{40}|[A-Fa-f0-9]{128})$"))
            .examples([serde_json::json!(
                "2aae6c35c94fcfb415dbe95f408b9ce91ee846ed"
            )])
            .description(Some("A sha1 or sha512 hash."))
            .build()
            .into()
    }
}

impl utoipa::ToSchema for FileHash {}

// utoipa-specific struct to use a value_type for docs.
/// A hashing algorithm (sha1 or sha256)
pub struct HashAlgorithm;

impl utoipa::PartialSchema for HashAlgorithm {
    fn schema() -> utoipa::openapi::RefOr<utoipa::openapi::schema::Schema> {
        utoipa::openapi::ObjectBuilder::new()
            .enum_values(Some([
                serde_json::json!("sha1"),
                serde_json::json!("sha512"),
            ]))
            .examples([serde_json::json!("sha1")])
            .description(Some("A supported hashing algorithm."))
            .build()
            .into()
    }
}

impl utoipa::ToSchema for HashAlgorithm {}

pub(crate) fn prefix_openapi_paths(
    openapi: &mut utoipa::openapi::OpenApi,
    prefix: &str,
    should_skip: impl Fn(&str) -> bool,
) {
    let paths = std::mem::take(&mut openapi.paths.paths);
    openapi.paths.paths = paths
        .into_iter()
        .map(|(path, item)| {
            if should_skip(&path) || path.starts_with(prefix) {
                (path, item)
            } else {
                (format!("{prefix}{}", normalize_openapi_path(&path)), item)
            }
        })
        .collect();
}

fn normalize_openapi_path(path: &str) -> String {
    if path.starts_with('/') {
        path.to_string()
    } else {
        format!("/{path}")
    }
}

pub(crate) struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.as_mut().unwrap();
        let mut bearer_auth = HttpBuilder::new()
			.scheme(HttpAuthScheme::Bearer)
			.description(Some(
				"Use a personal access token. Example: `mrp_RNtLRSPmGj2pd1v1ubi52nX7TJJM9sznrmwhAuj511oe4t1jAqAQ3D6Wc8Ic`.",
			))
			.build();
        bearer_auth.extensions = Some(
			ExtensionsBuilder::new()
				.add(
					"x-example",
					"mrp_RNtLRSPmGj2pd1v1ubi52nX7TJJM9sznrmwhAuj511oe4t1jAqAQ3D6Wc8Ic",
				)
				.build(),
		);

        components.add_security_scheme(
            "bearer_auth",
            SecurityScheme::Http(bearer_auth),
        );
    }
}

pub fn root_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            .wrap(default_cors())
            .wrap_fn(|req, _srv| {
            async {
                Ok(req.into_response(
                    HttpResponse::Gone()
                        .content_type("application/json")
                        .body(r#"{"error":"api_deprecated","description":"You are using an application that uses an outdated version of Modrinth's API. Please either update it or switch to another application. For developers: https://docs.modrinth.com/api/#versioning"}"#)
                ))
            }.boxed_local()
        })
    );
    cfg.service(
        web::scope("")
            .wrap(default_cors())
            .service(index::index_get)
            .service(index::build_get)
            .service(Files::new(
                "/",
                concat!(env!("CARGO_MANIFEST_DIR"), "/assets"),
            )),
    );
}

pub fn public_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/maven")
            .wrap(default_cors())
            .configure(maven::config),
    );
    cfg.service(
        web::scope("/updates")
            .wrap(default_cors())
            .configure(updates::config),
    );
    cfg.service(
        web::scope("/analytics")
            .wrap(
                Cors::default()
                    .allowed_origin_fn(|origin, _req_head| {
                        let allowed_origins = &ENV.ANALYTICS_ALLOWED_ORIGINS;
                        allowed_origins.contains(&"*".to_string())
                            || allowed_origins.contains(
                                &origin
                                    .to_str()
                                    .unwrap_or_default()
                                    .to_string(),
                            )
                    })
                    .allowed_methods(vec!["GET", "POST"])
                    .allowed_headers(vec![
                        actix_web::http::header::AUTHORIZATION,
                        actix_web::http::header::ACCEPT,
                        actix_web::http::header::CONTENT_TYPE,
                    ])
                    .max_age(3600),
            )
            .configure(analytics::config),
    );
}

/// Error when calling an HTTP endpoint.
#[derive(thiserror::Error, Debug)]
pub enum ApiError {
    /// Error occurred on the server side, which the caller has no fault in.
    #[error(transparent)]
    Internal(eyre::Report),
    /// Caller made an invalid or malformed request.
    #[error(transparent)]
    Request(eyre::Report),
    /// Caller attempted a request which they are not allowed to make.
    #[error(transparent)]
    Auth(eyre::Report),
    /// The requested resource does not exist.
    #[error(transparent)]
    NotFound(eyre::Report),
    /// The request conflicts with the current state of the resource.
    #[error(transparent)]
    Conflict(eyre::Report),
    /// A service dependency failed to complete the request.
    #[error(transparent)]
    FailedDependency(eyre::Report),
    /// The request requires a precondition.
    #[error(transparent)]
    PreconditionRequired(eyre::Report),
    /// A request precondition was not met.
    #[error(transparent)]
    PreconditionFailed(eyre::Report),
    /// The caller exceeded a request rate limit.
    #[error(transparent)]
    RateLimit(eyre::Report),
}

impl ApiError {
    /// Adds context to the contained report while preserving the HTTP status.
    pub(crate) fn wrap_err<D>(self, msg: D) -> Self
    where
        D: Send + Sync + std::fmt::Debug + std::fmt::Display + 'static,
    {
        match self {
            Self::Internal(report) => Self::Internal(report.wrap_err(msg)),
            Self::Request(report) => Self::Request(report.wrap_err(msg)),
            Self::Auth(report) => Self::Auth(report.wrap_err(msg)),
            Self::NotFound(report) => Self::NotFound(report.wrap_err(msg)),
            Self::Conflict(report) => Self::Conflict(report.wrap_err(msg)),
            Self::FailedDependency(report) => {
                Self::FailedDependency(report.wrap_err(msg))
            }
            Self::PreconditionRequired(report) => {
                Self::PreconditionRequired(report.wrap_err(msg))
            }
            Self::PreconditionFailed(report) => {
                Self::PreconditionFailed(report.wrap_err(msg))
            }
            Self::RateLimit(report) => Self::RateLimit(report.wrap_err(msg)),
        }
    }

    pub fn as_api_error<'a>(&self) -> crate::models::error::ApiError<'a> {
        let report = match self {
            Self::Internal(report)
            | Self::Request(report)
            | Self::Auth(report)
            | Self::NotFound(report)
            | Self::Conflict(report)
            | Self::FailedDependency(report)
            | Self::PreconditionRequired(report)
            | Self::PreconditionFailed(report)
            | Self::RateLimit(report) => report,
        };
        let details = report
            .chain()
            .skip(1)
            .map(ToString::to_string)
            .collect::<Vec<_>>();

        let validation = report
            .downcast_ref::<v3::projects::validate::ProjectValidationError>();

        crate::models::error::ApiError {
            error: match self {
                Self::Internal(..) => "internal_error",
                Self::Request(..) => "request_error",
                Self::Auth(..) => "auth_error",
                Self::NotFound(..) => "not_found",
                Self::Conflict(..) => "conflict",
                Self::FailedDependency(..) => "failed_dependency",
                Self::PreconditionRequired(..) => "precondition_required",
                Self::PreconditionFailed(..) => "precondition_failed",
                Self::RateLimit(..) => "ratelimit_error",
            },
            description: report.to_string(),
            details: validation
                .map(|error| serde_json::json!({ "nags": error.0 }))
                .or_else(|| {
                    (!details.is_empty()).then(|| serde_json::json!(details))
                }),
        }
    }
}

impl actix_web::ResponseError for ApiError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::Internal(..) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Request(..) => StatusCode::BAD_REQUEST,
            Self::Auth(..) => StatusCode::UNAUTHORIZED,
            Self::NotFound(..) => StatusCode::NOT_FOUND,
            Self::Conflict(..) => StatusCode::CONFLICT,
            Self::FailedDependency(..) => StatusCode::FAILED_DEPENDENCY,
            Self::PreconditionRequired(..) => StatusCode::PRECONDITION_REQUIRED,
            Self::PreconditionFailed(..) => StatusCode::PRECONDITION_FAILED,
            Self::RateLimit(..) => StatusCode::TOO_MANY_REQUESTS,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).json(self.as_api_error())
    }
}

#[cfg(test)]
mod tests {
    use super::ApiError;

    #[test]
    fn api_error_serializes_source_chain_as_details() {
        let error = ApiError::Request(
            eyre::eyre!("root cause")
                .wrap_err("intermediate context")
                .wrap_err("request failed"),
        );

        let response = error.as_api_error();

        assert_eq!(response.description, "request failed");
        assert_eq!(
            response.details,
            Some(serde_json::json!(["intermediate context", "root cause"])),
        );
    }
}

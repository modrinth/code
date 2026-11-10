use xredis::RedisPool;

use crate::auth::checks::is_visible_version;
use crate::database::{PgPool, models::DBVersion};
use crate::models::{exp::minecraft::ServerContentQuery, projects::Project};
use crate::routes::ApiError;
use crate::util::error::Context as _;

use super::{ProjectNag, ProjectNagKind, ProjectNagSeverity};

const MAX_LANGUAGE_COUNT: usize = 10;

pub(super) fn validate(project: &Project) -> Vec<ProjectNag> {
    let mut nags = Vec::new();
    let server = project.components.minecraft_server.as_ref();

    if server.is_some_and(|server| server.region.is_none()) {
        nags.push(ProjectNag::new(
            ProjectNagKind::SelectCountry,
            ProjectNagSeverity::Required,
        ));
    }

    if server.is_some()
        && project
            .components
            .minecraft_java_server
            .as_ref()
            .is_none_or(|server| server.address.is_empty())
    {
        nags.push(ProjectNag::new(
            ProjectNagKind::AddJavaAddress,
            ProjectNagSeverity::Required,
        ));
    }

    if project
        .components
        .minecraft_java_server
        .as_ref()
        .is_some_and(|server| {
            matches!(
                &server.content,
                ServerContentQuery::Vanilla {
                    recommended_game_version,
                    ..
                } if recommended_game_version
                    .as_deref()
                    .is_none_or(str::is_empty)
            )
        })
    {
        nags.push(ProjectNag::new(
            ProjectNagKind::SelectCompatibility,
            ProjectNagSeverity::Required,
        ));
    }

    if let Some(language_count) = server
        .map(|server| server.languages.len())
        .filter(|language_count| *language_count > MAX_LANGUAGE_COUNT)
    {
        nags.push(
            ProjectNag::new(
                ProjectNagKind::TooManyLanguages,
                ProjectNagSeverity::Warning,
            )
            .with_details(serde_json::json!({
                "language_count": language_count,
            })),
        );
    }

    if server.is_some_and(|server| server.languages.is_empty()) {
        nags.push(ProjectNag::new(
            ProjectNagKind::SelectLanguage,
            ProjectNagSeverity::Suggestion,
        ));
    }

    nags
}

pub(super) async fn validate_server_modpack(
    project: &Project,
    pool: &PgPool,
    redis: &RedisPool,
) -> Result<Vec<ProjectNag>, ApiError> {
    let Some(server) = &project.components.minecraft_java_server else {
        return Ok(Vec::new());
    };
    let ServerContentQuery::Modpack {
        version_id,
        project_id,
        ..
    } = &server.content
    else {
        return Ok(Vec::new());
    };
    if *project_id == project.id {
        return Ok(Vec::new());
    }
    let version = DBVersion::get((*version_id).into(), pool, redis)
        .await
        .wrap_internal_err("fetching server modpack version")?;
    let visible = if let Some(version) = version {
        is_visible_version(&version.inner, &None, pool, redis).await?
    } else {
        false
    };
    Ok(if visible {
        Vec::new()
    } else {
        vec![ProjectNag {
            kind: ProjectNagKind::ModpackVersionNotPublic,
            severity: ProjectNagSeverity::Required,
            details: serde_json::json!({

                "version_id": version_id,
            }),
        }]
    })
}

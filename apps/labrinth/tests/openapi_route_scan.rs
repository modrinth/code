use std::collections::BTreeSet;

use utoipa::OpenApi;

#[derive(OpenApi)]
struct DocsV3;

#[derive(OpenApi)]
struct DocsInternal;

fn collect_v3_paths() -> BTreeSet<String> {
    let docs = DocsV3::openapi()
        .merge_from(labrinth::routes::PublicApiDoc::openapi())
        .merge_from(labrinth::routes::v3::ApiDoc::openapi());

    docs.paths.paths.keys().cloned().collect()
}

fn collect_internal_paths() -> BTreeSet<String> {
    let docs = DocsInternal::openapi()
        .merge_from(labrinth::routes::internal::ApiDoc::openapi());

    docs.paths.paths.keys().cloned().collect()
}

fn assert_paths(paths: &BTreeSet<String>, expected: &[&str]) {
    let missing = expected
        .iter()
        .filter(|path| !paths.contains(**path))
        .copied()
        .collect::<Vec<_>>();

    assert!(
        missing.is_empty(),
        "missing OpenAPI paths: {missing:#?}\nregistered paths:\n{}",
        paths.iter().cloned().collect::<Vec<_>>().join("\n")
    );
}

#[test]
fn v3_openapi_includes_configured_routes() {
    let paths = collect_v3_paths();

    assert_paths(
        &paths,
        &[
            "/analytics/minecraft-server-play",
            "/analytics/playtime",
            "/analytics/view",
            "/maven/maven/modrinth/{id}/maven-metadata.xml",
            "/maven/maven/modrinth/{id}/{versionnum}/{file}",
            "/updates/{id}/forge_updates.json",
            "/v3/analytics",
            "/v3/analytics/facets",
            "/v3/collection",
            "/v3/collection/{id}",
            "/v3/content/resolve",
            "/v3/games",
            "/v3/image",
            "/v3/limits/collections",
            "/v3/notifications",
            "/v3/oauth/app",
            "/v3/organization",
            "/v3/organization/{id}",
            "/v3/project",
            "/v3/project/{id}",
            "/v3/project/{id}/gallery",
            "/v3/project/{project_id}/dependencies",
            "/v3/report",
            "/v3/reports",
            "/v3/shared-instance",
            "/v3/shared-instance/{id}/version",
            "/v3/statistics",
            "/v3/tag/category",
            "/v3/team/{id}/members",
            "/v3/thread/{id}",
            "/v3/user/{id}",
            "/v3/users",
            "/v3/version",
            "/v3/version/{id}",
            "/v3/version/{version_id}/file",
            "/v3/version_file/{version_id}",
            "/v3/version_files",
        ],
    );
}

#[test]
fn internal_openapi_includes_collapsed_route_groups() {
    let paths = collect_internal_paths();

    assert_paths(
        &paths,
        &[
            "/_internal/admin/_count-download",
            "/_internal/affiliate",
            "/_internal/attribution/assign",
            "/_internal/auth/init",
            "/_internal/billing/charge/{id}/refund",
            "/_internal/delphi/ingest",
            "/_internal/external_notifications",
            "/_internal/gdpr/export",
            "/_internal/globals",
            "/_internal/gotenberg/success",
            "/_internal/medal/verify",
            "/_internal/moderation/project/{id}",
            "/_internal/moderation/tech-review/report/{id}",
            "/_internal/mural/bank-details",
            "/_internal/pat",
            "/_internal/search-management/tasks",
            "/_internal/server-ping/minecraft-java",
            "/_internal/session/list",
            "/_internal/launcher_socket",
            "/v3/analytics-event",
        ],
    );
}

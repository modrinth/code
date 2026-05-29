use super::super::all_download_sources;
use super::{
    AnalyticsFacets, FacetValue, ProjectDownloadsFacets, ProjectPlaytimeFacets,
    ProjectViewsFacets,
};
use crate::{
    database::{PgPool, redis::RedisPool},
    models::{users::User, v3::analytics::DownloadReason},
    routes::ApiError,
    util::tags::valid_download_tags,
};

pub async fn fetch(
    _req: &super::super::GetRequest,
    _user: &User,
    pool: &PgPool,
    redis: &RedisPool,
) -> Result<AnalyticsFacets, ApiError> {
    let tags = valid_download_tags(pool, redis).await?;
    let mut loaders = tags.loaders.iter().cloned().collect::<Vec<_>>();
    loaders.sort();
    let mut game_versions =
        tags.game_versions.iter().cloned().collect::<Vec<_>>();
    game_versions.sort();

    let loader_facets = string_facets(loaders);
    let game_version_facets = string_facets(game_versions);
    let country_facets = country_facets();

    Ok(AnalyticsFacets {
        project_views: ProjectViewsFacets {
            domain: Vec::new(),
            site_path: Vec::new(),
            monetized: bool_facets(),
            country: country_facets.clone(),
        },
        project_downloads: ProjectDownloadsFacets {
            domain: Vec::new(),
            user_agent: download_source_facets(),
            version_id: Vec::new(),
            monetized: bool_facets(),
            country: country_facets.clone(),
            reason: download_reason_facets(),
            game_version: game_version_facets.clone(),
            loader: loader_facets.clone(),
        },
        project_playtime: ProjectPlaytimeFacets {
            version_id: Vec::new(),
            loader: loader_facets,
            game_version: game_version_facets,
            country: country_facets,
        },
    })
}

fn bool_facets() -> Vec<FacetValue<bool>> {
    vec![
        FacetValue {
            value: false,
            count: 0,
        },
        FacetValue {
            value: true,
            count: 0,
        },
    ]
}

fn download_reason_facets() -> Vec<FacetValue<DownloadReason>> {
    [
        DownloadReason::Standalone,
        DownloadReason::Dependency,
        DownloadReason::Modpack,
        DownloadReason::Update,
    ]
    .into_iter()
    .map(|value| FacetValue { value, count: 0 })
    .collect()
}

fn download_source_facets() -> Vec<FacetValue<super::super::DownloadSource>> {
    all_download_sources()
        .into_iter()
        .map(|value| FacetValue { value, count: 0 })
        .collect()
}

fn country_facets() -> Vec<FacetValue<String>> {
    let mut countries = rust_iso3166::ALL_ALPHA2
        .iter()
        .map(|country| country.to_string())
        .collect::<Vec<_>>();
    countries.push("XX".to_string());
    countries.sort();
    string_facets(countries)
}

fn string_facets(
    values: impl IntoIterator<Item = String>,
) -> Vec<FacetValue<String>> {
    values
        .into_iter()
        .map(|value| FacetValue { value, count: 0 })
        .collect()
}

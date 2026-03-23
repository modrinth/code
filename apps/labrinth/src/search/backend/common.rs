use crate::routes::ApiError;
use crate::search::SearchRequest;
use crate::util::error::Context;
use eyre::eyre;
use std::borrow::Cow;

pub struct ParsedSearchRequest<'a> {
    pub offset: usize,
    pub hits_per_page: usize,
    pub page: usize,
    pub index: &'a str,
    pub query: &'a str,
}

pub fn parse_search_request(
    info: &SearchRequest,
) -> Result<ParsedSearchRequest<'_>, ApiError> {
    let offset = info
        .offset
        .as_deref()
        .unwrap_or("0")
        .parse::<usize>()
        .wrap_request_err("invalid offset")?;
    let limit = info
        .limit
        .as_deref()
        .unwrap_or("10")
        .parse::<usize>()
        .wrap_request_err("invalid limit")?
        .min(100);
    let hits_per_page = if limit == 0 { 1 } else { limit };

    Ok(ParsedSearchRequest {
        offset,
        hits_per_page,
        page: offset / hits_per_page + 1,
        index: info.index.as_deref().unwrap_or("relevance"),
        query: info.query.as_deref().unwrap_or_default(),
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchIndex {
    Relevance,
    Downloads,
    Follows,
    Updated,
    Newest,
    MinecraftJavaServerVerifiedPlays2w,
    MinecraftJavaServerPlayersOnline,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchIndexName {
    Projects,
    ProjectsFiltered,
}

pub struct SearchSort {
    pub index_name: SearchIndexName,
    pub index: SearchIndex,
}

pub fn parse_search_index(
    index: &str,
    new_filters: Option<&str>,
) -> Result<SearchSort, ApiError> {
    let projects_name = SearchIndexName::Projects;
    let projects_filtered_name = SearchIndexName::ProjectsFiltered;

    // TODO: this is a dumb hack, the frontend should pass the project type it's filtering directly
    let is_server = new_filters
        .is_some_and(|f| f.contains("project_types = minecraft_java_server"));

    Ok(match index {
        "relevance" => SearchSort {
            index_name: projects_name,
            index: if is_server {
                SearchIndex::MinecraftJavaServerVerifiedPlays2w
            } else {
                SearchIndex::Relevance
            },
        },
        "downloads" => SearchSort {
            index_name: projects_filtered_name,
            index: SearchIndex::Downloads,
        },
        "follows" => SearchSort {
            index_name: projects_name,
            index: SearchIndex::Follows,
        },
        "updated" | "date_modified" => SearchSort {
            index_name: projects_name,
            index: SearchIndex::Updated,
        },
        "newest" | "date_created" => SearchSort {
            index_name: projects_name,
            index: SearchIndex::Newest,
        },
        "minecraft_java_server.verified_plays_2w" => SearchSort {
            index_name: projects_name,
            index: SearchIndex::MinecraftJavaServerVerifiedPlays2w,
        },
        "minecraft_java_server.ping.data.players_online" => SearchSort {
            index_name: projects_name,
            index: SearchIndex::MinecraftJavaServerPlayersOnline,
        },
        i => return Err(ApiError::Request(eyre!("invalid index '{i}'"))),
    })
}

pub fn combined_search_filters(info: &SearchRequest) -> Option<Cow<'_, str>> {
    if let Some(filters) = info.new_filters.as_deref() {
        return Some(filters.into());
    }

    match (info.filters.as_deref(), info.version.as_deref()) {
        (Some(f), Some(v)) => Some(format!("({f}) AND ({v})").into()),
        (Some(f), None) => Some(f.into()),
        (None, Some(v)) => Some(v.into()),
        (None, None) => None,
    }
}

use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::{routes::v2_reroute, search::ResultSearchProject};

#[derive(Serialize, Deserialize, Debug)]
pub struct LegacySearchResults {
    pub hits: Vec<LegacyResultSearchProject>,
    pub offset: usize,
    pub limit: usize,
    pub total_hits: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LegacyResultSearchProject {
    pub project_id: String,
    pub project_type: String,
    pub slug: Option<String>,
    pub author: String,
    pub title: String,
    pub description: String,
    pub categories: Vec<String>,
    pub display_categories: Vec<String>,
    pub versions: Vec<String>,
    pub downloads: i32,
    pub follows: i32,
    pub icon_url: String,
    /// RFC 3339 formatted creation date of the project
    pub date_created: String,
    /// RFC 3339 formatted modification date of the project
    pub date_modified: String,
    pub latest_version: String,
    pub license: String,
    pub client_side: String,
    pub server_side: String,
    pub gallery: Vec<String>,
    pub featured_gallery: Option<String>,
    pub color: Option<u32>,
}

// TODO: In other PR, when these are merged, make sure the v2 search testing functions use these
impl LegacyResultSearchProject {
    pub fn from(result_search_project: ResultSearchProject) -> Self {
        let mut categories = result_search_project.categories;
        categories.extend(result_search_project.loaders.clone());
        if categories.contains(&"mrpack".to_string()) {
            if let Some(mrpack_loaders) = result_search_project
                .project_loader_fields
                .get("mrpack_loaders")
            {
                categories.extend(
                    mrpack_loaders
                        .iter()
                        .filter_map(|c| c.as_str())
                        .map(String::from),
                );
                categories.retain(|c| c != "mrpack");
            }
        }
        let mut display_categories = result_search_project.display_categories;
        display_categories.extend(result_search_project.loaders);
        if display_categories.contains(&"mrpack".to_string()) {
            if let Some(mrpack_loaders) = result_search_project
                .project_loader_fields
                .get("mrpack_loaders")
            {
                categories.extend(
                    mrpack_loaders
                        .iter()
                        .filter_map(|c| c.as_str())
                        .map(String::from),
                );
                display_categories.retain(|c| c != "mrpack");
            }
        }

        // Sort then remove duplicates
        categories.sort();
        categories.dedup();
        display_categories.sort();
        display_categories.dedup();

        // V2 versions only have one project type- v3 versions can rarely have multiple.
        // We'll prioritize 'modpack' first, and if neither are found, use the first one.
        // If there are no project types, default to 'project'
        let mut project_types = result_search_project.project_types;
        if project_types.contains(&"modpack".to_string()) {
            project_types = vec!["modpack".to_string()];
        }
        let og_project_type = project_types
            .first()
            .cloned()
            .unwrap_or("project".to_string()); // Default to 'project' if none are found

        let project_type =
            if og_project_type == "datapack" || og_project_type == "plugin" {
                // These are not supported in V2, so we'll just use 'mod' instead
                "mod".to_string()
            } else {
                og_project_type.clone()
            };

        let project_loader_fields =
            result_search_project.project_loader_fields.clone();
        let get_one_string_loader_field = |key: &str| {
            project_loader_fields
                .get(key)
                .map_or(&[][..], |values| values.as_slice())
                .first()
                .and_then(|s| s.as_str())
        };

        let environment =
            get_one_string_loader_field("environment").unwrap_or("unknown");

        let (client_side, server_side) =
            v2_reroute::convert_v3_environment_to_v2_side_types(
                environment,
                Some(&*og_project_type),
            );
        let client_side = client_side.to_string();
        let server_side = server_side.to_string();

        let versions = result_search_project
            .project_loader_fields
            .get("game_versions")
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .filter_map(|s| s.as_str().map(String::from))
            .collect_vec();

        Self {
            project_type,
            client_side,
            server_side,
            versions,
            latest_version: result_search_project.version_id,
            categories,

            project_id: result_search_project.project_id,
            slug: result_search_project.slug,
            author: result_search_project.author,
            title: result_search_project.name,
            description: result_search_project.summary,
            display_categories,
            downloads: result_search_project.downloads,
            follows: result_search_project.follows,
            icon_url: result_search_project.icon_url.unwrap_or_default(),
            license: result_search_project.license,
            date_created: result_search_project.date_created,
            date_modified: result_search_project.date_modified,
            gallery: result_search_project.gallery,
            featured_gallery: result_search_project.featured_gallery,
            color: result_search_project.color,
        }
    }
}

impl LegacySearchResults {
    pub fn from(search_results: crate::search::SearchResults) -> Self {
        let limit = search_results.hits_per_page;
        let offset = (search_results.page - 1) * limit;
        Self {
            hits: search_results
                .hits
                .into_iter()
                .map(LegacyResultSearchProject::from)
                .collect(),
            offset,
            limit,
            total_hits: search_results.total_hits,
        }
    }
}

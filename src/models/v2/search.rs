use serde::{Deserialize, Serialize};

use crate::search::ResultSearchProject;

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
        if categories.contains(&"mrpack".to_string()) {
            if let Some(mrpack_loaders) = result_search_project.loader_fields.get("mrpack_loaders")
            {
                categories.extend(mrpack_loaders.clone());
                categories.retain(|c| c != "mrpack");
            }
        }
        let mut display_categories = result_search_project.display_categories;
        if display_categories.contains(&"mrpack".to_string()) {
            if let Some(mrpack_loaders) = result_search_project.loader_fields.get("mrpack_loaders")
            {
                display_categories.extend(mrpack_loaders.clone());
                display_categories.retain(|c| c != "mrpack");
            }
        }

        // Sort then remove duplicates
        categories.sort();
        categories.dedup();
        display_categories.sort();
        display_categories.dedup();

        Self {
            project_type: result_search_project
                .project_types
                .first()
                .cloned()
                .unwrap_or_default(),
            client_side: result_search_project
                .loader_fields
                .get("client_side")
                .cloned()
                .unwrap_or_default()
                .join(","),
            server_side: result_search_project
                .loader_fields
                .get("server_side")
                .cloned()
                .unwrap_or_default()
                .join(","),
            versions: result_search_project
                .loader_fields
                .get("game_versions")
                .cloned()
                .unwrap_or_default(),
            latest_version: result_search_project.version_id,
            categories,

            project_id: result_search_project.project_id,
            slug: result_search_project.slug,
            author: result_search_project.author,
            title: result_search_project.title,
            description: result_search_project.description,
            display_categories,
            downloads: result_search_project.downloads,
            follows: result_search_project.follows,
            icon_url: result_search_project.icon_url,
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
        Self {
            hits: search_results
                .hits
                .into_iter()
                .map(LegacyResultSearchProject::from)
                .collect(),
            offset: search_results.offset,
            limit: search_results.limit,
            total_hits: search_results.total_hits,
        }
    }
}

use std::collections::HashMap;

use self::models::{
    CommonCategoryData, CommonImageData, CommonLoaderData, CommonNotification, CommonProject,
    CommonTeamMember, CommonVersion,
};
use actix_web::dev::ServiceResponse;
use async_trait::async_trait;
use labrinth::{
    models::{
        projects::{ProjectId, VersionType},
        teams::{OrganizationPermissions, ProjectPermissions},
    },
    search::SearchResults,
    LabrinthConfig,
};

use super::dummy_data::TestFile;

pub mod generic;
pub mod models;
#[async_trait(?Send)]
pub trait ApiBuildable: Api {
    async fn build(labrinth_config: LabrinthConfig) -> Self;
}

#[async_trait(?Send)]
pub trait Api: ApiProject + ApiTags + ApiTeams + ApiVersion {
    async fn call(&self, req: actix_http::Request) -> ServiceResponse;
    async fn reset_search_index(&self) -> ServiceResponse;
}

#[async_trait(?Send)]
pub trait ApiProject {
    async fn add_public_project(
        &self,
        slug: &str,
        version_jar: Option<TestFile>,
        modify_json: Option<json_patch::Patch>,
        pat: &str,
    ) -> (CommonProject, Vec<CommonVersion>);
    async fn remove_project(&self, id_or_slug: &str, pat: &str) -> ServiceResponse;
    async fn get_project(&self, id_or_slug: &str, pat: &str) -> ServiceResponse;
    async fn get_project_deserialized_common(&self, id_or_slug: &str, pat: &str) -> CommonProject;
    async fn get_user_projects(&self, user_id_or_username: &str, pat: &str) -> ServiceResponse;
    async fn get_user_projects_deserialized_common(
        &self,
        user_id_or_username: &str,
        pat: &str,
    ) -> Vec<CommonProject>;
    async fn edit_project(
        &self,
        id_or_slug: &str,
        patch: serde_json::Value,
        pat: &str,
    ) -> ServiceResponse;
    async fn edit_project_bulk(
        &self,
        ids_or_slugs: &[&str],
        patch: serde_json::Value,
        pat: &str,
    ) -> ServiceResponse;
    async fn edit_project_icon(
        &self,
        id_or_slug: &str,
        icon: Option<CommonImageData>,
        pat: &str,
    ) -> ServiceResponse;
    async fn search_deserialized_common(
        &self,
        query: Option<&str>,
        facets: Option<serde_json::Value>,
        pat: &str,
    ) -> SearchResults;
}

#[async_trait(?Send)]
pub trait ApiTags {
    async fn get_loaders(&self) -> ServiceResponse;
    async fn get_loaders_deserialized_common(&self) -> Vec<CommonLoaderData>;
    async fn get_categories(&self) -> ServiceResponse;
    async fn get_categories_deserialized_common(&self) -> Vec<CommonCategoryData>;
}

#[async_trait(?Send)]
pub trait ApiTeams {
    async fn get_team_members(&self, team_id: &str, pat: &str) -> ServiceResponse;
    async fn get_team_members_deserialized_common(
        &self,
        team_id: &str,
        pat: &str,
    ) -> Vec<CommonTeamMember>;
    async fn get_project_members(&self, id_or_slug: &str, pat: &str) -> ServiceResponse;
    async fn get_project_members_deserialized_common(
        &self,
        id_or_slug: &str,
        pat: &str,
    ) -> Vec<CommonTeamMember>;
    async fn get_organization_members(&self, id_or_title: &str, pat: &str) -> ServiceResponse;
    async fn get_organization_members_deserialized_common(
        &self,
        id_or_title: &str,
        pat: &str,
    ) -> Vec<CommonTeamMember>;
    async fn join_team(&self, team_id: &str, pat: &str) -> ServiceResponse;
    async fn remove_from_team(&self, team_id: &str, user_id: &str, pat: &str) -> ServiceResponse;
    async fn edit_team_member(
        &self,
        team_id: &str,
        user_id: &str,
        patch: serde_json::Value,
        pat: &str,
    ) -> ServiceResponse;
    async fn transfer_team_ownership(
        &self,
        team_id: &str,
        user_id: &str,
        pat: &str,
    ) -> ServiceResponse;
    async fn get_user_notifications(&self, user_id: &str, pat: &str) -> ServiceResponse;
    async fn get_user_notifications_deserialized_common(
        &self,
        user_id: &str,
        pat: &str,
    ) -> Vec<CommonNotification>;
    async fn mark_notification_read(&self, notification_id: &str, pat: &str) -> ServiceResponse;
    async fn add_user_to_team(
        &self,
        team_id: &str,
        user_id: &str,
        project_permissions: Option<ProjectPermissions>,
        organization_permissions: Option<OrganizationPermissions>,
        pat: &str,
    ) -> ServiceResponse;
    async fn delete_notification(&self, notification_id: &str, pat: &str) -> ServiceResponse;
}

#[async_trait(?Send)]
pub trait ApiVersion {
    async fn add_public_version(
        &self,
        project_id: ProjectId,
        version_number: &str,
        version_jar: TestFile,
        ordering: Option<i32>,
        modify_json: Option<json_patch::Patch>,
        pat: &str,
    ) -> ServiceResponse;
    async fn add_public_version_deserialized_common(
        &self,
        project_id: ProjectId,
        version_number: &str,
        version_jar: TestFile,
        ordering: Option<i32>,
        modify_json: Option<json_patch::Patch>,
        pat: &str,
    ) -> CommonVersion;
    async fn get_version(&self, id_or_slug: &str, pat: &str) -> ServiceResponse;
    async fn get_version_deserialized_common(&self, id_or_slug: &str, pat: &str) -> CommonVersion;
    async fn get_versions(&self, ids_or_slugs: Vec<String>, pat: &str) -> ServiceResponse;
    async fn get_versions_deserialized_common(
        &self,
        ids_or_slugs: Vec<String>,
        pat: &str,
    ) -> Vec<CommonVersion>;
    async fn edit_version(
        &self,
        id_or_slug: &str,
        patch: serde_json::Value,
        pat: &str,
    ) -> ServiceResponse;
    async fn get_version_from_hash(
        &self,
        id_or_slug: &str,
        hash: &str,
        pat: &str,
    ) -> ServiceResponse;
    async fn get_version_from_hash_deserialized_common(
        &self,
        id_or_slug: &str,
        hash: &str,
        pat: &str,
    ) -> CommonVersion;
    async fn get_versions_from_hashes(
        &self,
        hashes: &[&str],
        algorithm: &str,
        pat: &str,
    ) -> ServiceResponse;
    async fn get_versions_from_hashes_deserialized_common(
        &self,
        hashes: &[&str],
        algorithm: &str,
        pat: &str,
    ) -> HashMap<String, CommonVersion>;
    async fn get_update_from_hash(
        &self,
        hash: &str,
        algorithm: &str,
        loaders: Option<Vec<String>>,
        game_versions: Option<Vec<String>>,
        version_types: Option<Vec<String>>,
        pat: &str,
    ) -> ServiceResponse;
    async fn get_update_from_hash_deserialized_common(
        &self,
        hash: &str,
        algorithm: &str,
        loaders: Option<Vec<String>>,
        game_versions: Option<Vec<String>>,
        version_types: Option<Vec<String>>,
        pat: &str,
    ) -> CommonVersion;
    async fn update_files(
        &self,
        algorithm: &str,
        hashes: Vec<String>,
        loaders: Option<Vec<String>>,
        game_versions: Option<Vec<String>>,
        version_types: Option<Vec<String>>,
        pat: &str,
    ) -> ServiceResponse;
    async fn update_files_deserialized_common(
        &self,
        algorithm: &str,
        hashes: Vec<String>,
        loaders: Option<Vec<String>>,
        game_versions: Option<Vec<String>>,
        version_types: Option<Vec<String>>,
        pat: &str,
    ) -> HashMap<String, CommonVersion>;
    #[allow(clippy::too_many_arguments)]
    async fn get_project_versions(
        &self,
        project_id_slug: &str,
        game_versions: Option<Vec<String>>,
        loaders: Option<Vec<String>>,
        featured: Option<bool>,
        version_type: Option<VersionType>,
        limit: Option<usize>,
        offset: Option<usize>,
        pat: &str,
    ) -> ServiceResponse;
    #[allow(clippy::too_many_arguments)]
    async fn get_project_versions_deserialized_common(
        &self,
        slug: &str,
        game_versions: Option<Vec<String>>,
        loaders: Option<Vec<String>>,
        featured: Option<bool>,
        version_type: Option<VersionType>,
        limit: Option<usize>,
        offset: Option<usize>,
        pat: &str,
    ) -> Vec<CommonVersion>;
    async fn edit_version_ordering(
        &self,
        version_id: &str,
        ordering: Option<i32>,
        pat: &str,
    ) -> ServiceResponse;
}

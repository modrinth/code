use std::collections::HashMap;

use self::models::{
    CommonCategoryData, CommonItemType, CommonLoaderData, CommonNotification,
    CommonProject, CommonTeamMember, CommonVersion,
};
use self::request_data::{ImageData, ProjectCreationRequestData};
use super::dummy_data::TestFile;
use actix_web::dev::ServiceResponse;
use async_trait::async_trait;
use labrinth::models::ids::ProjectId;
use labrinth::{
    LabrinthConfig,
    models::{
        projects::VersionType,
        teams::{OrganizationPermissions, ProjectPermissions},
    },
};

pub mod generic;
pub mod models;
pub mod request_data;
#[async_trait(?Send)]
pub trait ApiBuildable: Api {
    async fn build(labrinth_config: LabrinthConfig) -> Self;
}

#[async_trait(?Send)]
pub trait Api: ApiProject + ApiTags + ApiTeams + ApiUser + ApiVersion {
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
        pat: Option<&str>,
    ) -> (CommonProject, Vec<CommonVersion>);
    async fn create_project(
        &self,
        creation_data: ProjectCreationRequestData,
        pat: Option<&str>,
    ) -> ServiceResponse;
    async fn get_public_project_creation_data_json(
        &self,
        slug: &str,
        version_jar: Option<&TestFile>,
    ) -> serde_json::Value;

    async fn remove_project(
        &self,
        id_or_slug: &str,
        pat: Option<&str>,
    ) -> ServiceResponse;
    async fn get_project(
        &self,
        id_or_slug: &str,
        pat: Option<&str>,
    ) -> ServiceResponse;
    async fn get_project_deserialized_common(
        &self,
        id_or_slug: &str,
        pat: Option<&str>,
    ) -> CommonProject;
    async fn get_projects(
        &self,
        ids_or_slugs: &[&str],
        pat: Option<&str>,
    ) -> ServiceResponse;
    async fn get_project_dependencies(
        &self,
        id_or_slug: &str,
        pat: Option<&str>,
    ) -> ServiceResponse;
    async fn get_user_projects(
        &self,
        user_id_or_username: &str,
        pat: Option<&str>,
    ) -> ServiceResponse;
    async fn get_user_projects_deserialized_common(
        &self,
        user_id_or_username: &str,
        pat: Option<&str>,
    ) -> Vec<CommonProject>;
    async fn edit_project(
        &self,
        id_or_slug: &str,
        patch: serde_json::Value,
        pat: Option<&str>,
    ) -> ServiceResponse;
    async fn edit_project_bulk(
        &self,
        ids_or_slugs: &[&str],
        patch: serde_json::Value,
        pat: Option<&str>,
    ) -> ServiceResponse;
    async fn edit_project_icon(
        &self,
        id_or_slug: &str,
        icon: Option<ImageData>,
        pat: Option<&str>,
    ) -> ServiceResponse;
    #[allow(clippy::too_many_arguments)]
    async fn add_gallery_item(
        &self,
        id_or_slug: &str,
        image: ImageData,
        featured: bool,
        title: Option<String>,
        description: Option<String>,
        ordering: Option<i32>,
        pat: Option<&str>,
    ) -> ServiceResponse;
    async fn remove_gallery_item(
        &self,
        id_or_slug: &str,
        url: &str,
        pat: Option<&str>,
    ) -> ServiceResponse;
    async fn edit_gallery_item(
        &self,
        id_or_slug: &str,
        url: &str,
        patch: HashMap<String, String>,
        pat: Option<&str>,
    ) -> ServiceResponse;
    async fn create_report(
        &self,
        report_type: &str,
        id: &str,
        item_type: CommonItemType,
        body: &str,
        pat: Option<&str>,
    ) -> ServiceResponse;
    async fn get_report(&self, id: &str, pat: Option<&str>) -> ServiceResponse;
    async fn get_reports(
        &self,
        ids: &[&str],
        pat: Option<&str>,
    ) -> ServiceResponse;
    async fn get_user_reports(&self, pat: Option<&str>) -> ServiceResponse;
    async fn edit_report(
        &self,
        id: &str,
        patch: serde_json::Value,
        pat: Option<&str>,
    ) -> ServiceResponse;
    async fn delete_report(
        &self,
        id: &str,
        pat: Option<&str>,
    ) -> ServiceResponse;
    async fn get_thread(&self, id: &str, pat: Option<&str>) -> ServiceResponse;
    async fn get_threads(
        &self,
        ids: &[&str],
        pat: Option<&str>,
    ) -> ServiceResponse;
    async fn write_to_thread(
        &self,
        id: &str,
        r#type: &str,
        message: &str,
        pat: Option<&str>,
    ) -> ServiceResponse;
    async fn get_moderation_inbox(&self, pat: Option<&str>) -> ServiceResponse;
    async fn read_thread(&self, id: &str, pat: Option<&str>)
    -> ServiceResponse;
    async fn delete_thread_message(
        &self,
        id: &str,
        pat: Option<&str>,
    ) -> ServiceResponse;
}

#[async_trait(?Send)]
pub trait ApiTags {
    async fn get_loaders(&self) -> ServiceResponse;
    async fn get_loaders_deserialized_common(&self) -> Vec<CommonLoaderData>;
    async fn get_categories(&self) -> ServiceResponse;
    async fn get_categories_deserialized_common(
        &self,
    ) -> Vec<CommonCategoryData>;
}

#[async_trait(?Send)]
pub trait ApiTeams {
    async fn get_team_members(
        &self,
        team_id: &str,
        pat: Option<&str>,
    ) -> ServiceResponse;
    async fn get_team_members_deserialized_common(
        &self,
        team_id: &str,
        pat: Option<&str>,
    ) -> Vec<CommonTeamMember>;
    async fn get_teams_members(
        &self,
        team_ids: &[&str],
        pat: Option<&str>,
    ) -> ServiceResponse;
    async fn get_project_members(
        &self,
        id_or_slug: &str,
        pat: Option<&str>,
    ) -> ServiceResponse;
    async fn get_project_members_deserialized_common(
        &self,
        id_or_slug: &str,
        pat: Option<&str>,
    ) -> Vec<CommonTeamMember>;
    async fn get_organization_members(
        &self,
        id_or_title: &str,
        pat: Option<&str>,
    ) -> ServiceResponse;
    async fn get_organization_members_deserialized_common(
        &self,
        id_or_title: &str,
        pat: Option<&str>,
    ) -> Vec<CommonTeamMember>;
    async fn join_team(
        &self,
        team_id: &str,
        pat: Option<&str>,
    ) -> ServiceResponse;
    async fn remove_from_team(
        &self,
        team_id: &str,
        user_id: &str,
        pat: Option<&str>,
    ) -> ServiceResponse;
    async fn edit_team_member(
        &self,
        team_id: &str,
        user_id: &str,
        patch: serde_json::Value,
        pat: Option<&str>,
    ) -> ServiceResponse;
    async fn transfer_team_ownership(
        &self,
        team_id: &str,
        user_id: &str,
        pat: Option<&str>,
    ) -> ServiceResponse;
    async fn get_user_notifications(
        &self,
        user_id: &str,
        pat: Option<&str>,
    ) -> ServiceResponse;
    async fn get_user_notifications_deserialized_common(
        &self,
        user_id: &str,
        pat: Option<&str>,
    ) -> Vec<CommonNotification>;
    async fn get_notification(
        &self,
        notification_id: &str,
        pat: Option<&str>,
    ) -> ServiceResponse;
    async fn get_notifications(
        &self,
        ids: &[&str],
        pat: Option<&str>,
    ) -> ServiceResponse;
    async fn mark_notification_read(
        &self,
        notification_id: &str,
        pat: Option<&str>,
    ) -> ServiceResponse;
    async fn mark_notifications_read(
        &self,
        ids: &[&str],
        pat: Option<&str>,
    ) -> ServiceResponse;
    async fn add_user_to_team(
        &self,
        team_id: &str,
        user_id: &str,
        project_permissions: Option<ProjectPermissions>,
        organization_permissions: Option<OrganizationPermissions>,
        pat: Option<&str>,
    ) -> ServiceResponse;
    async fn delete_notification(
        &self,
        notification_id: &str,
        pat: Option<&str>,
    ) -> ServiceResponse;
    async fn delete_notifications(
        &self,
        ids: &[&str],
        pat: Option<&str>,
    ) -> ServiceResponse;
}

#[async_trait(?Send)]
pub trait ApiUser {
    async fn get_user(
        &self,
        id_or_username: &str,
        pat: Option<&str>,
    ) -> ServiceResponse;
    async fn get_current_user(&self, pat: Option<&str>) -> ServiceResponse;
    async fn edit_user(
        &self,
        id_or_username: &str,
        patch: serde_json::Value,
        pat: Option<&str>,
    ) -> ServiceResponse;
    async fn delete_user(
        &self,
        id_or_username: &str,
        pat: Option<&str>,
    ) -> ServiceResponse;
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
        pat: Option<&str>,
    ) -> ServiceResponse;
    async fn add_public_version_deserialized_common(
        &self,
        project_id: ProjectId,
        version_number: &str,
        version_jar: TestFile,
        ordering: Option<i32>,
        modify_json: Option<json_patch::Patch>,
        pat: Option<&str>,
    ) -> CommonVersion;
    async fn get_version(&self, id: &str, pat: Option<&str>)
    -> ServiceResponse;
    async fn get_version_deserialized_common(
        &self,
        id_or_slug: &str,
        pat: Option<&str>,
    ) -> CommonVersion;
    async fn get_versions(
        &self,
        ids: Vec<String>,
        pat: Option<&str>,
    ) -> ServiceResponse;
    async fn get_versions_deserialized_common(
        &self,
        ids: Vec<String>,
        pat: Option<&str>,
    ) -> Vec<CommonVersion>;
    async fn download_version_redirect(
        &self,
        hash: &str,
        algorithm: &str,
        pat: Option<&str>,
    ) -> ServiceResponse;
    async fn edit_version(
        &self,
        id: &str,
        patch: serde_json::Value,
        pat: Option<&str>,
    ) -> ServiceResponse;
    async fn get_version_from_hash(
        &self,
        hash: &str,
        algorithm: &str,
        pat: Option<&str>,
    ) -> ServiceResponse;
    async fn get_version_from_hash_deserialized_common(
        &self,
        hash: &str,
        algorithm: &str,
        pat: Option<&str>,
    ) -> CommonVersion;
    async fn get_versions_from_hashes(
        &self,
        hashes: &[&str],
        algorithm: &str,
        pat: Option<&str>,
    ) -> ServiceResponse;
    async fn get_versions_from_hashes_deserialized_common(
        &self,
        hashes: &[&str],
        algorithm: &str,
        pat: Option<&str>,
    ) -> HashMap<String, CommonVersion>;
    async fn get_update_from_hash(
        &self,
        hash: &str,
        algorithm: &str,
        loaders: Option<Vec<String>>,
        game_versions: Option<Vec<String>>,
        version_types: Option<Vec<String>>,
        pat: Option<&str>,
    ) -> ServiceResponse;
    async fn get_update_from_hash_deserialized_common(
        &self,
        hash: &str,
        algorithm: &str,
        loaders: Option<Vec<String>>,
        game_versions: Option<Vec<String>>,
        version_types: Option<Vec<String>>,
        pat: Option<&str>,
    ) -> CommonVersion;
    async fn update_files(
        &self,
        algorithm: &str,
        hashes: Vec<String>,
        loaders: Option<Vec<String>>,
        game_versions: Option<Vec<String>>,
        version_types: Option<Vec<String>>,
        pat: Option<&str>,
    ) -> ServiceResponse;
    async fn update_files_deserialized_common(
        &self,
        algorithm: &str,
        hashes: Vec<String>,
        loaders: Option<Vec<String>>,
        game_versions: Option<Vec<String>>,
        version_types: Option<Vec<String>>,
        pat: Option<&str>,
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
        pat: Option<&str>,
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
        pat: Option<&str>,
    ) -> Vec<CommonVersion>;
    async fn edit_version_ordering(
        &self,
        version_id: &str,
        ordering: Option<i32>,
        pat: Option<&str>,
    ) -> ServiceResponse;
    async fn upload_file_to_version(
        &self,
        version_id: &str,
        file: &TestFile,
        pat: Option<&str>,
    ) -> ServiceResponse;
    async fn remove_version(
        &self,
        id_or_slug: &str,
        pat: Option<&str>,
    ) -> ServiceResponse;
    async fn remove_version_file(
        &self,
        hash: &str,
        pat: Option<&str>,
    ) -> ServiceResponse;
}

pub trait AppendsOptionalPat {
    fn append_pat(self, pat: Option<&str>) -> Self;
}
// Impl this on all actix_web::test::TestRequest
impl AppendsOptionalPat for actix_web::test::TestRequest {
    fn append_pat(self, pat: Option<&str>) -> Self {
        if let Some(pat) = pat {
            self.append_header(("Authorization", pat))
        } else {
            self
        }
    }
}

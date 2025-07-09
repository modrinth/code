use std::collections::HashMap;

use crate::common::{api_v2::ApiV2, api_v3::ApiV3, dummy_data::TestFile};
use actix_web::dev::ServiceResponse;
use async_trait::async_trait;
use labrinth::models::ids::ProjectId;
use labrinth::models::{
    projects::VersionType,
    teams::{OrganizationPermissions, ProjectPermissions},
};

use super::{
    Api, ApiProject, ApiTags, ApiTeams, ApiUser, ApiVersion,
    models::{CommonProject, CommonVersion},
    request_data::{ImageData, ProjectCreationRequestData},
};

#[derive(Clone)]
pub enum GenericApi {
    V2(ApiV2),
    V3(ApiV3),
}

macro_rules! delegate_api_variant {
    (
        $(#[$meta:meta])*
        impl $impl_name:ident for $struct_name:ident {
            $(
                [$method_name:ident, $ret:ty, $($param_name:ident: $param_type:ty),*]
            ),* $(,)?
        }

    ) => {
        $(#[$meta])*
        impl $impl_name for $struct_name {
            $(
                async fn $method_name(&self, $($param_name: $param_type),*) -> $ret {
                    match self {
                        $struct_name::V2(api) => api.$method_name($($param_name),*).await,
                        $struct_name::V3(api) => api.$method_name($($param_name),*).await,
                    }
                }
            )*
        }
    };
}

#[async_trait(?Send)]
impl Api for GenericApi {
    async fn call(&self, req: actix_http::Request) -> ServiceResponse {
        match self {
            Self::V2(api) => api.call(req).await,
            Self::V3(api) => api.call(req).await,
        }
    }

    async fn reset_search_index(&self) -> ServiceResponse {
        match self {
            Self::V2(api) => api.reset_search_index().await,
            Self::V3(api) => api.reset_search_index().await,
        }
    }
}

delegate_api_variant!(
    #[async_trait(?Send)]
    impl ApiProject for GenericApi {
        [add_public_project, (CommonProject, Vec<CommonVersion>), slug: &str, version_jar: Option<TestFile>, modify_json: Option<json_patch::Patch>, pat: Option<&str>],
        [get_public_project_creation_data_json, serde_json::Value, slug: &str, version_jar: Option<&TestFile>],
        [create_project, ServiceResponse, creation_data: ProjectCreationRequestData, pat: Option<&str>],
        [remove_project, ServiceResponse, project_slug_or_id: &str, pat: Option<&str>],
        [get_project, ServiceResponse, id_or_slug: &str, pat: Option<&str>],
        [get_project_deserialized_common, CommonProject, id_or_slug: &str, pat: Option<&str>],
        [get_projects, ServiceResponse, ids_or_slugs: &[&str], pat: Option<&str>],
        [get_project_dependencies, ServiceResponse, id_or_slug: &str, pat: Option<&str>],
        [get_user_projects, ServiceResponse, user_id_or_username: &str, pat: Option<&str>],
        [get_user_projects_deserialized_common, Vec<CommonProject>, user_id_or_username: &str, pat: Option<&str>],
        [edit_project, ServiceResponse, id_or_slug: &str, patch: serde_json::Value, pat: Option<&str>],
        [edit_project_bulk, ServiceResponse, ids_or_slugs: &[&str], patch: serde_json::Value, pat: Option<&str>],
        [edit_project_icon, ServiceResponse, id_or_slug: &str, icon: Option<ImageData>, pat: Option<&str>],
        [add_gallery_item, ServiceResponse, id_or_slug: &str, image: ImageData,  featured: bool, title: Option<String>, description: Option<String>, ordering: Option<i32>, pat: Option<&str>],
        [remove_gallery_item, ServiceResponse, id_or_slug: &str, image_url: &str, pat: Option<&str>],
        [edit_gallery_item, ServiceResponse, id_or_slug: &str, image_url: &str, patch: HashMap<String, String>, pat: Option<&str>],
        [create_report, ServiceResponse, report_type: &str, id: &str, item_type: crate::common::api_common::models::CommonItemType, body: &str, pat: Option<&str>],
        [get_report, ServiceResponse, id: &str, pat: Option<&str>],
        [get_reports, ServiceResponse, ids: &[&str], pat: Option<&str>],
        [get_user_reports, ServiceResponse, pat: Option<&str>],
        [edit_report, ServiceResponse, id: &str, patch: serde_json::Value, pat: Option<&str>],
        [delete_report, ServiceResponse, id: &str, pat: Option<&str>],
        [get_thread, ServiceResponse, id: &str, pat: Option<&str>],
        [get_threads, ServiceResponse, ids: &[&str], pat: Option<&str>],
        [write_to_thread, ServiceResponse, id: &str, r#type : &str, message: &str, pat: Option<&str>],
        [get_moderation_inbox, ServiceResponse, pat: Option<&str>],
        [read_thread, ServiceResponse, id: &str, pat: Option<&str>],
        [delete_thread_message, ServiceResponse, id: &str, pat: Option<&str>],
    }
);

delegate_api_variant!(
    #[async_trait(?Send)]
    impl ApiTags for GenericApi {
        [get_loaders, ServiceResponse,],
        [get_loaders_deserialized_common, Vec<crate::common::api_common::models::CommonLoaderData>,],
        [get_categories, ServiceResponse,],
        [get_categories_deserialized_common, Vec<crate::common::api_common::models::CommonCategoryData>,],
    }
);

delegate_api_variant!(
    #[async_trait(?Send)]
    impl ApiTeams for GenericApi {
        [get_team_members, ServiceResponse, team_id: &str, pat: Option<&str>],
        [get_team_members_deserialized_common, Vec<crate::common::api_common::models::CommonTeamMember>, team_id: &str, pat: Option<&str>],
        [get_teams_members, ServiceResponse, ids: &[&str], pat: Option<&str>],
        [get_project_members, ServiceResponse, id_or_slug: &str, pat: Option<&str>],
        [get_project_members_deserialized_common, Vec<crate::common::api_common::models::CommonTeamMember>, id_or_slug: &str, pat: Option<&str>],
        [get_organization_members, ServiceResponse, id_or_title: &str, pat: Option<&str>],
        [get_organization_members_deserialized_common, Vec<crate::common::api_common::models::CommonTeamMember>, id_or_title: &str, pat: Option<&str>],
        [join_team, ServiceResponse, team_id: &str, pat: Option<&str>],
        [remove_from_team, ServiceResponse, team_id: &str, user_id: &str, pat: Option<&str>],
        [edit_team_member, ServiceResponse, team_id: &str, user_id: &str, patch: serde_json::Value, pat: Option<&str>],
        [transfer_team_ownership, ServiceResponse, team_id: &str, user_id: &str, pat: Option<&str>],
        [get_user_notifications, ServiceResponse, user_id: &str, pat: Option<&str>],
        [get_user_notifications_deserialized_common, Vec<crate::common::api_common::models::CommonNotification>, user_id: &str, pat: Option<&str>],
        [get_notification, ServiceResponse, notification_id: &str, pat: Option<&str>],
        [get_notifications, ServiceResponse, ids: &[&str], pat: Option<&str>],
        [mark_notification_read, ServiceResponse, notification_id: &str, pat: Option<&str>],
        [mark_notifications_read, ServiceResponse, ids: &[&str], pat: Option<&str>],
        [add_user_to_team, ServiceResponse, team_id: &str, user_id: &str, project_permissions: Option<ProjectPermissions>, organization_permissions: Option<OrganizationPermissions>, pat: Option<&str>],
        [delete_notification, ServiceResponse, notification_id: &str, pat: Option<&str>],
        [delete_notifications, ServiceResponse, ids: &[&str], pat: Option<&str>],
    }
);

delegate_api_variant!(
    #[async_trait(?Send)]
    impl ApiUser for GenericApi {
        [get_user, ServiceResponse, id_or_username: &str, pat: Option<&str>],
        [get_current_user, ServiceResponse, pat: Option<&str>],
        [edit_user, ServiceResponse, id_or_username: &str, patch: serde_json::Value, pat: Option<&str>],
        [delete_user, ServiceResponse, id_or_username: &str, pat: Option<&str>],
    }
);

delegate_api_variant!(
    #[async_trait(?Send)]
    impl ApiVersion for GenericApi {
        [add_public_version, ServiceResponse, project_id: ProjectId, version_number: &str, version_jar: TestFile, ordering: Option<i32>, modify_json: Option<json_patch::Patch>, pat: Option<&str>],
        [add_public_version_deserialized_common, CommonVersion, project_id: ProjectId, version_number: &str, version_jar: TestFile, ordering: Option<i32>, modify_json: Option<json_patch::Patch>, pat: Option<&str>],
        [get_version, ServiceResponse, id_or_slug: &str, pat: Option<&str>],
        [get_version_deserialized_common, CommonVersion, id_or_slug: &str, pat: Option<&str>],
        [get_versions, ServiceResponse, ids_or_slugs: Vec<String>, pat: Option<&str>],
        [get_versions_deserialized_common, Vec<CommonVersion>, ids_or_slugs: Vec<String>, pat: Option<&str>],
        [download_version_redirect, ServiceResponse, hash: &str, algorithm: &str, pat: Option<&str>],
        [edit_version, ServiceResponse, id_or_slug: &str, patch: serde_json::Value, pat: Option<&str>],
        [get_version_from_hash, ServiceResponse, id_or_slug: &str, hash: &str, pat: Option<&str>],
        [get_version_from_hash_deserialized_common, CommonVersion, id_or_slug: &str, hash: &str, pat: Option<&str>],
        [get_versions_from_hashes, ServiceResponse, hashes: &[&str], algorithm: &str, pat: Option<&str>],
        [get_versions_from_hashes_deserialized_common, HashMap<String, CommonVersion>, hashes: &[&str],        algorithm: &str,        pat: Option<&str>],
        [get_update_from_hash, ServiceResponse, hash: &str, algorithm: &str, loaders: Option<Vec<String>>,game_versions: Option<Vec<String>>, version_types: Option<Vec<String>>, pat: Option<&str>],
        [get_update_from_hash_deserialized_common, CommonVersion, hash: &str,        algorithm: &str,loaders: Option<Vec<String>>,game_versions: Option<Vec<String>>,version_types: Option<Vec<String>>,        pat: Option<&str>],
        [update_files, ServiceResponse, algorithm: &str,        hashes: Vec<String>,        loaders: Option<Vec<String>>,        game_versions: Option<Vec<String>>,        version_types: Option<Vec<String>>,        pat: Option<&str>],
        [update_files_deserialized_common, HashMap<String, CommonVersion>, algorithm: &str,        hashes: Vec<String>,        loaders: Option<Vec<String>>,        game_versions: Option<Vec<String>>,        version_types: Option<Vec<String>>,        pat: Option<&str>],
        [get_project_versions, ServiceResponse, project_id_slug: &str,        game_versions: Option<Vec<String>>,loaders: Option<Vec<String>>,featured: Option<bool>,        version_type: Option<VersionType>,        limit: Option<usize>,        offset: Option<usize>,pat: Option<&str>],
        [get_project_versions_deserialized_common, Vec<CommonVersion>, project_id_slug: &str, game_versions: Option<Vec<String>>, loaders: Option<Vec<String>>,featured: Option<bool>,version_type: Option<VersionType>,limit: Option<usize>,offset: Option<usize>,pat: Option<&str>],
        [edit_version_ordering, ServiceResponse, version_id: &str,ordering: Option<i32>,pat: Option<&str>],
        [upload_file_to_version, ServiceResponse, version_id: &str, file: &TestFile, pat: Option<&str>],
        [remove_version, ServiceResponse, version_id: &str, pat: Option<&str>],
        [remove_version_file, ServiceResponse, hash: &str, pat: Option<&str>],
    }
);

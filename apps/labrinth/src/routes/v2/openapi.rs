use crate::routes::SecurityAddon;
use crate::routes::prefix_openapi_paths;
use crate::routes::v2::{
    moderation, notifications, project_creation, projects, reports,
    statistics, tags, teams, threads, users, version_creation, version_file,
    versions,
};

use utoipa::openapi::extensions::{Extensions, ExtensionsBuilder};
use utoipa::openapi::path::{Operation, PathItem};
use utoipa::openapi::schema::{AdditionalProperties, AllOf, Schema};
use utoipa::openapi::{RefOr, Response};

#[derive(utoipa::OpenApi)]
#[openapi(
	info(
    	title = "Modrinth API v2",
    	version = "2.0.0",
    	description = include_str!("../../api_v2_description.md"),
	),
	paths(
		notifications::notifications_get,
		notifications::notifications_read,
		notifications::notifications_delete,
		notifications::notification_get,
		notifications::notification_read,
		notifications::notification_delete,
		project_creation::project_create,
		moderation::get_projects,
		projects::project_search,
		projects::random_projects_get,
		projects::projects_get,
		projects::projects_edit,
		projects::project_get,
		projects::project_get_check,
		projects::project_edit,
		projects::project_icon_edit,
		projects::delete_project_icon,
		projects::add_gallery_item,
		projects::edit_gallery_item,
		projects::delete_gallery_item,
		projects::project_delete,
		projects::project_follow,
		projects::project_unfollow,
		projects::dependency_list,
		reports::report_create,
		reports::reports,
		reports::reports_get,
		reports::report_get,
		reports::report_edit,
		reports::report_delete,
		statistics::get_stats,
		tags::category_list,
		tags::loader_list,
		tags::game_version_list,
		tags::license_list,
		tags::license_text,
		tags::donation_platform_list,
		tags::report_type_list,
		tags::project_type_list,
		tags::side_type_list,
		teams::teams_get,
		teams::team_members_get_project,
		teams::team_members_get,
		teams::join_team,
		teams::add_team_member,
		teams::edit_team_member,
		teams::transfer_ownership,
		teams::remove_team_member,
		threads::threads_get,
		threads::thread_get,
		threads::thread_send_message,
		threads::message_delete,
		users::users_get,
		users::user_auth_get,
		users::user_get,
		users::projects_list,
		users::user_edit,
		users::user_icon_edit,
		users::user_icon_delete,
		users::user_delete,
		users::user_follows,
		users::user_notifications,
		version_file::get_version_from_hash,
		version_file::download_version,
		version_file::delete_file,
		version_file::get_update_from_hash,
		version_file::get_projects_from_hashes,
		version_file::get_versions_from_hashes,
		version_file::update_files,
		version_file::update_files_many,
		version_file::update_individual_files,
		versions::versions_get,
		version_creation::version_create,
		versions::version_get,
		versions::version_edit,
		versions::version_delete,
		versions::version_list,
		versions::version_project_get,
		version_creation::upload_file_to_version,
	),
	modifiers(&V2PathModifier, &SecurityAddon, &V2Modifier)
)]
pub struct ApiDoc;

const FILE_HASH_EXAMPLE: &str = "2aae6c35c94fcfb415dbe95f408b9ce91ee846ed";
const FILE_HASH_MAP_RESPONSE_OPERATION_IDS: [&str; 4] = [
    "versionsFromHashes",
    "projectsFromHashes",
    "getLatestVersionsFromHashes",
    "getLatestVersionsFromHashesMany",
];

struct V2PathModifier;

impl utoipa::Modify for V2PathModifier {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        prefix_openapi_paths(openapi, "/v2", |_| false);
    }
}

struct V2Modifier;

impl utoipa::Modify for V2Modifier {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        // We need to add an extension field to some routes to give a clearer example of a file hash in responses. Else,
        // it'll render as "additionalProperty"
        for path_item in openapi.paths.paths.values_mut() {
            add_file_hash_map_response_extension_to_path_item(path_item);
        }
    }
}

fn add_file_hash_map_response_extension_to_path_item(path_item: &mut PathItem) {
    add_file_hash_map_response_extension(&mut path_item.get);
    add_file_hash_map_response_extension(&mut path_item.put);
    add_file_hash_map_response_extension(&mut path_item.post);
    add_file_hash_map_response_extension(&mut path_item.delete);
    add_file_hash_map_response_extension(&mut path_item.options);
    add_file_hash_map_response_extension(&mut path_item.head);
    add_file_hash_map_response_extension(&mut path_item.patch);
    add_file_hash_map_response_extension(&mut path_item.trace);
}

fn add_file_hash_map_response_extension(operation: &mut Option<Operation>) {
    let Some(operation) = operation else {
        return;
    };
    let Some(operation_id) = operation.operation_id.as_deref() else {
        return;
    };

    if !FILE_HASH_MAP_RESPONSE_OPERATION_IDS.contains(&operation_id) {
        return;
    }

    let Some(RefOr::T(response)) = operation.responses.responses.get_mut("200")
    else {
        return;
    };

    add_file_hash_map_response_extension_to_response(response);
}

fn add_file_hash_map_response_extension_to_response(response: &mut Response) {
    let Some(content) = response.content.get_mut("application/json") else {
        return;
    };
    let Some(RefOr::T(Schema::Object(schema))) = content.schema.as_mut() else {
        return;
    };
    let Some(additional_properties) = schema.additional_properties.as_mut()
    else {
        return;
    };

    match additional_properties.as_mut() {
        AdditionalProperties::RefOr(RefOr::T(schema)) => {
            add_file_hash_map_response_extension_to_schema(schema);
        }
        AdditionalProperties::RefOr(RefOr::Ref(reference)) => {
            let mut all_of = AllOf::new();
            all_of.items.push(RefOr::Ref(reference.clone()));
            let mut schema = Schema::AllOf(all_of);
            add_file_hash_map_response_extension_to_schema(&mut schema);
            **additional_properties =
                AdditionalProperties::RefOr(RefOr::T(schema));
        }
        AdditionalProperties::FreeForm(_) => {}
    }
}

fn add_file_hash_map_response_extension_to_schema(schema: &mut Schema) {
    match schema {
        Schema::Array(schema) => {
            add_file_hash_map_response_extension_to_extensions(
                &mut schema.extensions,
            );
        }
        Schema::Object(schema) => {
            add_file_hash_map_response_extension_to_extensions(
                &mut schema.extensions,
            );
        }
        Schema::OneOf(schema) => {
            add_file_hash_map_response_extension_to_extensions(
                &mut schema.extensions,
            );
        }
        Schema::AllOf(schema) => {
            add_file_hash_map_response_extension_to_extensions(
                &mut schema.extensions,
            );
        }
        Schema::AnyOf(schema) => {
            add_file_hash_map_response_extension_to_extensions(
                &mut schema.extensions,
            );
        }
        _ => {}
    }
}

fn add_file_hash_map_response_extension_to_extensions(
    extensions: &mut Option<Extensions>,
) {
    let file_hash_extension = ExtensionsBuilder::new()
        .add("x-additionalPropertiesName", FILE_HASH_EXAMPLE)
        .build();

    if let Some(extensions) = extensions {
        extensions.merge(file_hash_extension);
    } else {
        *extensions = Some(file_hash_extension);
    }
}

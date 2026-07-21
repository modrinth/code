pub use super::ApiError;
use super::SecurityAddon;
use crate::util::cors::default_cors;
use actix_web::{HttpResponse, web};
use serde_json::json;

pub mod analytics_event;
pub mod analytics_get;
pub mod collections;
pub mod content;
pub mod friends;
pub mod images;
pub mod limits;
pub mod notifications;
pub mod organizations;
pub mod payouts;
pub mod project_creation;
pub mod projects;
pub mod reports;
pub mod statistics;
pub mod tags;
pub mod teams;
pub mod threads;
pub mod users;
pub mod version_creation;
pub mod version_file;
pub mod versions;

pub mod oauth_clients;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/v3/analytics")
            .wrap(default_cors())
            .configure(analytics_get::config),
    );
    cfg.service(
        web::scope("/v3/payout")
            .wrap(default_cors())
            .configure(payouts::config),
    );
    cfg.service(
        web::scope("/v3/project")
            .wrap(default_cors())
            .configure(projects::project_config)
            .configure(project_creation::config),
    );
    cfg.service(
        web::scope("/v3")
            .wrap(default_cors())
            .configure(limits::config)
            .configure(collections::config)
            .configure(images::config)
            .configure(notifications::config)
            .configure(oauth_clients::config)
            .configure(organizations::config)
            .service(payouts::paypal_webhook)
            .service(payouts::tremendous_webhook)
            .configure(projects::config)
            .configure(reports::config)
            .configure(statistics::config)
            .configure(tags::config)
            .configure(teams::config)
            .configure(threads::config)
            .configure(users::config)
            .configure(version_file::config)
            .configure(versions::config)
            .configure(friends::config)
            .configure(content::config),
    );
}

#[derive(utoipa::OpenApi)]
#[openapi(
	info(
		title = "API v3 (UNSTABLE)",
		version = "3.0.0",
		description = include_str!("../../api_v3_description.md"),
	),
	paths(
		analytics_get::fetch_analytics,
		analytics_get::facets::fetch_facets,
		analytics_get::old::playtimes_get,
		analytics_get::old::views_get,
		analytics_get::old::downloads_get,
		analytics_get::old::revenue_get,
		analytics_get::old::countries_downloads_get,
		analytics_get::old::countries_views_get,
		payouts::post_compliance_form,
		payouts::calculate_fees,
		payouts::create_payout,
		payouts::transaction_history,
		payouts::cancel_payout,
		payouts::payment_methods,
		payouts::get_balance,
		payouts::platform_revenue,
		projects::project_get,
		projects::project_get_check,
		projects::project_delete,
		projects::project_edit,
		projects::project_icon_edit,
		projects::delete_project_icon,
		projects::add_gallery_item,
		projects::edit_gallery_item,
		projects::delete_gallery_item,
		projects::project_follow,
		projects::project_unfollow,
		projects::project_get_organization,
		projects::dependency_list,
		project_creation::project_create,
		project_creation::project_create_with_id,
		project_creation::new::create,
		teams::team_members_get_project,
		versions::version_project_get,
		versions::version_list,
		limits::get_project_limits,
		limits::get_organization_limits,
		limits::get_collection_limits,
		collections::collection_create,
		collections::collections_get,
		collections::collection_get,
		collections::collection_edit,
		collections::collection_icon_edit,
		collections::delete_collection_icon,
		collections::collection_delete,
		images::images_add,
		notifications::notifications_get_route,
		notifications::notification_get_route,
		notifications::notification_read_route,
		notifications::notification_delete_route,
		notifications::notifications_read_route,
		notifications::notifications_delete_route,
		oauth_clients::get_user_clients,
		oauth_clients::get_client,
		oauth_clients::get_clients,
		oauth_clients::oauth_client_create,
		oauth_clients::oauth_client_delete,
		oauth_clients::oauth_client_edit,
		oauth_clients::oauth_client_icon_edit,
		oauth_clients::oauth_client_icon_delete,
		oauth_clients::get_user_oauth_authorizations,
		oauth_clients::revoke_oauth_authorization,
		super::super::auth::oauth::init_oauth,
		super::super::auth::oauth::accept_client_scopes,
		super::super::auth::oauth::reject_client_scopes,
		super::super::auth::oauth::request_token,
		organizations::organization_projects_get,
		organizations::organization_create,
		organizations::organization_get,
		organizations::organization_notes_edit,
		organizations::organizations_get,
		organizations::organizations_edit,
		organizations::organization_delete,
		organizations::organization_projects_add,
		organizations::organization_projects_remove,
		organizations::organization_icon_edit,
		organizations::delete_organization_icon,
		super::maven::maven_metadata,
		super::maven::version_file,
		super::maven::version_file_sha1,
		super::maven::version_file_sha512,
		super::updates::forge_updates,
		projects::project_search,
		projects::project_search_post,
		projects::projects_get_route,
		projects::projects_edit_route,
		projects::random_projects_get_route,
		reports::report_create_route,
		reports::reports_route,
		reports::reports_get_route,
		reports::report_get_route,
		reports::report_edit_route,
		reports::report_delete_route,
		statistics::get_stats_route,
		tags::games_list_route,
		tags::category_list_route,
		tags::loader_list_route,
		tags::loader_fields_list_route,
		tags::license_list_route,
		tags::license_text_route,
		tags::link_platform_list_route,
		tags::report_type_list_route,
		tags::project_type_list_route,
		teams::team_members_get_organization,
		teams::team_members_get_route,
		teams::teams_get_route,
		teams::join_team_route,
		teams::add_team_member_route,
		teams::edit_team_member_route,
		teams::transfer_ownership_route,
		teams::remove_team_member_route,
		threads::thread_get_route,
		threads::threads_get_route,
		threads::thread_send_message_route,
		threads::message_delete_route,
		users::all_projects,
		users::admin_user_email,
		users::projects_list_route,
		users::user_auth_get_route,
		users::users_search,
		users::users_get_route,
		users::user_get_route,
		users::user_notes_edit,
		users::collections_list,
		users::orgs_list,
		users::user_edit_route,
		users::user_icon_edit_route,
		users::user_icon_delete_route,
		users::user_delete_route,
		users::user_follows_route,
		users::user_notifications_route,
		version_creation::version_create_route,
		version_creation::upload_file_to_version_route,
		version_file::get_version_from_hash_route,
		version_file::get_update_from_hash_route,
		version_file::get_versions_from_hashes_route,
		version_file::get_projects_from_hashes_route,
		version_file::update_files_many_route,
		version_file::update_files_route,
		version_file::update_individual_files_route,
		version_file::delete_file_route,
		version_file::download_version_route,
		versions::versions_get_route,
		versions::version_get_route,
		versions::version_edit_route,
		versions::version_delete_route,
		friends::add_friend,
		friends::remove_friend,
		friends::friends,
		content::resolve_content,
	),
	modifiers(&V3PathModifier, &SecurityAddon)
)]
pub struct ApiDoc;

struct V3PathModifier;

impl utoipa::Modify for V3PathModifier {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        super::prefix_openapi_paths(openapi, "/v3", |path| {
            path.starts_with("/analytics/")
                || path.starts_with("/maven/")
                || path.starts_with("/updates/")
                || path.starts_with("/debug/")
        });
    }
}

pub async fn hello_world() -> Result<HttpResponse, ApiError> {
    Ok(HttpResponse::Ok().json(json!({
        "hello": "world",
    })))
}

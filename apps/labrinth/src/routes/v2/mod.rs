pub(crate) mod moderation;
mod notifications;
pub(crate) mod project_creation;
mod projects;
mod reports;
mod statistics;
pub mod tags;
mod teams;
mod threads;
mod users;
mod version_creation;
pub mod version_file;
mod versions;

pub use super::ApiError;
use super::SecurityAddon;
use crate::util::cors::default_cors;
use actix_web::web;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/v2")
            .wrap(default_cors())
            .configure(super::internal::session::config)
            .configure(super::internal::flows::config)
            .configure(super::internal::pats::config)
            .configure(super::internal::admin::config)
            .configure(moderation::config)
            .configure(notifications::config)
            .configure(project_creation::config)
            .configure(projects::config)
            .configure(reports::config)
            .configure(statistics::config)
            .configure(tags::config)
            .configure(teams::config)
            .configure(threads::config)
            .configure(users::config)
            .configure(version_file::config)
            .configure(versions::config),
    );
}

#[derive(utoipa::OpenApi)]
#[openapi(
	info(
    	title = "Modrinth API v2",
    	version = "2.0.0",
    	description = include_str!("../../api_v2_description.md"),
	),
	paths(
		super::internal::session::list,
		super::internal::session::delete,
		super::internal::session::refresh,
		super::internal::flows::init,
		super::internal::flows::auth_callback,
		super::internal::flows::create_oauth_account,
		super::internal::flows::discord_community_link,
		super::internal::flows::delete_auth_provider,
		super::internal::flows::validate_create_account_with_password,
		super::internal::flows::create_account_with_password,
		super::internal::flows::login_password,
		super::internal::flows::login_2fa,
		super::internal::flows::begin_2fa_flow,
		super::internal::flows::finish_2fa_flow,
		super::internal::flows::remove_2fa,
		super::internal::flows::reset_password_begin,
		super::internal::flows::change_password,
		super::internal::flows::set_email,
		super::internal::flows::resend_verify_email,
		super::internal::flows::verify_email,
		super::internal::flows::subscribe_newsletter,
		super::internal::flows::get_newsletter_subscription_status,
		super::internal::flows::register_passkey_start,
		super::internal::flows::register_passkey_finish,
		super::internal::flows::authenticate_passkey_start,
		super::internal::flows::authenticate_passkey_finish,
		super::internal::flows::list_passkeys,
		super::internal::flows::rename_passkey,
		super::internal::flows::delete_passkey,
		super::internal::pats::get_pats,
		super::internal::pats::create_pat,
		super::internal::pats::edit_pat,
		super::internal::pats::delete_pat,
		moderation::get_projects,
		notifications::notifications_get,
		notifications::notifications_read,
		notifications::notifications_delete,
		notifications::notification_get,
		notifications::notification_read,
		notifications::notification_delete,
		project_creation::project_create,
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
	),
	modifiers(&V2PathModifier, &SecurityAddon)
)]
pub struct ApiDoc;

struct V2PathModifier;

impl utoipa::Modify for V2PathModifier {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        super::prefix_openapi_paths(openapi, "/v2", |_| false);
    }
}

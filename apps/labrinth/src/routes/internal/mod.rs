pub mod admin;
pub mod affiliate;
pub mod attribution;
pub mod billing;
pub mod campaign;
pub mod delphi;
pub mod external_notifications;
pub mod flows;
pub mod gdpr;
pub mod globals;
pub mod gotenberg;
pub mod medal;
pub mod moderation;
pub mod mural;
pub mod pats;
pub mod search;
pub mod server_ping;
pub mod session;
pub mod statuses;

pub use super::ApiError;
use super::SecurityAddon;
use super::v3::oauth_clients;
use crate::util::cors::default_cors;
use actix_web::web;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/_internal")
            .wrap(default_cors())
            .configure(admin::config)
            .configure(session::config)
            .configure(flows::config)
            .configure(pats::config)
            .configure(oauth_clients::config)
            .service(web::scope("/moderation").configure(moderation::config))
            .service(web::scope("/affiliate").configure(affiliate::config))
            .service(web::scope("/campaign").configure(campaign::config))
            .service(web::scope("/search-management").configure(search::config))
            .service(web::scope("/globals").configure(globals::config))
            .service(web::scope("/server-ping").configure(server_ping::config))
            .service(web::scope("/attribution").configure(attribution::config))
            .configure(billing::config)
            .configure(delphi::config)
            .configure(external_notifications::config)
            .configure(gdpr::config)
            .configure(gotenberg::config)
            .configure(medal::config)
            .configure(mural::config)
            .configure(statuses::config),
    )
    .service(
        web::scope("/v3/analytics-event")
            .wrap(default_cors())
            .configure(super::v3::analytics_event::config),
    );
}

#[derive(utoipa::OpenApi)]
#[openapi(
	info(
		title = "Internal API (UNSTABLE)",
		version = "internal",
		description = include_str!("../../api_internal_description.md"),
	),
	paths(
		admin::count_download,
		admin::force_reindex,
		admin::force_reindex_project,
		session::list,
		session::delete,
		session::refresh,
		flows::init,
		flows::auth_callback,
		flows::create_oauth_account,
		flows::discord_community_link,
		flows::delete_auth_provider,
		flows::validate_create_account_with_password,
		flows::create_account_with_password,
		flows::login_password,
		flows::login_2fa,
		flows::begin_2fa_flow,
		flows::finish_2fa_flow,
		flows::remove_2fa,
		flows::reset_password_begin,
		flows::change_password,
		flows::set_email,
		flows::resend_verify_email,
		flows::verify_email,
		flows::subscribe_newsletter,
		flows::get_newsletter_subscription_status,
		flows::register_passkey_start,
		flows::register_passkey_finish,
		flows::authenticate_passkey_start,
		flows::authenticate_passkey_finish,
		flows::list_passkeys,
		flows::rename_passkey,
		flows::delete_passkey,
		pats::get_pats,
		pats::create_pat,
		pats::edit_pat,
		pats::delete_pat,
		moderation::get_projects,
		moderation::get_project_ids,
		moderation::get_project_meta,
		moderation::set_project_meta,
		moderation::acquire_lock,
		moderation::override_lock,
		moderation::get_lock_status,
		moderation::release_lock,
		moderation::release_lock_beacon,
		moderation::delete_all_locks,
		moderation::tech_review::get_issue,
		moderation::tech_review::get_report,
		moderation::tech_review::search_projects,
		moderation::tech_review::global::search_global_issue_details,
		moderation::tech_review::global::get_global_issue_detail,
		moderation::tech_review::rules::get_rules,
		moderation::tech_review::rules::test_rule,
		moderation::tech_review::rules::get_rule_affected_details,
		moderation::tech_review::rules::create_rule,
		moderation::tech_review::rules::update_rule,
		moderation::tech_review::rules::delete_rule,
		moderation::tech_review::rules_scan::get_rule_schema,
		moderation::tech_review::rules_scan::scan_rules,
		moderation::tech_review::get_project_report,
		moderation::tech_review::submit_report,
		moderation::tech_review::update_issue_details,
		moderation::tech_review::update_global_issue_details,
		moderation::tech_review::add_report,
		moderation::external_license::search,
		moderation::external_license::lookup,
		moderation::external_license::get_by_sha1,
		moderation::external_license::get_by_sha1_bulk,
		moderation::external_license::add_file,
		moderation::external_license::reassign_file,
		moderation::external_license::update_license,
		affiliate::ingest_click,
		affiliate::get_all,
		affiliate::create,
		affiliate::get,
		affiliate::delete,
		affiliate::patch,
		campaign::tiltify_webhook,
		campaign::pride_26,
		search::tasks,
		search::tasks_cancel,
		globals::get_globals,
		server_ping::ping_minecraft_java,
		attribution::scan,
		attribution::list,
		attribution::update_group,
		attribution::delete_group,
		attribution::assign,
		attribution::split,
		billing::products,
		billing::subscriptions,
		billing::refund_charge,
		billing::reprocess_charge_tax,
		billing::edit_subscription,
		billing::user_customer,
		billing::charges,
		billing::add_payment_method_flow,
		billing::edit_payment_method,
		billing::remove_payment_method,
		billing::payment_methods,
		billing::active_servers,
		billing::initiate_payment,
		billing::stripe_webhook,
		billing::credit,
		delphi::ingest_report,
		delphi::_run,
		delphi::version,
		delphi::issue_type_schema,
		external_notifications::create,
		external_notifications::create_email_sync,
		external_notifications::remove,
		external_notifications::send_custom_email,
		gdpr::export,
		gotenberg::success_callback,
		gotenberg::error_callback,
		medal::verify,
		medal::redeem,
		mural::get_bank_details,
		statuses::ws_init,
		super::v3::analytics_event::analytics_events_get,
		super::v3::analytics_event::analytics_event_create,
		super::v3::analytics_event::analytics_event_edit,
		super::v3::analytics_event::analytics_event_delete,
	),
	modifiers(&InternalPathModifier, &SecurityAddon)
)]
pub struct ApiDoc;

struct InternalPathModifier;

impl utoipa::Modify for InternalPathModifier {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        super::prefix_openapi_paths(openapi, "/_internal", |path| {
            path.starts_with("/v3/")
        });
    }
}

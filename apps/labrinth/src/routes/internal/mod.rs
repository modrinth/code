pub(crate) mod admin;
pub mod affiliate;
pub mod billing;
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
use super::v3::oauth_clients;
use crate::util::cors::default_cors;

pub fn config(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(
        actix_web::web::scope("/_internal")
            .wrap(default_cors())
            .configure(|cfg| {
                cfg.service(
                    actix_web::web::scope("/admin")
                        .service(admin::count_download)
                        .service(admin::force_reindex),
                );
                cfg.service(
                    actix_web::web::scope("/session")
                        .service(session::list)
                        .service(session::delete)
                        .service(session::refresh),
                );
                cfg.service(
                    actix_web::web::scope("/auth")
                        .service(flows::init)
                        .service(flows::auth_callback)
                        .service(flows::delete_auth_provider)
                        .service(flows::create_account_with_password)
                        .service(flows::login_password)
                        .service(flows::login_2fa)
                        .service(flows::begin_2fa_flow)
                        .service(flows::finish_2fa_flow)
                        .service(flows::remove_2fa)
                        .service(flows::reset_password_begin)
                        .service(flows::change_password)
                        .service(flows::resend_verify_email)
                        .service(flows::set_email)
                        .service(flows::verify_email)
                        .service(flows::subscribe_newsletter)
                        .service(flows::get_newsletter_subscription_status),
                );
                cfg.service(pats::get_pats);
                cfg.service(pats::create_pat);
                cfg.service(pats::edit_pat);
                cfg.service(pats::delete_pat);
            })
            .configure(oauth_clients::config)
            .configure(billing::config)
            .configure(gdpr::config)
            .configure(gotenberg::config)
            .configure(statuses::config)
            .configure(medal::config)
            .configure(external_notifications::config)
            .configure(mural::config)
            .configure(delphi::config),
    );
}

pub fn utoipa_config(
    cfg: &mut utoipa_actix_web::service_config::ServiceConfig,
) {
    cfg.service(
        utoipa_actix_web::scope("/_internal/moderation")
            .wrap(default_cors())
            .configure(moderation::config),
    )
    .service(
        utoipa_actix_web::scope("/_internal/affiliate")
            .wrap(default_cors())
            .configure(affiliate::config),
    )
    .service(
        utoipa_actix_web::scope("/_internal/search-management")
            .wrap(default_cors())
            .configure(search::config),
    )
    .service(
        utoipa_actix_web::scope("/_internal/globals")
            .wrap(default_cors())
            .configure(globals::config),
    )
    .service(
        utoipa_actix_web::scope("/_internal/server-ping")
            .wrap(default_cors())
            .configure(server_ping::config),
    );
}

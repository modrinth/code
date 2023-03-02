use actix_web::{dev::Service, http::Method, web, HttpResponse};
use chrono::{Timelike, Utc};
use futures::FutureExt;

mod mods;
mod tags;
mod teams;
mod users;
mod versions;

pub fn v1_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("api/v1")
            .wrap_fn(|req, srv| {
                let time = Utc::now();

                if req.method() == Method::GET && time.hour12().1 < 6 && time.minute() % 10 < 5 {
                    srv.call(req).boxed_local()
                } else {
                    async {
                        Ok(
                            req.into_response(
                                HttpResponse::Gone()
                                    .content_type("application/json")
                                    .body(r#"{"error":"api_deprecated","description":"You are using an application that uses an outdated version of Modrinth's API. Please either update it or switch to another application. For developers: https://docs.modrinth.com/docs/migrations/v1-to-v2/"}"#)
                            )
                        )
                    }.boxed_local()
                }
            })
            .configure(tags_config)
            .configure(mods_config)
            .configure(versions_config)
            .configure(teams_config)
            .configure(users_config)
            .configure(notifications_config),
    );
}

pub fn tags_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("tag")
            .service(tags::category_list)
            .service(tags::loader_list)
            .service(tags::game_version_list)
            .service(super::tags::license_list)
            .service(super::tags::report_type_list),
    );
}

pub fn mods_config(cfg: &mut web::ServiceConfig) {
    cfg.service(mods::mod_search);
    cfg.service(mods::mods_get);

    cfg.service(
        web::scope("mod")
            .service(mods::mod_get)
            .service(web::scope("{mod_id}").service(versions::version_list)),
    );
}

pub fn versions_config(cfg: &mut web::ServiceConfig) {
    cfg.service(versions::versions_get);
    cfg.service(web::scope("version").service(versions::version_get));
    cfg.service(
        web::scope("version_file")
            .service(super::version_file::get_version_from_hash),
    );
}

pub fn users_config(cfg: &mut web::ServiceConfig) {
    cfg.service(super::users::user_auth_get);

    cfg.service(super::users::users_get);
    cfg.service(
        web::scope("user")
            .service(super::users::user_get)
            .service(users::mods_list)
            .service(super::users::user_notifications)
            .service(users::user_follows),
    );
}

pub fn teams_config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("team").service(teams::team_members_get));
}

pub fn notifications_config(cfg: &mut web::ServiceConfig) {
    cfg.service(super::notifications::notifications_get);

    cfg.service(
        web::scope("notification")
            .service(super::notifications::notification_get),
    );
}

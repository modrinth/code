use actix_web::{dev::Service, web, HttpResponse};
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
                let current_minute = Utc::now().minute();

                if current_minute % 10 > 5 {
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
            .configure(reports_config)
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
    cfg.service(mods::mod_create);

    cfg.service(
        web::scope("mod")
            .service(mods::mod_get)
            .service(super::projects::project_delete)
            .service(super::projects::project_edit)
            .service(super::projects::project_icon_edit)
            .service(super::projects::project_follow)
            .service(super::projects::project_unfollow)
            .service(web::scope("{mod_id}").service(versions::version_list)),
    );
}

pub fn versions_config(cfg: &mut web::ServiceConfig) {
    cfg.service(versions::versions_get);
    cfg.service(super::version_creation::version_create);
    cfg.service(
        web::scope("version")
            .service(versions::version_get)
            .service(super::versions::version_delete)
            .service(super::version_creation::upload_file_to_version)
            .service(super::versions::version_edit),
    );
    cfg.service(
        web::scope("version_file")
            .service(super::version_file::delete_file)
            .service(super::version_file::get_version_from_hash)
            .service(super::version_file::download_version),
    );
}

pub fn users_config(cfg: &mut web::ServiceConfig) {
    cfg.service(super::users::user_auth_get);

    cfg.service(super::users::users_get);
    cfg.service(
        web::scope("user")
            .service(super::users::user_get)
            .service(users::mods_list)
            .service(super::users::user_delete)
            .service(super::users::user_edit)
            .service(super::users::user_icon_edit)
            .service(super::users::user_notifications)
            .service(users::user_follows),
    );
}

pub fn teams_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("team")
            .service(teams::team_members_get)
            .service(super::teams::edit_team_member)
            .service(super::teams::add_team_member)
            .service(super::teams::join_team)
            .service(super::teams::remove_team_member),
    );
}

pub fn notifications_config(cfg: &mut web::ServiceConfig) {
    cfg.service(super::notifications::notifications_get);

    cfg.service(
        web::scope("notification")
            .service(super::notifications::notification_get)
            .service(super::notifications::notification_delete),
    );
}

pub fn reports_config(cfg: &mut web::ServiceConfig) {
    cfg.service(super::reports::report_create);
}

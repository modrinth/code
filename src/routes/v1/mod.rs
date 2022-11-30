use actix_web::web;

mod moderation;
mod mods;
mod reports;
mod tags;
mod teams;
mod users;
mod versions;

pub fn v1_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("api/v1")
            .configure(super::auth_config)
            .configure(tags_config)
            .configure(mods_config)
            .configure(versions_config)
            .configure(teams_config)
            .configure(users_config)
            .configure(moderation_config)
            .configure(reports_config)
            .configure(notifications_config),
    );
}

pub fn tags_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("tag")
            .service(tags::category_list)
            .service(tags::category_create)
            .service(super::tags::category_delete)
            .service(tags::loader_list)
            .service(tags::loader_create)
            .service(super::tags::loader_delete)
            .service(tags::game_version_list)
            .service(super::tags::game_version_create)
            .service(super::tags::game_version_delete)
            .service(super::tags::license_list)
            .service(super::tags::donation_platform_create)
            .service(super::tags::donation_platform_list)
            .service(super::tags::donation_platform_delete)
            .service(super::tags::report_type_create)
            .service(super::tags::report_type_delete)
            .service(super::tags::report_type_list),
    );
}

pub fn mods_config(cfg: &mut web::ServiceConfig) {
    cfg.service(mods::mod_search);
    cfg.service(mods::mods_get);
    cfg.service(mods::mod_create);

    cfg.service(
        web::scope("mod")
            .service(super::projects::project_get)
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
            .service(versions::delete_file)
            .service(versions::get_version_from_hash)
            .service(versions::download_version),
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

pub fn moderation_config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("moderation").service(moderation::get_mods));
}

pub fn reports_config(cfg: &mut web::ServiceConfig) {
    cfg.service(reports::reports);
    cfg.service(reports::report_create);
    cfg.service(super::reports::delete_report);
}

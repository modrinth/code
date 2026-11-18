pub mod project;
pub mod rule;
pub mod run_response;
mod schema_example;

pub fn config(cfg: &mut utoipa_actix_web::service_config::ServiceConfig<'_>) {
    cfg.service(rule::list_rules)
        .service(rule::get_rule_schema)
        .service(rule::list_runs)
        .service(rule::get_run_activity)
        .service(rule::list_run_results)
        .service(rule::start_run)
        .service(rule::delete_run)
        .service(rule::get_rule)
        .service(rule::create_rule)
        .service(rule::update_rule)
        .service(rule::delete_rule)
        .service(rule::eval_rule)
        .service(project::get_project_flags);
}

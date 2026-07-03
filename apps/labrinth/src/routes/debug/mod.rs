use std::time::Duration;

use actix_web::web;
use eyre::Context;
use eyre::eyre;
use prometheus::IntGauge;

use super::SecurityAddon;
use crate::util::cors::default_cors;

#[cfg(target_os = "linux")]
pub(crate) mod pprof;

#[cfg(target_os = "linux")]
#[derive(utoipa::OpenApi)]
#[openapi(
	paths(pprof::heap, pprof::flame_graph),
	modifiers(&DebugPathModifier, &SecurityAddon)
)]
pub struct ApiDoc;

#[cfg(target_os = "linux")]
struct DebugPathModifier;

#[cfg(target_os = "linux")]
impl utoipa::Modify for DebugPathModifier {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        super::prefix_openapi_paths(openapi, "/debug", |_| false);
    }
}

pub fn config(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(web::scope("/debug").wrap(default_cors()).configure({
        #[cfg(target_os = "linux")]
        {
            pprof::config
        }
        #[cfg(not(target_os = "linux"))]
        {
            |_cfg| ()
        }
    }));
}

pub fn register_and_set_metrics(
    registry: &prometheus::Registry,
) -> eyre::Result<()> {
    #[cfg(target_os = "linux")]
    {
        pprof::register_and_set_metrics(registry)
            .wrap_err("failed to register jemalloc metrics")?;
    }

    let make_gauge = |key: &str, name: &str| {
        IntGauge::new(key, name)
            .wrap_err_with(|| eyre!("failed to create gauge for '{key}'"))
    };

    let num_workers = make_gauge(
        "labrinth_tokio_num_workers",
        "number of Tokio worker threads, excluding Actix HTTP server threads",
    )?;
    let num_alive_tasks = make_gauge(
        "labrinth_tokio_num_alive_tasks",
        "number of alive Tokio tasks, excluding Actix HTTP server tasks",
    )?;
    let global_queue_depth = make_gauge(
        "labrinth_tokio_global_queue_depth",
        "number of tasks in the global queue, excluding Actix runtime",
    )?;

    for gauge in [&num_workers, &num_alive_tasks, &global_queue_depth] {
        registry
            .register(Box::new(gauge.clone()))
            .wrap_err("failed to register gauge")?;
    }

    tokio::spawn(async move {
        let metrics = tokio::runtime::Handle::current().metrics();

        loop {
            num_workers.set(metrics.num_workers() as i64);
            num_alive_tasks.set(metrics.num_alive_tasks() as i64);
            global_queue_depth.set(metrics.global_queue_depth() as i64);

            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    });

    Ok(())
}

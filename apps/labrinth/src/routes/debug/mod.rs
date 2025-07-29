use crate::routes::ApiError;
use crate::util::cors::default_cors;
use crate::util::guards::admin_key_guard;
use actix_web::{HttpResponse, get};
use prometheus::{IntGauge, Registry};
use std::time::Duration;

pub fn config(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(
        actix_web::web::scope("/debug")
            .wrap(default_cors())
            .service(heap)
            .service(flame_graph),
    );
}

#[get("pprof/heap", guard = "admin_key_guard")]
pub async fn heap() -> Result<HttpResponse, ApiError> {
    let mut prof_ctl = jemalloc_pprof::PROF_CTL.as_ref().unwrap().lock().await;
    require_profiling_activated(&prof_ctl)?;
    let pprof = prof_ctl
        .dump_pprof()
        .map_err(|err| ApiError::InvalidInput(err.to_string()))?;

    Ok(HttpResponse::Ok()
        .content_type("application/octet-stream")
        .body(pprof))
}

#[get("pprof/heap/flamegraph", guard = "admin_key_guard")]
pub async fn flame_graph() -> Result<HttpResponse, ApiError> {
    let mut prof_ctl = jemalloc_pprof::PROF_CTL.as_ref().unwrap().lock().await;
    require_profiling_activated(&prof_ctl)?;
    let svg = prof_ctl
        .dump_flamegraph()
        .map_err(|err| ApiError::InvalidInput(err.to_string()))?;

    Ok(HttpResponse::Ok().content_type("image/svg+xml").body(svg))
}

fn require_profiling_activated(
    prof_ctl: &jemalloc_pprof::JemallocProfCtl,
) -> Result<(), ApiError> {
    if prof_ctl.activated() {
        Ok(())
    } else {
        Err(ApiError::InvalidInput(
            "Profiling is not activated".to_string(),
        ))
    }
}

pub fn jemalloc_memory_stats(
    registry: &Registry,
) -> Result<(), prometheus::Error> {
    let allocated_mem = IntGauge::new(
        "labrinth_memory_allocated",
        "Labrinth allocated memory",
    )?;
    let resident_mem =
        IntGauge::new("labrinth_resident_memory", "Labrinth resident memory")?;

    registry.register(Box::new(allocated_mem.clone()))?;
    registry.register(Box::new(resident_mem.clone()))?;

    tokio::spawn(async move {
        let e = tikv_jemalloc_ctl::epoch::mib().unwrap();
        let allocated = tikv_jemalloc_ctl::stats::allocated::mib().unwrap();
        let resident = tikv_jemalloc_ctl::stats::resident::mib().unwrap();

        loop {
            e.advance().unwrap();

            if let Ok(allocated) = allocated.read() {
                allocated_mem.set(allocated as i64);
            }

            if let Ok(resident) = resident.read() {
                resident_mem.set(resident as i64);
            }

            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    });

    Ok(())
}

use crate::routes::ApiError;
use crate::util::guards::admin_key_guard;
use actix_web::{HttpResponse, get};
use eyre::{Context, eyre};
use prometheus::{IntGauge, Registry};
use std::time::Duration;

pub fn config(cfg: &mut utoipa_actix_web::service_config::ServiceConfig) {
    cfg.service(heap).service(flame_graph);
}

#[utoipa::path]
#[get("/pprof/heap", guard = "admin_key_guard")]
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

#[utoipa::path]
#[get("/pprof/heap/flamegraph", guard = "admin_key_guard")]
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

pub fn register_and_set_metrics(registry: &Registry) -> eyre::Result<()> {
    let make_gauge = |key: &str, name: &str| {
        IntGauge::new(key, name)
            .wrap_err_with(|| eyre!("failed to create gauge for '{key}'"))
    };

    let active_mem =
        make_gauge("labrinth_memory_active", "labrinth active memory")?;
    let allocated_mem =
        make_gauge("labrinth_memory_allocated", "labrinth allocated memory")?;
    let mapped_mem =
        make_gauge("labrinth_memory_mapped", "labrinth mapped memory")?;
    let metadata_mem =
        make_gauge("labrinth_memory_metadata", "labrinth metadata memory")?;
    let resident_mem =
        make_gauge("labrinth_memory_resident", "labrinth resident memory")?;

    for gauge in [
        &active_mem,
        &allocated_mem,
        &mapped_mem,
        &metadata_mem,
        &resident_mem,
    ] {
        registry
            .register(Box::new(gauge.clone()))
            .wrap_err("failed to register gauge")?;
    }

    tokio::spawn(async move {
        let epoch =
            tikv_jemalloc_ctl::epoch::mib().expect("failed to get epoch");
        let active = tikv_jemalloc_ctl::stats::active::mib().unwrap();
        let allocated = tikv_jemalloc_ctl::stats::allocated::mib().unwrap();
        let mapped = tikv_jemalloc_ctl::stats::mapped::mib().unwrap();
        let metadata = tikv_jemalloc_ctl::stats::metadata::mib().unwrap();
        let resident = tikv_jemalloc_ctl::stats::resident::mib().unwrap();

        loop {
            epoch.advance().unwrap();

            _ = active.read().inspect(|x| active_mem.set(*x as i64));
            _ = allocated.read().inspect(|x| allocated_mem.set(*x as i64));
            _ = mapped.read().inspect(|x| mapped_mem.set(*x as i64));
            _ = metadata.read().inspect(|x| metadata_mem.set(*x as i64));
            _ = resident.read().inspect(|x| resident_mem.set(*x as i64));

            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    });

    Ok(())
}

pub mod project_flag;

use self::project_flag::{
    PROJECT_FLAG_TASK_QUEUE, project_flag_worker_options,
};
use crate::file_hosting::FileHost;
use anyhow::{Context, Result};
use std::sync::Arc;
use temporalio_sdk::{Runtime, Worker};
use tracing::info;
use xredis::RedisPool;

use crate::database::PgPool;
use crate::env::ProjectFlags;
use crate::util::temporal::TemporalClient;

pub async fn run(
    pool: PgPool,
    redis: RedisPool,
    file_host: Arc<dyn FileHost>,
    project_flags: ProjectFlags,
    kafka: actix_web::web::Data<crate::util::kafka::KafkaClientState>,
) -> Result<()> {
    let runtime = Runtime::from_current_tokio(
        temporalio_sdk::runtime::RuntimeOptions::default(),
    )
    .context("initializing temporal runtime")?;
    let temporal = TemporalClient::from_env()
        .await
        .map_err(|error| anyhow::Error::from_boxed(error.into()))?;
    let worker = Worker::new(
        &runtime,
        temporal.client.clone(),
        project_flag_worker_options(
            project_flags,
            pool,
            redis,
            file_host,
            temporal.client,
            kafka,
        )?,
    )
    .context("creating temporal worker")?;

    #[cfg(unix)]
    let mut terminate = tokio::signal::unix::signal(
        tokio::signal::unix::SignalKind::terminate(),
    )
    .context("registering temporal worker termination signal")?;

    info!(
        task_queue = PROJECT_FLAG_TASK_QUEUE,
        "Starting temporal worker"
    );

    worker
        .run_until(async {
            #[cfg(unix)]
            tokio::select! {
                result = tokio::signal::ctrl_c() => {
                    if let Err(error) = result {
                        tracing::error!(%error, "Failed to listen for interrupt signal");
                    }
                }
                _ = terminate.recv() => {}
            }

            #[cfg(not(unix))]
            if let Err(error) = tokio::signal::ctrl_c().await {
                tracing::error!(%error, "Failed to listen for interrupt signal");
            }

            info!("Shutting down temporal worker");
        })
        .await
        .context("running temporal worker")
}

pub mod project_flag;

use self::project_flag::{
    PROJECT_FLAG_TASK_QUEUE, project_flag_worker_options,
};
use eyre::{Result, WrapErr};
use temporalio_sdk::{Runtime, Worker};
use tracing::info;

use crate::database::PgPool;
use crate::env::ProjectFlags;
use crate::util::temporal::TemporalClient;

pub async fn run(pool: PgPool, project_flags: ProjectFlags) -> Result<()> {
    let runtime = Runtime::from_current_tokio(Default::default())
        .wrap_err("initializing temporal runtime")?;
    let temporal = TemporalClient::from_env(pool).await?;
    let worker = Worker::new(
        &runtime,
        temporal.client,
        project_flag_worker_options(project_flags)?,
    )
    .wrap_err("creating temporal worker")?;

    #[cfg(unix)]
    let mut terminate = tokio::signal::unix::signal(
        tokio::signal::unix::SignalKind::terminate(),
    )
    .wrap_err("registering temporal worker termination signal")?;

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
        .wrap_err("running temporal worker")
}

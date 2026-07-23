use std::time::Duration;

use prometheus::{IntGauge, Registry};

const METRICS_UPDATE_INTERVAL: Duration = Duration::from_secs(5);

#[derive(Debug, Clone, Copy)]
pub(super) struct LogicalPoolStatus {
    max_size: usize,
    size: usize,
    available: usize,
    waiting: usize,
}

impl LogicalPoolStatus {
    pub(super) fn from_deadpool(status: deadpool_redis::Status) -> Self {
        Self {
            max_size: status.max_size,
            size: status.size,
            available: status.available,
            waiting: status.waiting,
        }
    }

    pub(super) fn shared_multiplexed() -> Self {
        Self {
            max_size: 1,
            size: 1,
            available: 1,
            waiting: 0,
        }
    }
}

pub(super) trait LogicalPoolStatusProvider:
    Clone + Send + 'static
{
    fn logical_pool_status(&self) -> LogicalPoolStatus;
}

#[derive(Debug, Clone, Copy)]
enum RedisPoolMetricsKind {
    Command,
    Blocking,
}

impl RedisPoolMetricsKind {
    fn metric_prefix(self) -> &'static str {
        match self {
            Self::Command => "labrinth_redis_pool",
            Self::Blocking => "labrinth_redis_blocking_pool",
        }
    }

    fn description(self) -> &'static str {
        match self {
            Self::Command => "Redis command pool",
            Self::Blocking => "Redis blocking-command pool",
        }
    }
}

#[derive(Clone)]
struct RedisPoolMetrics {
    max_size: IntGauge,
    size: IntGauge,
    available: IntGauge,
    waiting: IntGauge,
}

impl RedisPoolMetrics {
    fn register(
        registry: &Registry,
        kind: RedisPoolMetricsKind,
    ) -> Result<Self, prometheus::Error> {
        let prefix = kind.metric_prefix();
        let description = kind.description();
        let max_size = IntGauge::new(
            format!("{prefix}_max_size"),
            format!(
                "Maximum logical connection count for the {description}; clustered logical connections may own multiple physical sockets"
            ),
        )?;
        let size = IntGauge::new(
            format!("{prefix}_size"),
            format!(
                "Current logical connection count for the {description}; clustered logical connections may own multiple physical sockets"
            ),
        )?;
        let available = IntGauge::new(
            format!("{prefix}_available"),
            format!("Available logical connections in the {description}"),
        )?;
        let waiting = IntGauge::new(
            format!("{prefix}_waiting"),
            format!(
                "Number of futures waiting for a logical connection from the {description}"
            ),
        )?;

        registry.register(Box::new(max_size.clone()))?;
        registry.register(Box::new(size.clone()))?;
        registry.register(Box::new(available.clone()))?;
        registry.register(Box::new(waiting.clone()))?;

        Ok(Self {
            max_size,
            size,
            available,
            waiting,
        })
    }

    fn set(&self, status: LogicalPoolStatus) {
        self.max_size.set(status.max_size as i64);
        self.size.set(status.size as i64);
        self.available.set(status.available as i64);
        self.waiting.set(status.waiting as i64);
    }
}

pub(super) fn register_command_pool_metrics<P>(
    registry: &Registry,
    provider: P,
) -> Result<(), prometheus::Error>
where
    P: LogicalPoolStatusProvider,
{
    register_pool_metrics(registry, RedisPoolMetricsKind::Command, provider)
}

pub(super) fn register_blocking_pool_metrics<P>(
    registry: &Registry,
    provider: P,
) -> Result<(), prometheus::Error>
where
    P: LogicalPoolStatusProvider,
{
    register_pool_metrics(registry, RedisPoolMetricsKind::Blocking, provider)
}

fn register_pool_metrics<P>(
    registry: &Registry,
    kind: RedisPoolMetricsKind,
    provider: P,
) -> Result<(), prometheus::Error>
where
    P: LogicalPoolStatusProvider,
{
    let metrics = RedisPoolMetrics::register(registry, kind)?;
    metrics.set(provider.logical_pool_status());

    tokio::spawn(async move {
        loop {
            tokio::time::sleep(METRICS_UPDATE_INTERVAL).await;
            metrics.set(provider.logical_pool_status());
        }
    });

    Ok(())
}

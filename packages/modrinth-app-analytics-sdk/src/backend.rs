use std::time::Duration;

use uuid::Uuid;

#[derive(Debug)]
pub(crate) enum AnalyticsBackend {
    OptedOut,
    Enabled {
        http_pool: reqwest::Client,
        installation_id: Uuid,
    },
}

impl AnalyticsBackend {
    /// Returns the HTTP pool and installation ID to send a batch with, or
    /// `None` when opted out.
    pub(crate) fn sender(&self) -> Option<(reqwest::Client, Uuid)> {
        match self {
            AnalyticsBackend::OptedOut => None,
            AnalyticsBackend::Enabled {
                http_pool,
                installation_id,
            } => Some((http_pool.clone(), *installation_id)),
        }
    }

    /// Returns the installation ID events are sent under, or `None` when
    /// opted out.
    pub(crate) fn installation_id(&self) -> Option<Uuid> {
        match self {
            AnalyticsBackend::OptedOut => None,
            AnalyticsBackend::Enabled {
                installation_id, ..
            } => Some(*installation_id),
        }
    }

    pub(crate) fn is_enabled(&self) -> bool {
        matches!(self, AnalyticsBackend::Enabled { .. })
    }
}

#[track_caller]
pub(crate) fn init_pool(user_agent: &str) -> reqwest::Client {
    reqwest::Client::builder()
        .user_agent(user_agent)
        .timeout(Duration::from_secs(10))
        .read_timeout(Duration::from_secs(10))
        .connect_timeout(Duration::from_secs(10))
        // we try to flush events every 10 seconds
        .pool_idle_timeout(Duration::from_secs(30))
        .build()
        .expect("reqwest::Client failed to build")
}

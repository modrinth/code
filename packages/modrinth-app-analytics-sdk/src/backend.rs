use std::time::Duration;

#[derive(Debug)]
pub(crate) enum AnalyticsBackend {
    OptedOut,
    Enabled { http_pool: reqwest::Client },
}

impl AnalyticsBackend {
    pub(crate) fn clone_pool(&self) -> Option<reqwest::Client> {
        match self {
            AnalyticsBackend::OptedOut => None,
            AnalyticsBackend::Enabled { http_pool } => Some(http_pool.clone()),
        }
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

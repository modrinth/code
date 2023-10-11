use crate::models::analytics::{Download, PageView, Playtime};
use dashmap::DashSet;

#[cfg(test)]
mod tests;

const VIEWS_TABLENAME: &str = "views";
const DOWNLOADS_TABLENAME: &str = "downloads";
const PLAYTIME_TABLENAME: &str = "playtime";

pub struct AnalyticsQueue {
    views_queue: DashSet<PageView>,
    downloads_queue: DashSet<Download>,
    playtime_queue: DashSet<Playtime>,
}

impl Default for AnalyticsQueue {
    fn default() -> Self {
        Self::new()
    }
}

// Batches analytics data points + transactions every few minutes
impl AnalyticsQueue {
    pub fn new() -> Self {
        AnalyticsQueue {
            views_queue: DashSet::with_capacity(1000),
            downloads_queue: DashSet::with_capacity(1000),
            playtime_queue: DashSet::with_capacity(1000),
        }
    }

    pub fn add_view(&self, page_view: PageView) {
        self.views_queue.insert(page_view);
    }

    pub fn add_download(&self, download: Download) {
        self.downloads_queue.insert(download);
    }

    pub fn add_playtime(&self, playtime: Playtime) {
        self.playtime_queue.insert(playtime);
    }

    pub async fn index(&self, client: clickhouse::Client) -> Result<(), clickhouse::error::Error> {
        Self::index_queue(&client, &self.views_queue, VIEWS_TABLENAME).await?;
        Self::index_queue(&client, &self.downloads_queue, DOWNLOADS_TABLENAME).await?;
        Self::index_queue(&client, &self.playtime_queue, PLAYTIME_TABLENAME).await?;

        Ok(())
    }

    async fn index_queue<T>(
        client: &clickhouse::Client,
        queue: &DashSet<T>,
        table_name: &str,
    ) -> Result<(), clickhouse::error::Error>
    where
        T: serde::Serialize + Eq + std::hash::Hash + Clone + clickhouse::Row,
    {
        if queue.is_empty() {
            return Ok(());
        }

        let current_queue = queue.clone();
        queue.clear();

        let mut inserter = client.inserter(table_name)?;

        for row in current_queue {
            inserter.write(&row).await?;
            inserter.commit().await?;
        }

        inserter.end().await?;

        Ok(())
    }
}

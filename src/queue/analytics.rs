use crate::database::models::DatabaseError;
use crate::database::redis::RedisPool;
use crate::models::analytics::{Download, PageView, Playtime};
use crate::routes::ApiError;
use dashmap::{DashMap, DashSet};
use redis::cmd;
use sqlx::PgPool;

const DOWNLOADS_NAMESPACE: &str = "downloads";

pub struct AnalyticsQueue {
    views_queue: DashSet<PageView>,
    downloads_queue: DashMap<String, Download>,
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
            downloads_queue: DashMap::with_capacity(1000),
            playtime_queue: DashSet::with_capacity(1000),
        }
    }

    pub fn add_view(&self, page_view: PageView) {
        self.views_queue.insert(page_view);
    }

    pub fn add_download(&self, download: Download) {
        let ip_stripped = if let Some(ip) = download.ip.to_ipv4_mapped() {
            let octets = ip.octets();
            u64::from_be_bytes([0, 0, 0, 0, octets[0], octets[1], octets[2], octets[3]])
        } else {
            let octets = download.ip.octets();
            u64::from_be_bytes([0, 0, 0, 0, octets[0], octets[1], octets[2], octets[3]])
        };
        self.downloads_queue
            .insert(format!("{}-{}", ip_stripped, download.project_id), download);
    }

    pub fn add_playtime(&self, playtime: Playtime) {
        self.playtime_queue.insert(playtime);
    }

    pub async fn index(
        &self,
        client: clickhouse::Client,
        redis: &RedisPool,
        pool: &PgPool,
    ) -> Result<(), ApiError> {
        let views_queue = self.views_queue.clone();
        self.views_queue.clear();

        let downloads_queue = self.downloads_queue.clone();
        self.downloads_queue.clear();

        let playtime_queue = self.playtime_queue.clone();
        self.playtime_queue.clear();

        if !views_queue.is_empty() {
            let mut views = client.insert("views")?;

            for view in views_queue {
                views.write(&view).await?;
            }

            views.end().await?;
        }

        if !playtime_queue.is_empty() {
            let mut playtimes = client.insert("playtime")?;

            for playtime in playtime_queue {
                playtimes.write(&playtime).await?;
            }

            playtimes.end().await?;
        }

        if !downloads_queue.is_empty() {
            let mut downloads_keys = Vec::new();
            let raw_downloads = DashMap::new();

            for (index, (key, download)) in downloads_queue.into_iter().enumerate() {
                downloads_keys.push(key);
                raw_downloads.insert(index, download);
            }

            let mut redis = redis.pool.get().await.map_err(DatabaseError::RedisPool)?;

            let results = cmd("MGET")
                .arg(
                    downloads_keys
                        .iter()
                        .map(|x| format!("{}:{}", DOWNLOADS_NAMESPACE, x))
                        .collect::<Vec<_>>(),
                )
                .query_async::<_, Vec<Option<u32>>>(&mut redis)
                .await
                .map_err(DatabaseError::CacheError)?;

            let mut pipe = redis::pipe();
            for (idx, count) in results.into_iter().enumerate() {
                let key = &downloads_keys[idx];

                let new_count = if let Some(count) = count {
                    if count > 5 {
                        raw_downloads.remove(&idx);
                        continue;
                    }

                    count + 1
                } else {
                    1
                };

                pipe.atomic().set_ex(
                    format!("{}:{}", DOWNLOADS_NAMESPACE, key),
                    new_count,
                    6 * 60 * 60,
                );
            }
            pipe.query_async(&mut *redis)
                .await
                .map_err(DatabaseError::CacheError)?;

            let version_ids = raw_downloads
                .iter()
                .map(|x| x.version_id as i64)
                .collect::<Vec<_>>();
            let project_ids = raw_downloads
                .iter()
                .map(|x| x.project_id as i64)
                .collect::<Vec<_>>();

            let mut transaction = pool.begin().await?;
            let mut downloads = client.insert("downloads")?;

            for (_, download) in raw_downloads {
                downloads.write(&download).await?;
            }

            sqlx::query!(
                "UPDATE versions
                SET downloads = downloads + 1
                WHERE id = ANY($1)",
                &version_ids
            )
            .execute(&mut *transaction)
            .await?;

            sqlx::query!(
                "UPDATE mods
                SET downloads = downloads + 1
                WHERE id = ANY($1)",
                &project_ids
            )
            .execute(&mut *transaction)
            .await?;

            transaction.commit().await?;
            downloads.end().await?;
        }

        Ok(())
    }
}

use crate::database::models::DatabaseError;
use crate::database::redis::RedisPool;
use crate::models::analytics::{Download, PageView, Playtime};
use crate::routes::ApiError;
use dashmap::{DashMap, DashSet};
use redis::cmd;
use sqlx::PgPool;
use std::collections::HashMap;

const DOWNLOADS_NAMESPACE: &str = "downloads";
const VIEWS_NAMESPACE: &str = "views";

pub struct AnalyticsQueue {
    views_queue: DashMap<(u64, u64), Vec<PageView>>,
    downloads_queue: DashMap<(u64, u64), Download>,
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
            views_queue: DashMap::with_capacity(1000),
            downloads_queue: DashMap::with_capacity(1000),
            playtime_queue: DashSet::with_capacity(1000),
        }
    }

    pub fn add_view(&self, page_view: PageView) {
        let ip_stripped = crate::util::ip::strip_ip(page_view.ip);

        self.views_queue
            .entry((ip_stripped, page_view.project_id))
            .or_default()
            .push(page_view);
    }
    pub fn add_download(&self, download: Download) {
        let ip_stripped = crate::util::ip::strip_ip(download.ip);
        self.downloads_queue
            .insert((ip_stripped, download.project_id), download);
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

        if !playtime_queue.is_empty() {
            let mut playtimes = client.insert("playtime")?;

            for playtime in playtime_queue {
                playtimes.write(&playtime).await?;
            }

            playtimes.end().await?;
        }

        if !views_queue.is_empty() {
            let mut views_keys = Vec::new();
            let mut raw_views = Vec::new();

            for (key, views) in views_queue {
                views_keys.push(key);
                raw_views.push((views, true));
            }

            let mut redis =
                redis.pool.get().await.map_err(DatabaseError::RedisPool)?;

            let results = cmd("MGET")
                .arg(
                    views_keys
                        .iter()
                        .map(|x| format!("{}:{}-{}", VIEWS_NAMESPACE, x.0, x.1))
                        .collect::<Vec<_>>(),
                )
                .query_async::<Vec<Option<u32>>>(&mut redis)
                .await
                .map_err(DatabaseError::CacheError)?;

            let mut pipe = redis::pipe();
            for (idx, count) in results.into_iter().enumerate() {
                let key = &views_keys[idx];

                let new_count =
                    if let Some((views, monetized)) = raw_views.get_mut(idx) {
                        if let Some(count) = count {
                            if count > 3 {
                                *monetized = false;
                                continue;
                            }

                            if (count + views.len() as u32) > 3 {
                                *monetized = false;
                            }

                            count + (views.len() as u32)
                        } else {
                            views.len() as u32
                        }
                    } else {
                        1
                    };

                pipe.atomic().set_ex(
                    format!("{}:{}-{}", VIEWS_NAMESPACE, key.0, key.1),
                    new_count,
                    6 * 60 * 60,
                );
            }
            pipe.query_async::<()>(&mut *redis)
                .await
                .map_err(DatabaseError::CacheError)?;

            let mut views = client.insert("views")?;

            for (all_views, monetized) in raw_views {
                for (idx, mut view) in all_views.into_iter().enumerate() {
                    if idx != 0 || !monetized {
                        view.monetized = false;
                    }

                    views.write(&view).await?;
                }
            }

            views.end().await?;
        }

        if !downloads_queue.is_empty() {
            let mut downloads_keys = Vec::new();
            let raw_downloads = DashMap::new();

            for (index, (key, download)) in
                downloads_queue.into_iter().enumerate()
            {
                downloads_keys.push(key);
                raw_downloads.insert(index, download);
            }

            let mut redis =
                redis.pool.get().await.map_err(DatabaseError::RedisPool)?;

            let results = cmd("MGET")
                .arg(
                    downloads_keys
                        .iter()
                        .map(|x| {
                            format!("{}:{}-{}", DOWNLOADS_NAMESPACE, x.0, x.1)
                        })
                        .collect::<Vec<_>>(),
                )
                .query_async::<Vec<Option<u32>>>(&mut redis)
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
                    format!("{}:{}-{}", DOWNLOADS_NAMESPACE, key.0, key.1),
                    new_count,
                    6 * 60 * 60,
                );
            }
            pipe.query_async::<()>(&mut *redis)
                .await
                .map_err(DatabaseError::CacheError)?;

            let mut transaction = pool.begin().await?;
            let mut downloads = client.insert("downloads")?;

            let mut version_downloads: HashMap<i64, i32> = HashMap::new();
            let mut project_downloads: HashMap<i64, i32> = HashMap::new();

            for (_, download) in raw_downloads {
                *version_downloads
                    .entry(download.version_id as i64)
                    .or_default() += 1;
                *project_downloads
                    .entry(download.project_id as i64)
                    .or_default() += 1;

                downloads.write(&download).await?;
            }

            sqlx::query(
                "
                UPDATE versions v
                SET downloads = v.downloads + x.amount
                FROM unnest($1::BIGINT[], $2::int[]) AS x(id, amount)
                WHERE v.id = x.id
                ",
            )
            .bind(version_downloads.keys().copied().collect::<Vec<_>>())
            .bind(version_downloads.values().copied().collect::<Vec<_>>())
            .execute(&mut *transaction)
            .await?;

            sqlx::query(
                "
                    UPDATE mods m
                    SET downloads = m.downloads + x.amount
                    FROM unnest($1::BIGINT[], $2::int[]) AS x(id, amount)
                    WHERE m.id = x.id
                    ",
            )
            .bind(project_downloads.keys().copied().collect::<Vec<_>>())
            .bind(project_downloads.values().copied().collect::<Vec<_>>())
            .execute(&mut *transaction)
            .await?;

            transaction.commit().await?;
            downloads.end().await?;
        }

        Ok(())
    }
}

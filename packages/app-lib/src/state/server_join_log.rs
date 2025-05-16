use std::collections::HashMap;

use chrono::{DateTime, TimeZone, Utc};

#[derive(Default)]
pub struct JoinLogEntry {
    pub profile_path: String,
    pub host: String,
    pub port: u16,
    pub join_time: DateTime<Utc>,
}

impl JoinLogEntry {
    pub async fn upsert(
        &self,
        exec: impl sqlx::Executor<'_, Database = sqlx::Sqlite>,
    ) -> crate::Result<()> {
        let join_time = self.join_time.timestamp();

        sqlx::query!(
            "
            INSERT INTO join_log (profile_path, host, port, join_time)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (profile_path, host, port) DO UPDATE SET
                join_time = $4
            ",
            self.profile_path,
            self.host,
            self.port,
            join_time
        )
        .execute(exec)
        .await?;

        Ok(())
    }
}

pub async fn get_joins(
    instance: &str,
    exec: impl sqlx::Executor<'_, Database = sqlx::Sqlite>,
) -> crate::Result<HashMap<(String, u16), DateTime<Utc>>> {
    let joins = sqlx::query!(
        "
        SELECT profile_path, host, port, join_time
        FROM join_log
        WHERE profile_path = $1
        ",
        instance
    )
    .fetch_all(exec)
    .await?;

    Ok(joins
        .into_iter()
        .map(|x| {
            (
                (x.host, x.port as u16),
                Utc.timestamp_opt(x.join_time, 0)
                    .single()
                    .unwrap_or_else(Utc::now),
            )
        })
        .collect())
}

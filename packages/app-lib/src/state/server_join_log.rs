use std::collections::HashMap;

use chrono::{DateTime, TimeZone, Utc};

#[derive(Default)]
pub struct JoinLogEntry {
    pub instance_id: String,
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
        let instance_id = self.instance_id.as_str();
        let host = self.host.as_str();
        let port = i64::from(self.port);

        sqlx::query!(
            "
			INSERT INTO join_log (instance_id, host, port, join_time)
			VALUES (?, ?, ?, ?)
			ON CONFLICT (instance_id, host, port) DO UPDATE SET
				join_time = excluded.join_time
			",
            instance_id,
            host,
            port,
            join_time,
        )
        .execute(exec)
        .await?;

        Ok(())
    }
}

pub async fn get_joins(
    instance_id: &str,
    exec: impl sqlx::Executor<'_, Database = sqlx::Sqlite>,
) -> crate::Result<HashMap<(String, u16), DateTime<Utc>>> {
    let joins = sqlx::query!(
        "
		SELECT host, port, join_time
		FROM join_log
		WHERE instance_id = ?
		",
        instance_id,
    )
    .fetch_all(exec)
    .await?;

    Ok(joins
        .into_iter()
        .map(|row| {
            (
                (row.host, row.port as u16),
                Utc.timestamp_opt(row.join_time, 0)
                    .single()
                    .unwrap_or_else(Utc::now),
            )
        })
        .collect())
}

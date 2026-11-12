use chrono::{DateTime, Utc};
use eyre::{Result, WrapErr};
use serde::Serialize;
use serde_json::Value;

use temporalio_client::{Client, ClientOptions, ConnectionOptions};

use crate::database::{Executor, PgPool};
use crate::env::ENV;

#[derive(Debug)]
pub struct TemporalClient {
    pub client: Client,
    pub memos: WorkflowMemos,
}

impl TemporalClient {
    pub fn new(client: Client, db: PgPool) -> Self {
        Self {
            client,
            memos: WorkflowMemos::new(db),
        }
    }

    pub async fn from_env(db: PgPool) -> Result<Self> {
        let client = Client::connect(
            ConnectionOptions::new(ENV.TEMPORAL_ADDRESS.clone()).build(),
            ClientOptions::new(ENV.TEMPORAL_NAMESPACE.clone()).build(),
        )
        .await
        .wrap_err("connecting to temporal")?;

        Ok(Self::new(client, db))
    }
}

#[derive(Debug)]
pub struct WorkflowMemos {
    db: PgPool,
}

impl WorkflowMemos {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    pub async fn get(
        &self,
        activity: &str,
        key: impl Serialize,
    ) -> Result<Option<Value>> {
        let key = memo_key(&key)?;
        Ok(WorkflowMemoModel::get(&self.db, activity, &key)
            .await?
            .map(|memo| memo.out))
    }

    pub async fn insert(
        &self,
        activity: &str,
        key: impl Serialize,
        out: Value,
    ) -> Result<()> {
        let key = memo_key(&key)?;
        WorkflowMemoModel::insert(&self.db, activity, &key, &out).await
    }

    pub async fn remove(
        &self,
        activity: &str,
        key: impl Serialize,
    ) -> Result<bool> {
        let key = memo_key(&key)?;
        WorkflowMemoModel::remove(&self.db, activity, &key).await
    }
}

// have to use an `impl Serialize` as the key, not an `impl Hash`,
// since although BLAKE3 is a deterministic hash,
// the *values* that Rust's `Hash` passes to BLAKE3 to combine it with
// the previous has are *not* deterministic.
// e.g. struct layout reordering, etc. can change the order of how
// inputs are fed in.
fn memo_key(key: &impl Serialize) -> Result<Vec<u8>> {
    let mut value =
        serde_json::to_value(key).wrap_err("serializing workflow memo key")?;
    value.sort_all_objects();
    let bytes =
        serde_json::to_vec(&value).wrap_err("encoding workflow memo key")?;
    Ok(blake3::hash(&bytes).as_bytes().to_vec())
}

#[derive(Debug, Clone)]
struct WorkflowMemoModel {
    activity: String,
    key: Vec<u8>,
    out: Value,
    created_at: DateTime<Utc>,
}

impl WorkflowMemoModel {
    async fn get(
        exec: impl Executor<'_, Database = sqlx::Postgres>,
        activity: &str,
        key: &[u8],
    ) -> Result<Option<Self>> {
        let row = sqlx::query!(
            r#"
            SELECT activity, key, out, created_at
            FROM workflow_memos
            WHERE activity = $1 AND key = $2
            "#,
            activity,
            key,
        )
        .fetch_optional(exec)
        .await
        .wrap_err("fetching workflow memo")?;

        Ok(row.map(|row| Self {
            activity: row.activity,
            key: row.key,
            out: row.out,
            created_at: row.created_at,
        }))
    }

    async fn insert(
        exec: impl Executor<'_, Database = sqlx::Postgres>,
        activity: &str,
        key: &[u8],
        out: &Value,
    ) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO workflow_memos (activity, key, out)
            VALUES ($1, $2, $3)
            ON CONFLICT (activity, key) DO NOTHING
            "#,
            activity,
            key,
            out,
        )
        .execute(exec)
        .await
        .wrap_err("inserting workflow memo")?;

        Ok(())
    }

    async fn remove(
        exec: impl Executor<'_, Database = sqlx::Postgres>,
        activity: &str,
        key: &[u8],
    ) -> Result<bool> {
        let result = sqlx::query!(
            r#"
            DELETE FROM workflow_memos
            WHERE activity = $1 AND key = $2
            "#,
            activity,
            key,
        )
        .execute(exec)
        .await
        .wrap_err("removing workflow memo")?;

        Ok(result.rows_affected() > 0)
    }
}

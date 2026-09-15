use std::fmt::{self, Display, Formatter};

use uuid::Uuid;

use crate::database::{PgTransaction, models::DBFileId};

/// A transaction-scoped PostgreSQL advisory lock used by Labrinth.
///
/// PostgreSQL advisory lock keys share one database-wide keyspace. Defining
/// them here gives each resource a stable namespace and prevents unrelated
/// numeric IDs from contending with each other.
pub enum AdvisoryLock {
    DelphiFile(DBFileId),
    DelphiScan(Uuid),
    DiscordRoleEmailCampaign,
}

impl AdvisoryLock {
    pub async fn acquire(
        &self,
        transaction: &mut PgTransaction<'_>,
    ) -> Result<(), sqlx::Error> {
        let key = self.to_string();
        sqlx::query!(
            "SELECT pg_advisory_xact_lock(hashtextextended($1, 0))",
            key,
        )
        .execute(&mut *transaction)
        .await?;

        Ok(())
    }

    pub async fn try_acquire(
        &self,
        transaction: &mut PgTransaction<'_>,
    ) -> Result<bool, sqlx::Error> {
        let key = self.to_string();
        sqlx::query_scalar!(
            r#"SELECT pg_try_advisory_xact_lock(hashtextextended($1, 0)) AS "lock_acquired!""#,
            key,
        )
        .fetch_one(&mut *transaction)
        .await
    }
}

impl Display for AdvisoryLock {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::DelphiFile(file_id) => {
                write!(formatter, "labrinth:delphi-file:{}", file_id.0)
            }
            Self::DelphiScan(scan_id) => {
                write!(formatter, "labrinth:delphi-scan:{scan_id}")
            }
            Self::DiscordRoleEmailCampaign => {
                formatter.write_str("discord_role_email_campaign")
            }
        }
    }
}

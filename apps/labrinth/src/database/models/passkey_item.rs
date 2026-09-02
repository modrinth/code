use super::ids::*;
use crate::database::PgTransaction;
use crate::database::models::DatabaseError;
use chrono::{DateTime, Utc};
use futures::TryStreamExt;
use serde::{Deserialize, Serialize};
use sqlx::types::Json;
use webauthn_rs::prelude::Passkey;

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct DBPasskey {
    pub id: DBPasskeyId,
    pub user_id: DBUserId,
    pub name: String,
    pub credential_id: Vec<u8>,
    pub passkey: Passkey,
    pub created_at: DateTime<Utc>,
    pub last_used: Option<DateTime<Utc>>,
}

impl DBPasskey {
    pub async fn insert(
        &self,
        transaction: &mut PgTransaction<'_>,
    ) -> Result<(), DatabaseError> {
        sqlx::query!(
            "
            INSERT INTO user_passkeys (
                id, user_id, name, credential_id, passkey, created_at, last_used
            )
            VALUES (
                $1, $2 ,$3, $4, $5, $6, $7
            )
            ",
            self.id as DBPasskeyId,
            self.user_id as DBUserId,
            self.name,
            self.credential_id,
            Json(&self.passkey) as _,
            self.created_at,
            self.last_used,
        )
        .execute(&mut *transaction)
        .await?;

        Ok(())
    }

    pub async fn get_by_credential_id<'a, E>(
        credential_id: &[u8],
        exec: E,
    ) -> Result<Option<DBPasskey>, DatabaseError>
    where
        E: crate::database::Executor<'a, Database = sqlx::Postgres>,
    {
        let row = sqlx::query!(
            r#"
            SELECT id, user_id, name, credential_id,
                   passkey AS "passkey: sqlx::types::Json<Passkey>",
                   last_used, created_at
            FROM user_passkeys
            WHERE credential_id = $1
            "#,
            credential_id,
        )
        .fetch_optional(exec)
        .await?
        .map(|x| DBPasskey {
            id: DBPasskeyId(x.id),
            user_id: DBUserId(x.user_id),
            name: x.name,
            credential_id: x.credential_id,
            passkey: x.passkey.0,
            created_at: x.created_at,
            last_used: x.last_used,
        });

        Ok(row)
    }

    pub async fn get_for_user<'a, E>(
        user_id: DBUserId,
        exec: E,
    ) -> Result<Vec<DBPasskey>, DatabaseError>
    where
        E: crate::database::Executor<'a, Database = sqlx::Postgres>,
    {
        let passkeys = sqlx::query!(
            r#"
            SELECT id, user_id, name, credential_id,
                   passkey AS "passkey: sqlx::types::Json<Passkey>",
                   last_used, created_at
            FROM user_passkeys
            WHERE user_id = $1
            ORDER BY created_at DESC
            "#,
            user_id.0,
        )
        .fetch(exec)
        .map_ok(|x| DBPasskey {
            id: DBPasskeyId(x.id),
            user_id: DBUserId(x.user_id),
            name: x.name,
            credential_id: x.credential_id,
            passkey: x.passkey.0,
            created_at: x.created_at,
            last_used: x.last_used,
        })
        .try_collect::<Vec<DBPasskey>>()
        .await?;

        Ok(passkeys)
    }

    pub async fn rename(
        id: DBPasskeyId,
        user_id: DBUserId,
        name: &str,
        transaction: &mut PgTransaction<'_>,
    ) -> Result<bool, DatabaseError> {
        let result = sqlx::query!(
            "
            UPDATE user_passkeys SET name = $1
            WHERE id = $2 AND user_id = $3
            ",
            name,
            id as DBPasskeyId,
            user_id as DBUserId,
        )
        .execute(&mut *transaction)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn update_after_auth(
        id: DBPasskeyId,
        passkey: Passkey,
        transaction: &mut PgTransaction<'_>,
    ) -> Result<bool, DatabaseError> {
        let result = sqlx::query!(
            "
            UPDATE user_passkeys
            SET passkey = $1, last_used = NOW()
            WHERE id = $2
            ",
            Json(&passkey) as _,
            id as DBPasskeyId,
        )
        .execute(&mut *transaction)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn remove(
        id: DBPasskeyId,
        transaction: &mut PgTransaction<'_>,
    ) -> Result<bool, DatabaseError> {
        let result = sqlx::query!(
            "
            DELETE FROM user_passkeys
            WHERE id = $1
            ",
            id as DBPasskeyId,
        )
        .execute(&mut *transaction)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn remove_for_user(
        id: DBPasskeyId,
        user_id: DBUserId,
        transaction: &mut PgTransaction<'_>,
    ) -> Result<bool, DatabaseError> {
        let result = sqlx::query!(
            "
            DELETE FROM user_passkeys
            WHERE id = $1 AND user_id = $2
            ",
            id as DBPasskeyId,
            user_id as DBUserId,
        )
        .execute(&mut *transaction)
        .await?;

        Ok(result.rows_affected() > 0)
    }
}

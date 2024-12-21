use chrono::{DateTime, Utc};
use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::models::pats::Scopes;

use super::{DatabaseError, OAuthClientAuthorizationId, OAuthClientId, UserId};

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct OAuthClientAuthorization {
    pub id: OAuthClientAuthorizationId,
    pub client_id: OAuthClientId,
    pub user_id: UserId,
    pub scopes: Scopes,
    pub created: DateTime<Utc>,
}

struct AuthorizationQueryResult {
    id: i64,
    client_id: i64,
    user_id: i64,
    scopes: i64,
    created: DateTime<Utc>,
}

impl From<AuthorizationQueryResult> for OAuthClientAuthorization {
    fn from(value: AuthorizationQueryResult) -> Self {
        OAuthClientAuthorization {
            id: OAuthClientAuthorizationId(value.id),
            client_id: OAuthClientId(value.client_id),
            user_id: UserId(value.user_id),
            scopes: Scopes::from_postgres(value.scopes),
            created: value.created,
        }
    }
}

impl OAuthClientAuthorization {
    pub async fn get(
        client_id: OAuthClientId,
        user_id: UserId,
        exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    ) -> Result<Option<OAuthClientAuthorization>, DatabaseError> {
        let value = sqlx::query_as!(
            AuthorizationQueryResult,
            "
            SELECT id, client_id, user_id, scopes, created
            FROM oauth_client_authorizations
            WHERE client_id=$1 AND user_id=$2
            ",
            client_id.0,
            user_id.0,
        )
        .fetch_optional(exec)
        .await?;

        Ok(value.map(|r| r.into()))
    }

    pub async fn get_all_for_user(
        user_id: UserId,
        exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    ) -> Result<Vec<OAuthClientAuthorization>, DatabaseError> {
        let results = sqlx::query_as!(
            AuthorizationQueryResult,
            "
            SELECT id, client_id, user_id, scopes, created
            FROM oauth_client_authorizations
            WHERE user_id=$1
            ",
            user_id.0
        )
        .fetch_all(exec)
        .await?;

        Ok(results.into_iter().map(|r| r.into()).collect_vec())
    }

    pub async fn upsert(
        id: OAuthClientAuthorizationId,
        client_id: OAuthClientId,
        user_id: UserId,
        scopes: Scopes,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<(), DatabaseError> {
        sqlx::query!(
            "
            INSERT INTO oauth_client_authorizations (
                id, client_id, user_id, scopes
            )
            VALUES (
                $1, $2, $3, $4
            )
            ON CONFLICT (id)
            DO UPDATE SET scopes = EXCLUDED.scopes
            ",
            id.0,
            client_id.0,
            user_id.0,
            scopes.bits() as i64,
        )
        .execute(&mut **transaction)
        .await?;

        Ok(())
    }

    pub async fn remove(
        client_id: OAuthClientId,
        user_id: UserId,
        exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    ) -> Result<(), DatabaseError> {
        sqlx::query!(
            "
            DELETE FROM oauth_client_authorizations
            WHERE client_id=$1 AND user_id=$2
            ",
            client_id.0,
            user_id.0
        )
        .execute(exec)
        .await?;

        Ok(())
    }
}

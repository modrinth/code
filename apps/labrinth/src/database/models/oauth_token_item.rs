use super::{
    DBOAuthAccessTokenId, DBOAuthClientAuthorizationId, DBOAuthClientId,
    DBUserId, DatabaseError,
};
use crate::models::pats::Scopes;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::Digest;

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct DBOAuthAccessToken {
    pub id: DBOAuthAccessTokenId,
    pub authorization_id: DBOAuthClientAuthorizationId,
    pub token_hash: String,
    pub scopes: Scopes,
    pub created: DateTime<Utc>,
    pub expires: DateTime<Utc>,
    pub last_used: Option<DateTime<Utc>>,

    // Stored separately inside oauth_client_authorizations table
    pub client_id: DBOAuthClientId,
    pub user_id: DBUserId,
}

impl DBOAuthAccessToken {
    pub async fn get(
        token_hash: String,
        exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    ) -> Result<Option<DBOAuthAccessToken>, DatabaseError> {
        let value = sqlx::query!(
            "
            SELECT
                tokens.id,
                tokens.authorization_id,
                tokens.token_hash,
                tokens.scopes,
                tokens.created,
                tokens.expires,
                tokens.last_used,
                auths.client_id,
                auths.user_id
            FROM oauth_access_tokens tokens
            JOIN oauth_client_authorizations auths
            ON tokens.authorization_id = auths.id
            WHERE tokens.token_hash = $1
            ",
            token_hash
        )
        .fetch_optional(exec)
        .await?;

        Ok(value.map(|r| DBOAuthAccessToken {
            id: DBOAuthAccessTokenId(r.id),
            authorization_id: DBOAuthClientAuthorizationId(r.authorization_id),
            token_hash: r.token_hash,
            scopes: Scopes::from_postgres(r.scopes),
            created: r.created,
            expires: r.expires,
            last_used: r.last_used,
            client_id: DBOAuthClientId(r.client_id),
            user_id: DBUserId(r.user_id),
        }))
    }

    /// Inserts and returns the time until the token expires
    pub async fn insert(
        &self,
        exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    ) -> Result<chrono::Duration, DatabaseError> {
        let r = sqlx::query!(
            "
            INSERT INTO oauth_access_tokens (
                id, authorization_id, token_hash, scopes, last_used
            )
            VALUES (
                $1, $2, $3, $4, $5
            )
            RETURNING created, expires
            ",
            self.id.0,
            self.authorization_id.0,
            self.token_hash,
            self.scopes.to_postgres(),
            Option::<DateTime<Utc>>::None
        )
        .fetch_one(exec)
        .await?;

        let (created, expires) = (r.created, r.expires);
        let time_until_expiration = expires - created;

        Ok(time_until_expiration)
    }

    pub fn hash_token(token: &str) -> String {
        format!("{:x}", sha2::Sha512::digest(token.as_bytes()))
    }
}

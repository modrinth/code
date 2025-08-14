use chrono::{DateTime, Utc};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use sha2::Digest;

use super::{DBOAuthClientId, DBOAuthRedirectUriId, DBUserId, DatabaseError};
use crate::models::pats::Scopes;

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct DBOAuthRedirectUri {
    pub id: DBOAuthRedirectUriId,
    pub client_id: DBOAuthClientId,
    pub uri: String,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct DBOAuthClient {
    pub id: DBOAuthClientId,
    pub name: String,
    pub icon_url: Option<String>,
    pub raw_icon_url: Option<String>,
    pub max_scopes: Scopes,
    pub secret_hash: String,
    pub redirect_uris: Vec<DBOAuthRedirectUri>,
    pub created: DateTime<Utc>,
    pub created_by: DBUserId,
    pub url: Option<String>,
    pub description: Option<String>,
}

struct OAuthClientQueryResult {
    id: i64,
    name: String,
    icon_url: Option<String>,
    raw_icon_url: Option<String>,
    max_scopes: i64,
    secret_hash: String,
    created: DateTime<Utc>,
    created_by: i64,
    url: Option<String>,
    description: Option<String>,
    uri_ids: Option<Vec<i64>>,
    uri_vals: Option<Vec<String>>,
}

macro_rules! select_clients_with_predicate {
    ($predicate:tt, $param:ident) => {
        // The columns in this query have nullability type hints, because for some reason
        // the combination of the JOIN and filter using ANY makes sqlx think all columns are nullable
        // https://docs.rs/sqlx/latest/sqlx/macro.query.html#force-nullable
        sqlx::query_as!(
            OAuthClientQueryResult,
            r#"
            SELECT
                clients.id as "id!",
                clients.name as "name!",
                clients.icon_url as "icon_url?",
                clients.raw_icon_url as "raw_icon_url?",
                clients.max_scopes as "max_scopes!",
                clients.secret_hash as "secret_hash!",
                clients.created as "created!",
                clients.created_by as "created_by!",
                clients.url as "url?",
                clients.description as "description?",
                uris.uri_ids as "uri_ids?",
                uris.uri_vals as "uri_vals?"
            FROM oauth_clients clients
            LEFT JOIN (
                SELECT client_id, array_agg(id) as uri_ids, array_agg(uri) as uri_vals
                FROM oauth_client_redirect_uris
                GROUP BY client_id
            ) uris ON clients.id = uris.client_id
            "#
                + $predicate,
            $param
        )
    };
}

impl DBOAuthClient {
    pub async fn get(
        id: DBOAuthClientId,
        exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    ) -> Result<Option<DBOAuthClient>, DatabaseError> {
        Ok(Self::get_many(&[id], exec).await?.into_iter().next())
    }

    pub async fn get_many(
        ids: &[DBOAuthClientId],
        exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    ) -> Result<Vec<DBOAuthClient>, DatabaseError> {
        let ids = ids.iter().map(|id| id.0).collect_vec();
        let ids_ref: &[i64] = &ids;
        let results = select_clients_with_predicate!(
            "WHERE clients.id = ANY($1::bigint[])",
            ids_ref
        )
        .fetch_all(exec)
        .await?;

        Ok(results.into_iter().map(|r| r.into()).collect_vec())
    }

    pub async fn get_all_user_clients(
        user_id: DBUserId,
        exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    ) -> Result<Vec<DBOAuthClient>, DatabaseError> {
        let user_id_param = user_id.0;
        let clients = select_clients_with_predicate!(
            "WHERE created_by = $1",
            user_id_param
        )
        .fetch_all(exec)
        .await?;

        Ok(clients.into_iter().map(|r| r.into()).collect())
    }

    pub async fn remove(
        id: DBOAuthClientId,
        exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    ) -> Result<(), DatabaseError> {
        // Cascades to oauth_client_redirect_uris, oauth_client_authorizations
        sqlx::query!(
            "
            DELETE FROM oauth_clients
            WHERE id = $1
            ",
            id.0
        )
        .execute(exec)
        .await?;

        Ok(())
    }

    pub async fn insert(
        &self,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<(), DatabaseError> {
        sqlx::query!(
            "
            INSERT INTO oauth_clients (
                id, name, icon_url, raw_icon_url, max_scopes, secret_hash, created_by
            )
            VALUES (
                $1, $2, $3, $4, $5, $6, $7
            )
            ",
            self.id.0,
            self.name,
            self.icon_url,
            self.raw_icon_url,
            self.max_scopes.to_postgres(),
            self.secret_hash,
            self.created_by.0
        )
        .execute(&mut **transaction)
        .await?;

        Self::insert_redirect_uris(&self.redirect_uris, &mut **transaction)
            .await?;

        Ok(())
    }

    pub async fn update_editable_fields(
        &self,
        exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    ) -> Result<(), DatabaseError> {
        sqlx::query!(
            "
            UPDATE oauth_clients
            SET name = $1, icon_url = $2, raw_icon_url = $3, max_scopes = $4, url = $5, description = $6
            WHERE (id = $7)
            ",
            self.name,
            self.icon_url,
            self.raw_icon_url,
            self.max_scopes.to_postgres(),
            self.url,
            self.description,
            self.id.0,
        )
        .execute(exec)
        .await?;

        Ok(())
    }

    pub async fn remove_redirect_uris(
        ids: impl IntoIterator<Item = DBOAuthRedirectUriId>,
        exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    ) -> Result<(), DatabaseError> {
        let ids = ids.into_iter().map(|id| id.0).collect_vec();
        sqlx::query!(
            "
            DELETE FROM oauth_client_redirect_uris
            WHERE id IN
            (SELECT * FROM UNNEST($1::bigint[]))
            ",
            &ids[..]
        )
        .execute(exec)
        .await?;

        Ok(())
    }

    pub async fn insert_redirect_uris(
        uris: &[DBOAuthRedirectUri],
        exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    ) -> Result<(), DatabaseError> {
        let (ids, client_ids, uris): (Vec<_>, Vec<_>, Vec<_>) = uris
            .iter()
            .map(|r| (r.id.0, r.client_id.0, r.uri.clone()))
            .multiunzip();
        sqlx::query!(
            "
            INSERT INTO oauth_client_redirect_uris (id, client_id, uri)
            SELECT * FROM UNNEST($1::bigint[], $2::bigint[], $3::varchar[])
            ",
            &ids[..],
            &client_ids[..],
            &uris[..],
        )
        .execute(exec)
        .await?;

        Ok(())
    }

    pub fn hash_secret(secret: &str) -> String {
        format!("{:x}", sha2::Sha512::digest(secret.as_bytes()))
    }
}

impl From<OAuthClientQueryResult> for DBOAuthClient {
    fn from(r: OAuthClientQueryResult) -> Self {
        let redirects = if let (Some(ids), Some(uris)) =
            (r.uri_ids.as_ref(), r.uri_vals.as_ref())
        {
            ids.iter()
                .zip(uris.iter())
                .map(|(id, uri)| DBOAuthRedirectUri {
                    id: DBOAuthRedirectUriId(*id),
                    client_id: DBOAuthClientId(r.id),
                    uri: uri.to_string(),
                })
                .collect()
        } else {
            vec![]
        };

        DBOAuthClient {
            id: DBOAuthClientId(r.id),
            name: r.name,
            icon_url: r.icon_url,
            raw_icon_url: r.raw_icon_url,
            max_scopes: Scopes::from_postgres(r.max_scopes),
            secret_hash: r.secret_hash,
            redirect_uris: redirects,
            created: r.created,
            created_by: DBUserId(r.created_by),
            url: r.url,
            description: r.description,
        }
    }
}

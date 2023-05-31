/*!
Current edition of Ory kratos does not support PAT access of data, so this module is how we allow for PAT authentication.


Just as a summary: Don't implement this flow in your application!
*/

use super::auth::AuthenticationError;
use crate::database;
use crate::database::models::{DatabaseError, UserId};
use crate::models::users::{self, Badges, RecipientType, RecipientWallet};
use censor::Censor;
use chrono::{NaiveDateTime, Utc};
use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct PersonalAccessToken {
    pub id: String,
    pub name: Option<String>,
    pub access_token: Option<String>,
    pub scope: i64,
    pub user_id: users::UserId,
    pub expires_at: NaiveDateTime,
}
// Find database user from PAT token
// Separate to user_items as it may yet include further behaviour.
pub async fn get_user_from_pat<'a, E>(
    access_token: &str,
    executor: E,
) -> Result<Option<database::models::User>, AuthenticationError>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    let row = sqlx::query!(
        "
                SELECT pats.expires_at,
                    u.id, u.name, u.kratos_id, u.email, u.github_id,
                    u.avatar_url, u.username, u.bio,
                    u.created, u.role, u.badges,
                    u.balance, u.payout_wallet, u.payout_wallet_type,
                    u.payout_address
                FROM pats LEFT OUTER JOIN users u ON pats.user_id = u.id
                WHERE access_token = $1
                ",
        access_token
    )
    .fetch_optional(executor)
    .await?;
    if let Some(row) = row {
        if row.expires_at < Utc::now().naive_utc() {
            return Ok(None);
        }

        return Ok(Some(database::models::User {
            id: UserId(row.id),
            kratos_id: row.kratos_id,
            name: row.name,
            github_id: row.github_id,
            email: row.email,
            avatar_url: row.avatar_url,
            username: row.username,
            bio: row.bio,
            created: row.created,
            role: row.role,
            badges: Badges::from_bits(row.badges as u64).unwrap_or_default(),
            balance: row.balance,
            payout_wallet: row.payout_wallet.map(|x| RecipientWallet::from_string(&x)),
            payout_wallet_type: row
                .payout_wallet_type
                .map(|x| RecipientType::from_string(&x)),
            payout_address: row.payout_address,
        }));
    }
    Ok(None)
}

// Generate a new 128 char PAT token starting with 'mod_'
pub async fn generate_pat(
    con: &mut sqlx::Transaction<'_, sqlx::Postgres>,
) -> Result<String, DatabaseError> {
    let mut rng = rand::thread_rng();
    let mut retry_count = 0;
    let censor = Censor::Standard + Censor::Sex;

    // First generate the PAT token as a random 128 char string. This may include uppercase and lowercase and numbers only.
    loop {
        let mut access_token = String::with_capacity(63);
        access_token.push_str("mod_");
        for _ in 0..60 {
            let c = rng.gen_range(0..60);
            if c < 10 {
                access_token.push(char::from_u32(c + 48).unwrap()); // 0-9
            } else if c < 36 {
                access_token.push(char::from_u32(c + 55).unwrap()); // A-Z
            } else {
                access_token.push(char::from_u32(c + 61).unwrap()); // a-z
            }
        }
        let results = sqlx::query!(
            "
            SELECT EXISTS(SELECT 1 FROM pats WHERE access_token=$1)
        ",
            access_token
        )
        .fetch_one(&mut *con)
        .await?;

        if !results.exists.unwrap_or(true) && !censor.check(&access_token) {
            break Ok(access_token);
        }

        retry_count += 1;
        if retry_count > 15 {
            return Err(DatabaseError::RandomId);
        }
    }
}

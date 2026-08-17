use ariadne::ids::base62_impl::parse_base62;
use xredis::RedisPool;

use super::AuthenticationError;
use crate::database::{PgTransaction, models::DBUserId};

const TOTP_NAMESPACE: &str = "used_totp:v4";

pub async fn verify_2fa_code(
    input: &str,
    secret: &str,
    user_id: DBUserId,
    redis: &RedisPool,
) -> Result<bool, AuthenticationError> {
    let totp = totp_rs::TOTP::new(
        totp_rs::Algorithm::SHA1,
        6,
        1,
        30,
        totp_rs::Secret::Encoded(secret.to_owned())
            .to_bytes()
            .map_err(|_| AuthenticationError::InvalidCredentials)?,
    )
    .map_err(|_| AuthenticationError::InvalidCredentials)?;

    let mut conn = redis.connect().await?;
    let logical_key = format!("{input}-{}", user_id.0);
    let key = redis
        .key()
        .with_slot(TOTP_NAMESPACE, &logical_key, &logical_key);

    if conn.get(&key).await?.is_some() {
        return Err(AuthenticationError::InvalidCredentials);
    }

    let is_valid = totp
        .check_current(input)
        .map_err(|_| AuthenticationError::InvalidCredentials)?;

    if is_valid {
        conn.set(&key, "", Some(60)).await?;
    }

    Ok(is_valid)
}

pub async fn use_backup_code(
    input: &str,
    user_id: DBUserId,
    transaction: &mut PgTransaction<'_>,
) -> Result<bool, AuthenticationError> {
    let Ok(code) = parse_base62(input) else {
        return Ok(false);
    };

    let deleted = sqlx::query_scalar!(
        r#"
		DELETE FROM user_backup_codes
		WHERE user_id = $1 AND code = $2
		RETURNING code
		"#,
        user_id.0,
        code as i64,
    )
    .fetch_optional(&mut *transaction)
    .await?;

    Ok(deleted.is_some())
}

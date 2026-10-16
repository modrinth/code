use crate::auth::{get_user_from_headers, require_verified_email};
use crate::database::PgPool;
use crate::models::pats::Scopes;
use crate::queue::session::AuthQueue;
use crate::routes::ApiError;
use crate::util::crowdin::{CrowdinClient, IssueType};
use crate::util::error::Context as _;
use crate::util::ratelimit::{AsyncRateLimiter, GCRAParameters};
use actix_web::{HttpRequest, post, web};
use ariadne::ids::UserId;
use chrono::Utc;
use eyre::{Result, eyre};
use redis::{AsyncCommands, ExistenceCheck, SetExpiry, SetOptions};
use serde::Deserialize;
use std::fmt::Write as _;
use validator::Validate;
use xredis::RedisPool;

const REPORT_NAMESPACE: &str = "translation_feedback:v1";
const REPORT_EXPIRY_SECONDS: u64 = 60 * 60 * 24 * 7;
const DAILY_LIMIT_NAMESPACE: &str = "translation_feedback_daily:v1";
const DAILY_LIMIT_EXPIRY_SECONDS: i64 = 60 * 60 * 24;
const DAILY_LIMIT: u64 = 5;

pub fn config(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(translation_feedback_create);
}

#[derive(Debug, Clone, Copy, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum TranslationProduct {
    Website,
    App,
}

#[derive(Debug, Clone, Copy, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum TranslationFeedbackKind {
    /// The translation has the wrong meaning.
    Incorrect,
    /// The translation has spelling, grammar or punctuation mistakes.
    Typo,
    /// The translation uses different wording than the rest of the interface.
    Inconsistent,
    /// Placeholders, links or line breaks are broken or shown as raw text.
    Formatting,
    /// The text is still shown in English.
    Untranslated,
    Other,
}

impl TranslationFeedbackKind {
    fn issue_type(self) -> IssueType {
        match self {
            Self::Incorrect
            | Self::Typo
            | Self::Inconsistent
            | Self::Formatting
            | Self::Untranslated => IssueType::TranslationMistake,
            Self::Other => IssueType::GeneralQuestion,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Incorrect => "Incorrect translation",
            Self::Typo => "Spelling or grammar mistake",
            Self::Inconsistent => "Inconsistent translation",
            Self::Formatting => "Broken formatting",
            Self::Untranslated => "Not translated",
            Self::Other => "Other",
        }
    }
}

#[derive(Debug, Deserialize, Validate, utoipa::ToSchema)]
pub struct TranslationFeedback {
    pub product: TranslationProduct,
    /// Locale code the player is using, e.g. `de-DE`.
    #[validate(length(min = 2, max = 16))]
    pub locale: String,
    /// Message ID of the string being reported.
    #[validate(length(min = 1, max = 255))]
    pub key: String,
    pub kind: TranslationFeedbackKind,
    #[validate(length(max = 2000))]
    pub details: Option<String>,
}

/// Send feedback about a translated string to the translators.
///
/// The feedback is filed as an issue on the string in Crowdin for the
/// reported locale.
#[utoipa::path(
	tag = "translation_feedback",
	request_body = TranslationFeedback,
	responses((status = NO_CONTENT))
)]
#[post("/translation-feedback")]
pub async fn translation_feedback_create(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
    body: web::Json<TranslationFeedback>,
) -> Result<(), ApiError> {
    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::SESSION_ACCESS,
    )
    .await
    .wrap_auth_err("authenticating API request")?
    .1;

    require_verified_email(&user)?;

    let feedback = body.into_inner();
    feedback
        .validate()
        .map_err(|err| eyre!(err))
        .wrap_request_err("validating request")?;

    let limiter = AsyncRateLimiter::new(
        redis.get_ref().clone(),
        GCRAParameters::new(1, 5),
    );
    let decision = limiter
        .check_rate_limit(&format!("translation_feedback:{}", user.id))
        .await;
    if !decision.allowed {
        return Err(ApiError::RateLimit(eyre!(
            "rate limit exceeded; retry after {} milliseconds",
            decision.retry_after_ms.unwrap_or(0)
        )));
    }

    let Some(crowdin) = CrowdinClient::from_env() else {
        tracing::info!("Crowdin not configured, dropping translation feedback");
        return Ok(());
    };

    let daily_key = daily_limit_key(&redis, user.id);
    let reports_today = count_daily_report(&redis, &daily_key)
        .await
        .wrap_internal_err("counting daily translation feedback")?;
    if reports_today > DAILY_LIMIT {
        return Err(ApiError::RateLimit(eyre!(
            "daily translation feedback limit reached"
        )));
    }

    let language = crowdin
        .resolve_language(&redis, &feedback.locale)
        .await
        .wrap_internal_err("resolving Crowdin language")?
        .wrap_request_err_with(|| {
            format!("unsupported locale: `{}`", feedback.locale)
        })?;

    let string_id = crowdin
        .find_string_id(&redis, &feedback.key)
        .await
        .wrap_internal_err("finding Crowdin string")?
        .wrap_request_err_with(|| {
            format!("unknown message ID: `{}`", feedback.key)
        })?;

    let report_key = redis.key().entity(
        REPORT_NAMESPACE,
        format!("{}:{}:{string_id}", user.id, language.id),
    );
    if !claim_report(&redis, &report_key)
        .await
        .wrap_internal_err("checking for duplicate translation feedback")?
    {
        uncount_daily_report(&redis, &daily_key).await;
        return Err(ApiError::Conflict(eyre!(
            "you already reported this string recently"
        )));
    }

    if let Err(err) = crowdin
        .create_issue(
            string_id,
            &language.id,
            feedback.kind.issue_type(),
            &issue_text(&feedback, &language.name, &user.username),
        )
        .await
    {
        release_report(&redis, &report_key).await;
        return Err(err).wrap_internal_err("filing translation feedback");
    }

    Ok(())
}

fn daily_limit_key(redis: &RedisPool, user_id: UserId) -> String {
    redis.key().entity(
        DAILY_LIMIT_NAMESPACE,
        format!("{user_id}:{}", Utc::now().date_naive()),
    )
}

/// Counts a report attempt towards the user's daily limit, returning how many
/// attempts they have made today (UTC).
async fn count_daily_report(redis: &RedisPool, key: &str) -> Result<u64> {
    let mut conn = redis.connect().await?;
    let (count,): (u64,) = redis::pipe()
        .atomic()
        .incr(key, 1)
        .expire(key, DAILY_LIMIT_EXPIRY_SECONDS)
        .ignore()
        .query_async(&mut conn)
        .await?;

    Ok(count)
}

/// Refunds an attempt counted by [`count_daily_report`].
async fn uncount_daily_report(redis: &RedisPool, key: &str) {
    let result = async {
        let _: i64 = redis.connect().await?.decr(key, 1).await?;
        eyre::Ok(())
    }
    .await;
    if let Err(err) = result {
        tracing::warn!("Failed to refund daily translation feedback: {err:#}");
    }
}

/// Marks a string as reported by a user in a language, returning `false` if
/// they already reported it within [`REPORT_EXPIRY_SECONDS`].
async fn claim_report(redis: &RedisPool, key: &str) -> Result<bool> {
    let options = SetOptions::default()
        .conditional_set(ExistenceCheck::NX)
        .with_expiration(SetExpiry::EX(REPORT_EXPIRY_SECONDS));
    let claimed: Option<String> =
        redis.connect().await?.set_options(key, 1, options).await?;

    Ok(claimed.is_some())
}

/// Releases a claim from [`claim_report`] so the user can report the string
/// again.
async fn release_report(redis: &RedisPool, key: &str) {
    let result = async { redis.connect().await?.delete(key).await }.await;
    if let Err(err) = result {
        tracing::warn!("Failed to release translation feedback claim: {err:#}");
    }
}

fn issue_text(
    feedback: &TranslationFeedback,
    language_name: &str,
    username: &str,
) -> String {
    let product = match feedback.product {
        TranslationProduct::Website => "website",
        TranslationProduct::App => "app",
    };

    let mut text = format!(
        "User feedback from the {product} for {language_name}, reported by {username}\n\nProblem: {}",
        feedback.kind.label(),
    );

    if let Some(details) = feedback
        .details
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        let _ = write!(text, "\nDetails: {}", escape_mentions(details));
    }

    text
}

/// Inserts a zero-width space after every `@` in user-provided text, so
/// Crowdin doesn't turn it into a mention that notifies project members.
fn escape_mentions(text: &str) -> String {
    text.replace('@', "@\u{200B}")
}

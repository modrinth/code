use crate::database::models::DatabaseError;
use crate::database::redis::RedisPool;
use crate::models::v3::notifications::{NotificationChannel, NotificationType};
use serde::{Deserialize, Serialize};

const TEMPLATES_NAMESPACE: &str = "notifications_templates";
const TEMPLATES_HTML_DATA_NAMESPACE: &str = "notifications_templates_html_data";
const HTML_DATA_CACHE_EXPIRY: i64 = 60 * 15; // 15 minutes

#[derive(Clone, Serialize, Deserialize)]
pub struct NotificationTemplate {
    pub id: i64,
    pub channel: NotificationChannel,
    pub notification_type: NotificationType,
    pub subject_line: String,
    pub body_fetch_url: String,
    pub plaintext_fallback: String,
}

struct NotificationTemplateQueryResult {
    id: i64,
    channel: String,
    notification_type: String,
    subject_line: String,
    body_fetch_url: String,
    plaintext_fallback: String,
}

impl From<NotificationTemplateQueryResult> for NotificationTemplate {
    fn from(r: NotificationTemplateQueryResult) -> Self {
        NotificationTemplate {
            id: r.id,
            channel: NotificationChannel::from_str_or_default(&r.channel),
            notification_type: NotificationType::from_str_or_default(
                &r.notification_type,
            ),
            subject_line: r.subject_line,
            body_fetch_url: r.body_fetch_url,
            plaintext_fallback: r.plaintext_fallback,
        }
    }
}

impl NotificationTemplate {
    pub async fn list_channel(
        channel: NotificationChannel,
        exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
        redis: &RedisPool,
    ) -> Result<Vec<NotificationTemplate>, DatabaseError> {
        let mut redis = redis.connect().await?;

        let maybe_cached_templates = redis
            .get_deserialized_from_json(TEMPLATES_NAMESPACE, channel.as_str())
            .await?;

        if let Some(cached) = maybe_cached_templates {
            return Ok(cached);
        }

        let results = sqlx::query_as!(
            NotificationTemplateQueryResult,
            r#"
            SELECT * FROM notifications_templates WHERE channel = $1
            "#,
            channel.as_str(),
        )
        .fetch_all(exec)
        .await?;

        let templates = results.into_iter().map(Into::into).collect();

        redis
            .set_serialized_to_json(
                TEMPLATES_NAMESPACE,
                channel.as_str(),
                &templates,
                None,
            )
            .await?;

        Ok(templates)
    }

    pub async fn get_cached_html_data(
        &self,
        redis: &RedisPool,
    ) -> Result<Option<String>, DatabaseError> {
        let mut redis = redis.connect().await?;
        redis
            .get_deserialized_from_json(
                TEMPLATES_HTML_DATA_NAMESPACE,
                &self.id.to_string(),
            )
            .await
    }

    pub async fn set_cached_html_data(
        &self,
        data: String,
        redis: &RedisPool,
    ) -> Result<(), DatabaseError> {
        let mut redis = redis.connect().await?;
        redis
            .set_serialized_to_json(
                TEMPLATES_HTML_DATA_NAMESPACE,
                &self.id.to_string(),
                &data,
                Some(HTML_DATA_CACHE_EXPIRY),
            )
            .await
    }
}

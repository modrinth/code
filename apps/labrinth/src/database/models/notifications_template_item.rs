use crate::database::models::DatabaseError;
use crate::models::v3::notifications::{NotificationChannel, NotificationType};

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
    pub async fn get_channel(
        channel: NotificationChannel,
        exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    ) -> Result<Vec<NotificationTemplate>, DatabaseError> {
        let results = sqlx::query_as!(
            NotificationTemplateQueryResult,
            r#"
            SELECT * FROM notifications_templates WHERE channel = $1
            "#,
            channel.as_str(),
        )
        .fetch_all(exec)
        .await?;

        Ok(results.into_iter().map(Into::into).collect())
    }
}

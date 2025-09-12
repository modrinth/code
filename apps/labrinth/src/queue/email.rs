use crate::database::models::DatabaseError;
use crate::database::models::ids::*;
use crate::database::models::notification_item::DBNotification;
use crate::database::models::notifications_deliveries_item::DBNotificationDelivery;
use crate::database::models::notifications_template_item::NotificationTemplate;
use crate::database::redis::RedisPool;
use crate::models::v3::notifications::{
    NotificationChannel, NotificationDeliveryStatus,
};
use crate::routes::ApiError;
use chrono::Utc;
use futures::stream::{FuturesUnordered, StreamExt};
use lettre::transport::smtp::authentication::Credentials;
use lettre::transport::smtp::client::{Tls, TlsParameters};
use lettre::{AsyncSmtpTransport, AsyncTransport, Tokio1Executor};
use reqwest::Client;
use sqlx::PgPool;
use std::sync::Arc;
use thiserror::Error;
use tracing::{error, info, instrument, trace, warn};

const EMAIL_RETRY_DELAY_SECONDS: i64 = 10;

#[derive(Error, Debug)]
pub enum MailError {
    #[error("Environment Error")]
    Env(#[from] dotenvy::Error),
    #[error("Mail Error: {0}")]
    Mail(#[from] lettre::error::Error),
    #[error("Address Parse Error: {0}")]
    Address(#[from] lettre::address::AddressError),
    #[error("SMTP Error: {0}")]
    Smtp(#[from] lettre::transport::smtp::Error),
    #[error("HTTP error fetching template: {0}")]
    HttpTemplate(#[from] reqwest::Error),
}

#[derive(Clone)]
pub struct EmailQueue {
    pg: PgPool,
    client: reqwest::Client,
    redis: RedisPool,
    mailer: Option<Arc<AsyncSmtpTransport<Tokio1Executor>>>,
    from_name: String,
    from_address: String,
}

impl EmailQueue {
    /// Initializes the email queue from environment variables, and tests the SMTP connection.
    ///
    /// # Panic
    ///
    /// Panics if a TLS backend cannot be initialized by [`reqwest::ClientBuilder`].
    pub fn init(pg: PgPool, redis: RedisPool) -> Self {
        const DEFAULT_SENDER_NAME: &str = "Modrinth";
        const DEFAULT_SENDER_ADDRESS: &str = "no-reply@mail.modrinth.com";

        let from_name = dotenvy::var("SMTP_FROM_NAME")
            .unwrap_or_else(|_| DEFAULT_SENDER_NAME.to_string());
        let from_address = dotenvy::var("SMTP_FROM_ADDRESS")
            .unwrap_or_else(|_| DEFAULT_SENDER_ADDRESS.to_string());

        Self {
            pg,
            redis,
            mailer: None,
            from_name,
            from_address,
            client: Client::builder()
                .user_agent("Modrinth")
                .build()
                .expect("Failed to build HTTP client"),
        }
    }

    /// Tries to initialize the mailer. Returns an error for missing environment variables,
    /// but does nothing if the connection fails (besides emitting logs.)
    async fn try_init_mailer(&mut self) -> Result<(), MailError> {
        let username = dotenvy::var("SMTP_USERNAME")?;
        let password = dotenvy::var("SMTP_PASSWORD")?;
        let host = dotenvy::var("SMTP_HOST")?;
        let port = dotenvy::var("SMTP_PORT")?.parse::<u16>().unwrap_or(465);
        let creds = (!username.is_empty())
            .then(|| Credentials::new(username, password));

        let tls_setting = match dotenvy::var("SMTP_TLS")?.as_str() {
            "none" => Tls::None,
            "opportunistic_start_tls" => {
                Tls::Opportunistic(TlsParameters::new(host.to_string())?)
            }
            "requires_start_tls" => {
                Tls::Required(TlsParameters::new(host.to_string())?)
            }
            "tls" => Tls::Wrapper(TlsParameters::new(host.to_string())?),
            _ => {
                warn!("Unrecognized SMTP TLS setting. Defaulting to TLS.");
                Tls::Wrapper(TlsParameters::new(host.to_string())?)
            }
        };

        let mut mailer = AsyncSmtpTransport::<Tokio1Executor>::relay(&host)?
            .port(port)
            .tls(tls_setting);
        if let Some(creds) = creds {
            mailer = mailer.credentials(creds);
        }

        let mailer = mailer.build();

        let result = mailer.test_connection().await;

        match &result {
            Ok(true) => self.mailer = Some(Arc::new(mailer)),
            Ok(false) => error!("SMTP NOOP failed, disabling mailer"),
            Err(error) => {
                error!(%error, "Failed to test SMTP connection, disabling mailer")
            }
        }

        Ok(())
    }

    #[instrument(name = "EmailQueue::index", skip_all)]
    pub async fn index(mut self) {
        let (mailer, pg, redis, client, from_name, from_address) = {
            if self.mailer.is_none() {
                let _ = self.try_init_mailer().await;
            }

            if let Self {
                mailer: Some(mailer),
                pg,
                redis,
                client,
                from_name,
                from_address,
            } = self
            {
                (mailer, pg, redis, client, from_name, from_address)
            } else {
                return;
            }
        };

        let result = poll_queue(
            Arc::clone(&mailer),
            &pg,
            &redis,
            &client,
            from_name.clone(),
            from_address.clone(),
        )
        .await;

        if let Err(error) = result {
            error!(%error, "Database error in email queue");
        }
    }
}

async fn poll_queue(
    mailer: Arc<AsyncSmtpTransport<Tokio1Executor>>,
    pg: &PgPool,
    redis: &RedisPool,
    client: &Client,
    from_name: String,
    from_address: String,
) -> Result<(), DatabaseError> {
    let mut txn = pg.begin().await?;

    let begin = std::time::Instant::now();

    let mut deliveries = DBNotificationDelivery::lock_channel_processable(
        NotificationChannel::Email,
        5,
        &mut *txn,
    )
    .await?;

    if deliveries.is_empty() {
        return Ok(());
    }

    let n_to_process = deliveries.len();

    // We hold a FOR UPDATE lock on the rows here, so no other workers are accessing them
    // at the same time.

    let templates = NotificationTemplate::list_channel(
        NotificationChannel::Email,
        &mut *txn,
        redis,
    )
    .await?;

    let notification_ids = deliveries
        .iter()
        .map(|d| d.notification_id)
        .collect::<Vec<_>>();
    let notifications =
        DBNotification::get_many(&notification_ids, &mut *txn).await?;

    struct DeliveryResult {
        notification_id: DBNotificationId,
        update_status: NotificationDeliveryStatus,
        advance_next_attempt_time: bool,
    }

    // For all notifications we collected, fill out the template
    // and send it via SMTP in parallel.

    let mut futures = FuturesUnordered::new();

    for notification in notifications {
        let redis = redis.clone();
        let pg = pg.clone();
        let client = client.clone();
        let from_name = from_name.clone();
        let from_address = from_address.clone();
        let mailer = Arc::clone(&mailer);

        let maybe_template = templates
            .iter()
            .find(|t| {
                t.notification_type == notification.body.notification_type()
            })
            .cloned();

        futures.push(async move {
            let mut result = DeliveryResult {
                notification_id: notification.id,
                update_status: NotificationDeliveryStatus::Pending,
                advance_next_attempt_time: false,
            };

            // If there isn't any template present in the database for the
            // notification type, skip it.

            let Some(template) = maybe_template else {
                trace!("No template was found for notification type.");

                result.update_status = NotificationDeliveryStatus::SkippedDefault;
                return Ok(result);
            };

            let maybe_message = templates::build_email(
                &pg,
                &redis,
                &client,
                &notification,
                &template,
                from_name,
                from_address,
            )
            .await?;

            let Some(message) = maybe_message else {
                // User has no email--skip it.
                trace!("Attempted to send email to user without email");
                result.update_status = NotificationDeliveryStatus::SkippedDefault;
                return Ok(result);
            };

            let send_result = mailer.send(message).await;

            match send_result {
                Ok(_) => {
                    result.update_status = NotificationDeliveryStatus::Delivered;
                }

                Err(error) => {
                    error!(%error, smtp.code = ?extract_smtp_code(&error), "Error sending email");

                    if error.is_permanent() {
                        result.update_status =
                            NotificationDeliveryStatus::PermanentlyFailed;
                    }
                }
            };

            Result::<DeliveryResult, ApiError>::Ok(result)
        });
    }

    while let Some(result) = futures.next().await {
        match result {
            Ok(result) => {
                // Find the matching delivery row

                if let Some(idx) = deliveries
                    .iter()
                    .position(|d| d.notification_id == result.notification_id)
                {
                    let mut delivery = deliveries.remove(idx);
                    delivery.status = result.update_status;
                    delivery.next_attempt = if result.advance_next_attempt_time
                    {
                        Utc::now()
                            + chrono::Duration::seconds(
                                EMAIL_RETRY_DELAY_SECONDS,
                            )
                    } else {
                        delivery.next_attempt
                    };

                    delivery.attempt_count += 1;
                    delivery.update(&mut *txn).await?;
                }
            }

            Err(error) => error!(%error, "Error building email"),
        }
    }

    for mut delivery in deliveries {
        // For these, there was an error building the email, like a
        // database error. Retry them after 30 seconds.

        delivery.next_attempt =
            Utc::now() + chrono::Duration::seconds(EMAIL_RETRY_DELAY_SECONDS);

        delivery.update(&mut *txn).await?;
    }

    txn.commit().await?;

    info!(
        "Processed {} email deliveries in {}ms",
        n_to_process,
        begin.elapsed().as_millis()
    );

    Ok(())
}

fn extract_smtp_code(e: &lettre::transport::smtp::Error) -> Option<u16> {
    e.status().map(|x| x.into())
}

mod templates;

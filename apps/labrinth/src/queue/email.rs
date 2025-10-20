use crate::database::models::ids::*;
use crate::database::models::notification_item::DBNotification;
use crate::database::models::notifications_deliveries_item::DBNotificationDelivery;
use crate::database::models::notifications_template_item::NotificationTemplate;
use crate::database::models::user_item::DBUser;
use crate::database::redis::RedisPool;
use crate::models::notifications::{NotificationBody, NotificationType};
use crate::models::v3::notifications::{
    NotificationChannel, NotificationDeliveryStatus,
};
use crate::routes::ApiError;
use chrono::Utc;
use futures::stream::{FuturesUnordered, StreamExt};
use lettre::message::Mailbox;
use lettre::transport::smtp::authentication::Credentials;
use lettre::transport::smtp::client::{Tls, TlsParameters};
use lettre::{AsyncSmtpTransport, AsyncTransport, Tokio1Executor};
use reqwest::Client;
use sqlx::PgPool;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::Mutex as TokioMutex;
use tokio::sync::Semaphore;
use tracing::{error, info, instrument, warn};

const EMAIL_RETRY_DELAY_SECONDS: i64 = 10;

pub enum Mailer {
    Uninitialized,
    Initialized(Arc<AsyncSmtpTransport<Tokio1Executor>>),
}

impl Mailer {
    pub async fn to_transport(
        &mut self,
    ) -> Result<Arc<AsyncSmtpTransport<Tokio1Executor>>, MailError> {
        let maybe_transport = match self {
            Mailer::Uninitialized => {
                let username = dotenvy::var("SMTP_USERNAME")?;
                let password = dotenvy::var("SMTP_PASSWORD")?;
                let host = dotenvy::var("SMTP_HOST")?;
                let port =
                    dotenvy::var("SMTP_PORT")?.parse::<u16>().unwrap_or(465);

                let creds = (!username.is_empty())
                    .then(|| Credentials::new(username, password));

                let tls_setting = match dotenvy::var("SMTP_TLS")?.as_str() {
                    "none" => Tls::None,
                    "opportunistic_start_tls" => Tls::Opportunistic(
                        TlsParameters::new(host.to_string())?,
                    ),
                    "requires_start_tls" => {
                        Tls::Required(TlsParameters::new(host.to_string())?)
                    }
                    "tls" => {
                        Tls::Wrapper(TlsParameters::new(host.to_string())?)
                    }
                    _ => {
                        warn!(
                            "Unrecognized SMTP TLS setting. Defaulting to TLS."
                        );
                        Tls::Wrapper(TlsParameters::new(host.to_string())?)
                    }
                };

                let mut mailer =
                    AsyncSmtpTransport::<Tokio1Executor>::relay(&host)?
                        .port(port)
                        .tls(tls_setting);

                if let Some(creds) = creds {
                    mailer = mailer.credentials(creds);
                }

                let mailer = mailer.build();

                let result = mailer.test_connection().await;

                match &result {
                    Ok(true) => Some(Arc::new(mailer)),
                    Ok(false) => {
                        error!("SMTP NOOP failed, disabling mailer");
                        None
                    }
                    Err(error) => {
                        error!(%error, "Failed to test SMTP connection, disabling mailer");
                        None
                    }
                }
            }
            Mailer::Initialized(transport) => Some(Arc::clone(transport)),
        };

        let transport =
            maybe_transport.ok_or_else(|| MailError::Uninitialized)?;
        *self = Mailer::Initialized(Arc::clone(&transport));
        Ok(transport)
    }
}

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
    #[error("Couldn't initialize SMTP transport")]
    Uninitialized,
    #[error("HTTP error fetching template: {0}")]
    HttpTemplate(#[from] reqwest::Error),
}

#[derive(Clone)]
pub struct EmailQueue {
    pg: PgPool,
    client: reqwest::Client,
    redis: RedisPool,
    mailer: Arc<TokioMutex<Mailer>>,
    identity: templates::MailingIdentity,
}

impl EmailQueue {
    /// Initializes the email queue from environment variables, and tests the SMTP connection.
    ///
    /// # Panic
    ///
    /// Panics if a TLS backend cannot be initialized by [`reqwest::ClientBuilder`].
    pub fn init(pg: PgPool, redis: RedisPool) -> Result<Self, MailError> {
        Ok(Self {
            pg,
            redis,
            mailer: Arc::new(TokioMutex::new(Mailer::Uninitialized)),
            identity: templates::MailingIdentity::from_env()?,
            client: Client::builder()
                .user_agent("Modrinth")
                .build()
                .expect("Failed to build HTTP client"),
        })
    }

    /// Works on the email queue for up to `limit` items.
    ///
    /// Don't use a value too large for `limit`, as this method uses a single long running transaction to hold locks
    /// on the deliveries. Something around 5 is good.
    ///
    /// Returns `Ok(false)` if no emails were processed, `Ok(true)` if some were processed.
    #[instrument(name = "EmailQueue::index", skip_all)]
    pub async fn index(&self, limit: i64) -> Result<bool, ApiError> {
        let transport = self.mailer.lock().await.to_transport().await?;

        let begin = std::time::Instant::now();

        let mut deliveries = DBNotificationDelivery::lock_channel_processable(
            NotificationChannel::Email,
            limit,
            &self.pg,
        )
        .await?;

        if deliveries.is_empty() {
            return Ok(false);
        }

        let n_to_process = deliveries.len();

        // Auto-fail deliveries which have been attempted over 3 times to avoid
        // ballooning the error rate.
        for d in deliveries.iter_mut().filter(|d| d.attempt_count >= 3) {
            d.status = NotificationDeliveryStatus::PermanentlyFailed;
            d.update(&self.pg).await?;
        }

        // We hold a FOR UPDATE lock on the rows here, so no other workers are accessing them
        // at the same time.

        let notification_ids = deliveries
            .iter()
            .filter(|d| d.attempt_count < 3)
            .map(|d| d.notification_id)
            .collect::<Vec<_>>();
        let notifications =
            DBNotification::get_many(&notification_ids, &self.pg).await?;

        // For all notifications we collected, fill out the template
        // and send it via SMTP in parallel.
        let mut futures = FuturesUnordered::new();

        // Some email notifications should still be processed sequentially. This is to avoid cache stampede in the
        // case that processing the email can be heavy. For example, custom emails always make a POST request to modrinth.com,
        // which, while not necessarily slow, is subject to heavy rate limiting.
        let sequential_processing = Arc::new(Semaphore::new(1));

        for notification in notifications {
            let this = self.clone();
            let transport = Arc::clone(&transport);

            let seq = Arc::clone(&sequential_processing);

            futures.push(async move {
                let mut txn = this.pg.begin().await?;

                let maybe_user = DBUser::get_id(
                    notification.user_id,
                    &mut *txn,
                    &this.redis,
                )
                .await?;

                let Some(mailbox) = maybe_user
                    .and_then(|user| user.email)
                    .and_then(|email| email.parse().ok())
                else {
                    return Ok((
                        notification.id,
                        NotificationDeliveryStatus::SkippedPreferences,
                    ));
                };

                // For the cache stampede reasons mentioned above, we process custom emails exclusively sequentially.
                // This could cause unnecessary slowness if we're sending a lot of custom emails with the same key in one go,
                // and the cache is already populated (thus the sequential processing would not be needed).
                let maybe_permit = if notification.body.notification_type()
                    == NotificationType::Custom
                {
                    Some(
                        seq.acquire()
                            .await
                            .expect("Semaphore should never be closed"),
                    )
                } else {
                    None
                };

                let result = this
                    .send_one_with_transport(
                        &mut txn,
                        transport,
                        notification.body,
                        notification.user_id,
                        mailbox,
                    )
                    .await
                    .map(|status| (notification.id, status));

                drop(maybe_permit);

                result
            });
        }

        while let Some(result) = futures.next().await {
            match result {
                Ok((notification_id, status)) => {
                    if let Some(idx) = deliveries
                        .iter()
                        .position(|d| d.notification_id == notification_id)
                    {
                        let update_next_attempt =
                            status == NotificationDeliveryStatus::Pending;

                        let mut delivery = deliveries.swap_remove(idx);
                        delivery.status = status;
                        delivery.next_attempt += if update_next_attempt {
                            chrono::Duration::seconds(EMAIL_RETRY_DELAY_SECONDS)
                        } else {
                            chrono::Duration::seconds(0)
                        };

                        delivery.attempt_count += 1;
                        delivery.update(&self.pg).await?;
                    }
                }

                Err(error) => error!(%error, "Error building email"),
            }
        }

        for mut delivery in deliveries {
            // For these, there was an error building the email, like a
            // database error. Retry them after a delay.

            delivery.next_attempt = Utc::now()
                + chrono::Duration::seconds(EMAIL_RETRY_DELAY_SECONDS);

            delivery.update(&self.pg).await?;
        }

        info!(
            "Processed {} email deliveries in {}ms",
            n_to_process,
            begin.elapsed().as_millis()
        );

        Ok(true)
    }

    pub async fn send_one(
        &self,
        txn: &mut sqlx::PgTransaction<'_>,
        notification: NotificationBody,
        user_id: DBUserId,
        address: Mailbox,
    ) -> Result<NotificationDeliveryStatus, ApiError> {
        let transport = self.mailer.lock().await.to_transport().await?;
        self.send_one_with_transport(
            txn,
            transport,
            notification,
            user_id,
            address,
        )
        .await
    }

    async fn send_one_with_transport(
        &self,
        txn: &mut sqlx::PgTransaction<'_>,
        transport: Arc<AsyncSmtpTransport<Tokio1Executor>>,
        notification: NotificationBody,
        user_id: DBUserId,
        address: Mailbox,
    ) -> Result<NotificationDeliveryStatus, ApiError> {
        // If there isn't any template present in the database for the
        // notification type, skip it.

        let Some(template) = NotificationTemplate::list_channel(
            NotificationChannel::Email,
            &mut **txn,
            &self.redis,
        )
        .await?
        .into_iter()
        .find(|t| t.notification_type == notification.notification_type()) else {
            return Ok(NotificationDeliveryStatus::SkippedDefault);
        };

        let message = templates::build_email(
            txn,
            &self.redis,
            &self.client,
            user_id,
            &notification,
            &template,
            self.identity.clone(),
            address,
        )
        .await?;

        let send_result = transport.send(message).await;

        Ok(send_result.map_or_else(|error| {
            error!(%error, smtp.code = ?extract_smtp_code(&error), "Error sending email");

            if error.is_permanent() {
                NotificationDeliveryStatus::PermanentlyFailed
            } else {
                NotificationDeliveryStatus::Pending
            }
        }, |_| NotificationDeliveryStatus::Delivered))
    }
}

fn extract_smtp_code(e: &lettre::transport::smtp::Error) -> Option<u16> {
    e.status().map(|x| x.into())
}

mod templates;

use super::MailError;
use crate::database::models::ids::*;
use crate::database::models::notifications_template_item::NotificationTemplate;
use crate::database::models::{
    DBOrganization, DBProject, DBUser, DatabaseError,
};
use crate::database::redis::RedisPool;
use crate::models::v3::notifications::NotificationBody;
use crate::routes::ApiError;
use ariadne::ids::base62_impl::to_base62;
use futures::TryFutureExt;
use lettre::Message;
use lettre::message::{Mailbox, MultiPart, SinglePart};
use sqlx::query;
use std::collections::HashMap;
use std::time::Duration;
use tracing::{error, warn};

const USER_NAME: &str = "user.name";
const USER_EMAIL: &str = "user.email";

const RESETPASSWORD_URL: &str = "resetpassword.url";
const VERIFYEMAIL_URL: &str = "verifyemail.url";
const AUTHPROVIDER_NAME: &str = "authprovider.name";
const EMAILCHANGED_NEW_EMAIL: &str = "emailchanged.new_email";
const BILLING_URL: &str = "billing.url";
const SUBSCRIPTION_ID: &str = "subscription.id";

const TAXNOTIFICATION_OLD_AMOUNT: &str = "taxnotification.old_amount";
const TAXNOTIFICATION_OLD_TAX_AMOUNT: &str = "taxnotification.old_tax_amount";
const TAXNOTIFICATION_OLD_TOTAL_AMOUNT: &str =
    "taxnotification.old_total_amount";
const TAXNOTIFICATION_NEW_AMOUNT: &str = "taxnotification.new_amount";
const TAXNOTIFICATION_NEW_TAX_AMOUNT: &str = "taxnotification.new_tax_amount";
const TAXNOTIFICATION_NEW_TOTAL_AMOUNT: &str =
    "taxnotification.new_total_amount";
const TAXNOTIFICATION_BILLING_INTERVAL: &str =
    "taxnotification.billing_interval";
const TAXNOTIFICATION_DUE: &str = "taxnotification.due";
const TAXNOTIFICATION_SERVICE: &str = "taxnotification.service";

const PAYMENTFAILED_AMOUNT: &str = "paymentfailed.amount";
const PAYMENTFAILED_SERVICE: &str = "paymentfailed.service";

const TEAMINVITE_INVITER_NAME: &str = "teaminvite.inviter.name";
const TEAMINVITE_PROJECT_NAME: &str = "teaminvite.project.name";
const TEAMINVITE_ROLE_NAME: &str = "teaminvite.role.name";

const ORGINVITE_INVITER_NAME: &str = "organizationinvite.inviter.name";
const ORGINVITE_ORG_NAME: &str = "organizationinvite.organization.name";
const ORGINVITE_ROLE_NAME: &str = "organizationinvite.role.name";

const STATUSCHANGE_PROJECT_NAME: &str = "statuschange.project.name";
const STATUSCHANGE_OLD_STATUS: &str = "statuschange.old.status";
const STATUSCHANGE_NEW_STATUS: &str = "statuschange.new.status";

const NEWPAT_TOKEN_NAME: &str = "newpat.token_name";

const PROJECT_ID: &str = "project.id";
const PROJECT_NAME: &str = "project.name";
const PROJECT_ICON_URL: &str = "project.icon_url";

const REPORT_ID: &str = "report.id";
const REPORT_TITLE: &str = "report.title";
const REPORT_DATE: &str = "report.date";
const NEWREPORT_ID: &str = "newreport.id";

const PROJECT_OLD_STATUS: &str = "project.oldstatus";
const PROJECT_NEW_STATUS: &str = "project.newstatus";

const NEWOWNER_TYPE: &str = "new_owner.type";
const NEWOWNER_TYPE_CAPITALIZED: &str = "new_owner.type_capitalized";
const NEWOWNER_NAME: &str = "new_owner.name";

const PAYOUTAVAILABLE_AMOUNT: &str = "payout.amount";
const PAYOUTAVAILABLE_PERIOD: &str = "payout.period";

#[derive(Clone)]
pub struct MailingIdentity {
    from_name: String,
    from_address: String,
    reply_name: Option<String>,
    reply_address: Option<String>,
}

impl MailingIdentity {
    pub fn from_env() -> dotenvy::Result<Self> {
        Ok(Self {
            from_name: dotenvy::var("SMTP_FROM_NAME")?,
            from_address: dotenvy::var("SMTP_FROM_ADDRESS")?,
            reply_name: dotenvy::var("SMTP_REPLY_TO_NAME").ok(),
            reply_address: dotenvy::var("SMTP_REPLY_TO_ADDRESS").ok(),
        })
    }
}

#[allow(clippy::too_many_arguments)]
pub async fn build_email(
    exec: &mut sqlx::PgTransaction<'_>,
    redis: &RedisPool,
    client: &reqwest::Client,
    user_id: DBUserId,
    body: &NotificationBody,
    template: &NotificationTemplate,
    from: MailingIdentity,
    to: Mailbox,
) -> Result<Message, ApiError> {
    let get_html_body = async {
        let result: Result<Result<String, reqwest::Error>, ApiError> =
            match template.get_cached_html_data(redis).await? {
                Some(html_body) => Ok(Ok(html_body)),
                None => {
                    let result = client
                        .get(&template.body_fetch_url)
                        .timeout(Duration::from_secs(3))
                        .send()
                        .and_then(|res| async move { res.error_for_status() })
                        .and_then(|res| res.text())
                        .await;

                    if let Ok(ref body) = result {
                        template
                            .set_cached_html_data(body.clone(), redis)
                            .await?;
                    }

                    Ok(result)
                }
            };

        result
    };

    let MailingIdentity {
        from_name,
        from_address,
        reply_name,
        reply_address,
    } = from;

    let (html_body_result, mut variables) = futures::try_join!(
        get_html_body,
        collect_template_variables(exec, redis, user_id, body)
    )?;

    variables.insert(USER_EMAIL, to.email.to_string());

    let mut message_builder = Message::builder().from(Mailbox::new(
        Some(from_name),
        from_address.parse().map_err(MailError::from)?,
    ));

    if let Some((name, address)) = reply_name.zip(reply_address) {
        message_builder = message_builder.reply_to(Mailbox::new(
            Some(name),
            address.parse().map_err(MailError::from)?,
        ));
    }

    let subject = fill_template(&template.subject_line, &variables);
    message_builder = message_builder.to(to).subject(subject);

    let plaintext_filled_body =
        fill_template(&template.plaintext_fallback, &variables);

    let email_message = match html_body_result {
        Ok(html_body) => {
            let html_filled_body = fill_template(&html_body, &variables);
            message_builder
                .multipart(MultiPart::alternative_plain_html(
                    plaintext_filled_body,
                    html_filled_body,
                ))
                .map_err(MailError::from)?
        }

        Err(error) => {
            error!(%error, "Failed to fetch template body");
            message_builder
                .singlepart(SinglePart::plain(plaintext_filled_body))
                .map_err(MailError::from)?
        }
    };

    Ok(email_message)
}

fn fill_template(
    mut text: &str,
    variables: &HashMap<&'static str, String>,
) -> String {
    let mut buffer = String::with_capacity(text.len());

    loop {
        if let Some((previous, start_variable)) = text.split_once('{') {
            buffer.push_str(previous);

            if let Some((variable_name, rest)) = start_variable.split_once('}')
            {
                // Replace variable with an empty string if it isn't matched
                buffer.push_str(
                    variables
                        .get(variable_name)
                        .map(|s| s.as_str())
                        .unwrap_or_default(),
                );
                text = rest;
            } else {
                warn!("Unmatched open brace in template");
                text = start_variable;
            }
        } else {
            buffer.push_str(text);
            break;
        }
    }

    buffer
}

async fn collect_template_variables(
    exec: &mut sqlx::PgTransaction<'_>,
    redis: &RedisPool,
    user_id: DBUserId,
    n: &NotificationBody,
) -> Result<HashMap<&'static str, String>, ApiError> {
    let db_user = DBUser::get_id(user_id, &mut **exec, redis)
        .await?
        .ok_or_else(|| DatabaseError::Database(sqlx::Error::RowNotFound))?;

    let mut map = HashMap::new();
    map.insert(USER_NAME, db_user.username);

    match &n {
        NotificationBody::PatCreated { token_name } => {
            map.insert(NEWPAT_TOKEN_NAME, token_name.clone());
            Ok(map)
        }

        NotificationBody::ModerationMessageReceived { project_id, .. } => {
            let result = DBProject::get_id(
                DBProjectId(project_id.0 as i64),
                exec,
                redis,
            )
            .await?
            .ok_or_else(|| DatabaseError::Database(sqlx::Error::RowNotFound))?
            .inner;

            map.insert(PROJECT_ID, to_base62(project_id.0));
            map.insert(PROJECT_NAME, result.name);
            map.insert(PROJECT_ICON_URL, result.icon_url.unwrap_or_default());
            Ok(map)
        }

        NotificationBody::ReportStatusUpdated { report_id } => {
            let result = query!(
                r#"
                SELECT
                  r.created,
                  COALESCE(m.name, v.version_number, u.username, 'unknown') "title!"
                FROM reports r
                LEFT JOIN mods m ON r.mod_id = m.id
                LEFT JOIN versions v ON r.version_id = v.id
                LEFT JOIN users u ON r.user_id = u.id
                WHERE r.id = $1
                "#,
                report_id.0 as i64
            )
            .fetch_one(&mut **exec)
            .await?;

            map.insert(REPORT_ID, to_base62(report_id.0));
            map.insert(REPORT_TITLE, result.title);
            map.insert(REPORT_DATE, date_human_readable(result.created));
            Ok(map)
        }

        NotificationBody::ReportSubmitted { report_id } => {
            let result = query!(
                r#"
                SELECT
                  COALESCE(m.name, v.version_number, u.username, 'unknown') "title!"
                FROM reports r
                LEFT JOIN mods m ON r.mod_id = m.id
                LEFT JOIN versions v ON r.version_id = v.id
                LEFT JOIN users u ON r.user_id = u.id
                WHERE r.id = $1
                "#,
                report_id.0 as i64
            )
            .fetch_one(&mut **exec)
            .await?;

            map.insert(REPORT_TITLE, result.title);
            map.insert(NEWREPORT_ID, to_base62(report_id.0));
            Ok(map)
        }

        NotificationBody::ProjectStatusApproved { project_id } => {
            let result = query!(
                r#"
                SELECT name, icon_url FROM mods WHERE id = $1
                "#,
                project_id.0 as i64
            )
            .fetch_one(&mut **exec)
            .await?;

            map.insert(PROJECT_ID, to_base62(project_id.0));
            map.insert(PROJECT_NAME, result.name);
            map.insert(PROJECT_ICON_URL, result.icon_url.unwrap_or_default());
            Ok(map)
        }

        NotificationBody::ProjectStatusNeutral {
            project_id,
            old_status,
            new_status,
        } => {
            let result = DBProject::get_id(
                DBProjectId(project_id.0 as i64),
                exec,
                redis,
            )
            .await?
            .ok_or_else(|| DatabaseError::Database(sqlx::Error::RowNotFound))?
            .inner;

            map.insert(PROJECT_ID, to_base62(project_id.0));
            map.insert(PROJECT_NAME, result.name);
            map.insert(PROJECT_ICON_URL, result.icon_url.unwrap_or_default());
            map.insert(PROJECT_OLD_STATUS, old_status.as_str().to_string());
            map.insert(PROJECT_NEW_STATUS, new_status.as_str().to_string());
            Ok(map)
        }

        NotificationBody::ProjectTransferred {
            project_id,
            new_owner_user_id,
            new_owner_organization_id,
        } => {
            let project = DBProject::get_id(
                DBProjectId(project_id.0 as i64),
                &mut **exec,
                redis,
            )
            .await?
            .ok_or_else(|| DatabaseError::Database(sqlx::Error::RowNotFound))?
            .inner;

            map.insert(PROJECT_ID, to_base62(project_id.0));
            map.insert(PROJECT_NAME, project.name);
            map.insert(PROJECT_ICON_URL, project.icon_url.unwrap_or_default());

            if let Some(new_owner_user_id) = new_owner_user_id {
                let user = DBUser::get_id(
                    DBUserId(new_owner_user_id.0 as i64),
                    &mut **exec,
                    redis,
                )
                .await?
                .ok_or_else(|| {
                    DatabaseError::Database(sqlx::Error::RowNotFound)
                })?;

                map.insert(NEWOWNER_TYPE, "user".to_string());
                map.insert(NEWOWNER_TYPE_CAPITALIZED, "User".to_string());
                map.insert(NEWOWNER_NAME, user.username);
            } else if let Some(new_owner_organization_id) =
                new_owner_organization_id
            {
                let org = DBOrganization::get_id(
                    DBOrganizationId(new_owner_organization_id.0 as i64),
                    &mut **exec,
                    redis,
                )
                .await?
                .ok_or_else(|| {
                    DatabaseError::Database(sqlx::Error::RowNotFound)
                })?;

                map.insert(NEWOWNER_TYPE, "organization".to_string());
                map.insert(
                    NEWOWNER_TYPE_CAPITALIZED,
                    "Organization".to_string(),
                );
                map.insert(NEWOWNER_NAME, org.name);
            }

            Ok(map)
        }
        NotificationBody::TeamInvite {
            team_id: _,
            project_id,
            invited_by,
            role,
        } => {
            let result = query!(
                r#"
                SELECT
                  users.username "user_name!",
                  users.email "user_email",
                  inviter.username "inviter_name!",
                  project.name "project_name!"
                FROM users
                INNER JOIN users inviter ON inviter.id = $1
                INNER JOIN mods project ON project.id = $2
                WHERE users.id = $3
                "#,
                invited_by.0 as i64,
                project_id.0 as i64,
                user_id.0 as i64
            )
            .fetch_one(&mut **exec)
            .await?;

            map.insert(TEAMINVITE_INVITER_NAME, result.inviter_name);
            map.insert(TEAMINVITE_PROJECT_NAME, result.project_name);
            map.insert(TEAMINVITE_ROLE_NAME, role.clone());

            Ok(map)
        }

        NotificationBody::OrganizationInvite {
            organization_id,
            invited_by,
            team_id: _,
            role,
        } => {
            let result = query!(
                r#"
                SELECT
                  users.username "user_name!",
                  users.email "user_email",
                  inviter.username "inviter_name!",
                  organization.name "organization_name!"
                FROM users
                INNER JOIN users inviter ON inviter.id = $1
                INNER JOIN organizations organization ON organization.id = $2
                WHERE users.id = $3
                "#,
                invited_by.0 as i64,
                organization_id.0 as i64,
                user_id.0 as i64
            )
            .fetch_one(&mut **exec)
            .await?;

            map.insert(ORGINVITE_INVITER_NAME, result.inviter_name);
            map.insert(ORGINVITE_ORG_NAME, result.organization_name);
            map.insert(ORGINVITE_ROLE_NAME, role.clone());

            Ok(map)
        }

        NotificationBody::StatusChange {
            project_id,
            old_status,
            new_status,
        } => {
            let result = query!(
                r#"
                SELECT
                  users.username "user_name!",
                  users.email "user_email",
                  project.name "project_name!"
                FROM users
                INNER JOIN mods project ON project.id = $1
                WHERE users.id = $2
                "#,
                project_id.0 as i64,
                user_id.0 as i64,
            )
            .fetch_one(&mut **exec)
            .await?;

            map.insert(STATUSCHANGE_PROJECT_NAME, result.project_name);
            map.insert(STATUSCHANGE_OLD_STATUS, old_status.as_str().to_owned());
            map.insert(STATUSCHANGE_NEW_STATUS, new_status.as_str().to_owned());

            Ok(map)
        }

        NotificationBody::ResetPassword { flow } => {
            let url = format!(
                "{}/{}?flow={}",
                dotenvy::var("SITE_URL")?,
                dotenvy::var("SITE_RESET_PASSWORD_PATH")?,
                flow
            );

            map.insert(RESETPASSWORD_URL, url);

            Ok(map)
        }

        NotificationBody::VerifyEmail { flow } => {
            let url = format!(
                "{}/{}?flow={}",
                dotenvy::var("SITE_URL")?,
                dotenvy::var("SITE_VERIFY_EMAIL_PATH")?,
                flow
            );

            map.insert(VERIFYEMAIL_URL, url);

            Ok(map)
        }

        NotificationBody::AuthProviderAdded { provider }
        | NotificationBody::AuthProviderRemoved { provider } => {
            map.insert(AUTHPROVIDER_NAME, provider.clone());

            Ok(map)
        }

        NotificationBody::TwoFactorEnabled
        | NotificationBody::TwoFactorRemoved
        | NotificationBody::PasswordChanged
        | NotificationBody::PasswordRemoved => Ok(map),

        NotificationBody::EmailChanged {
            new_email,
            to_email: _,
        } => {
            map.insert(EMAILCHANGED_NEW_EMAIL, new_email.clone());

            Ok(map)
        }

        NotificationBody::PaymentFailed { amount, service } => {
            let url = format!(
                "{}/{}",
                dotenvy::var("SITE_URL")?,
                dotenvy::var("SITE_BILLING_PATH")?,
            );

            let mut map = HashMap::new();
            map.insert(PAYMENTFAILED_AMOUNT, amount.clone());
            map.insert(PAYMENTFAILED_SERVICE, service.clone());
            map.insert(BILLING_URL, url);

            Ok(map)
        }

        NotificationBody::PayoutAvailable {
            amount,
            date_available,
        } => {
            if let Some(period_month) =
                date_available.checked_sub_months(chrono::Months::new(2))
            {
                map.insert(
                    PAYOUTAVAILABLE_PERIOD,
                    period_month.format("%B %Y").to_string(),
                );
            }

            map.insert(
                PAYOUTAVAILABLE_AMOUNT,
                format!("USD${:.2}", *amount as f64 / 100.0),
            );

            Ok(map)
        }

        NotificationBody::TaxNotification {
            subscription_id,
            old_amount,
            old_tax_amount,
            new_amount,
            new_tax_amount,
            billing_interval,
            currency,
            due,
            service,
        } => {
            map.insert(
                TAXNOTIFICATION_OLD_AMOUNT,
                fmt_money(*old_amount, currency),
            );
            map.insert(
                TAXNOTIFICATION_OLD_TAX_AMOUNT,
                fmt_money(*old_tax_amount, currency),
            );
            map.insert(
                TAXNOTIFICATION_OLD_TOTAL_AMOUNT,
                fmt_money(*old_amount + *old_tax_amount, currency),
            );
            map.insert(
                TAXNOTIFICATION_NEW_AMOUNT,
                fmt_money(*new_amount, currency),
            );
            map.insert(
                TAXNOTIFICATION_NEW_TAX_AMOUNT,
                fmt_money(*new_tax_amount, currency),
            );
            map.insert(
                TAXNOTIFICATION_NEW_TOTAL_AMOUNT,
                fmt_money(*new_amount + *new_tax_amount, currency),
            );
            map.insert(
                TAXNOTIFICATION_BILLING_INTERVAL,
                billing_interval.as_str().to_owned(),
            );
            map.insert(TAXNOTIFICATION_DUE, date_human_readable(*due));
            map.insert(TAXNOTIFICATION_SERVICE, service.clone());
            map.insert(SUBSCRIPTION_ID, to_base62(subscription_id.0));
            Ok(map)
        }

        NotificationBody::ProjectUpdate { .. }
        | NotificationBody::ModeratorMessage { .. }
        | NotificationBody::LegacyMarkdown { .. }
        | NotificationBody::Unknown => Ok(map),
    }
}

fn date_human_readable(date: chrono::DateTime<chrono::Utc>) -> String {
    date.format("%B %d, %Y").to_string()
}

fn fmt_money(amount: i64, currency: &str) -> String {
    rusty_money::Money::from_minor(
        amount,
        rusty_money::iso::find(currency).unwrap_or(rusty_money::iso::USD),
    )
    .to_string()
}

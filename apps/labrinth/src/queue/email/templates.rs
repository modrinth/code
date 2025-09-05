use super::MailError;
use crate::database::models::DatabaseError;
use crate::database::models::ids::*;
use crate::database::models::notification_item::DBNotification;
use crate::database::models::notifications_template_item::NotificationTemplate;
use crate::database::models::user_item::DBUser;
use crate::database::redis::RedisPool;
use crate::models::v3::notifications::NotificationBody;
use crate::routes::ApiError;
use futures::TryFutureExt;
use lettre::Message;
use lettre::message::{Mailbox, MultiPart, SinglePart};
use sqlx::query;
use std::collections::HashMap;
use std::time::Duration;
use tracing::{error, warn};

const USER_NAME: &str = "user.name";
const USER_EMAIL: &str = "user.email";

const TEAMINVITE_INVITER_NAME: &str = "teaminvite.inviter.name";
const TEAMINVITE_PROJECT_NAME: &str = "teaminvite.project.name";
const TEAMINVITE_ROLE_NAME: &str = "teaminvite.role.name";

const ORGINVITE_INVITER_NAME: &str = "organizationinvite.inviter.name";
const ORGINVITE_ORG_NAME: &str = "organizationinvite.organization.name";
const ORGINVITE_ROLE_NAME: &str = "organizationinvite.role.name";

const STATUSCHANGE_PROJECT_NAME: &str = "statuschange.project.name";
const STATUSCHANGE_OLD_STATUS: &str = "statuschange.old.status";
const STATUSCHANGE_NEW_STATUS: &str = "statuschange.new.status";

pub async fn build_email(
    exec: impl sqlx::PgExecutor<'_>,
    redis: &RedisPool,
    client: &reqwest::Client,
    notification: &DBNotification,
    template: &NotificationTemplate,
    from_name: String,
    from_address: String,
) -> Result<Option<Message>, ApiError> {
    let get_html_body = async {
        let result: Result<Result<String, reqwest::Error>, DatabaseError> =
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

    let (html_body_result, template_variables_result) = futures::join!(
        get_html_body,
        collect_template_variables(exec, redis, notification)
    );

    let TemplateVariables {
        user_email,
        variables,
    } = template_variables_result?;

    let Some(target_email) = user_email else {
        return Ok(None);
    };

    let message_builder = Message::builder()
        .from(Mailbox::new(
            Some(from_name),
            from_address.parse().map_err(MailError::from)?,
        ))
        .to(target_email.parse().map_err(MailError::from)?)
        .subject(&template.subject_line);

    let plaintext_filled_body =
        fill_template(&template.plaintext_fallback, &variables);

    let email_message = match html_body_result? {
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

    Ok(Some(email_message))
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

struct TemplateVariables {
    user_email: Option<String>,
    variables: HashMap<&'static str, String>,
}

impl TemplateVariables {
    fn with_email(
        mut variables: HashMap<&'static str, String>,
        maybe_user_email: Option<String>,
    ) -> Self {
        if let Some(user_email) = &maybe_user_email {
            variables.insert(USER_EMAIL, user_email.clone());
        }

        Self {
            user_email: maybe_user_email,
            variables,
        }
    }
}

async fn collect_template_variables(
    exec: impl sqlx::PgExecutor<'_>,
    redis: &RedisPool,
    n: &DBNotification,
) -> Result<TemplateVariables, DatabaseError> {
    async fn only_select_default_variables(
        exec: impl sqlx::PgExecutor<'_>,
        redis: &RedisPool,
        user_id: DBUserId,
    ) -> Result<TemplateVariables, DatabaseError> {
        let mut map = HashMap::new();

        let user = DBUser::get_id(user_id, exec, redis)
            .await?
            .ok_or_else(|| DatabaseError::Database(sqlx::Error::RowNotFound))?;

        map.insert(USER_NAME, user.username);
        Ok(TemplateVariables::with_email(map, user.email))
    }

    match &n.body {
        NotificationBody::ProjectUpdate { .. } => {
            only_select_default_variables(exec, redis, n.user_id).await
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
                n.user_id.0 as i64
            )
            .fetch_one(exec)
            .await?;

            let mut map = HashMap::new();
            map.insert(USER_NAME, result.user_name);
            map.insert(TEAMINVITE_INVITER_NAME, result.inviter_name);
            map.insert(TEAMINVITE_PROJECT_NAME, result.project_name);
            map.insert(TEAMINVITE_ROLE_NAME, role.clone());

            Ok(TemplateVariables::with_email(map, result.user_email))
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
                n.user_id.0 as i64
            )
            .fetch_one(exec)
            .await?;

            let mut map = HashMap::new();
            map.insert(USER_NAME, result.user_name);
            map.insert(ORGINVITE_INVITER_NAME, result.inviter_name);
            map.insert(ORGINVITE_ORG_NAME, result.organization_name);
            map.insert(ORGINVITE_ROLE_NAME, role.clone());

            Ok(TemplateVariables::with_email(map, result.user_email))
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
                n.user_id.0 as i64,
            )
            .fetch_one(exec)
            .await?;

            let mut map = HashMap::new();
            map.insert(USER_NAME, result.user_name);
            map.insert(STATUSCHANGE_PROJECT_NAME, result.project_name);
            map.insert(STATUSCHANGE_OLD_STATUS, old_status.as_str().to_owned());
            map.insert(STATUSCHANGE_NEW_STATUS, new_status.as_str().to_owned());

            Ok(TemplateVariables::with_email(map, result.user_email))
        }

        _ => only_select_default_variables(exec, redis, n.user_id).await,
    }
}

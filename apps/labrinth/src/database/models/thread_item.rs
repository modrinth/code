use super::ids::*;
use crate::database::PgTransaction;
use crate::models::thread_issues::{ThreadIssueTarget, ThreadIssueVerdict};
use crate::models::threads::{MessageBody, ThreadType};
use chrono::{DateTime, Utc};
use eyre::{Result, WrapErr};
use serde::{Deserialize, Serialize};

pub struct ThreadBuilder {
    pub type_: ThreadType,
    pub members: Vec<DBUserId>,
    pub project_id: Option<DBProjectId>,
    pub report_id: Option<DBReportId>,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct DBThread {
    pub id: DBThreadId,

    pub project_id: Option<DBProjectId>,
    pub report_id: Option<DBReportId>,
    pub type_: ThreadType,

    pub messages: Vec<DBThreadMessage>,
    pub members: Vec<DBUserId>,
}

pub struct ThreadMessageBuilder {
    pub author_id: Option<DBUserId>,
    pub body: MessageBody,
    pub thread_id: DBThreadId,
    pub hide_identity: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct DBThreadMessage {
    pub id: DBThreadMessageId,
    pub thread_id: DBThreadId,
    pub author_id: Option<DBUserId>,
    pub body: MessageBody,
    pub created: DateTime<Utc>,
    pub hide_identity: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct DBThreadIssue {
    pub id: DBThreadIssueId,
    pub thread_id: DBThreadId,
    pub what: ThreadIssueTarget,
    pub why: serde_json::Value,
    pub user_addressed: bool,
    pub moderator_verified: bool,
    pub verdict: ThreadIssueVerdict,
    pub created_at: DateTime<Utc>,
}

impl ThreadMessageBuilder {
    pub async fn insert(
        &self,
        transaction: &mut PgTransaction<'_>,
    ) -> Result<DBThreadMessageId> {
        let thread_message_id = generate_thread_message_id(transaction)
            .await
            .wrap_err("generating thread message ID")?;
        let body = serde_json::value::to_value(self.body.clone())
            .wrap_err("serializing thread message body")?;

        sqlx::query!(
            "
            INSERT INTO threads_messages (
                id, author_id, body, thread_id, hide_identity
            )
            VALUES (
                $1, $2, $3, $4, $5
            )
            ",
            thread_message_id as DBThreadMessageId,
            self.author_id.map(|x| x.0),
            body,
            self.thread_id as DBThreadId,
            self.hide_identity
        )
        .execute(&mut *transaction)
        .await
        .wrap_err("inserting thread message")?;

        Ok(thread_message_id)
    }
}

impl ThreadBuilder {
    pub async fn insert(
        &self,
        transaction: &mut PgTransaction<'_>,
    ) -> Result<DBThreadId> {
        let thread_id = generate_thread_id(&mut *transaction)
            .await
            .wrap_err("generating thread ID")?;
        sqlx::query!(
            "
            INSERT INTO threads (
                id, thread_type, mod_id, report_id
            )
            VALUES (
                $1, $2, $3, $4
            )
            ",
            thread_id as DBThreadId,
            self.type_.as_str(),
            self.project_id.map(|x| x.0),
            self.report_id.map(|x| x.0),
        )
        .execute(&mut *transaction)
        .await
        .wrap_err("inserting thread")?;

        let (thread_ids, members): (Vec<_>, Vec<_>) =
            self.members.iter().map(|m| (thread_id.0, m.0)).unzip();
        sqlx::query!(
            "
            INSERT INTO threads_members (
                thread_id, user_id
            )
            SELECT * FROM UNNEST ($1::int8[], $2::int8[])
            ",
            &thread_ids[..],
            &members[..],
        )
        .execute(&mut *transaction)
        .await
        .wrap_err("inserting thread members")?;

        Ok(thread_id)
    }
}

impl DBThread {
    pub async fn get<'a, E>(id: DBThreadId, exec: E) -> Result<Option<DBThread>>
    where
        E: crate::database::Executor<'a, Database = sqlx::Postgres> + Copy,
    {
        Ok(Self::get_many(&[id], exec)
            .await
            .wrap_err("fetching thread")?
            .into_iter()
            .next())
    }

    pub async fn get_many<'a, E>(
        thread_ids: &[DBThreadId],
        exec: E,
    ) -> Result<Vec<DBThread>>
    where
        E: crate::database::Executor<'a, Database = sqlx::Postgres> + Copy,
    {
        use futures::stream::TryStreamExt;

        let thread_ids_parsed: Vec<i64> =
            thread_ids.iter().map(|x| x.0).collect();
        let threads = sqlx::query!(
            "
            SELECT t.id, t.thread_type, t.mod_id, t.report_id,
            ARRAY_AGG(DISTINCT tm.user_id) filter (where tm.user_id is not null) members,
            JSONB_AGG(DISTINCT jsonb_build_object('id', tmsg.id, 'author_id', tmsg.author_id, 'thread_id', tmsg.thread_id, 'body', tmsg.body, 'created', tmsg.created, 'hide_identity', tmsg.hide_identity)) filter (where tmsg.id is not null) messages
            FROM threads t
            LEFT OUTER JOIN threads_messages tmsg ON tmsg.thread_id = t.id
            LEFT OUTER JOIN threads_members tm ON tm.thread_id = t.id
            WHERE t.id = ANY($1)
            GROUP BY t.id
            ",
            &thread_ids_parsed
        )
        .fetch(exec)
            .map_ok(|x| DBThread {
                id: DBThreadId(x.id),
                project_id: x.mod_id.map(DBProjectId),
                report_id: x.report_id.map(DBReportId),
                type_: ThreadType::from_string(&x.thread_type),
                messages: {
                    let mut messages: Vec<DBThreadMessage> = serde_json::from_value(
                        x.messages.unwrap_or_default(),
                    )
                        .ok()
                        .unwrap_or_default();
                    messages.sort_by_key(|a| a.created);
                    messages
                },
                members: x.members.unwrap_or_default().into_iter().map(DBUserId).collect(),
            })
        .try_collect::<Vec<DBThread>>()
        .await
        .wrap_err("fetching threads")?;

        Ok(threads)
    }

    pub async fn remove_full(
        id: DBThreadId,
        transaction: &mut PgTransaction<'_>,
    ) -> Result<Option<()>> {
        sqlx::query!(
            "
            DELETE FROM threads_messages
            WHERE thread_id = $1
            ",
            id as DBThreadId,
        )
        .execute(&mut *transaction)
        .await
        .wrap_err("removing thread messages")?;
        sqlx::query!(
            "
            DELETE FROM threads_members
            WHERE thread_id = $1
            ",
            id as DBThreadId
        )
        .execute(&mut *transaction)
        .await
        .wrap_err("removing thread members")?;
        sqlx::query!(
            "
            DELETE FROM threads
            WHERE id = $1
            ",
            id as DBThreadId,
        )
        .execute(&mut *transaction)
        .await
        .wrap_err("removing thread")?;

        Ok(Some(()))
    }
}

impl DBThreadMessage {
    pub async fn get<'a, E>(
        id: DBThreadMessageId,
        exec: E,
    ) -> Result<Option<DBThreadMessage>>
    where
        E: crate::database::Executor<'a, Database = sqlx::Postgres>,
    {
        Ok(Self::get_many(&[id], exec)
            .await
            .wrap_err("fetching thread message")?
            .into_iter()
            .next())
    }

    pub async fn get_many<'a, E>(
        message_ids: &[DBThreadMessageId],
        exec: E,
    ) -> Result<Vec<DBThreadMessage>>
    where
        E: crate::database::Executor<'a, Database = sqlx::Postgres>,
    {
        use futures::stream::TryStreamExt;

        let message_ids_parsed: Vec<i64> =
            message_ids.iter().map(|x| x.0).collect();
        let messages = sqlx::query!(
            "
            SELECT tm.id, tm.author_id, tm.thread_id, tm.body, tm.created, tm.hide_identity
            FROM threads_messages tm
            WHERE tm.id = ANY($1)
            ",
            &message_ids_parsed
        )
        .fetch(exec)
        .map_ok(|x| DBThreadMessage {
            id: DBThreadMessageId(x.id),
            thread_id: DBThreadId(x.thread_id),
            author_id: x.author_id.map(DBUserId),
            body: serde_json::from_value(x.body).unwrap_or(MessageBody::Deleted { private: false }),
            created: x.created,
            hide_identity: x.hide_identity,
        })
        .try_collect::<Vec<DBThreadMessage>>()
        .await
        .wrap_err("fetching thread messages")?;

        Ok(messages)
    }

    pub async fn remove_full(
        id: DBThreadMessageId,
        private: bool,
        transaction: &mut PgTransaction<'_>,
    ) -> Result<Option<()>> {
        let body = serde_json::to_value(MessageBody::Deleted { private })
            .wrap_err("serializing deleted thread message body")?;

        sqlx::query!(
            "
            UPDATE threads_messages
            SET body = $2
            WHERE id = $1
            ",
            id as DBThreadMessageId,
            body,
        )
        .execute(&mut *transaction)
        .await
        .wrap_err("removing thread message")?;

        Ok(Some(()))
    }
}

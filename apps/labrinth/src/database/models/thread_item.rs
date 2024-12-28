use super::ids::*;
use crate::database::models::DatabaseError;
use crate::models::threads::{MessageBody, ThreadType};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub struct ThreadBuilder {
    pub type_: ThreadType,
    pub members: Vec<UserId>,
    pub project_id: Option<ProjectId>,
    pub report_id: Option<ReportId>,
}

#[derive(Clone, Serialize)]
pub struct Thread {
    pub id: ThreadId,

    pub project_id: Option<ProjectId>,
    pub report_id: Option<ReportId>,
    pub type_: ThreadType,

    pub messages: Vec<ThreadMessage>,
    pub members: Vec<UserId>,
}

pub struct ThreadMessageBuilder {
    pub author_id: Option<UserId>,
    pub body: MessageBody,
    pub thread_id: ThreadId,
    pub hide_identity: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ThreadMessage {
    pub id: ThreadMessageId,
    pub thread_id: ThreadId,
    pub author_id: Option<UserId>,
    pub body: MessageBody,
    pub created: DateTime<Utc>,
    pub hide_identity: bool,
}

impl ThreadMessageBuilder {
    pub async fn insert(
        &self,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<ThreadMessageId, DatabaseError> {
        let thread_message_id = generate_thread_message_id(transaction).await?;

        sqlx::query!(
            "
            INSERT INTO threads_messages (
                id, author_id, body, thread_id, hide_identity
            )
            VALUES (
                $1, $2, $3, $4, $5
            )
            ",
            thread_message_id as ThreadMessageId,
            self.author_id.map(|x| x.0),
            serde_json::value::to_value(self.body.clone())?,
            self.thread_id as ThreadId,
            self.hide_identity
        )
        .execute(&mut **transaction)
        .await?;

        Ok(thread_message_id)
    }
}

impl ThreadBuilder {
    pub async fn insert(
        &self,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<ThreadId, DatabaseError> {
        let thread_id = generate_thread_id(&mut *transaction).await?;
        sqlx::query!(
            "
            INSERT INTO threads (
                id, thread_type, mod_id, report_id
            )
            VALUES (
                $1, $2, $3, $4
            )
            ",
            thread_id as ThreadId,
            self.type_.as_str(),
            self.project_id.map(|x| x.0),
            self.report_id.map(|x| x.0),
        )
        .execute(&mut **transaction)
        .await?;

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
        .execute(&mut **transaction)
        .await?;

        Ok(thread_id)
    }
}

impl Thread {
    pub async fn get<'a, E>(
        id: ThreadId,
        exec: E,
    ) -> Result<Option<Thread>, sqlx::Error>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres> + Copy,
    {
        Self::get_many(&[id], exec)
            .await
            .map(|x| x.into_iter().next())
    }

    pub async fn get_many<'a, E>(
        thread_ids: &[ThreadId],
        exec: E,
    ) -> Result<Vec<Thread>, sqlx::Error>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres> + Copy,
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
            .map_ok(|x| Thread {
                id: ThreadId(x.id),
                project_id: x.mod_id.map(ProjectId),
                report_id: x.report_id.map(ReportId),
                type_: ThreadType::from_string(&x.thread_type),
                messages: {
                    let mut messages: Vec<ThreadMessage> = serde_json::from_value(
                        x.messages.unwrap_or_default(),
                    )
                        .ok()
                        .unwrap_or_default();
                    messages.sort_by(|a, b| a.created.cmp(&b.created));
                    messages
                },
                members: x.members.unwrap_or_default().into_iter().map(UserId).collect(),
            })
        .try_collect::<Vec<Thread>>()
        .await?;

        Ok(threads)
    }

    pub async fn remove_full(
        id: ThreadId,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<Option<()>, sqlx::error::Error> {
        sqlx::query!(
            "
            DELETE FROM threads_messages
            WHERE thread_id = $1
            ",
            id as ThreadId,
        )
        .execute(&mut **transaction)
        .await?;
        sqlx::query!(
            "
            DELETE FROM threads_members
            WHERE thread_id = $1
            ",
            id as ThreadId
        )
        .execute(&mut **transaction)
        .await?;
        sqlx::query!(
            "
            DELETE FROM threads
            WHERE id = $1
            ",
            id as ThreadId,
        )
        .execute(&mut **transaction)
        .await?;

        Ok(Some(()))
    }
}

impl ThreadMessage {
    pub async fn get<'a, E>(
        id: ThreadMessageId,
        exec: E,
    ) -> Result<Option<ThreadMessage>, sqlx::Error>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        Self::get_many(&[id], exec)
            .await
            .map(|x| x.into_iter().next())
    }

    pub async fn get_many<'a, E>(
        message_ids: &[ThreadMessageId],
        exec: E,
    ) -> Result<Vec<ThreadMessage>, sqlx::Error>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
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
        .map_ok(|x| ThreadMessage {
            id: ThreadMessageId(x.id),
            thread_id: ThreadId(x.thread_id),
            author_id: x.author_id.map(UserId),
            body: serde_json::from_value(x.body).unwrap_or(MessageBody::Deleted { private: false }),
            created: x.created,
            hide_identity: x.hide_identity,
        })
        .try_collect::<Vec<ThreadMessage>>()
        .await?;

        Ok(messages)
    }

    pub async fn remove_full(
        id: ThreadMessageId,
        private: bool,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<Option<()>, sqlx::error::Error> {
        sqlx::query!(
            "
            UPDATE threads_messages
            SET body = $2
            WHERE id = $1
            ",
            id as ThreadMessageId,
            serde_json::to_value(MessageBody::Deleted { private })
                .unwrap_or(serde_json::json!({}))
        )
        .execute(&mut **transaction)
        .await?;

        Ok(Some(()))
    }
}

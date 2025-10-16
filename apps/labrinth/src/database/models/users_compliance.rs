use crate::database::models::DBUserId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{query, query_scalar};
use std::fmt;

#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize,
)]
pub enum FormType {
    #[serde(rename = "W-8BEN")]
    ForeignIndividual,
    #[serde(rename = "W-8BEN-E")]
    ForeignEntity,
    #[default]
    #[serde(rename = "W-9")]
    DomesticPerson,
}

impl FormType {
    pub fn as_str(&self) -> &'static str {
        match self {
            FormType::ForeignIndividual => "W8-BEN",
            FormType::ForeignEntity => "W8-BEN-E",
            FormType::DomesticPerson => "W-9",
        }
    }

    pub fn from_str_or_default(s: &str) -> Self {
        match s {
            "W8-BEN" => FormType::ForeignIndividual,
            "W8-BEN-E" => FormType::ForeignEntity,
            "W-9" => FormType::DomesticPerson,
            _ => FormType::default(),
        }
    }

    pub fn requires_domestic_tin_match(self) -> bool {
        match self {
            FormType::ForeignIndividual | FormType::ForeignEntity => false,
            FormType::DomesticPerson => true,
        }
    }
}

impl fmt::Display for FormType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug)]
pub struct UserCompliance {
    pub id: i64,
    pub user_id: DBUserId,
    pub requested: DateTime<Utc>,
    pub signed: Option<DateTime<Utc>>,
    pub e_delivery_consented: bool,
    pub tin_matched: bool,
    pub last_checked: DateTime<Utc>,
    pub external_request_id: String,
    pub reference_id: String,
    pub form_type: Option<FormType>,
    pub requires_manual_review: bool,
}

impl UserCompliance {
    pub async fn get_by_user_id<'a, E>(
        exec: E,
        id: DBUserId,
    ) -> sqlx::Result<Option<Self>>
    where
        E: sqlx::PgExecutor<'a>,
    {
        let maybe_compliance = query!(
            r#"SELECT * FROM users_compliance WHERE user_id = $1"#,
            id.0
        )
        .fetch_optional(exec)
        .await?
        .map(|row| UserCompliance {
            id: row.id,
            user_id: id,
            requested: row.requested,
            signed: row.signed,
            e_delivery_consented: row.e_delivery_consented,
            tin_matched: row.tin_matched,
            last_checked: row.last_checked,
            external_request_id: row.external_request_id,
            reference_id: row.reference_id,
            form_type: row
                .form_type
                .as_deref()
                .map(FormType::from_str_or_default),
            requires_manual_review: row.requires_manual_review,
        });

        Ok(maybe_compliance)
    }

    /// This either inserts the row into the table or updates the row, *except the requires_manual_review* column.
    pub async fn upsert_partial<'a, E>(&mut self, exec: E) -> sqlx::Result<()>
    where
        E: sqlx::PgExecutor<'a>,
    {
        let id = query_scalar!(
            r#"
            INSERT INTO users_compliance
              (
                user_id,
                requested,
                signed,
                e_delivery_consented,
                tin_matched,
                last_checked,
                external_request_id,
                reference_id,
                form_type,
                requires_manual_review
              )
              VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            ON CONFLICT (user_id)
            DO UPDATE SET
              requested = EXCLUDED.requested,
              signed = EXCLUDED.signed,
              e_delivery_consented = EXCLUDED.e_delivery_consented,
              tin_matched = EXCLUDED.tin_matched,
              last_checked = EXCLUDED.last_checked,
              external_request_id = EXCLUDED.external_request_id,
              reference_id = EXCLUDED.reference_id,
              form_type = EXCLUDED.form_type
            RETURNING id
            "#,
            self.user_id.0,
            self.requested,
            self.signed,
            self.e_delivery_consented,
            self.tin_matched,
            self.last_checked,
            self.external_request_id,
            self.reference_id,
            self.form_type.map(|s| s.as_str()),
            self.requires_manual_review,
        )
        .fetch_one(exec)
        .await?;

        self.id = id;

        Ok(())
    }

    pub async fn update<'a, E>(&self, exec: E) -> sqlx::Result<()>
    where
        E: sqlx::PgExecutor<'a>,
    {
        query!(
            r#"
            UPDATE users_compliance
            SET
              requested = $2,
              signed = $3,
              e_delivery_consented = $4,
              tin_matched = $5,
              last_checked = $6,
              external_request_id = $7,
              reference_id = $8,
              form_type = $9,
              requires_manual_review = $10
            WHERE id = $1
            "#,
            self.id,
            self.requested,
            self.signed,
            self.e_delivery_consented,
            self.tin_matched,
            self.last_checked,
            self.external_request_id,
            self.reference_id,
            self.form_type.map(|s| s.as_str()),
            self.requires_manual_review,
        )
        .execute(exec)
        .await?;

        Ok(())
    }
}

use std::{
    collections::HashMap,
    fmt::{self, Display, Formatter},
};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::types::Json;

use crate::database::models::{
    DBFileId, DBProjectId, DatabaseError, DelphiReportId,
    DelphiReportIssueDetailsId, DelphiReportIssueId,
};

/// A Delphi malware analysis report for a project version file.
///
/// Malware analysis reports usually belong to a specific project file,
/// but they can get orphaned if the versions they belong to are deleted.
/// Thus, deleting versions does not delete these reports.
#[derive(Serialize)]
pub struct DBDelphiReport {
    pub id: DelphiReportId,
    pub file_id: Option<DBFileId>,
    /// A sequential, monotonically increasing version number for the
    /// Delphi version that generated this report.
    pub delphi_version: i32,
    pub artifact_url: String,
    pub created: DateTime<Utc>,
    pub severity: DelphiSeverity,
}

impl DBDelphiReport {
    pub async fn upsert(
        &self,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<DelphiReportId, DatabaseError> {
        Ok(DelphiReportId(sqlx::query_scalar!(
            "
            INSERT INTO delphi_reports (file_id, delphi_version, artifact_url, severity)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (file_id, delphi_version) DO UPDATE SET
                delphi_version = $2, artifact_url = $3, created = CURRENT_TIMESTAMP, severity = $4
            RETURNING id
            ",
            self.file_id as Option<DBFileId>,
            self.delphi_version,
            self.artifact_url,
            self.severity as DelphiSeverity,
        )
        .fetch_one(&mut **transaction)
        .await?))
    }
}

/// A severity level reported by Delphi.
#[derive(
    Deserialize,
    Serialize,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    sqlx::Type,
    utoipa::ToSchema,
)]
// The canonical serialized form of this enum is the snake_case representation.
// We add `alias`es so we can deserialize it from how Delphi sends it,
// which follows the Java conventions of `SCREAMING_SNAKE_CASE`.
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "delphi_severity", rename_all = "snake_case")]
pub enum DelphiSeverity {
    #[serde(alias = "LOW")]
    Low,
    #[serde(alias = "MEDIUM")]
    Medium,
    #[serde(alias = "HIGH")]
    High,
    #[serde(alias = "SEVERE")]
    Severe,
}

/// An issue found in a Delphi report. Every issue belongs to a report,
/// and a report can have zero, one, or more issues attached to it.
#[derive(Deserialize, Serialize)]
pub struct DBDelphiReportIssue {
    pub id: DelphiReportIssueId,
    pub report_id: DelphiReportId,
    pub issue_type: String,
}

/// A status a Delphi report issue can have.
#[derive(
    Deserialize,
    Serialize,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    sqlx::Type,
    utoipa::ToSchema,
)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "delphi_report_issue_status", rename_all = "snake_case")]
pub enum DelphiStatus {
    /// The issue is pending review by the moderation team.
    Pending,
    /// The issue has been rejected (i.e., reviewed as a false positive).
    /// The affected artifact has thus been verified to be clean, other issues
    /// with it notwithstanding.
    Safe,
    /// The issue has been approved (i.e., reviewed as a valid, true positive).
    /// The affected artifact has thus been verified to be potentially malicious.
    Unsafe,
}

impl Display for DelphiStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.serialize(f)
    }
}

/// What verdict a moderator can give to a project flagged for technical review.
#[derive(
    Deserialize,
    Serialize,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    sqlx::Type,
    utoipa::ToSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum DelphiVerdict {
    /// The issue has been rejected (i.e., reviewed as a false positive).
    /// The affected artifact has thus been verified to be clean, other issues
    /// with it notwithstanding.
    Safe,
    /// The issue has been approved (i.e., reviewed as a valid, true positive).
    /// The affected artifact has thus been verified to be potentially malicious.
    Unsafe,
}

/// An order in which Delphi report issues can be sorted during queries.
#[derive(Deserialize, Serialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum DelphiReportListOrder {
    CreatedAsc,
    CreatedDesc,
    PendingStatusFirst,
    SeverityAsc,
    SeverityDesc,
}

impl Display for DelphiReportListOrder {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.serialize(f)
    }
}

/// A result returned from a Delphi report issue query, slightly
/// denormalized with related entity information for ease of
/// consumption by clients.
#[derive(Serialize)]
pub struct DelphiReportIssueResult {
    pub issue: DBDelphiReportIssue,
    pub report: DBDelphiReport,
    pub details: Vec<ReportIssueDetail>,
    pub project_id: Option<DBProjectId>,
    pub project_published: Option<DateTime<Utc>>,
}

impl DBDelphiReportIssue {
    pub async fn insert(
        &self,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<DelphiReportIssueId, DatabaseError> {
        Ok(DelphiReportIssueId(
            sqlx::query_scalar!(
                "
                INSERT INTO delphi_report_issues (report_id, issue_type)
                VALUES ($1, $2)
                RETURNING id
                ",
                self.report_id as DelphiReportId,
                self.issue_type,
            )
            .fetch_one(&mut **transaction)
            .await?,
        ))
    }
}

/// The details of a Delphi report issue, which contain data about a
/// Java class affected by it. Every Delphi report issue details object
/// belongs to a specific issue, and an issue can have zero, one, or
/// more details attached to it. (Some issues may be artifact-wide,
/// or otherwise not really specific to any particular class.)
#[derive(
    Debug, Clone, Deserialize, Serialize, utoipa::ToSchema, sqlx::FromRow,
)]
pub struct ReportIssueDetail {
    /// ID of this issue detail.
    pub id: DelphiReportIssueDetailsId,
    /// ID of the issue this detail belongs to.
    pub issue_id: DelphiReportIssueId,
    /// Opaque identifier for where this issue detail is located, relative to
    /// the file scanned.
    ///
    /// This acts as a stable identifier for an issue detail, even across
    /// different versions of the same file.
    pub key: String,
    /// Name of the Java class path in which this issue was found.
    pub file_path: String,
    /// Decompiled, pretty-printed source of the Java class.
    pub decompiled_source: Option<String>,
    /// Extra detail-specific info for this detail.
    #[sqlx(json)]
    pub data: HashMap<String, serde_json::Value>,
    /// How important is this issue, as flagged by Delphi?
    pub severity: DelphiSeverity,
    /// Has this issue detail been marked as safe or unsafe?
    pub status: DelphiStatus,
}

impl ReportIssueDetail {
    pub async fn insert(
        &self,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<DelphiReportIssueDetailsId, DatabaseError> {
        Ok(DelphiReportIssueDetailsId(sqlx::query_scalar!(
            "
            INSERT INTO delphi_report_issue_details (issue_id, key, file_path, decompiled_source, data, severity)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id
            ",
            self.issue_id as DelphiReportIssueId,
            self.key,
            self.file_path,
            self.decompiled_source,
            sqlx::types::Json(&self.data) as Json<&HashMap<String, serde_json::Value>>,
            self.severity as DelphiSeverity,
        )
        .fetch_one(&mut **transaction)
        .await?))
    }

    pub async fn remove_all_by_issue_id(
        issue_id: DelphiReportIssueId,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<u64, DatabaseError> {
        Ok(sqlx::query!(
            "DELETE FROM delphi_report_issue_details WHERE issue_id = $1",
            issue_id as DelphiReportIssueId,
        )
        .execute(&mut **transaction)
        .await?
        .rows_affected())
    }
}

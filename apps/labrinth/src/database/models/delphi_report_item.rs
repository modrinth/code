use std::{
    fmt::{self, Display, Formatter},
    ops::Deref,
};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::database::models::{
    DBFileId, DBProjectId, DatabaseError, DelphiReportId, DelphiReportIssueId,
    DelphiReportIssueJavaClassId,
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
}

impl DBDelphiReport {
    pub async fn upsert(
        &self,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<DelphiReportId, DatabaseError> {
        Ok(DelphiReportId(sqlx::query_scalar!(
            "
            INSERT INTO delphi_reports (file_id, delphi_version, artifact_url)
            VALUES ($1, $2, $3)
            ON CONFLICT (file_id, delphi_version) DO UPDATE SET
                delphi_version = $2, artifact_url = $3, created = CURRENT_TIMESTAMP
            RETURNING id
            ",
            self.file_id as Option<DBFileId>,
            self.delphi_version,
            self.artifact_url,
        )
        .fetch_one(&mut **transaction)
        .await?))
    }
}

/// An issue found in a Delphi report. Every issue belongs to a report,
/// and a report can have zero, one, or more issues attached to it.
#[derive(Deserialize, Serialize)]
pub struct DBDelphiReportIssue {
    pub id: DelphiReportIssueId,
    pub report_id: DelphiReportId,
    pub issue_type: DelphiReportIssueType,
    pub status: DelphiReportIssueStatus,
}

/// An status a Delphi report issue can have.
#[derive(
    Deserialize, Serialize, Debug, Clone, Copy, PartialEq, Eq, Hash, sqlx::Type,
)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "delphi_report_issue_status")]
#[sqlx(rename_all = "snake_case")]
pub enum DelphiReportIssueStatus {
    /// The issue is pending review by the moderation team.
    Pending,
    /// The issue has been approved (i.e., reviewed as a valid, true positive).
    /// The affected artifact has thus been verified to be potentially malicious.
    Approved,
    /// The issue has been rejected (i.e., reviewed as a false positive).
    /// The affected artifact has thus been verified to be clean, other issues
    /// with it notwithstanding.
    Rejected,
}

impl Display for DelphiReportIssueStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.serialize(f)
    }
}

/// An order in which Delphi report issues can be sorted during queries.
#[derive(Deserialize, Serialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum DelphiReportListOrder {
    CreatedAsc,
    CreatedDesc,
    PendingStatusFirst,
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
    pub java_classes: Vec<DBDelphiReportIssueJavaClass>,
    pub project_id: Option<DBProjectId>,
    pub project_published: Option<DateTime<Utc>>,
}

impl DBDelphiReportIssue {
    pub async fn upsert(
        &self,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<DelphiReportIssueId, DatabaseError> {
        Ok(DelphiReportIssueId(
            sqlx::query_scalar!(
                "
                INSERT INTO delphi_report_issues (report_id, issue_type, status)
                VALUES ($1, $2, $3)
                ON CONFLICT (report_id, issue_type) DO UPDATE SET status = $3
                RETURNING id
                ",
                self.report_id as DelphiReportId,
                self.issue_type as DelphiReportIssueType,
                self.status as DelphiReportIssueStatus,
            )
            .fetch_one(&mut **transaction)
            .await?,
        ))
    }

    pub async fn find_all_by(
        ty: Option<DelphiReportIssueType>,
        status: Option<DelphiReportIssueStatus>,
        order_by: Option<DelphiReportListOrder>,
        count: Option<u16>,
        offset: Option<i64>,
        exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    ) -> Result<Vec<DelphiReportIssueResult>, DatabaseError> {
        Ok(sqlx::query!(
            r#"
            SELECT
                delphi_report_issues.id AS "id", report_id,
                issue_type AS "issue_type: DelphiReportIssueType",
                delphi_report_issues.status as "status: DelphiReportIssueStatus",

                file_id, delphi_version, artifact_url, created,
                json_array(SELECT to_jsonb(delphi_report_issue_java_classes)
                    FROM delphi_report_issue_java_classes
                    WHERE issue_id = delphi_report_issues.id
                ) AS "classes: sqlx::types::Json<Vec<DBDelphiReportIssueJavaClass>>",
                versions.mod_id AS "project_id?", mods.published AS "project_published?"
            FROM delphi_report_issues
            INNER JOIN delphi_reports ON delphi_reports.id = report_id
            LEFT OUTER JOIN files ON files.id = file_id
            LEFT OUTER JOIN versions ON versions.id = files.version_id
            LEFT OUTER JOIN mods ON mods.id = versions.mod_id
            WHERE
                (issue_type = $1 OR $1 IS NULL)
                AND (delphi_report_issues.status = $2 OR $2 IS NULL)
            ORDER BY
                CASE WHEN $3 = 'created_asc' THEN delphi_reports.created ELSE TO_TIMESTAMP(0) END ASC,
                CASE WHEN $3 = 'created_desc' THEN delphi_reports.created ELSE TO_TIMESTAMP(0) END DESC,
                CASE WHEN $3 = 'pending_status_first' THEN delphi_report_issues.status ELSE 'pending'::delphi_report_issue_status END ASC
            OFFSET $5
            LIMIT $4
            "#,
            ty as Option<DelphiReportIssueType>,
            status as Option<DelphiReportIssueStatus>,
            order_by.map(|order_by| order_by.to_string()),
            count.map(|count| count as i64),
            offset,
        )
        .map(|row| DelphiReportIssueResult {
            issue: DBDelphiReportIssue {
                id: DelphiReportIssueId(row.id),
                report_id: DelphiReportId(row.report_id),
                issue_type: row.issue_type,
                status: row.status,
            },
            report: DBDelphiReport {
                id: DelphiReportId(row.report_id),
                file_id: row.file_id.map(DBFileId),
                delphi_version: row.delphi_version,
                artifact_url: row.artifact_url,
                created: row.created,
            },
            java_classes: row
                .classes
                .into_iter()
                .flat_map(|class_list| class_list.0)
                .collect(),
            project_id: row.project_id.map(DBProjectId),
            project_published: row.project_published,
        })
        .fetch_all(exec)
        .await?)
    }
}

/// A type of issue found by Delphi for an artifact.
#[derive(
    Deserialize, Serialize, Debug, Clone, Copy, PartialEq, Eq, Hash, sqlx::Type,
)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "delphi_report_issue_type")]
#[sqlx(rename_all = "snake_case")]
pub enum DelphiReportIssueType {
    ReflectionIndirection,
    XorObfuscation,
    IncludedLibraries,
    SuspiciousBinaries,
    CorruptClasses,
    SuspiciousClasses,

    UrlUsage,
    ClassloaderUsage,
    ProcessbuilderUsage,
    RuntimeExecUsage,
    #[serde(rename = "jni_usage")]
    #[sqlx(rename = "jni_usage")]
    JNIUsage,

    MainMethod,
    NativeLoading,

    MalformedJar,
    NestedJarTooDeep,
    FailedDecompilation,
    #[serde(alias = "ANALYSIS FAILURE!")]
    AnalysisFailure,

    MalwareEasyforme,
    MalwareSimplyloader,

    /// An issue reported by Delphi but not known by labrinth yet.
    #[serde(other)]
    Unknown,
}

impl Display for DelphiReportIssueType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.serialize(f)
    }
}

/// A Java class affected by a Delphi report issue. Every affected
/// Java class belongs to a specific issue, and an issue can have zero,
/// one, or more affected classes. (Some issues may be artifact-wide,
/// or otherwise not really specific to any particular class.)
#[derive(Debug, Deserialize, Serialize)]
pub struct DBDelphiReportIssueJavaClass {
    pub id: DelphiReportIssueJavaClassId,
    pub issue_id: DelphiReportIssueId,
    pub internal_class_name: InternalJavaClassName,
    pub decompiled_source: DecompiledJavaClassSource,
}

impl DBDelphiReportIssueJavaClass {
    pub async fn upsert(
        &self,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<DelphiReportIssueJavaClassId, DatabaseError> {
        Ok(DelphiReportIssueJavaClassId(sqlx::query_scalar!(
            "
            INSERT INTO delphi_report_issue_java_classes (issue_id, internal_class_name, decompiled_source)
            VALUES ($1, $2, $3)
            ON CONFLICT (issue_id, internal_class_name) DO UPDATE SET decompiled_source = $3
            RETURNING id
            ",
            self.issue_id as DelphiReportIssueId,
            self.internal_class_name.0,
            self.decompiled_source.0,
        )
        .fetch_one(&mut **transaction)
        .await?))
    }
}

/// A [Java class name] with dots replaced by forward slashes (/).
///
/// Because class names are usually the [binary names] passed to a classloader, top level interfaces and classes
/// have a binary name that matches its canonical, fully qualified name, such canonical names are prefixed by the
/// package path the class is in, and packages usually match the directory structure within a JAR for typical
/// classloaders, this usually (but not necessarily) corresponds to the path to the class file within its JAR.
///
/// [Java class name]: https://docs.oracle.com/en/java/javase/21/docs/api/java.base/java/lang/Class.html#getName()
/// [binary names]: https://docs.oracle.com/javase/specs/jls/se21/html/jls-13.html#jls-13.1
#[derive(
    Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash, sqlx::Type,
)]
#[serde(transparent)]
#[sqlx(transparent)]
pub struct InternalJavaClassName(String);

impl Deref for InternalJavaClassName {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for InternalJavaClassName {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// The decompiled source code of a Java class.
#[derive(
    Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash, sqlx::Type,
)]
#[serde(transparent)]
#[sqlx(transparent)]
pub struct DecompiledJavaClassSource(String);

impl Deref for DecompiledJavaClassSource {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for DecompiledJavaClassSource {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

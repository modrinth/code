use crate::util::error::ApiContext as _;
use futures::StreamExt;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use ariadne::ids::UserId;

use crate::{
    database::{
        PgPool,
        models::{DBProjectId, DBUserId},
    },
    models::ids::ProjectId,
    routes::ApiError,
    util::error::Context,
};

use super::super::{TimeSlice, add_to_time_slice};
use super::{AnalyticsData, Metrics, ProjectAnalytics, ProjectMetrics};

/// Fields for [`super::ReturnMetrics::project_revenue`].
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum ProjectRevenueField {
    /// Project ID.
    ProjectId,
    /// User ID.
    ///
    /// You can only bucket by user if you are a member on the project.
    /// If you are a member of the parent organization (and have view analytics
    /// permissions), but not a member of the project, you cannot bucket by
    /// user.
    UserId,
}

/// Filters for [`super::ReturnMetrics::project_revenue`].
#[derive(Debug, Clone, Default, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ProjectRevenueFilters {}

/// Whether project revenue is provisional or finalized.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum ProjectRevenueKind {
    /// Provisional revenue which can change before finalization.
    Estimated,
    /// Finalized creator revenue.
    Actual,
}

/// [`super::ReturnMetrics::project_revenue`].
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ProjectRevenue {
    /// User these metrics are for.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<UserId>,
    /// Whether this revenue is estimated or actual.
    pub revenue_kind: ProjectRevenueKind,
    /// Total revenue for this bucket.
    #[serde(with = "rust_decimal::serde::float")]
    #[schema(value_type = f64)]
    pub(crate) revenue: Decimal,
}

pub(crate) async fn fetch(
    pool: &PgPool,
    time_slices: &mut [TimeSlice],
    req: &super::super::GetRequest,
    num_time_slices: usize,
    project_id_values: &[i64],
    user_id_bucket_project_ids: &[i64],
    can_view_all_revenue_splits: bool,
    metrics: &Metrics<ProjectRevenueField, ProjectRevenueFilters>,
) -> Result<(), ApiError> {
    let bucket_by_user_id =
        metrics.bucket_by.contains(&ProjectRevenueField::UserId);

    let mut rows = sqlx::query!(
        r#"
        SELECT
            WIDTH_BUCKET(
                EXTRACT(EPOCH FROM created)::bigint,
                EXTRACT(EPOCH FROM $1::timestamp with time zone AT TIME ZONE 'UTC')::bigint,
                EXTRACT(EPOCH FROM $2::timestamp with time zone AT TIME ZONE 'UTC')::bigint,
                $3::integer
            ) AS "bucket?",
            mod_id AS "mod_id?",
            CASE
                WHEN $5 AND ($6 OR mod_id = ANY($7)) THEN user_id
                ELSE 0
            END AS "user_id?",
            estimated AS "estimated!",
            SUM(amount) AS "amount_sum?"
        FROM (
            SELECT user_id, mod_id, amount, created, FALSE AS estimated
            FROM payouts_values
            WHERE mod_id IS NOT NULL

            UNION ALL

            SELECT user_id, mod_id, amount, created, TRUE AS estimated
            FROM payout_estimates
        ) creator_revenue
        WHERE
            -- only project revenue is counted here
            -- for affiliate code revenue, see `affiliate_code_revenue`
            creator_revenue.mod_id = ANY($4)
            AND created >= $1
            AND created < $2
        GROUP BY 1, 2, 3, 4
        "#,
        req.time_range.start,
        req.time_range.end,
        num_time_slices as i64,
        project_id_values,
        bucket_by_user_id,
        can_view_all_revenue_splits,
        user_id_bucket_project_ids,
    )
    .fetch(pool);
    while let Some(row) = rows
        .next()
        .await
        .transpose()
        .wrap_internal_err("fetching project revenue")?
    {
        let bucket = row
            .bucket
            .wrap_internal_err("bucket should be non-null - query bug!")?;
        let bucket = usize::try_from(bucket).wrap_internal_err_with(|| {
            eyre::eyre!(
                "bucket value {bucket} does not fit into `usize` - query bug!"
            )
        })?;

        if let Some(source_project) =
            row.mod_id.map(DBProjectId).map(ProjectId::from)
            && let Some(revenue) = row.amount_sum
        {
            add_to_time_slice(
                time_slices,
                bucket,
                AnalyticsData::Project(ProjectAnalytics {
                    source_project,
                    metrics: ProjectMetrics::Revenue(ProjectRevenue {
                        user_id: row
                            .user_id
                            .filter(|id| bucket_by_user_id && *id != 0)
                            .map(DBUserId)
                            .map(UserId::from),
                        revenue_kind: if row.estimated {
                            ProjectRevenueKind::Estimated
                        } else {
                            ProjectRevenueKind::Actual
                        },
                        revenue,
                    }),
                }),
            )
            .wrap_api_err("adding project revenue to time slice")?;
        }
    }

    Ok(())
}

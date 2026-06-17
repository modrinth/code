use futures::StreamExt;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::Row;

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
    /// If you bucket revenue by this field, and are not an admin, you will only
    /// get revenue metrics for your own user.
    UserId,
}

/// Filters for [`super::ReturnMetrics::project_revenue`].
#[derive(Debug, Clone, Default, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ProjectRevenueFilters {}

/// [`super::ReturnMetrics::project_revenue`].
#[derive(Debug, Clone, Default, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ProjectRevenue {
    /// User these metrics are for.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<UserId>,
    /// Total revenue for this bucket.
    pub(crate) revenue: Decimal,
}

pub(crate) async fn fetch(
    pool: &PgPool,
    time_slices: &mut [TimeSlice],
    req: &super::super::GetRequest,
    num_time_slices: usize,
    project_id_values: &[i64],
    current_user_id: DBUserId,
    is_admin: bool,
    metrics: &Metrics<ProjectRevenueField, ProjectRevenueFilters>,
) -> Result<(), ApiError> {
    let bucket_by_user_id =
        metrics.bucket_by.contains(&ProjectRevenueField::UserId);

    let mut rows = sqlx::query(
        "SELECT
            WIDTH_BUCKET(
                EXTRACT(EPOCH FROM created)::bigint,
                EXTRACT(EPOCH FROM $1::timestamp with time zone AT TIME ZONE 'UTC')::bigint,
                EXTRACT(EPOCH FROM $2::timestamp with time zone AT TIME ZONE 'UTC')::bigint,
                $3::integer
            ) AS bucket,
            mod_id,
            CASE WHEN $5 THEN user_id ELSE 0 END AS user_id,
            SUM(amount) amount_sum
        FROM payouts_values
        WHERE
            -- only project revenue is counted here
            -- for affiliate code revenue, see `affiliate_code_revenue`
            payouts_values.mod_id IS NOT NULL
            AND payouts_values.mod_id = ANY($4)
            AND created >= $1
            AND created < $2
            AND (NOT $5 OR $6 OR user_id = $7)
        GROUP BY bucket, mod_id, user_id",
    )
    .bind(req.time_range.start)
    .bind(req.time_range.end)
    .bind(num_time_slices as i64)
    .bind(project_id_values)
    .bind(bucket_by_user_id)
    .bind(is_admin)
    .bind(current_user_id as DBUserId)
    .fetch(pool);
    while let Some(row) = rows.next().await.transpose()? {
        let bucket = row
            .try_get::<Option<i32>, _>("bucket")?
            .wrap_internal_err("bucket should be non-null - query bug!")?;
        let bucket = usize::try_from(bucket).wrap_internal_err_with(|| {
            eyre::eyre!(
                "bucket value {bucket} does not fit into `usize` - query bug!"
            )
        })?;

        let mod_id = row.try_get::<Option<i64>, _>("mod_id")?;
        let user_id = row.try_get::<Option<i64>, _>("user_id")?;
        let amount_sum = row.try_get::<Option<Decimal>, _>("amount_sum")?;
        if let Some(source_project) =
            mod_id.map(DBProjectId).map(ProjectId::from)
            && let Some(revenue) = amount_sum
        {
            add_to_time_slice(
                time_slices,
                bucket,
                AnalyticsData::Project(ProjectAnalytics {
                    source_project,
                    metrics: ProjectMetrics::Revenue(ProjectRevenue {
                        user_id: user_id
                            .filter(|_| bucket_by_user_id)
                            .map(DBUserId)
                            .map(UserId::from),
                        revenue,
                    }),
                }),
            )?;
        }
    }

    Ok(())
}

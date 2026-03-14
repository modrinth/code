use eyre::{Result, WrapErr, eyre};
use futures::future::try_join_all;
use tracing::info;

use super::DelphiRunParameters;
use crate::{database::PgPool, env::ENV, models::ids::FileId};

pub async fn rescan_projects_in_queue(
    pool: &PgPool,
    http: &reqwest::Client,
) -> Result<()> {
    let delphi_version = fetch_delphi_version(http).await?;
    let old_delphi_version = fetch_stored_delphi_version(pool).await?;

    if old_delphi_version == Some(delphi_version) {
        info!(
            ?delphi_version,
            "Delphi version unchanged; skipping startup tech review rescan"
        );
        return Ok(());
    }

    info!(
        ?old_delphi_version,
        ?delphi_version,
        delphi_version,
        "Delphi version changed; rescanning tech review queue"
    );

    let project_ids = fetch_unreviewed_tech_review_project_ids(pool).await?;
    if project_ids.is_empty() {
        info!("No fully unreviewed tech review projects found to rescan");
        return Ok(());
    }

    let file_ids = fetch_project_file_ids(pool, &project_ids).await?;
    if file_ids.is_empty() {
        info!(
            project_count = project_ids.len(),
            "No files found for tech review projects selected for rescan"
        );
        return Ok(());
    }

    let file_ids = file_ids
        .into_iter()
        .map(|file_id| FileId(file_id.cast_unsigned()));

    try_join_all(file_ids.map(|file_id| async move {
        super::run(pool, DelphiRunParameters { file_id }, http)
            .await
            .wrap_err_with(|| {
                eyre!("failed to submit Delphi rescan for `{file_id:?}`")
            })
    }))
    .await?;

    info!(
        project_count = project_ids.len(),
        "Submitted Delphi rescans for all unreviewed tech review project files"
    );

    Ok(())
}

async fn fetch_delphi_version(http: &reqwest::Client) -> Result<i32> {
    let response = http
        .get(format!("{}/version", ENV.DELPHI_URL))
        .send()
        .await
        .and_then(|res| res.error_for_status())
        .wrap_err("failed to fetch Delphi version")?;

    let version = response
        .text()
        .await
        .wrap_err("failed to read Delphi version response body")?;
    let version = version.trim().parse::<i32>().wrap_err_with(|| {
        eyre!("invalid Delphi version response body: {version}")
    })?;
    Ok(version)
}

async fn fetch_stored_delphi_version(pool: &PgPool) -> Result<Option<i32>> {
    let row =
        sqlx::query_scalar!("SELECT MAX(delphi_version) FROM delphi_reports")
            .fetch_one(pool)
            .await
            .wrap_err("failed to fetch latest stored Delphi version")?;
    Ok(row)
}

async fn fetch_unreviewed_tech_review_project_ids(
    pool: &PgPool,
) -> Result<Vec<i64>> {
    sqlx::query_scalar!(
        r#"
        SELECT DISTINCT m.id
        FROM mods m
        WHERE
            EXISTS(
                SELECT 1
                FROM delphi_issue_details_with_statuses didws
                INNER JOIN delphi_report_issues dri ON dri.id = didws.issue_id
                WHERE
                    didws.project_id = m.id
                    AND didws.status = 'pending'
                    -- see delphi.rs todo comment
                    AND dri.issue_type != '__dummy'
            )
            AND NOT EXISTS(
                SELECT 1
                FROM delphi_issue_details_with_statuses didws
                INNER JOIN delphi_report_issues dri ON dri.id = didws.issue_id
                WHERE
                    didws.project_id = m.id
                    AND didws.status IN ('safe', 'unsafe')
                    -- see delphi.rs todo comment
                    AND dri.issue_type != '__dummy'
            )
        "#,
    )
    .fetch_all(pool)
    .await
    .wrap_err("failed to fetch fully unreviewed tech review project ids")
}

async fn fetch_project_file_ids(
    pool: &PgPool,
    project_ids: &[i64],
) -> Result<Vec<i64>> {
    let rows = sqlx::query_scalar!(
        r#"
        SELECT DISTINCT dr.file_id
        FROM delphi_reports dr
        INNER JOIN files f ON f.id = dr.file_id
        INNER JOIN versions v ON v.id = f.version_id
        WHERE v.mod_id = ANY($1::bigint[])
        "#,
        project_ids,
    )
    .fetch_all(pool)
    .await
    .wrap_err("failed to fetch file ids for tech review Delphi rescan")?;

    Ok(rows.into_iter().flatten().collect())
}

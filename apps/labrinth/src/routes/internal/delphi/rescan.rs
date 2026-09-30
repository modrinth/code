use eyre::{Result, WrapErr};
use tracing::info;

use crate::{
    database::{PgPool, models::DBFileId},
    queue::delphi_scan,
    util::kafka::KafkaClientState,
};

pub async fn enqueue_tech_review_files_for_new_delphi_version(
    pool: &PgPool,
    kafka_client: &KafkaClientState,
) -> Result<()> {
    let delphi_version = delphi_scan::fetch_delphi_version().await?;
    let stored_delphi_version =
        sqlx::query_scalar!("SELECT MAX(delphi_version) FROM delphi_reports")
            .fetch_one(pool)
            .await
            .wrap_err("fetching latest stored Delphi version")?;

    if stored_delphi_version == Some(delphi_version) {
        info!(
            %delphi_version,
            "Delphi version unchanged; skipping tech review rescan enqueue"
        );
        return Ok(());
    }

    let mut transaction = pool
        .begin()
        .await
        .wrap_err("beginning Delphi tech review rescan transaction")?;
    let file_ids = sqlx::query_scalar!(
        r#"
		SELECT DISTINCT file.id AS "file_id!: DBFileId"
		FROM delphi_tech_review_queue queue
		INNER JOIN versions version ON version.mod_id = queue.project_id
		INNER JOIN files file ON file.version_id = version.id
		ORDER BY file.id
		"#,
    )
    .fetch_all(&mut transaction)
    .await
    .wrap_err("fetching technical review files for Delphi rescan")?;

    for file_id in &file_ids {
        delphi_scan::enqueue_file(&mut transaction, kafka_client, *file_id)
            .await
            .wrap_err_with(|| {
                format!("enqueueing file `{file_id:?}` for Delphi rescan")
            })?;
    }

    transaction
        .commit()
        .await
        .wrap_err("committing Delphi tech review rescan enqueue")?;

    info!(
        %delphi_version,
        file_count = file_ids.len(),
        "Enqueued technical review files for new Delphi version"
    );

    Ok(())
}

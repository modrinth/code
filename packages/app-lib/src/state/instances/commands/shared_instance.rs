use crate::state::instances::adapters::sqlite::{content_rows, instance_rows};
use crate::state::instances::{
    ContentSetRemoteRef, ContentSetRemoteRefType, ContentSetSyncProvider,
    ContentSetSyncState, ContentSetSyncStatus, InstanceLink,
    SharedInstanceAttachment, SharedInstanceAttachmentInput,
    SharedInstanceRole,
};
use chrono::Utc;
use sqlx::SqlitePool;

pub(crate) async fn attach_shared_instance(
    instance_id: &str,
    input: SharedInstanceAttachmentInput,
    pool: &SqlitePool,
) -> crate::Result<()> {
    let metadata =
        instance_rows::get_instance_metadata_by_id(instance_id, pool)
            .await?
            .ok_or_else(|| {
                crate::ErrorKind::InputError("Unknown instance".to_string())
            })?;
    let content_set_id = metadata.applied_content_set.id;
    let attachment = SharedInstanceAttachment::from(input);
    let sync_state =
        shared_sync_state(&content_set_id, &attachment, Some(Utc::now()));

    let mut tx = pool.begin_with("BEGIN IMMEDIATE").await?;
    instance_rows::set_shared_instance_attachment(
        instance_id,
        Some(&attachment),
        &mut tx,
    )
    .await?;
    content_rows::upsert_content_set_remote_ref(
        &ContentSetRemoteRef {
            content_set_id: content_set_id.clone(),
            ref_type: ContentSetRemoteRefType::SharedContentSet,
            ref_id: attachment.id.clone(),
        },
        &mut tx,
    )
    .await?;
    content_rows::upsert_content_set_sync_state(&sync_state, &mut tx).await?;
    tx.commit().await?;

    Ok(())
}

pub(crate) async fn clear_shared_instance(
    instance_id: &str,
    pool: &SqlitePool,
) -> crate::Result<()> {
    detach_shared_instance(instance_id, false, pool).await
}

pub(crate) async fn quarantine_shared_instance(
    instance_id: &str,
    pool: &SqlitePool,
) -> crate::Result<()> {
    detach_shared_instance(instance_id, true, pool).await
}

async fn detach_shared_instance(
    instance_id: &str,
    quarantine: bool,
    pool: &SqlitePool,
) -> crate::Result<()> {
    let Some(metadata) =
        instance_rows::get_instance_metadata_by_id(instance_id, pool).await?
    else {
        return Ok(());
    };
    let content_set_id = metadata.applied_content_set.id;
    let retained_modpack_link = match metadata.link {
        InstanceLink::SharedInstance {
            modpack_project_id: Some(project_id),
            modpack_version_id: Some(version_id),
        } => Some(InstanceLink::ModrinthModpack {
            project_id,
            version_id,
        }),
        _ => None,
    };

    let mut tx = pool.begin_with("BEGIN IMMEDIATE").await?;
    instance_rows::set_shared_instance_attachment(instance_id, None, &mut tx)
        .await?;
    if quarantine {
        instance_rows::set_instance_quarantined(instance_id, true, &mut tx)
            .await?;
    }
    if let Some(link) = retained_modpack_link.as_ref() {
        instance_rows::upsert_instance_link(instance_id, link, &mut tx).await?;
    }
    content_rows::delete_content_set_remote_ref(
        &content_set_id,
        ContentSetRemoteRefType::SharedContentSet,
        &mut tx,
    )
    .await?;
    content_rows::delete_content_set_sync_state(&content_set_id, &mut tx)
        .await?;
    tx.commit().await?;

    Ok(())
}

pub(crate) async fn set_shared_instance_sync_status(
    instance_id: &str,
    status: ContentSetSyncStatus,
    applied_version: Option<i32>,
    latest_version: Option<i32>,
    pool: &SqlitePool,
) -> crate::Result<()> {
    let metadata =
        instance_rows::get_instance_metadata_by_id(instance_id, pool)
            .await?
            .ok_or_else(|| {
                crate::ErrorKind::InputError("Unknown instance".to_string())
            })?;
    let Some(mut attachment) = metadata.shared_instance else {
        return Ok(());
    };

    attachment.status = status;
    attachment.applied_version = applied_version;
    attachment.latest_version = latest_version;

    let sync_state = shared_sync_state(
        &metadata.applied_content_set.id,
        &attachment,
        Some(Utc::now()),
    );
    let mut tx = pool.begin_with("BEGIN IMMEDIATE").await?;
    content_rows::upsert_content_set_sync_state(&sync_state, &mut tx).await?;
    tx.commit().await?;

    Ok(())
}

pub(crate) async fn mark_shared_instance_stale(
    instance_id: &str,
    pool: &SqlitePool,
) -> crate::Result<()> {
    if let Some(sync_state) = stale_sync_state(instance_id, pool).await? {
        let mut tx = pool.begin_with("BEGIN IMMEDIATE").await?;
        content_rows::upsert_content_set_sync_state(&sync_state, &mut tx)
            .await?;
        tx.commit().await?;
    }
    Ok(())
}

pub(super) async fn stale_sync_state(
    instance_id: &str,
    pool: &SqlitePool,
) -> crate::Result<Option<ContentSetSyncState>> {
    let Some(metadata) =
        instance_rows::get_instance_metadata_by_id(instance_id, pool).await?
    else {
        return Ok(None);
    };
    let Some(mut attachment) = metadata.shared_instance else {
        return Ok(None);
    };
    if attachment.role != SharedInstanceRole::Owner {
        return Ok(None);
    }
    attachment.status = ContentSetSyncStatus::Stale;
    Ok(Some(shared_sync_state(
        &metadata.applied_content_set.id,
        &attachment,
        Some(Utc::now()),
    )))
}

fn shared_sync_state(
    content_set_id: &str,
    attachment: &SharedInstanceAttachment,
    checked_at: Option<chrono::DateTime<Utc>>,
) -> ContentSetSyncState {
    ContentSetSyncState {
        content_set_id: content_set_id.to_string(),
        provider: ContentSetSyncProvider::SharedInstance,
        applied_update_id: attachment
            .applied_version
            .map(|value| value.to_string()),
        latest_available_update_id: attachment
            .latest_version
            .map(|value| value.to_string()),
        checked_at,
        status: attachment.status,
    }
}

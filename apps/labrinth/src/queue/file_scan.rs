use std::collections::HashMap;
use std::io::{Cursor, Read};
use std::sync::Arc;

use chrono::Utc;
use eyre::{Result, eyre};
use hex::ToHex;
use sha1::Digest;
use tokio::task::{spawn, spawn_blocking};
use tracing::{Instrument, info, info_span, warn};
use zip::ZipArchive;

use crate::database::models::ids::{
    DBAttributionGroupId, DBProjectId, DBVersionId,
    generate_attribution_group_id,
};
use crate::database::models::moderation_external_item::ExternalLicense;
use crate::database::models::{DBFileId, DBUserId, DBVersion};
use crate::database::{PgPool, PgTransaction, redis::RedisPool};
use crate::env::ENV;
use crate::file_hosting::{FileHost, FileHostPublicity};
use crate::models::error::ApiError;
use crate::models::ids::FileId;
use crate::models::projects::{
    AttributionResolution, AttributionResolutionKind, DependencyAttribution,
    FlameProject,
};
use crate::queue::moderation::{
    ApprovalType, FingerprintResponse, FlameResponse,
};
use crate::util::error::Context;
use crate::util::http::HTTP_CLIENT;

const PENDING_FILE_SCAN_BATCH_SIZE: i64 = 100;

type FileScanResult<'a> = std::result::Result<(), ApiError<'a>>;

#[derive(Clone)]
struct PendingFileScan {
    file_id: DBFileId,
    url: String,
    project_id: DBProjectId,
}

/// Attribution enforcement is version/project-scoped, not file-hash-scoped.
///
/// Versions or projects listed in `attributions_exemptions` predate this
/// attribution system. They are not scanned for attribution requirements and
/// must not cause missing-attribution withholding. A later non-exempt version
/// can still contain the same override SHA1 and create attribution groups/files
/// for that SHA1. Because of that, reverse lookups from override SHA1s to
/// versions must go through the `attribution_enforced_versions` view so
/// grandfathered versions and projects are ignored without making the SHA1
/// itself exempt.
pub async fn scan_all_pending_files(
    db: &PgPool,
    redis: &RedisPool,
    file_host: Arc<dyn FileHost>,
) -> Result<()> {
    let scan_concurrency = ENV.FILE_SCAN_CONCURRENCY.max(1);

    let total_to_scan = sqlx::query_scalar!(
        r#"
        select count(*) as "count!"
        from file_scans fa
        inner join files f on f.id = fa.file_id
        inner join attribution_enforced_versions aev on aev.id = f.version_id
        where fa.attributions_scanned_at is null
        "#,
    )
    .fetch_one(db)
    .await
    .wrap_err("fetching number of files to scan")?;

    info!(
        "Found {total_to_scan} total pending files to scan, running in batches of {PENDING_FILE_SCAN_BATCH_SIZE} with concurrency {scan_concurrency}"
    );

    loop {
        let scanned_count = scan_pending_files_batch(
            db,
            redis,
            file_host.clone(),
            scan_concurrency * PENDING_FILE_SCAN_BATCH_SIZE,
        )
        .await?;

        if scanned_count == 0 {
            break;
        }
    }

    Ok(())
}

async fn scan_pending_files_batch(
    db: &PgPool,
    redis: &RedisPool,
    file_host: Arc<dyn FileHost>,
    scan_limit: i64,
) -> Result<usize> {
    let files_to_scan = sqlx::query!(
        r#"
        select
            fa.file_id as "file_id: DBFileId",
            f.url,
            v.mod_id as "project_id: DBProjectId"
        from file_scans fa
        inner join files f on f.id = fa.file_id
        inner join attribution_enforced_versions aev on aev.id = f.version_id
        inner join versions v on v.id = f.version_id
        where fa.attributions_scanned_at is null
        order by fa.file_id
        limit $1
        "#,
        scan_limit,
    )
    .fetch_all(db)
    .await
    .wrap_err("fetching files to scan")?;

    let fetched_count = files_to_scan.len();

    info!(
        "Found {fetched_count} pending files to scan, splitting into jobs of {PENDING_FILE_SCAN_BATCH_SIZE}",
    );

    let files_to_scan: Vec<_> = files_to_scan
        .into_iter()
        .map(|row| PendingFileScan {
            file_id: row.file_id,
            url: row.url,
            project_id: row.project_id,
        })
        .collect();

    let mut tasks = Vec::new();
    for chunk in files_to_scan.chunks(PENDING_FILE_SCAN_BATCH_SIZE as usize) {
        let db = db.clone();
        let redis = redis.clone();
        let file_host = file_host.clone();
        let chunk = chunk.to_vec();

        tasks.push(spawn(async move {
            scan_pending_files_chunk(&db, &redis, &*file_host, chunk).await
        }));
    }

    let mut scanned_count = 0;
    let mut errors = Vec::new();
    for task in tasks {
        match task.await.wrap_err("joining file scan task")? {
            Ok(count) => scanned_count += count,
            Err(err) => {
                errors.push(err);
            }
        }
    }

    if !errors.is_empty() {
        let error_count = errors.len();
        let error_messages = errors
            .into_iter()
            .enumerate()
            .map(|(index, err)| format!("chunk {}: {err:?}", index + 1))
            .collect::<Vec<_>>()
            .join("\n\n");

        return Err(eyre!(
            "failed to scan {error_count} pending file chunks:\n\n{error_messages}"
        ));
    }

    if fetched_count > 0 && scanned_count == 0 {
        return Err(eyre!(
            "file scan batch made no progress after fetching {fetched_count} files"
        ));
    }

    info!("Marked {} files as scanned", scanned_count);

    Ok(scanned_count)
}

async fn scan_pending_files_chunk(
    db: &PgPool,
    redis: &RedisPool,
    file_host: &dyn FileHost,
    files_to_scan: Vec<PendingFileScan>,
) -> Result<usize> {
    info!("Scanning {} files", files_to_scan.len());
    let mut scanned_count = 0;

    for row in files_to_scan {
        let human_file_id = FileId::from(row.file_id);
        let span = info_span!("scan", file_id = %human_file_id);

        let file_id = row.file_id;
        let result = async {
            info!("Scanning file");

            let overrides = extract_override_files_from_storage(
                file_host, file_id, &row.url,
            )
            .await
            .wrap_err_with(|| {
                eyre!("extracting overrides for file {file_id:?}")
            })?;

            if overrides.is_empty() {
                info!("Found no overrides");

                return Ok(());
            }

            info!("Found {} overrides", overrides.len());

            let mut txn = db
                .begin()
                .await
                .wrap_err("beginning file scan transaction")?;

            let resolved = resolve_overrides(&overrides, redis, &mut txn)
                .await
                .wrap_err_with(|| {
                    eyre!("resolving overrides for file {file_id:?}")
                })?;
            info!("Resolved: {resolved:#?}");

            persist_attribution_results(
                row.project_id,
                file_id,
                &overrides,
                &resolved,
                redis,
                &mut txn,
            )
            .await
            .wrap_err_with(|| {
                eyre!("persisting attribution results for file {file_id:?}")
            })?;

            txn.commit()
                .await
                .wrap_err("committing file scan transaction")?;

            eyre::Ok(())
        }
        .instrument(span)
        .await;

        let scan_result = file_scan_result(&result);
        match result {
            Ok(()) => {
                info!(%human_file_id, "Successfully scanned file");
            }
            Err(err) => {
                warn!(%human_file_id, "Failed to scan file: {err:?}");
            }
        }

        update_file_scan_result(db, file_id, scan_result)
            .await
            .wrap_err("marking file as scanned")?;

        scanned_count += 1;
    }

    Ok(scanned_count)
}

pub async fn scan_file(
    txn: &mut PgTransaction<'_>,
    redis: &RedisPool,
    file_host: &dyn FileHost,
    project_id: DBProjectId,
    file_id: DBFileId,
    file_url: &str,
) -> Result<()> {
    let result =
        scan_file_inner(txn, redis, file_host, project_id, file_id, file_url)
            .await;

    upsert_file_scan_result(txn, file_id, file_scan_result(&result))
        .await
        .wrap_err("marking file as scanned")?;

    result
}

async fn scan_file_inner(
    txn: &mut PgTransaction<'_>,
    redis: &RedisPool,
    file_host: &dyn FileHost,
    project_id: DBProjectId,
    file_id: DBFileId,
    file_url: &str,
) -> Result<()> {
    let overrides =
        extract_override_files_from_storage(file_host, file_id, file_url)
            .await
            .wrap_err_with(|| {
                eyre!("extracting overrides for file {file_id:?}")
            })?;

    if !overrides.is_empty() {
        let resolved = resolve_overrides(&overrides, redis, txn)
            .await
            .wrap_err_with(|| {
                eyre!("resolving overrides for file {file_id:?}")
            })?;

        persist_attribution_results(
            project_id, file_id, &overrides, &resolved, redis, txn,
        )
        .await
        .wrap_err_with(|| {
            eyre!("persisting attribution results for file {file_id:?}")
        })?;
    }

    Ok(())
}

fn file_scan_result(result: &Result<()>) -> FileScanResult<'static> {
    match result {
        Ok(()) => Ok(()),
        Err(err) => Err(ApiError {
            error: "internal_error",
            description: format!("{err:#}"),
            details: None,
        }),
    }
}

async fn update_file_scan_result(
    db: &PgPool,
    file_id: DBFileId,
    result: FileScanResult<'_>,
) -> Result<()> {
    let now = Utc::now();

    sqlx::query!(
        r#"
        update file_scans
        set
            attributions_scanned_at = $2,
            attributions_scan_result = $3
        where file_id = $1
        "#,
        file_id.0,
        now,
        sqlx::types::Json(result) as _,
    )
    .execute(db)
    .await
    .wrap_err("updating file scan result")?;

    Ok(())
}

async fn upsert_file_scan_result(
    txn: &mut PgTransaction<'_>,
    file_id: DBFileId,
    result: FileScanResult<'_>,
) -> Result<()> {
    sqlx::query!(
        r#"
        insert into file_scans (
            file_id,
            attributions_scanned_at,
            attributions_scan_result
        )
        values ($1, now(), $2)
        on conflict (file_id) do update set
            attributions_scanned_at = now(),
            attributions_scan_result = $2
        "#,
        file_id.0,
        sqlx::types::Json(result) as _,
    )
    .execute(&mut *txn)
    .await
    .wrap_err("upserting file scan result")?;

    Ok(())
}

pub async fn scan_override_files(
    file_host: &dyn FileHost,
    file_id: DBFileId,
    file_url: &str,
) -> Result<Vec<OverrideFile>> {
    extract_override_files_from_storage(file_host, file_id, file_url)
        .await
        .wrap_err_with(|| eyre!("extracting overrides for file {file_id:?}"))
}

async fn extract_override_files_from_storage(
    file_host: &dyn FileHost,
    file_id: DBFileId,
    file_url: &str,
) -> Result<Vec<OverrideFile>> {
    let key = file_url
        .strip_prefix(&ENV.CDN_URL)
        .unwrap_or(file_url)
        .trim_start_matches('/');
    let key = urlencoding::decode(key).wrap_err("decoding file URL path")?;

    let file_data = file_host
        .read_file(&key, FileHostPublicity::Public)
        .await
        .wrap_err_with(|| {
            eyre!("reading file {file_id:?} from storage at {key}")
        })?;

    spawn_blocking(move || extract_override_files(&file_data))
        .await
        .wrap_err("extracting override files")?
        .wrap_err("extracting override files")
}

#[derive(Debug)]
pub struct OverrideFile {
    pub path: String,
    pub sha1: String,
    pub murmur2: u32,
}

#[derive(Debug)]
pub enum OverrideResolution {
    OnModrinth,
    ExternalLicense {
        id: i64,
        status: ApprovalType,
        link: Option<String>,
        flame_project: Option<FlameProject>,
    },
    Flame(FlameProject),
    Unknown,
}

const OVERRIDE_PREFIXES: &[&str] = &[
    "overrides/mods",
    "client-overrides/mods",
    "server-overrides/mods",
    "overrides/shaderpacks",
    "client-overrides/shaderpacks",
    "overrides/resourcepacks",
    "client-overrides/resourcepacks",
];

fn should_scan(name: &str) -> bool {
    let should_skip = name.starts_with("mods/.connector/")
        || name.starts_with(".sable/natives/")
        || name.starts_with("local/crash_assistant/")
        || name.starts_with("mods/mcef-libraries/")
        || name.starts_with("mods/mcef-cache/")
        || name.starts_with("config/super_resolution/libraries/")
        || name.starts_with("config/Veinminer/update/")
        || name.starts_with("config/epicfight/native/")
        || name.starts_with("essential/")
        || name.ends_with(".rpo")
        || name.ends_with(".txt");
    let is_archive = name.contains(".jar") || name.contains(".zip");

    is_archive && !should_skip
}

fn extract_override_files(data: &[u8]) -> Result<Vec<OverrideFile>> {
    let reader = Cursor::new(data);
    let mut zip =
        ZipArchive::new(reader).wrap_err("creating zip archive reader")?;

    let mut files = Vec::new();

    for i in 0..zip.len() {
        let mut file = zip
            .by_index(i)
            .wrap_err_with(|| eyre!("reading file {i}"))?;
        let name = file.name().to_string();

        if file.is_dir() {
            continue;
        }

        if !OVERRIDE_PREFIXES
            .iter()
            .any(|prefix| name.starts_with(prefix))
        {
            continue;
        }

        if !should_scan(&name) {
            continue;
        }

        let mut contents = Vec::new();
        file.read_to_end(&mut contents)?;

        let sha1 = sha1::Sha1::digest(&contents).encode_hex::<String>();
        let murmur = hash_flame_murmur32(contents);

        files.push(OverrideFile {
            sha1,
            murmur2: murmur,
            path: name,
        });
    }

    Ok(files)
}

async fn persist_attribution_results(
    project_id: DBProjectId,
    file_id: DBFileId,
    overrides: &[OverrideFile],
    resolved: &HashMap<String, OverrideResolution>,
    redis: &RedisPool,
    txn: &mut PgTransaction<'_>,
) -> Result<()> {
    let all_sha1s: Vec<Vec<u8>> = overrides
        .iter()
        .map(|f| f.sha1.as_bytes().to_vec())
        .collect();

    let already_persisted: Vec<Vec<u8>> = sqlx::query_scalar!(
        "
        select paf.sha1 from project_attribution_files paf
        inner join project_attribution_groups pag on pag.id = paf.group_id
        where pag.project_id = $1 and paf.sha1 = ANY($2)
        ",
        project_id as DBProjectId,
        &all_sha1s,
    )
    .fetch_all(&mut *txn)
    .await
    .wrap_err("checking existing attribution files")?;

    let mut flame_groups: HashMap<
        u32,
        (Vec<&OverrideFile>, Option<&OverrideResolution>),
    > = HashMap::new();
    let mut external_license_files: Vec<(
        &OverrideFile,
        i64,
        ApprovalType,
        Option<String>,
        Option<FlameProject>,
    )> = Vec::new();
    let mut unknown_files: Vec<&OverrideFile> = Vec::new();

    for file in overrides {
        if already_persisted
            .iter()
            .any(|s| s.as_slice() == file.sha1.as_bytes())
        {
            continue;
        }

        match resolved.get(&file.sha1) {
            Some(OverrideResolution::OnModrinth) => continue,
            Some(OverrideResolution::ExternalLicense {
                id,
                status,
                link,
                flame_project,
            }) => {
                external_license_files.push((
                    file,
                    *id,
                    *status,
                    link.clone(),
                    flame_project.clone(),
                ));
            }
            Some(res @ OverrideResolution::Flame(flame_project)) => {
                let entry = flame_groups.entry(flame_project.id).or_default();
                entry.0.push(file);
                if entry.1.is_none() {
                    entry.1 = Some(res);
                }
            }
            Some(OverrideResolution::Unknown) | None => {
                unknown_files.push(file);
            }
        }
    }

    let existing_flame_groups = sqlx::query!(
        r#"
        select id as "id: DBAttributionGroupId", flame_project
        from project_attribution_groups
        where project_id = $1 and flame_project is not null
        "#,
        project_id as DBProjectId,
    )
    .fetch_all(&mut *txn)
    .await
    .wrap_err("fetching existing flame attribution groups")?;

    let mut existing_flame_group_ids = HashMap::new();
    for group in existing_flame_groups {
        if let Some(flame_project) = group
            .flame_project
            .and_then(|fp| serde_json::from_value::<FlameProject>(fp).ok())
        {
            existing_flame_group_ids.insert(flame_project.id, group.id);
        }
    }

    for (file, external_license_id, status, link, flame_project) in
        external_license_files
    {
        if let Some(group_id) = flame_project
            .as_ref()
            .and_then(|fp| existing_flame_group_ids.get(&fp.id))
        {
            sqlx::query!(
                r#"
                insert into project_attribution_files (group_id, name, sha1, moderation_external_license_id)
                values ($1, $2, $3, $4)
                on conflict (group_id, sha1) do update
                set moderation_external_license_id = excluded.moderation_external_license_id
                "#,
                *group_id as DBAttributionGroupId,
                &file.path,
                &file.sha1.as_bytes().to_vec() as &[u8],
                external_license_id,
            )
            .execute(&mut *txn)
            .await
            .wrap_err("inserting external license attribution file into existing flame group")?;

            continue;
        }

        let attribution = default_external_license_attribution(status, link);
        let flame_project =
            flame_project.and_then(|fp| serde_json::to_value(fp).ok());
        let group_id = generate_attribution_group_id(&mut *txn).await?;
        sqlx::query!(
			"
            insert into project_attribution_groups (id, project_id, attribution, flame_project)
            values ($1, $2, $3, $4)
            ",
			group_id as DBAttributionGroupId,
			project_id as DBProjectId,
			attribution,
			flame_project,
		)
        .execute(&mut *txn)
        .await
        .wrap_err("inserting external license attribution group")?;

        sqlx::query!(
            r#"
            insert into project_attribution_files (group_id, name, sha1, moderation_external_license_id)
            values ($1, $2, $3, $4)
            on conflict (group_id, sha1) do update
            set moderation_external_license_id = excluded.moderation_external_license_id
            "#,
            group_id as DBAttributionGroupId,
            &file.path,
            &file.sha1.as_bytes().to_vec() as &[u8],
            external_license_id,
        )
        .execute(&mut *txn)
        .await
        .wrap_err("inserting external license attribution file")?;
    }

    for (flame_project_id, (files, resolution)) in &flame_groups {
        let group_id = if let Some(group_id) =
            existing_flame_group_ids.get(flame_project_id)
        {
            *group_id
        } else {
            let fp = resolution
                .and_then(|r| {
                    if let OverrideResolution::Flame(flame_project) = r {
                        Some(serde_json::to_value(flame_project).ok())
                    } else {
                        None
                    }
                })
                .flatten();

            let id = generate_attribution_group_id(&mut *txn).await?;
            sqlx::query!(
                "
                insert into project_attribution_groups (id, project_id, flame_project)
                values ($1, $2, $3)
                ",
                id as DBAttributionGroupId,
                project_id as DBProjectId,
                fp,
            )
            .execute(&mut *txn)
            .await
            .wrap_err("inserting attribution group")?;
            existing_flame_group_ids.insert(*flame_project_id, id);
            id
        };

        let names: Vec<String> = files.iter().map(|f| f.path.clone()).collect();
        let sha1s: Vec<Vec<u8>> =
            files.iter().map(|f| f.sha1.as_bytes().to_vec()).collect();

        sqlx::query!(
            r#"
            insert into project_attribution_files (group_id, name, sha1)
            select $1, unnest($2::text[]), unnest($3::bytea[])
            on conflict (group_id, sha1) do nothing
            "#,
            group_id as DBAttributionGroupId,
            &names,
            &sha1s,
        )
        .execute(&mut *txn)
        .await
        .wrap_err("inserting attribution files")?;
    }

    for file in &unknown_files {
        let group_id = generate_attribution_group_id(&mut *txn).await?;
        sqlx::query!(
            "
            insert into project_attribution_groups (id, project_id)
            values ($1, $2)
            ",
            group_id as DBAttributionGroupId,
            project_id as DBProjectId,
        )
        .execute(&mut *txn)
        .await
        .wrap_err("inserting unknown attribution group")?;

        sqlx::query!(
            r#"
            insert into project_attribution_files (group_id, name, sha1)
            values ($1, $2, $3)
            on conflict (group_id, sha1) do nothing
            "#,
            group_id as DBAttributionGroupId,
            &file.path,
            &file.sha1.as_bytes().to_vec() as &[u8],
        )
        .execute(&mut *txn)
        .await
        .wrap_err("inserting unknown attribution file")?;
    }

    if !all_sha1s.is_empty() {
        sqlx::query!(
            "
            insert into override_file_sources (sha1, file_id)
            select unnest($1::bytea[]), $2
            on conflict do nothing
            ",
            &all_sha1s,
            file_id as DBFileId,
        )
        .execute(&mut *txn)
        .await
        .wrap_err("inserting override file sources")?;
    }

    let version_id = sqlx::query_scalar!(
        r#"
        select version_id as "version_id: DBVersionId"
        from files
        where id = $1
        "#,
        file_id as DBFileId,
    )
    .fetch_one(&mut *txn)
    .await
    .wrap_err("fetching scanned file version")?;

    DBVersion::clear_cache_ids(&[version_id], redis)
        .await
        .wrap_err("clearing version cache after attribution scan")?;

    Ok(())
}

fn default_external_license_attribution(
    status: ApprovalType,
    link: Option<String>,
) -> Option<serde_json::Value> {
    match status {
        ApprovalType::Yes
        | ApprovalType::WithAttributionAndSource
        | ApprovalType::WithAttribution => link
            .and_then(|link| url::Url::parse(&link).ok())
            .and_then(|link_to_work| {
                serde_json::to_value(AttributionResolution {
                    kind: AttributionResolutionKind::GloballyAllowed {
                        link_to_work,
                    },
                    moderation_status: None,
                    updated_by_moderator: false,
                    notes: String::new(),
                    image_urls: Vec::new(),
                })
                .ok()
            }),
        ApprovalType::No => {
            let link_to_work =
                link.and_then(|link| url::Url::parse(&link).ok());

            serde_json::to_value(AttributionResolution {
                kind: AttributionResolutionKind::NoPermission { link_to_work },
                moderation_status: None,
                updated_by_moderator: false,
                notes: String::new(),
                image_urls: Vec::new(),
            })
            .ok()
        }
        ApprovalType::PermanentNo | ApprovalType::Unidentified => None,
    }
}

async fn resolve_overrides(
    overrides: &[OverrideFile],
    redis: &RedisPool,
    txn: &mut PgTransaction<'_>,
) -> Result<HashMap<String, OverrideResolution>> {
    let mut results: HashMap<String, OverrideResolution> = HashMap::new();
    let mut remaining: Vec<usize> = (0..overrides.len()).collect();

    if overrides.is_empty() {
        return Ok(results);
    }

    let hashes: Vec<String> =
        overrides.iter().map(|x| x.sha1.clone()).collect();
    let files = DBVersion::get_files_from_hash(
        "sha1".to_string(),
        &hashes,
        &mut *txn,
        redis,
    )
    .await
    .wrap_err("fetching files on platform by hash")?;

    let version_ids: Vec<_> = files.iter().map(|x| x.version_id).collect();
    let versions_data = DBVersion::get_many(&version_ids, &mut *txn, redis)
        .await
        .wrap_err("fetching versions")?;

    for file in &files {
        if !versions_data.iter().any(|v| v.inner.id == file.version_id) {
            continue;
        }

        if let Some(hash) = file.hashes.get("sha1")
            && let Some(pos) =
                remaining.iter().position(|i| overrides[*i].sha1 == *hash)
        {
            let idx = remaining.remove(pos);
            results.insert(
                overrides[idx].sha1.clone(),
                OverrideResolution::OnModrinth,
            );
        }
    }

    if remaining.is_empty() {
        return Ok(results);
    }

    let rows = sqlx::query!(
		"
        SELECT encode(mef.sha1, 'escape') sha1, mel.id, mel.status status, mel.link
        FROM moderation_external_files mef
        INNER JOIN moderation_external_licenses mel ON mef.external_license_id = mel.id
        WHERE mef.sha1 = ANY($1)
        ",
        &remaining
            .iter()
            .map(|i| overrides[*i].sha1.as_bytes().to_vec())
            .collect::<Vec<_>>()
    )
    .fetch_all(&mut *txn)
	.await
	.wrap_err("fetching external file licenses")?;

    let mut direct_external_licenses = HashMap::new();
    for row in rows {
        if let Some(sha1) = row.sha1 {
            direct_external_licenses.insert(
                sha1,
                (
                    row.id,
                    ApprovalType::from_string(&row.status)
                        .unwrap_or(ApprovalType::Unidentified),
                    row.link,
                ),
            );
        }
    }

    let fingerprints: Vec<u32> =
        remaining.iter().map(|i| overrides[*i].murmur2).collect();
    let res = HTTP_CLIENT
        .post(format!("{}/v1/fingerprints", ENV.FLAME_ANVIL_URL))
        .json(&serde_json::json!({
            "fingerprints": fingerprints
        }))
        .send()
        .await;

    if let Err(e) = &res {
        warn!("Flame fingerprint request failed: {e}");
    }

    if let Ok(res) = res {
        let body = res
            .text()
            .await
            .wrap_err("reading Flame fingerprint response")?;

        let flame_files: Vec<_> =
            serde_json::from_str::<FlameResponse<FingerprintResponse>>(&body)
                .ok()
                .map(|x| {
                    x.data
                        .exact_matches
                        .into_iter()
                        .map(|m| m.file)
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();

        let mut flame_matches: Vec<(String, u32)> = Vec::new();
        for flame_file in &flame_files {
            if let Some(hash) = flame_file
                .hashes
                .iter()
                .find(|x| x.algo == 1)
                .map(|x| x.value.clone())
            {
                flame_matches.push((hash, flame_file.mod_id));
            }
        }

        let project_license_rows = sqlx::query!(
            "
            SELECT mel.id, mel.flame_project_id, mel.status status, mel.link
            FROM moderation_external_licenses mel
            WHERE mel.flame_project_id = ANY($1)
            ",
            &flame_matches.iter().map(|x| x.1 as i32).collect::<Vec<_>>()
        )
        .fetch_all(&mut *txn)
        .await
        .wrap_err("fetching Flame project licenses")?;

        let mut project_external_licenses = HashMap::new();
        for row in project_license_rows {
            if let Some(flame_project_id) = row.flame_project_id {
                project_external_licenses.insert(
                    flame_project_id as u32,
                    (
                        row.id,
                        ApprovalType::from_string(&row.status)
                            .unwrap_or(ApprovalType::Unidentified),
                        row.link,
                    ),
                );
            }
        }

        let flame_projects_res = HTTP_CLIENT
            .post(format!("{}/v1/mods", ENV.FLAME_ANVIL_URL))
            .json(&serde_json::json!({
                "modIds": flame_matches.iter().map(|x| x.1).collect::<Vec<_>>()
            }))
            .send()
            .await;

        let flame_projects = match flame_projects_res {
            Ok(res) => res
                .text()
                .await
                .ok()
                .and_then(|t| {
                    serde_json::from_str::<
                        FlameResponse<
                            Vec<crate::queue::moderation::FlameProjectResponse>,
                        >,
                    >(&t)
                    .ok()
                })
                .map(|x| x.data)
                .unwrap_or_default(),
            Err(e) => {
                warn!("Flame projects request failed: {e}");
                Vec::new()
            }
        };
        let mut flame_project_metadata = HashMap::new();
        for project in flame_projects {
            if flame_project_url_is_not_found(&project.links.website_url).await
            {
                info!(
                    "Flame project {} at {:?} returned 404, ignoring",
                    project.id, project.links.website_url,
                );
                continue;
            }

            flame_project_metadata.insert(
                project.id,
                FlameProject {
                    id: project.id,
                    title: project.name,
                    url: project.links.website_url,
                    icon_url: project.logo.thumbnail_url,
                },
            );
        }

        let mut insert_hashes = Vec::new();
        let mut insert_filenames = Vec::new();
        let mut insert_ids = Vec::new();

        for (sha1, flame_project_id) in &flame_matches {
            if let Some(remaining_pos) =
                remaining.iter().position(|i| overrides[*i].sha1 == *sha1)
            {
                let idx = remaining.remove(remaining_pos);
                let flame_project =
                    flame_project_metadata.get(flame_project_id).cloned();

                if let Some((id, status, link)) =
                    direct_external_licenses.remove(&overrides[idx].sha1)
                {
                    results.insert(
                        overrides[idx].sha1.clone(),
                        OverrideResolution::ExternalLicense {
                            id,
                            status,
                            link,
                            flame_project,
                        },
                    );
                } else if let Some((id, status, link)) =
                    project_external_licenses.get(flame_project_id)
                {
                    results.insert(
                        overrides[idx].sha1.clone(),
                        OverrideResolution::ExternalLicense {
                            id: *id,
                            status: *status,
                            link: link.clone(),
                            flame_project,
                        },
                    );

                    insert_hashes.push(overrides[idx].sha1.as_bytes().to_vec());
                    insert_filenames.push(Some(overrides[idx].path.clone()));
                    insert_ids.push(*id);
                } else if let Some(flame_project) = flame_project {
                    results.insert(
                        overrides[idx].sha1.clone(),
                        OverrideResolution::Flame(flame_project),
                    );
                }
            }
        }

        if !insert_hashes.is_empty() {
            ExternalLicense::insert_files(
                &mut *txn,
                &insert_hashes,
                &insert_filenames,
                &insert_ids,
                DBUserId(0),
            )
            .await
            .wrap_err("inserting external license files")?;
        }
    }

    remaining.retain(|idx| {
        if let Some((id, status, link)) =
            direct_external_licenses.remove(&overrides[*idx].sha1)
        {
            results.insert(
                overrides[*idx].sha1.clone(),
                OverrideResolution::ExternalLicense {
                    id,
                    status,
                    link,
                    flame_project: None,
                },
            );
            false
        } else {
            true
        }
    });

    for idx in remaining {
        results
            .insert(overrides[idx].sha1.clone(), OverrideResolution::Unknown);
    }

    Ok(results)
}

async fn flame_project_url_is_not_found(url: &str) -> bool {
    match HTTP_CLIENT.get(url).send().await {
        Ok(response) => response.status() == reqwest::StatusCode::NOT_FOUND,
        Err(err) => {
            warn!("Flame project URL check failed for {url}: {err}");
            false
        }
    }
}

fn hash_flame_murmur32(input: Vec<u8>) -> u32 {
    murmur2::murmur2(
        &input
            .into_iter()
            .filter(|x| *x != 9 && *x != 10 && *x != 13 && *x != 32)
            .collect::<Vec<u8>>(),
        1,
    )
}

pub async fn get_files_missing_attribution<'a, E>(
    exec: E,
    version_ids: &[DBVersionId],
) -> Result<
    std::collections::HashMap<
        DBVersionId,
        Vec<(DBFileId, Option<FlameProject>)>,
    >,
>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    if version_ids.is_empty() {
        return Ok(std::collections::HashMap::new());
    }

    let rows = sqlx::query!(
        r#"
        select distinct f.version_id as "version_id: DBVersionId", f.id as "file_id: DBFileId",
            pag.flame_project
        from files f
        inner join attribution_enforced_versions aev on aev.id = f.version_id
        inner join versions v on v.id = f.version_id
        inner join override_file_sources ofs on ofs.file_id = f.id
        inner join project_attribution_files paf on paf.sha1 = ofs.sha1
        inner join project_attribution_groups pag on pag.id = paf.group_id
        where f.version_id = ANY($1)
          and pag.project_id = v.mod_id
          and (
            pag.attribution is null
            or pag.attribution->>'kind' = 'no_permission'
            or coalesce(
              pag.attribution->'moderation_status'->>'kind',
              'approved'
            ) != 'approved'
          )
        "#,
        &version_ids.iter().map(|v| v.0).collect::<Vec<_>>(),
    )
    .fetch_all(exec)
    .await
    .wrap_err("fetching files missing attribution")?;

    let mut result = std::collections::HashMap::new();
    for row in rows {
        let flame_project = row
            .flame_project
            .and_then(|v| serde_json::from_value(v).ok());
        result
            .entry(row.version_id)
            .or_insert_with(Vec::new)
            .push((row.file_id, flame_project));
    }

    Ok(result)
}

pub struct DependencyAttributionData {
    pub attribution: DependencyAttribution,
}

pub async fn get_dependency_attributions<'a, E>(
    exec: E,
    version_ids: &[DBVersionId],
) -> Result<HashMap<i32, DependencyAttributionData>>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    if version_ids.is_empty() {
        return Ok(HashMap::new());
    }

    let version_ids_vec: Vec<_> = version_ids.iter().map(|v| v.0).collect();

    let rows = sqlx::query!(
        r#"
        select
            d.id as "dependency_id!",
            pag.attribution,
            pag.flame_project,
            pag.project_id as "project_id: DBProjectId"
        from dependencies d
        inner join files f on f.version_id = d.dependent_id
        inner join attribution_enforced_versions aev on aev.id = f.version_id
        inner join versions v on v.id = f.version_id
        inner join override_file_sources ofs on ofs.file_id = f.id
        inner join project_attribution_files paf on paf.sha1 = ofs.sha1
        inner join project_attribution_groups pag on pag.id = paf.group_id
        where d.dependent_id = ANY($1)
          and pag.project_id = v.mod_id
          and d.dependency_file_name is not null
          and (
            pag.flame_project is not null
            or pag.attribution is not null
          )
          and split_part(paf.name, '/', -1) = d.dependency_file_name
        "#,
        &version_ids_vec,
    )
    .fetch_all(exec)
    .await
    .wrap_err("fetching dependency attributions")?;

    let mut result = HashMap::new();
    for row in rows {
        let attribution: Option<AttributionResolution> =
            row.attribution.and_then(|v| serde_json::from_value(v).ok());

        let flame_project: Option<FlameProject> = row
            .flame_project
            .and_then(|v| serde_json::from_value(v).ok());

        let resolution = attribution.map(|a| a.kind);

        result.insert(
            row.dependency_id,
            DependencyAttributionData {
                attribution: DependencyAttribution {
                    flame_project,
                    resolution,
                },
            },
        );
    }

    Ok(result)
}

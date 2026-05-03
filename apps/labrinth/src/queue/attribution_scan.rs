use std::collections::HashMap;
use std::io::{Cursor, Read};

use chrono::Utc;
use eyre::{Result, eyre};
use hex::ToHex;
use sha1::Digest;
use tokio::task::spawn_blocking;
use tracing::{Instrument, info, info_span, warn};
use zip::ZipArchive;

use crate::database::models::ids::{
    DBAttributionGroupId, DBProjectId, generate_attribution_group_id,
};
use crate::database::models::moderation_external_item::ExternalLicense;
use crate::database::models::{DBFileId, DBUserId, DBVersion};
use crate::database::{PgPool, PgTransaction, redis::RedisPool};
use crate::env::ENV;
use crate::file_hosting::{FileHost, FileHostPublicity};
use crate::models::ids::FileId;
use crate::queue::moderation::{
    ApprovalType, FingerprintResponse, FlameProject, FlameResponse,
};
use crate::util::error::Context;
use crate::util::http::HTTP_CLIENT;

pub async fn scan_all_file_override_attributions(
    db: &PgPool,
    redis: &RedisPool,
    file_host: &dyn FileHost,
) -> Result<()> {
    let mut txn = db.begin().await.wrap_err("beginning transaction")?;

    let files_to_scan = sqlx::query!(
        r#"
        select
            fa.file_id as "file_id: DBFileId",
            f.url,
            v.mod_id as "project_id: DBProjectId"
        from file_attributions fa
        inner join files f on f.id = fa.file_id
        inner join versions v on v.id = f.version_id
        where fa.scanned_at is null
        "#
    )
    .fetch_all(&mut txn)
    .await
    .wrap_err("fetching files to scan")?;

    info!("Found {} files to scan", files_to_scan.len());

    let mut scanned_ids = Vec::new();

    for row in files_to_scan {
        let human_file_id = FileId::from(row.file_id);
        let span = info_span!("scan", file_id = %human_file_id);
        async {
            info!("Scanning file");

            let file_id = row.file_id;

            let overrides = extract_override_files_from_storage(
                file_host, file_id, &row.url,
            )
            .await
            .wrap_err_with(|| {
                eyre!("extracting overrides for file {file_id:?}")
            })?;

            if overrides.is_empty() {
                info!("Found no overrides");
            } else {
                info!("Found {} overrides", overrides.len());

                let resolved = resolve_overrides(&overrides, redis, &mut txn)
                    .await
                    .wrap_err_with(|| {
                        eyre!("resolving overrides for file {file_id:?}")
                    })?;
                info!("Resolved: {resolved:#?}");

                persist_attribution_results(
                    row.project_id,
                    &overrides,
                    &resolved,
                    &mut txn,
                )
                .await
                .wrap_err_with(|| {
                    eyre!("persisting attribution results for file {file_id:?}")
                })?;
            }

            scanned_ids.push(file_id.0);
            eyre::Ok(())
        }
        .instrument(span)
        .await?;
    }

    if !scanned_ids.is_empty() {
        let now = Utc::now();
        sqlx::query!(
            "
            update file_attributions
            set scanned_at = now
            from unnest($1::bigint[], $2::timestamptz[]) as u(id, now)
            where file_attributions.file_id = u.id
            ",
            &scanned_ids,
            &vec![now; scanned_ids.len()],
        )
        .execute(&mut txn)
        .await
        .wrap_err("marking files as scanned")?;
    }

    info!("Marked {} files as scanned", scanned_ids.len());

    txn.commit().await.wrap_err("committing transaction")?;

    Ok(())
}

pub async fn scan_file_override_attributions(
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

        persist_attribution_results(project_id, &overrides, &resolved, txn)
            .await
            .wrap_err_with(|| {
                eyre!("persisting attribution results for file {file_id:?}")
            })?;
    }

    sqlx::query!(
        "
        insert into file_attributions (file_id, scanned_at)
        values ($1, now())
        on conflict (file_id) do update set scanned_at = now()
        ",
        file_id.0,
    )
    .execute(&mut *txn)
    .await
    .wrap_err("marking file as scanned")?;

    Ok(())
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

    let file_data = file_host
        .read_file(key, FileHostPublicity::Public)
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
    ExternalLicense(ApprovalType),
    Flame {
        project_id: u32,
        title: String,
        url: String,
    },
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

        if !OVERRIDE_PREFIXES
            .iter()
            .any(|prefix| name.starts_with(prefix))
        {
            continue;
        }

        if name.matches('/').count() > 2
            || name.ends_with(".txt")
            || name.ends_with(".rpo")
        {
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
    overrides: &[OverrideFile],
    resolved: &HashMap<String, OverrideResolution>,
    txn: &mut PgTransaction<'_>,
) -> Result<()> {
    let existing_sha1s: Vec<Vec<u8>> = overrides
        .iter()
        .map(|f| f.sha1.as_bytes().to_vec())
        .collect();

    let already_persisted: Vec<Vec<u8>> = sqlx::query_scalar!(
        "
        select sha1 from project_attribution_files
        where sha1 = ANY($1)
        ",
        &existing_sha1s,
    )
    .fetch_all(&mut *txn)
    .await
    .wrap_err("checking existing attribution files")?;

    let mut flame_groups: HashMap<u32, (Vec<&OverrideFile>, Option<String>)> =
        HashMap::new();
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
            Some(OverrideResolution::ExternalLicense(_)) => {
                // TODO: handle external license attribution
                continue;
            }
            Some(OverrideResolution::Flame {
                project_id: fp_id,
                title,
                ..
            }) => {
                let entry = flame_groups.entry(*fp_id).or_default();
                entry.0.push(file);
                if entry.1.is_none() {
                    entry.1 = Some(title.clone());
                }
            }
            Some(OverrideResolution::Unknown) | None => {
                unknown_files.push(file);
            }
        }
    }

    let existing_flame_groups = sqlx::query!(
        r#"
        select id as "id: DBAttributionGroupId", flame_project_id
        from project_attribution_groups
        where project_id = $1 and flame_project_id is not null
        "#,
        project_id as DBProjectId,
    )
    .fetch_all(&mut *txn)
    .await
    .wrap_err("fetching existing flame attribution groups")?;

    for (flame_project_id, (files, title)) in &flame_groups {
        let existing = existing_flame_groups
            .iter()
            .find(|g| g.flame_project_id == Some(*flame_project_id as i64));

        let group_id = if let Some(group) = existing {
            group.id
        } else {
            let id = generate_attribution_group_id(&mut *txn).await?;
            sqlx::query!(
                "
                insert into project_attribution_groups (id, project_id, flame_project_id, flame_project_title)
                values ($1, $2, $3, $4)
                ",
                id as DBAttributionGroupId,
                project_id as DBProjectId,
                *flame_project_id as i64,
                title.as_deref().unwrap_or_default(),
            )
            .execute(&mut *txn)
            .await
            .wrap_err("inserting attribution group")?;
            id
        };

        let names: Vec<String> = files.iter().map(|f| f.path.clone()).collect();
        let sha1s: Vec<Vec<u8>> =
            files.iter().map(|f| f.sha1.as_bytes().to_vec()).collect();

        sqlx::query!(
            "
            insert into project_attribution_files (group_id, name, sha1)
            select $1, unnest($2::text[]), unnest($3::bytea[])
            ",
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
            "
            insert into project_attribution_files (group_id, name, sha1)
            values ($1, $2, $3)
            ",
            group_id as DBAttributionGroupId,
            &file.path,
            &file.sha1.as_bytes().to_vec() as &[u8],
        )
        .execute(&mut *txn)
        .await
        .wrap_err("inserting unknown attribution file")?;
    }

    Ok(())
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
        SELECT encode(mef.sha1, 'escape') sha1, mel.status status
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

    for row in rows {
        if let Some(sha1) = row.sha1
            && let Some(pos) =
                remaining.iter().position(|i| overrides[*i].sha1 == sha1)
        {
            let idx = remaining.remove(pos);
            results.insert(
                overrides[idx].sha1.clone(),
                OverrideResolution::ExternalLicense(
                    ApprovalType::from_string(&row.status)
                        .unwrap_or(ApprovalType::Unidentified),
                ),
            );
        }
    }

    if remaining.is_empty() {
        return Ok(results);
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

        let rows = sqlx::query!(
            "
            SELECT mel.id, mel.flame_project_id, mel.status status
            FROM moderation_external_licenses mel
            WHERE mel.flame_project_id = ANY($1)
            ",
            &flame_matches.iter().map(|x| x.1 as i32).collect::<Vec<_>>()
        )
        .fetch_all(&mut *txn)
        .await
        .wrap_err("fetching Flame project licenses")?;

        let mut insert_hashes = Vec::new();
        let mut insert_filenames = Vec::new();
        let mut insert_ids = Vec::new();

        for row in &rows {
            if let Some((curse_idx, (hash, _))) = flame_matches
                .iter()
                .enumerate()
                .find(|(_, x)| Some(x.1 as i32) == row.flame_project_id)
                && let Some(remaining_pos) =
                    remaining.iter().position(|i| overrides[*i].sha1 == *hash)
            {
                let idx = remaining.remove(remaining_pos);

                results.insert(
                    overrides[idx].sha1.clone(),
                    OverrideResolution::ExternalLicense(
                        ApprovalType::from_string(&row.status)
                            .unwrap_or(ApprovalType::Unidentified),
                    ),
                );

                insert_hashes.push(overrides[idx].sha1.as_bytes().to_vec());
                insert_filenames.push(Some(overrides[idx].path.clone()));
                insert_ids.push(row.id);

                flame_matches.remove(curse_idx);
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

        if !flame_matches.is_empty() {
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
                                FlameResponse<Vec<FlameProject>>,
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

            for (sha1, flame_project_id) in &flame_matches {
                if let Some(pos) =
                    remaining.iter().position(|i| overrides[*i].sha1 == *sha1)
                {
                    let idx = remaining.remove(pos);

                    let project = flame_projects
                        .iter()
                        .find(|p| p.id == *flame_project_id);

                    results.insert(
                        overrides[idx].sha1.clone(),
                        OverrideResolution::Flame {
                            project_id: *flame_project_id,
                            title: project
                                .map(|p| p.name.clone())
                                .unwrap_or_else(|| {
                                    format!("Flame project {flame_project_id}")
                                }),
                            url: project
                                .map(|p| p.links.website_url.clone())
                                .unwrap_or_default(),
                        },
                    );
                }
            }
        }
    }

    for idx in remaining {
        results
            .insert(overrides[idx].sha1.clone(), OverrideResolution::Unknown);
    }

    Ok(results)
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

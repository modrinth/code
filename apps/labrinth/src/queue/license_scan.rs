use std::io::Cursor;

use bytes::Bytes;
use eyre::{Result, eyre};
use tokio::task::spawn_blocking;
use tracing::info;
use zip::ZipArchive;

use crate::{
    database::{PgPool, PgTransaction, models::DBFileId, redis::RedisPool},
    file_hosting::FileHost,
    util::error::Context,
};

pub async fn scan_all_file_overrides(
    db: &PgPool,
    redis: &RedisPool,
    file_host: &dyn FileHost,
) -> Result<()> {
    let mut txn = db.begin().await.wrap_err("beginning transaction")?;

    let files_to_scan = sqlx::query!(
        r#"
        select file_id as "file_id: DBFileId"
        from file_attributions
        where scanned_at is null
        "#
    )
    .fetch_all(&db)
    .await
    .wrap_err("fetching files to scan")?;

    info!("Found {} files to scan", files_to_scan.len());

    for file_id in files_to_scan {
        let file_data = (); // todo get

        scan_file_overrides(redis, &mut txn, file_id.file_id, file_data)
            .await
            .wrap_err_with(|| eyre!("scanning file {file_id}"))?;
    }

    txn.commit().await.wrap_err("committing transaction")?;

    Ok(())
}

pub async fn scan_file_overrides(
    redis: &RedisPool,
    txn: &mut PgTransaction<'_>,
    file_id: DBFileId,
    file_data: Bytes,
) -> Result<()> {
    let overrides = {
        let file_data = file_data.clone();
        spawn_blocking(move || extract_override_files(&file_data))
            .await
            .wrap_err("extracting override files")?
            .wrap_err("extracting override files")?
    };

    if !overrides.is_empty() {}

    sqlx::query!(
        "
        update file_attributions
        set scanned_at = now()
        where file_id = $1
        ",
        file_id,
    )
    .execute(&mut *txn)
    .await
    .wrap_err("marking file as scanned")?;

    Ok(())
}

#[derive(Debug)]
pub struct OverrideFile {
    pub path: String,
    pub sha1: String,
    pub murmur2: u32,
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

/// Takes a modpack zip file and finds all override files within it.
///
/// Each override file is also bundled with its SHA-1 hash and Murmur2 hashes,
/// for use in looking up license info in Flame's and our own database.
fn extract_override_files(data: &[u8]) -> Result<Vec<OverrideFile>> {
    let reader = Cursor::new(data);
    let mut zip =
        ZipArchive::new(reader).wrap_err("creating zip archive reader")?;

    let mut files = Vec::new();

    for i in 0..zip.len() {
        let mut file =
            zip.by_index(i).wrap_err_with(|| eyre("reading file {i}"))?;
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

fn hash_flame_murmur32(input: Vec<u8>) -> u32 {
    murmur2::murmur2(
        &input
            .into_iter()
            .filter(|x| *x != 9 && *x != 10 && *x != 13 && *x != 32)
            .collect::<Vec<u8>>(),
        1,
    )
}

/*
*
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


// pub async fn resolve_overrides(
//     overrides: &[OverrideFile],
//     transaction: &mut PgTransaction<'_>,
//     pool: &PgPool,
//     redis: &RedisPool,
//     http: &HttpClient,
// ) -> Result<HashMap<String, OverrideResolution>> {
//     let mut results: HashMap<String, OverrideResolution> = HashMap::new();
//     let mut remaining: Vec<usize> = (0..overrides.len()).collect();

//     if overrides.is_empty() {
//         return Ok(results);
//     }

//     // Tier 1: already on Modrinth
//     let hashes = overrides.iter().map(|x| x.sha1.clone()).collect::<Vec<_>>();
//     let files = DBVersion::get_files_from_hash(
//         "sha1".to_string(),
//         &hashes,
//         pool,
//         redis,
//     )
//     .await
//     .context("fetching files on platform by hash")?;

//     let version_ids = files
//         .iter()
//         .map(|x| x.version_id)
//         .collect::<HashSet<_>>()
//         .into_iter()
//         .collect::<Vec<_>>();
//     let versions_data = DBVersion::get_many(&version_ids, pool, redis)
//         .await
//         .context("fetching versions")?;

//     for file in &files {
//         //
//         if !versions_data.iter().any(|v| v.inner.id == file.version_id) {
//             continue;
//         }

//         if let Some(hash) = file.hashes.get("sha1")
//             && let Some(pos) =
//                 remaining.iter().position(|i| overrides[*i].sha1 == *hash)
//         {
//             let idx = remaining.remove(pos);
//             results.insert(
//                 overrides[idx].sha1.clone(),
//                 OverrideResolution::OnModrinth,
//             );
//         }
//     }

//     if remaining.is_empty() {
//         return results;
//     }

//     // Tier 2: previously reviewed by SHA1
//     let rows = sqlx::query!(
//         "
 //         SELECT encode(mef.sha1, 'escape') sha1, mel.status status
 //         FROM moderation_external_files mef
 //         INNER JOIN moderation_external_licenses mel ON mef.external_license_id = mel.id
 //         WHERE mef.sha1 = ANY($1)
 //         ",
//         &remaining
//             .iter()
//             .map(|i| overrides[*i].sha1.as_bytes().to_vec())
//             .collect::<Vec<_>>()
//     )
//     .fetch_all(&mut *transaction)
//     .await
//     .unwrap_or_default();

//     for row in rows {
//         if let Some(sha1) = row.sha1
//             && let Some(pos) =
//                 remaining.iter().position(|i| overrides[*i].sha1 == sha1)
//         {
//             let idx = remaining.remove(pos);
//             results.insert(
//                 overrides[idx].sha1.clone(),
//                 OverrideResolution::ExternalLicense(
//                     ApprovalType::from_string(&row.status)
//                         .unwrap_or(ApprovalType::Unidentified),
//                 ),
//             );
//         }
//     }

//     if remaining.is_empty() {
//         return results;
//     }

//     // Tier 3: Flame fingerprint lookup
//     let fingerprints: Vec<u32> =
//         remaining.iter().map(|i| overrides[*i].murmur2).collect();
//     let res = http
//         .post(format!("{}/v1/fingerprints", ENV.FLAME_ANVIL_URL))
//         .json(&serde_json::json!({
//             "fingerprints": fingerprints
//         }))
//         .send()
//         .await;

//     if let Ok(res) = res {
//         let body = res.text().await.unwrap_or_default();

//         let flame_files: Vec<_> =
//             serde_json::from_str::<FlameResponse<FingerprintResponse>>(&body)
//                 .ok()
//                 .map(|x| {
//                     x.data
//                         .exact_matches
//                         .into_iter()
//                         .map(|m| m.file)
//                         .collect::<Vec<_>>()
//                 })
//                 .unwrap_or_default();

//         let mut flame_matches: Vec<(String, u32)> = Vec::new();
//         for flame_file in &flame_files {
//             if let Some(hash) = flame_file
//                 .hashes
//                 .iter()
//                 .find(|x| x.algo == 1)
//                 .map(|x| x.value.clone())
//             {
//                 flame_matches.push((hash, flame_file.mod_id));
//             }
//         }

//         let rows = sqlx::query!(
//             "
 //             SELECT mel.id, mel.flame_project_id, mel.status status
 //             FROM moderation_external_licenses mel
 //             WHERE mel.flame_project_id = ANY($1)
 //             ",
//             &flame_matches.iter().map(|x| x.1 as i32).collect::<Vec<_>>()
//         )
//         .fetch_all(&mut *transaction)
//         .await
//         .unwrap_or_default();

//         let mut insert_hashes = Vec::new();
//         let mut insert_filenames = Vec::new();
//         let mut insert_ids = Vec::new();

//         for row in &rows {
//             if let Some((curse_idx, (hash, _))) = flame_matches
//                 .iter()
//                 .enumerate()
//                 .find(|(_, x)| Some(x.1 as i32) == row.flame_project_id)
//                 && let Some(remaining_pos) =
//                     remaining.iter().position(|i| overrides[*i].sha1 == *hash)
//             {
//                 let idx = remaining.remove(remaining_pos);

//                 results.insert(
//                     overrides[idx].sha1.clone(),
//                     OverrideResolution::ExternalLicense(
//                         ApprovalType::from_string(&row.status)
//                             .unwrap_or(ApprovalType::Unidentified),
//                     ),
//                 );

//                 insert_hashes.push(overrides[idx].sha1.as_bytes().to_vec());
//                 insert_filenames.push(Some(overrides[idx].path.clone()));
//                 insert_ids.push(row.id);

//                 flame_matches.remove(curse_idx);
//             }
//         }

//         if !insert_hashes.is_empty() {
//             ExternalLicense::insert_files(
//                 &mut *transaction,
//                 &insert_hashes,
//                 &insert_filenames,
//                 &insert_ids,
//                 crate::database::models::DBUserId(0),
//             )
//             .await
//             .ok();
//         }

//         // Tier 4: found on Flame but no license ruling — fetch project info
//         if !flame_matches.is_empty() {
//             let flame_projects_res = http
//                 .post(format!("{}v1/mods", ENV.FLAME_ANVIL_URL))
//                 .json(&serde_json::json!({
//                     "modIds": flame_matches.iter().map(|x| x.1).collect::<Vec<_>>()
//                 }))
//                 .send()
//                 .await;

//             let flame_projects = match flame_projects_res {
//                 Ok(res) => res
//                     .text()
//                     .await
//                     .ok()
//                     .and_then(|t| {
//                         serde_json::from_str::<
//                                 FlameResponse<Vec<FlameProject>>,
//                             >(&t)
//                             .ok()
//                     })
//                     .map(|x| x.data)
//                     .unwrap_or_default(),
//                 Err(_) => Vec::new(),
//             };

//             for (sha1, flame_project_id) in &flame_matches {
//                 if let Some(pos) =
//                     remaining.iter().position(|i| overrides[*i].sha1 == *sha1)
//                 {
//                     let idx = remaining.remove(pos);

//                     let project = flame_projects
//                         .iter()
//                         .find(|p| p.id == *flame_project_id);

//                     results.insert(
//                         overrides[idx].sha1.clone(),
//                         OverrideResolution::Flame {
//                             project_id: *flame_project_id,
//                             title: project
//                                 .map(|p| p.name.clone())
//                                 .unwrap_or_else(|| {
//                                     format!("Flame project {flame_project_id}")
//                                 }),
//                             url: project
//                                 .map(|p| p.links.website_url.clone())
//                                 .unwrap_or_default(),
//                         },
//                     );
//                 }
//             }
//         }
//     }

//     // Remaining: truly unknown
//     for idx in remaining {
//         results
//             .insert(overrides[idx].sha1.clone(), OverrideResolution::Unknown);
//     }

//     results
// }

*/

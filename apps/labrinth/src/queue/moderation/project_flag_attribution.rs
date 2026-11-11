use std::collections::{HashMap, HashSet};

use ariadne::ids::UserId;
use chrono::{DateTime, Utc};
use eyre::{Result, WrapErr};
use serde::Serialize;
use utoipa::ToSchema;

use super::ApprovalType;
use crate::{
    database::{ReadOnlyPgPool, models::DBProjectId},
    models::{
        ids::{AttributionGroupId, ProjectId, VersionId},
        projects::{AttributionResolution, FlameProject},
    },
};

#[derive(Clone, Serialize, ToSchema)]
pub struct FlagAttributionGroup {
    pub id: AttributionGroupId,
    pub flame_project: Option<FlameProject>,
    pub attribution: Option<AttributionResolution>,
    pub attributed_at: Option<DateTime<Utc>>,
    pub attributed_by: Option<UserId>,
    pub files: Vec<FlagAttributionFile>,
    pub versions: Vec<FlagAttributionVersion>,
    pub override_files_on_platform: Vec<FlagAttributionOverrideFile>,
}

#[derive(Clone, Serialize, ToSchema)]
pub struct FlagAttributionFile {
    pub name: String,
    pub sha1: String,
    pub versions: Vec<VersionId>,
    pub moderation_external_license_id: Option<i64>,
    pub moderation_external_license: Option<FlagAttributionExternalLicense>,
}

#[derive(Clone, Serialize, ToSchema)]
pub struct FlagAttributionVersion {
    pub id: VersionId,
    pub name: String,
    pub version_number: String,
    pub date_created: DateTime<Utc>,
}

#[derive(Clone, Serialize, ToSchema)]
pub struct FlagAttributionOverrideFile {
    pub file_path: String,
    pub sha1: String,
    pub version_id: VersionId,
    pub platform_version_id: VersionId,
    pub platform_project_id: ProjectId,
}

#[derive(Clone, Serialize, ToSchema)]
pub struct FlagAttributionExternalLicense {
    pub id: i64,
    pub title: Option<String>,
    pub status: ApprovalType,
    pub link: Option<String>,
    pub exceptions: Option<String>,
    pub proof: Option<String>,
    pub flame_project_id: Option<i32>,
    pub inserted_at: Option<DateTime<Utc>>,
    pub inserted_by: Option<UserId>,
    pub updated_at: Option<DateTime<Utc>>,
    pub updated_by: Option<UserId>,
}

/// Loads unredacted moderator-only permission evidence for flag evaluation.
pub(super) async fn load_attributions(
    project_ids: &[DBProjectId],
    pool: &ReadOnlyPgPool,
) -> Result<HashMap<DBProjectId, Vec<FlagAttributionGroup>>> {
    let mut result: HashMap<_, Vec<_>> = project_ids
        .iter()
        .copied()
        .map(|id| (id, Vec::new()))
        .collect();
    if project_ids.is_empty() {
        return Ok(result);
    }
    let ids: Vec<_> = project_ids.iter().map(|id| id.0).collect();
    let groups = sqlx::query!(
        r#"
        SELECT id, project_id, flame_project, attribution, attributed_at, attributed_by
        FROM project_attribution_groups
        WHERE project_id = ANY($1)
        ORDER BY project_id, id
        "#,
        &ids,
    )
    .fetch_all(&**pool)
    .await
    .wrap_err("fetching attribution groups for flag evaluation")?;
    if groups.is_empty() {
        return Ok(result);
    }
    let group_ids: Vec<_> = groups.iter().map(|group| group.id).collect();
    let files = sqlx::query!(
        r#"
        SELECT paf.group_id AS "group_id!", paf.name AS "name!",
            CONVERT_FROM(paf.sha1, 'UTF8') AS "sha1!",
            paf.moderation_external_license_id,
            COALESCE(ARRAY_AGG(DISTINCT aev.id) FILTER (WHERE aev.id IS NOT NULL), '{}') AS "version_ids!: Vec<i64>"
        FROM project_attribution_files paf
        INNER JOIN project_attribution_groups pag ON pag.id = paf.group_id
        LEFT JOIN override_file_sources ofs ON ofs.sha1 = paf.sha1
        LEFT JOIN files f ON f.id = ofs.file_id
        LEFT JOIN versions v ON v.id = f.version_id AND v.mod_id = pag.project_id
        LEFT JOIN attribution_enforced_versions aev ON aev.id = v.id
        WHERE paf.group_id = ANY($1)
        GROUP BY paf.group_id, paf.name, paf.sha1, paf.moderation_external_license_id
        ORDER BY paf.group_id, paf.name, paf.sha1
        "#,
        &group_ids,
    )
    .fetch_all(&**pool)
    .await
    .wrap_err("fetching attribution files for flag evaluation")?;

    let mut license_ids: Vec<_> = files
        .iter()
        .filter_map(|file| file.moderation_external_license_id)
        .collect();
    license_ids.sort_unstable();
    license_ids.dedup();
    let mut licenses = HashMap::new();
    if !license_ids.is_empty() {
        let rows = sqlx::query!(
            r#"
            SELECT id, title, status, link, exceptions, proof, flame_project_id,
                inserted_at, inserted_by, updated_at, updated_by
            FROM moderation_external_licenses
            WHERE id = ANY($1)
            "#,
            &license_ids,
        )
        .fetch_all(&**pool)
        .await
        .wrap_err(
            "fetching attribution external licenses for flag evaluation",
        )?;
        for row in rows {
            licenses.insert(
                row.id,
                FlagAttributionExternalLicense {
                    id: row.id,
                    title: row.title,
                    status: ApprovalType::from_string(&row.status)
                        .unwrap_or(ApprovalType::Unidentified),
                    link: row.link,
                    exceptions: row.exceptions,
                    proof: row.proof,
                    flame_project_id: row.flame_project_id,
                    inserted_at: row.inserted_at,
                    inserted_by: row.inserted_by.map(|id| UserId(id as u64)),
                    updated_at: row.updated_at,
                    updated_by: row.updated_by.map(|id| UserId(id as u64)),
                },
            );
        }
    }

    let mut version_ids: Vec<_> = files
        .iter()
        .flat_map(|file| file.version_ids.iter().copied())
        .collect();
    version_ids.sort_unstable();
    version_ids.dedup();
    let mut versions = Vec::new();
    if !version_ids.is_empty() {
        let rows = sqlx::query!(
            r#"
            SELECT id, name, version_number, date_published
            FROM versions
            WHERE id = ANY($1)
            ORDER BY date_published DESC, id
            "#,
            &version_ids,
        )
        .fetch_all(&**pool)
        .await
        .wrap_err("fetching attribution versions for flag evaluation")?;
        versions.extend(rows.into_iter().map(|row| FlagAttributionVersion {
            id: VersionId(row.id as u64),
            name: row.name,
            version_number: row.version_number,
            date_created: row.date_published,
        }));
    }
    let version_order: HashMap<_, _> = versions
        .iter()
        .enumerate()
        .map(|(index, version)| (version.id, index))
        .collect();
    let mut files_by_group: HashMap<_, Vec<_>> = HashMap::new();
    for file in files {
        let mut file_versions: Vec<_> = file
            .version_ids
            .into_iter()
            .map(|id| VersionId(id as u64))
            .collect();
        file_versions.sort_by_key(|id| {
            version_order.get(id).copied().unwrap_or(usize::MAX)
        });
        files_by_group.entry(file.group_id).or_default().push(
            FlagAttributionFile {
                name: file.name,
                sha1: file.sha1,
                versions: file_versions,
                moderation_external_license_id: file
                    .moderation_external_license_id,
                moderation_external_license: file
                    .moderation_external_license_id
                    .and_then(|id| licenses.get(&id).cloned()),
            },
        );
    }

    let platform_files = sqlx::query!(
        r#"
        SELECT source_version.mod_id AS project_id,
            ofs.file_path AS "file_path!", CONVERT_FROM(ofs.sha1, 'UTF8') AS "sha1!",
            source_file.version_id, platform_file.version_id AS platform_version_id,
            platform_version.mod_id AS platform_project_id
        FROM files source_file
        INNER JOIN versions source_version ON source_version.id = source_file.version_id
        INNER JOIN override_file_sources ofs ON ofs.file_id = source_file.id
        INNER JOIN hashes h ON h.algorithm = 'sha1' AND h.hash = ofs.sha1
        INNER JOIN files platform_file ON platform_file.id = h.file_id
        INNER JOIN versions platform_version ON platform_version.id = platform_file.version_id
        WHERE source_version.mod_id = ANY($1)
        ORDER BY source_version.mod_id, source_file.version_id, ofs.file_path,
            platform_file.version_id, platform_version.mod_id
        "#,
        &ids,
    )
    .fetch_all(&**pool)
    .await
    .wrap_err("fetching attribution override files on platform for flag evaluation")?;
    let mut platform_files_by_project: HashMap<_, Vec<_>> = HashMap::new();
    for row in platform_files {
        platform_files_by_project
            .entry(row.project_id)
            .or_default()
            .push(FlagAttributionOverrideFile {
                file_path: row.file_path,
                sha1: row.sha1,
                version_id: VersionId(row.version_id as u64),
                platform_version_id: VersionId(row.platform_version_id as u64),
                platform_project_id: ProjectId(row.platform_project_id as u64),
            });
    }

    for group in groups {
        let files = files_by_group.remove(&group.id).unwrap_or_default();
        let group_version_ids: HashSet<_> = files
            .iter()
            .flat_map(|file| file.versions.iter().copied())
            .collect();
        let sha1s: HashSet<_> =
            files.iter().map(|file| file.sha1.as_str()).collect();
        let override_files_on_platform = platform_files_by_project
            .get(&group.project_id)
            .into_iter()
            .flatten()
            .filter(|file| sha1s.contains(file.sha1.as_str()))
            .cloned()
            .collect();
        let attribution = group
            .attribution
            .map(serde_json::from_value::<AttributionResolution>)
            .transpose()
            .wrap_err(
                "deserializing attribution resolution for flag evaluation",
            )?;
        let flame_project = group
            .flame_project
            .map(serde_json::from_value::<FlameProject>)
            .transpose()
            .wrap_err("deserializing attribution source for flag evaluation")?;
        result
            .entry(DBProjectId(group.project_id))
            .or_default()
            .push(FlagAttributionGroup {
                id: AttributionGroupId(group.id as u64),
                flame_project,
                attribution,
                attributed_at: group.attributed_at,
                attributed_by: group.attributed_by.map(|id| UserId(id as u64)),
                versions: versions
                    .iter()
                    .filter(|version| group_version_ids.contains(&version.id))
                    .cloned()
                    .collect(),
                files,
                override_files_on_platform,
            });
    }
    Ok(result)
}

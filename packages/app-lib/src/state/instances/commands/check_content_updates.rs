use crate::state::instances::{
    ContentEntry, InstanceFile,
    adapters::sqlite::{content_rows, instance_rows},
};
use crate::state::{
    CacheBehaviour, CachedEntry, ProjectType, ReleaseChannel, State,
};
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub(crate) struct ContentUpdate {
    pub relative_path: String,
    pub current_version_id: String,
    pub update_version_id: String,
}

pub(crate) async fn check_content_updates(
    instance_id: &str,
    cache_behaviour: Option<CacheBehaviour>,
    state: &State,
) -> crate::Result<Vec<ContentUpdate>> {
    let instance = instance_rows::get_instance_by_id(instance_id, &state.pool)
        .await?
        .ok_or_else(|| {
            crate::ErrorKind::InputError("Unknown instance".to_string())
        })?;
    let content_set =
        content_rows::get_applied_content_set(&instance.id, &state.pool)
            .await?
            .ok_or_else(|| {
                crate::ErrorKind::InputError(format!(
                    "Instance {} has no applied content set",
                    instance.id
                ))
            })?;
    let entries =
        content_rows::get_content_entries(&content_set.id, &state.pool).await?;
    let files =
        content_rows::get_instance_files(&instance.id, &state.pool).await?;
    let files_by_id = files
        .into_iter()
        .map(|file| (file.id.clone(), file))
        .collect::<HashMap<_, _>>();
    let candidates = entries
        .into_iter()
        .filter_map(|entry| {
            let file = files_by_id.get(entry.file_id.as_ref()?)?;
            let version_id = entry.version_id.clone()?;
            Some((entry, file.clone(), version_id))
        })
        .collect::<Vec<_>>();

    if candidates.is_empty() {
        return Ok(Vec::new());
    }

    let installed_channels =
        installed_update_channels(&candidates, cache_behaviour, state).await?;
    let update_keys = candidates
        .iter()
        .map(|(entry, file, _)| {
            update_cache_key(
                file,
                entry.project_type,
                effective_update_channel(
                    instance.update_channel,
                    installed_channels.get(&file.sha1).copied(),
                ),
                &content_set.game_version,
                content_set.loader.as_str(),
            )
        })
        .collect::<Vec<_>>();
    let update_key_refs = update_keys
        .iter()
        .map(|key| key.as_str())
        .collect::<Vec<_>>();
    let updates = CachedEntry::get_file_update_many(
        &update_key_refs,
        cache_behaviour,
        &state.pool,
        &state.api_semaphore,
    )
    .await?;
    let mut updates_by_hash: HashMap<String, Vec<String>> = HashMap::new();
    for update in updates {
        updates_by_hash
            .entry(update.hash)
            .or_default()
            .push(update.update_version_id);
    }

    let mut output = Vec::new();
    for (entry, file, current_version_id) in candidates {
        let update_version_id = updates_by_hash
            .remove(&file.sha1)
            .unwrap_or_default()
            .into_iter()
            .find(|update_version_id| update_version_id != &current_version_id);

        content_rows::upsert_content_update_check(
            &entry.id,
            instance.update_channel,
            update_version_id.as_deref(),
            &state.pool,
        )
        .await?;

        if let Some(update_version_id) = update_version_id {
            output.push(ContentUpdate {
                relative_path: file.relative_path,
                current_version_id,
                update_version_id,
            });
        }
    }

    Ok(output)
}

async fn installed_update_channels(
    candidates: &[(ContentEntry, InstanceFile, String)],
    cache_behaviour: Option<CacheBehaviour>,
    state: &State,
) -> crate::Result<HashMap<String, ReleaseChannel>> {
    let version_ids = candidates
        .iter()
        .map(|(_, _, version_id)| version_id.as_str())
        .collect::<Vec<_>>();
    let versions = CachedEntry::get_version_many(
        &version_ids,
        cache_behaviour,
        &state.pool,
        &state.api_semaphore,
    )
    .await?;
    let channels_by_version_id = versions
        .into_iter()
        .map(|version| {
            (
                version.id,
                ReleaseChannel::from_version_type(&version.version_type),
            )
        })
        .collect::<HashMap<_, _>>();

    Ok(candidates
        .iter()
        .filter_map(|(_, file, version_id)| {
            channels_by_version_id
                .get(version_id)
                .copied()
                .map(|channel| (file.sha1.clone(), channel))
        })
        .collect())
}

fn effective_update_channel(
    preferred: ReleaseChannel,
    installed: Option<ReleaseChannel>,
) -> ReleaseChannel {
    installed.map_or(preferred, |channel| preferred.least_stable(channel))
}

fn update_cache_key(
    file: &InstanceFile,
    project_type: ProjectType,
    channel: ReleaseChannel,
    game_version: &str,
    loader: &str,
) -> String {
    format!(
        "{}-{}-{}-{}",
        file.sha1,
        if project_type == ProjectType::Mod {
            loader.to_string()
        } else {
            project_type.get_loaders().join("+")
        },
        channel.key(),
        game_version
    )
}

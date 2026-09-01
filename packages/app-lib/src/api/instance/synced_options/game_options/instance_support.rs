//! Works out which instances can receive each shared setting.

use super::super as synced_options;
use super::CATALOG_REVISION;
use super::api_types::*;
use super::catalog::*;
use super::options_file::{
    GameOptionsDocument, options_path, read_document, sha1_bytes,
};
use crate::state::{
    CanonicalValue, InstanceMetadata, State, StoredOption,
    game_options_sync_is_enabled,
};
use std::collections::BTreeMap;

/// The instance data needed to decide whether a setting can be synced.
#[derive(Clone)]
pub(super) struct ParticipatingInstance {
    metadata: InstanceMetadata,
    deferred: bool,
    fullscreen_controlled: bool,
    pub(super) document: Option<GameOptionsDocument>,
    inspection_error: Option<String>,
}

/// Loads every instance with game-settings sync turned on.
///
/// An instance may not have an `options.txt` until its first launch. It still takes
/// part in sync, but its settings are marked as waiting for the file.
pub(super) async fn load_participating_instances(
    state: &State,
) -> crate::Result<Vec<ParticipatingInstance>> {
    if !game_options_sync_is_enabled(&state.pool).await? {
        return Ok(Vec::new());
    }
    let instances = crate::state::list_instances(&state.pool).await?;
    let mut participants = Vec::new();
    for metadata in instances {
        if !metadata.synced_options.game_options {
            continue;
        }
        let deferred = synced_options::sync_files_are_protected(&metadata)
            || synced_options::instance_is_running(&metadata, state).await?;
        let path = options_path(&metadata, state);
        let (document, inspection_error) = if path.exists() {
            match read_document(&path).await {
                Ok((document, _)) => (Some(document), None),
                Err(error) => {
                    tracing::warn!(
                        "Could not inspect game options compatibility for {}: {error}",
                        metadata.instance.id
                    );
                    (None, Some(error.to_string()))
                }
            }
        } else {
            (None, None)
        };
        participants.push(ParticipatingInstance {
            fullscreen_controlled: metadata
                .launch_overrides
                .force_fullscreen
                .is_some(),
            metadata,
            deferred,
            document,
            inspection_error,
        });
    }
    Ok(participants)
}

pub(super) fn summary_revision(
    canonical_revision: u64,
    participants: &[ParticipatingInstance],
) -> String {
    let mut summary = format!("{CATALOG_REVISION}:{canonical_revision}");
    let mut participants = participants.iter().collect::<Vec<_>>();
    participants.sort_by(|left, right| {
        left.metadata.instance.id.cmp(&right.metadata.instance.id)
    });
    for participant in participants {
        summary.push('|');
        summary.push_str(&participant.metadata.instance.id);
        summary.push(':');
        summary
            .push_str(&participant.metadata.applied_content_set.game_version);
        summary.push(':');
        summary.push_str(if participant.deferred {
            "deferred"
        } else {
            "ready"
        });
        summary.push(':');
        summary.push_str(if participant.inspection_error.is_some() {
            "degraded"
        } else if participant.document.is_some() {
            "file"
        } else {
            "missing"
        });
        summary.push(':');
        summary.push_str(if participant.fullscreen_controlled {
            "fullscreen-controlled"
        } else {
            "fullscreen-shared"
        });
    }
    sha1_bytes(summary.as_bytes())
}
/// Counts which instances can receive a setting now, later, or not at all.
///
/// The settings screen uses these results for its preview, and the file writer uses
/// the same checks when it saves. A setting shown as blocked therefore cannot be
/// written behind the user's back.
pub(super) fn describe_instance_support(
    definition: Option<&SupportedSetting>,
    stored: &StoredOption,
    value: Option<&CanonicalValue>,
    participants: &[ParticipatingInstance],
) -> GameOptionCompatibility {
    let total = participants.len() as u32;
    let mut will_receive = 0;
    let mut write_now = 0;
    let mut buckets: BTreeMap<
        (String, String, String, String),
        GameOptionCompatibilityBucket,
    > = BTreeMap::new();

    for participant in participants {
        let version = participant
            .metadata
            .applied_content_set
            .game_version
            .clone();
        // Check launcher-controlled settings first. Finding the key in options.txt
        // does not make a launcher-owned setting safe to overwrite.
        let (status, key, mapping, reason) = if stored.option_id == "fullscreen"
            && participant.fullscreen_controlled
        {
            (
                GameOptionCompatibilityStatus::Controlled,
                None,
                None,
                Some(GameOptionCompatibilityReason::LauncherControlled),
            )
        } else if definition.is_some()
            && !supported_settings_cover_game_version(&version)
        {
            (
                GameOptionCompatibilityStatus::CatalogUncovered,
                None,
                None,
                Some(GameOptionCompatibilityReason::CatalogUncovered),
            )
        } else if participant.inspection_error.is_some() {
            (
                GameOptionCompatibilityStatus::Degraded,
                None,
                None,
                Some(GameOptionCompatibilityReason::InspectionFailed),
            )
        } else {
            match &participant.document {
                None => {
                    // An instance that has not launched yet has no options.txt. If we know
                    // the key Minecraft will create, wait for the file instead of excluding it.
                    if definition.is_none()
                        && stored.raw_key.is_some()
                        && matches!(value, Some(CanonicalValue::ExternalRaw(_)))
                    {
                        (
						GameOptionCompatibilityStatus::WaitingForFile,
						stored.raw_key.clone(),
						Some(GameOptionMappingKind::Direct),
						Some(GameOptionCompatibilityReason::WaitingForOptionsFile),
					)
                    } else if let Some((key, mapping)) =
                        definition.and_then(|definition| {
                            physical_variant_for_version(definition, &version)
                                .map(|variant| (variant.key, variant.mapping))
                                .or_else(|| {
                                    definition
                                        .versioned_keys
                                        .is_empty()
                                        .then(|| {
                                            definition
										.keys
										.first()
										.copied()
										.map(|key| (key, GameOptionMappingKind::Direct))
                                        })
                                        .flatten()
                                })
                        })
                    {
                        let representable = value.is_none_or(|value| {
                            definition.is_some_and(|definition| {
                                encode_value(
                                    definition, key, value, &version, None,
                                )
                                .is_some()
                            })
                        });
                        if representable {
                            (
							GameOptionCompatibilityStatus::WaitingForFile,
							Some(key.to_string()),
							Some(mapping),
							Some(GameOptionCompatibilityReason::WaitingForOptionsFile),
						)
                        } else {
                            (
							GameOptionCompatibilityStatus::UnsupportedValue,
							Some(key.to_string()),
							None,
							Some(GameOptionCompatibilityReason::UnsupportedValue),
						)
                        }
                    } else {
                        (
                            GameOptionCompatibilityStatus::NotAvailable,
                            None,
                            None,
                            Some(GameOptionCompatibilityReason::MissingSetting),
                        )
                    }
                }
                Some(document) => {
                    // Vanilla keys sometimes change between versions. Replace an old key with
                    // its newer name only when we know the value can be converted safely.
                    let key = target_physical_key(
                        definition, stored, document, &version,
                    );
                    if let Some(key) = key {
                        let needs_alias_migration =
                            definition.is_some_and(|definition| {
                                alias_migration_needed(
                                    definition, document, &key,
                                )
                            });
                        let representable =
                            value.is_none_or(|value| match definition {
                                Some(definition) => encode_value(
                                    definition,
                                    &key,
                                    value,
                                    &version,
                                    document.value(&key),
                                )
                                .is_some(),
                                None => matches!(
                                    value,
                                    CanonicalValue::ExternalRaw(_)
                                ),
                            });
                        if !representable {
                            (
							GameOptionCompatibilityStatus::UnsupportedValue,
							Some(key.to_string()),
							None,
							Some(GameOptionCompatibilityReason::UnsupportedValue),
						)
                        } else {
                            let mapped = if needs_alias_migration {
                                Some(GameOptionMappingKind::Migrated)
                            } else {
                                definition.and_then(|definition| {
                                    physical_variant_for_present_key(
                                        definition, &key, &version,
                                    )
                                    .map(|variant| variant.mapping)
                                    .or_else(|| {
                                        (matches!(
                                            definition.encoding,
                                            ValueEncoding::KeyBinding
                                        ) && document.value(&key).is_some_and(
                                            |raw| {
                                                split_key_binding(raw)
                                                    .0
                                                    .parse::<i32>()
                                                    .is_ok()
                                            },
                                        ))
                                        .then_some(
                                            GameOptionMappingKind::Migrated,
                                        )
                                    })
                                })
                            };
                            let legacy = definition.is_some_and(|definition| {
                                definition.keys.first().copied()
                                    != Some(key.as_str())
                            });
                            (
							if participant.deferred {
								GameOptionCompatibilityStatus::Deferred
							} else {
								GameOptionCompatibilityStatus::Ready
							},
							Some(key.to_string()),
							Some(mapped.unwrap_or(if legacy {
								GameOptionMappingKind::Legacy
							} else {
								GameOptionMappingKind::Direct
							})),
							needs_alias_migration
								.then_some(GameOptionCompatibilityReason::MigratesOnWrite),
						)
                        }
                    } else if let Some(definition) =
                        definition.filter(|definition| {
                            (definition.versioned_keys.is_empty()
                                || physical_variant_for_version(
                                    definition, &version,
                                )
                                .is_some())
                                && definition
                                    .keys
                                    .iter()
                                    .any(|key| document.value(key).is_some())
                        })
                    {
                        let eventual_key =
                            physical_variant_for_version(definition, &version)
                                .map(|variant| variant.key.to_string())
                                .or_else(|| {
                                    definition
                                        .keys
                                        .iter()
                                        .find(|key| {
                                            document.value(key).is_some()
                                        })
                                        .map(|key| (*key).to_string())
                                });
                        let representable =
                            eventual_key.as_deref().is_some_and(|key| {
                                value.is_none_or(|value| {
                                    encode_value(
                                        definition, key, value, &version, None,
                                    )
                                    .is_some()
                                })
                            });
                        if representable {
                            (
							GameOptionCompatibilityStatus::WaitingForBase,
							eventual_key,
							Some(GameOptionMappingKind::Migrated),
							Some(GameOptionCompatibilityReason::WaitingForCompatibleBase),
						)
                        } else {
                            (
							GameOptionCompatibilityStatus::UnsupportedValue,
							eventual_key,
							None,
							Some(GameOptionCompatibilityReason::UnsupportedValue),
						)
                        }
                    } else {
                        (
                            GameOptionCompatibilityStatus::NotAvailable,
                            None,
                            None,
                            Some(GameOptionCompatibilityReason::MissingSetting),
                        )
                    }
                }
            }
        };

        let receives = matches!(
            status,
            GameOptionCompatibilityStatus::Ready
                | GameOptionCompatibilityStatus::Deferred
                | GameOptionCompatibilityStatus::WaitingForFile
                | GameOptionCompatibilityStatus::WaitingForBase
        );
        if receives {
            will_receive += 1;
        }
        if matches!(status, GameOptionCompatibilityStatus::Ready) {
            write_now += 1;
        }
        let status_key = serde_json::to_string(&status).unwrap_or_default();
        let key_name = key.clone().unwrap_or_default();
        let mapping_key = mapping
            .as_ref()
            .and_then(|mapping| serde_json::to_string(mapping).ok())
            .unwrap_or_default();
        let reason_key = reason
            .as_ref()
            .and_then(|reason| serde_json::to_string(reason).ok())
            .unwrap_or_default();
        // Group instances with the same result to keep the response small. Keep their
        // Minecraft versions so the UI can explain which versions are affected.
        let bucket = buckets
            .entry((status_key, key_name.clone(), mapping_key, reason_key))
            .or_insert_with(|| GameOptionCompatibilityBucket {
                instance_count: 0,
                write_keys: if matches!(
                    status,
                    GameOptionCompatibilityStatus::Ready
                        | GameOptionCompatibilityStatus::Deferred
                ) {
                    key.clone().into_iter().collect()
                } else {
                    Vec::new()
                },
                eventual_keys: if matches!(
                    status,
                    GameOptionCompatibilityStatus::WaitingForFile
                        | GameOptionCompatibilityStatus::WaitingForBase
                ) {
                    key.clone().into_iter().collect()
                } else {
                    Vec::new()
                },
                game_versions: vec![version.clone()],
                status,
                mapping,
                reason,
            });
        bucket.instance_count += 1;
        if !bucket.game_versions.contains(&version) {
            bucket.game_versions.push(version);
            bucket.game_versions.sort();
        }
    }

    let left_local = total.saturating_sub(will_receive);
    GameOptionCompatibility {
        total_participating: total,
        will_receive,
        write_now,
        left_local,
        buckets: buckets.into_values().collect(),
    }
}

/// Uses the local value only when every readable instance currently agrees on it.
pub(super) fn find_common_local_value(
    definition: Option<&SupportedSetting>,
    stored: &StoredOption,
    participants: &[ParticipatingInstance],
) -> (Option<CanonicalValue>, GameOptionValueState) {
    let mut values = Vec::new();
    let mut invalid = false;
    for participant in participants {
        if stored.option_id == "fullscreen" && participant.fullscreen_controlled
        {
            continue;
        }
        if definition.is_some()
            && !supported_settings_cover_game_version(
                &participant.metadata.applied_content_set.game_version,
            )
        {
            continue;
        }
        let Some(document) = &participant.document else {
            continue;
        };
        let key = if let Some(definition) = definition {
            observed_physical_key(
                definition,
                document,
                &participant.metadata.applied_content_set.game_version,
            )
        } else {
            target_physical_key(
                None,
                stored,
                document,
                &participant.metadata.applied_content_set.game_version,
            )
        };
        let Some(key) = key else {
            continue;
        };
        let Some(raw_value) = document.value(&key) else {
            continue;
        };
        let value = if let Some(definition) = definition {
            decode_value(definition, &key, raw_value)
        } else {
            Some(CanonicalValue::ExternalRaw(raw_value.to_string()))
        };
        if let Some(value) = value
            .filter(|value| validate_canonical_value(definition, value).is_ok())
        {
            values.push(value);
        } else {
            invalid = true;
        }
    }
    if invalid && values.is_empty() {
        return (None, GameOptionValueState::Invalid);
    }
    let Some(first) = values.first().cloned() else {
        return (None, GameOptionValueState::Unset);
    };
    if values
        .iter()
        .all(|value| canonical_values_equal(value, &first))
    {
        (Some(first), GameOptionValueState::UniformLocal)
    } else {
        (None, GameOptionValueState::Mixed)
    }
}

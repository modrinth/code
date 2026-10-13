//! Keeps game settings intact when a modpack is installed or updated.

use super::MAX_OPTIONS_BYTES;
use super::options_file::{
    GameOptionsDocument, input_error, options_path, sha1_bytes,
};
use super::write_shared_settings::{
    apply_shared_settings_to_instance, capture_instance_options,
    sync_is_active_for_instance,
};
use crate::state::{InstanceLink, InstanceMetadata, State, SyncedOption};
use crate::util::io;
use encoding_rs::{UTF_8, UTF_16BE, UTF_16LE};
use std::io::ErrorKind;
use std::path::Path;

const YOSBR_OPTIONS_PATH: &str = "config/yosbr/options.txt";
const DEFAULT_OPTIONS_PATH: &str = "config/defaultoptions/options.txt";
const DEFAULT_KEYBINDINGS_PATH: &str = "config/defaultoptions/keybindings.txt";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GameOptionsPackSource {
    ClientOverrides,
    Overrides,
    ClientOverridesYosbr,
    OverridesYosbr,
    ClientOverridesDefaultOptions,
    OverridesDefaultOptions,
}

impl GameOptionsPackSource {
    fn as_str(self) -> &'static str {
        match self {
            Self::ClientOverrides
            | Self::ClientOverridesYosbr
            | Self::ClientOverridesDefaultOptions => "client_overrides",
            Self::Overrides
            | Self::OverridesYosbr
            | Self::OverridesDefaultOptions => "overrides",
        }
    }

    fn template_path(self) -> Option<&'static str> {
        match self {
            Self::ClientOverridesYosbr | Self::OverridesYosbr => {
                Some(YOSBR_OPTIONS_PATH)
            }
            Self::ClientOverridesDefaultOptions
            | Self::OverridesDefaultOptions => Some(DEFAULT_OPTIONS_PATH),
            Self::ClientOverrides | Self::Overrides => None,
        }
    }
}

fn detected_encoding_name(bytes: &[u8]) -> String {
    if bytes.starts_with(&[0xFF, 0xFE]) {
        return UTF_16LE.name().to_string();
    }
    if bytes.starts_with(&[0xFE, 0xFF]) {
        return UTF_16BE.name().to_string();
    }
    if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        return UTF_8.name().to_string();
    }
    let mut detector = chardetng::EncodingDetector::new();
    detector.feed(bytes, true);
    detector.guess(None, true).name().to_string()
}

fn pack_version_id(metadata: &InstanceMetadata) -> Option<&str> {
    match &metadata.link {
        InstanceLink::ModrinthModpack { version_id, .. } => Some(version_id),
        InstanceLink::ServerProjectModpack {
            content_version_id, ..
        } => Some(content_version_id),
        InstanceLink::ImportedModpack { version_id, .. } => {
            version_id.as_deref()
        }
        InstanceLink::SharedInstance {
            modpack_version_id, ..
        } => modpack_version_id.as_deref(),
        _ => None,
    }
}

async fn read_pack_options(
    path: &Path,
    name: &str,
) -> crate::Result<Option<(Vec<u8>, String, String)>> {
    let metadata = match tokio::fs::symlink_metadata(path).await {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(io::IOError::with_path(error, path).into()),
    };
    if metadata.file_type().is_symlink() {
        return Err(input_error(format!(
            "Refusing to capture a symlinked {name}"
        )));
    }
    if metadata.len() > MAX_OPTIONS_BYTES as u64 {
        return Err(input_error(format!(
            "The modpack {name} is too large to capture for syncing"
        )));
    }
    let bytes = io::read(path).await?;
    let sha1 = sha1_bytes(&bytes);
    let encoding = detected_encoding_name(&bytes);
    Ok(Some((bytes, sha1, encoding)))
}

async fn options_target_exists(path: &Path) -> crate::Result<bool> {
    match tokio::fs::symlink_metadata(path).await {
        Ok(metadata) if metadata.file_type().is_symlink() => {
            Err(input_error("Refusing to replace a symlinked options.txt"))
        }
        Ok(_) => Ok(true),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(false),
        Err(error) => Err(io::IOError::with_path(error, path).into()),
    }
}

fn merge_default_keybindings(
    document: &mut GameOptionsDocument,
    keybindings: &GameOptionsDocument,
) -> crate::Result<bool> {
    let mut changed = false;
    for line in &keybindings.lines {
        let Some(entry) = &line.entry else {
            continue;
        };
        if !entry.key.starts_with("key_")
            || document.value(&entry.key).is_some()
        {
            continue;
        }
        let Some(value) = keybindings.value(&entry.key) else {
            continue;
        };
        let (key, modifier) = super::catalog::split_key_binding(value);
        // Modified bindings use loader-specific storage. Leaving them absent lets
        // Default Options apply them without marking an incomplete binding as seen.
        if modifier.is_some_and(|value| !value.is_empty() && value != "NONE") {
            continue;
        }
        changed |= document.set(&entry.key, key, true)?;
    }
    Ok(changed)
}

async fn read_pack_options_template(
    metadata: &InstanceMetadata,
    state: &State,
    relative_path: &str,
) -> crate::Result<Option<(Vec<u8>, String, String)>> {
    let directory = super::super::instance_dir(metadata, state);
    let captured =
        read_pack_options(&directory.join(relative_path), relative_path)
            .await?;
    if captured.is_none() && relative_path != DEFAULT_OPTIONS_PATH {
        return Ok(None);
    }
    let keybindings = read_pack_options(
        &directory.join(DEFAULT_KEYBINDINGS_PATH),
        "Default Options keybindings.txt",
    )
    .await?;
    let Some((keybindings, _, _)) = keybindings else {
        return Ok(captured);
    };
    let mut document = if let Some((bytes, _, _)) = &captured {
        GameOptionsDocument::parse(bytes)?
    } else if relative_path == DEFAULT_OPTIONS_PATH {
        let Some(document) =
            GameOptionsDocument::for_instance(metadata, state).await?
        else {
            return Ok(None);
        };
        document
    } else {
        return Ok(None);
    };
    let changed = merge_default_keybindings(
        &mut document,
        &GameOptionsDocument::parse(&keybindings)?,
    )?;
    if !changed && captured.is_some() {
        return Ok(captured);
    }
    let bytes = document.serialize()?;
    let sha1 = sha1_bytes(&bytes);
    let encoding = document.encoding.name().to_string();
    Ok(Some((bytes, sha1, encoding)))
}

pub(super) async fn materialize_pack_options_if_missing(
    metadata: &InstanceMetadata,
    state: &State,
) -> crate::Result<()> {
    let path = options_path(metadata, state);
    if options_target_exists(&path).await? {
        return Ok(());
    }
    for template_path in [YOSBR_OPTIONS_PATH, DEFAULT_OPTIONS_PATH] {
        if let Some((bytes, _, _)) =
            read_pack_options_template(metadata, state, template_path).await?
        {
            io::write(&path, bytes).await?;
            break;
        }
    }
    Ok(())
}

pub(in crate::api::instance) async fn detach_instance(
    metadata: &InstanceMetadata,
    state: &State,
) -> crate::Result<()> {
    let feature = SyncedOption::GameOptions.as_str();
    sqlx::query!(
        "
		DELETE FROM instance_sync_checkpoints
		WHERE instance_id = ? AND feature = ?
		",
        metadata.instance.id,
        feature,
    )
    .execute(&state.pool)
    .await?;
    Ok(())
}

/// Before replacing modpack files, saves `options.txt` and shares any settings the
/// player changed in Minecraft.
pub(in crate::api::instance) async fn prepare_instance_update_with_state(
    metadata: &InstanceMetadata,
    state: &State,
) -> crate::Result<()> {
    sqlx::query!(
        "DELETE FROM instance_game_option_update_state WHERE instance_id = ?",
        metadata.instance.id,
    )
    .execute(&state.pool)
    .await?;
    if sync_is_active_for_instance(metadata, state).await? {
        let _ = capture_instance_options(metadata, state, true).await?;
    }
    let path = options_path(metadata, state);
    let (had_file, bytes, sha1) = if path.exists() {
        let file_metadata = tokio::fs::symlink_metadata(&path)
            .await
            .map_err(|error| io::IOError::with_path(error, &path))?;
        if file_metadata.file_type().is_symlink() {
            return Err(input_error(
                "Refusing to prepare a symlinked options.txt for a modpack update",
            ));
        }
        if file_metadata.len() > MAX_OPTIONS_BYTES as u64 {
            return Err(input_error(
                "options.txt is too large to preserve for a modpack update",
            ));
        }
        let bytes = io::read(&path).await?;
        (true, Some(bytes.clone()), Some(sha1_bytes(&bytes)))
    } else {
        (false, None, None)
    };
    sqlx::query!(
        "
		INSERT INTO instance_game_option_update_state
			(instance_id, had_file, sha1, document)
		VALUES (?, ?, ?, ?)
		ON CONFLICT(instance_id) DO UPDATE SET
			had_file = excluded.had_file, sha1 = excluded.sha1,
			document = excluded.document
		",
        metadata.instance.id,
        had_file,
        sha1,
        bytes,
    )
    .execute(&state.pool)
    .await?;
    Ok(())
}

/// Keeps the new modpack's `options.txt` as its default, then reapplies the user's
/// shared settings.
pub async fn capture_pack_base(
    instance_id: &str,
    source: Option<GameOptionsPackSource>,
) -> crate::Result<()> {
    let state = State::get().await?;
    let _guard = state.lock_synced_options().await;
    let metadata = crate::state::get_instance(instance_id, &state.pool)
        .await?
        .ok_or_else(|| input_error("Unknown instance"))?;
    let path = options_path(&metadata, &state);
    let feature = SyncedOption::GameOptions.as_str();
    sqlx::query!(
        "
		DELETE FROM instance_sync_checkpoints
		WHERE instance_id = ? AND feature = ?
		",
        instance_id,
        feature,
    )
    .execute(&state.pool)
    .await?;

    if source.is_none() {
        let pending = sqlx::query!(
            r#"
			SELECT had_file AS "had_file!: bool", document
			FROM instance_game_option_update_state
			WHERE instance_id = ?
			"#,
            instance_id,
        )
        .fetch_optional(&state.pool)
        .await?;
        if let Some(pending) = pending {
            if pending.had_file {
                if let Some(document) = pending.document {
                    io::write(&path, document).await?;
                }
            } else if path.exists() {
                io::remove_file(&path).await?;
            }
        }
    }

    let template_path = source.and_then(GameOptionsPackSource::template_path);
    let captured =
        if let Some(template_path) = template_path {
            let template =
			read_pack_options_template(&metadata, &state, template_path)
			.await?
			.ok_or_else(|| {
				input_error(format!(
					"The modpack {template_path} defaults could not be materialized"
				))
			})?;
            if !options_target_exists(&path).await? {
                io::write(&path, template.0.clone()).await?;
            }
            Some(template)
        } else {
            read_pack_options_template(&metadata, &state, "options.txt").await?
        };
    if let Some((bytes, _, _)) =
        read_pack_options_template(&metadata, &state, "options.txt").await?
        && io::read(&path).await? != bytes
    {
        io::write(&path, bytes).await?;
    }
    let (document_bytes, sha1, encoding) = captured
        .map(|(document, sha1, encoding)| {
            (Some(document), Some(sha1), Some(encoding))
        })
        .unwrap_or((None, None, None));
    let source_name =
        source.map(GameOptionsPackSource::as_str).unwrap_or("none");
    let version_id = pack_version_id(&metadata);
    sqlx::query!(
        "
		INSERT INTO instance_game_option_pack_bases
			(instance_id, pack_version_id, source, sha1, encoding, document)
		VALUES (?, ?, ?, ?, ?, ?)
		ON CONFLICT(instance_id) DO UPDATE SET
			pack_version_id = excluded.pack_version_id,
			source = excluded.source, sha1 = excluded.sha1,
			encoding = excluded.encoding, document = excluded.document
		",
        instance_id,
        version_id,
        source_name,
        sha1,
        encoding,
        document_bytes,
    )
    .execute(&state.pool)
    .await?;
    sqlx::query!(
        "DELETE FROM instance_game_option_update_state WHERE instance_id = ?",
        instance_id,
    )
    .execute(&state.pool)
    .await?;

    if sync_is_active_for_instance(&metadata, &state).await?
        && let Err(error) =
            apply_shared_settings_to_instance(&metadata, &state, true).await
    {
        tracing::warn!(
            "Captured the modpack options.txt for {}, but shared settings could not be overlaid yet: {error}",
            metadata.instance.id
        );
    }
    Ok(())
}

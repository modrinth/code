//! # Minecraft Skins API
//!
//! ## Data Flow
//!
//! 1. Frontend calls `get_available_skins()` and `get_available_capes()`
//! 2. Backend gets the selected Minecraft account and a recent Mojang profile.
//!    If skins and capes load at the same time, they share the recent profile
//!    instead of sending the same request twice.
//! 3. The skin list is built from three places:
//!    - saved custom skins in the local app database
//!    - bundled Minecraft default skins
//!    - the active Mojang skin, if it is not already known locally
//! 4. Before changing a skin, the current Mojang-only skin is saved locally so
//!    switching to a default skin does not lose it.
//! 5. After a Mojang change, the returned profile is saved in memory when
//!    possible. If that response cannot be read, or a later step fails, the
//!    backend asks Mojang for the profile again.
//!
//! ## Ownership
//!
//! Mojang decides which skin and cape are currently equipped. The local database
//! stores saved custom skins and the app's selected default cape. A saved skin is
//! the same saved skin when its `texture_key` and `variant` match; changing its
//! cape updates that saved skin instead of creating another row.
//!
//! `cape_id = Some(_)` means a skin should apply that specific cape.
//! `cape_id = None` means the skin follows the app's default cape.
//!
//! ## Consistency
//!
//! A Mojang request and a SQLite write cannot be one all-or-nothing operation.
//! The backend handles this by saving skins that might be lost before changing
//! Mojang, saving uploaded skins with the texture key Mojang returns, and asking
//! Mojang for the latest profile again whenever the result is unclear.

use std::sync::Arc;

pub use bytes::Bytes;
use futures::{StreamExt, TryStreamExt, stream};
use serde::{Deserialize, Serialize};
use url::Url;
use uuid::Uuid;

pub use crate::state::MinecraftSkinVariant;
use crate::{
    ErrorKind, State,
    state::{
        MinecraftCharacterExpressionState, MinecraftProfile,
        minecraft_skins::{
            CustomMinecraftSkin, DefaultMinecraftCape, mojang_api,
        },
    },
};

use super::data::Credentials;

mod assets {
    mod default {
        mod default_skins;
        pub use default_skins::DEFAULT_SKINS;
    }
    pub use default::DEFAULT_SKINS;
}

mod png_util;

#[derive(Deserialize, Serialize, Debug)]
pub struct Cape {
    /// An identifier for this cape, potentially unique to the owning player.
    pub id: Uuid,
    /// The name of the cape.
    pub name: Arc<str>,
    /// The URL of the cape PNG texture.
    pub texture: Arc<Url>,
    /// Whether the cape is the default one, used when the currently selected cape does not
    /// override it.
    pub is_default: bool,
    /// Whether the cape is currently equipped in the Minecraft profile of its corresponding
    /// player.
    pub is_equipped: bool,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Skin {
    /// A key used to recognize this skin texture.
    pub texture_key: Arc<str>,
    /// The name of the skin, if available.
    pub name: Option<Arc<str>>,
    /// The variant of the skin model.
    pub variant: MinecraftSkinVariant,
    /// The UUID of the cape that this skin uses, if any.
    ///
    /// If `None`, this skin uses the app's default cape for this player.
    pub cape_id: Option<Uuid>,
    /// The URL of the skin PNG texture. Can also be a data URL.
    pub texture: Arc<Url>,
    /// The source of the skin, which represents how the app knows about it.
    pub source: SkinSource,
    /// Whether the skin is currently equipped in the Minecraft profile of its corresponding
    /// player.
    pub is_equipped: bool,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum SkinSource {
    /// A default Minecraft skin, which may be assigned to players at random by default.
    Default,
    /// A skin that is not the default, but is not a custom skin managed by our app either.
    CustomExternal,
    /// A custom skin we have set up in our app.
    Custom,
}

/// Represents either a URL or a blob for a Minecraft skin PNG texture.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum UrlOrBlob {
    Url(Url),
    Blob(Bytes),
}

/// Gets the capes for the selected Minecraft profile.
/// Only one cape can be equipped, and only one cape can be the app default.
#[tracing::instrument]
pub async fn get_available_capes() -> crate::Result<Vec<Cape>> {
    let state = State::get().await?;

    let selected_credentials = Credentials::get_default_credential(&state.pool)
        .await?
        .ok_or(ErrorKind::NoCredentialsError)?;

    let profile = selected_credentials
        .online_profile_fresh()
        .await
        .ok_or_else(|| ErrorKind::OnlineMinecraftProfileUnavailable {
            user_name: selected_credentials.offline_profile.name.clone(),
        })?;

    let default_cape_id = DefaultMinecraftCape::get(profile.id, &state.pool)
        .await?
        .map(|cape| cape.id);

    Ok(profile
        .capes
        .iter()
        .map(|cape| Cape {
            id: cape.id,
            name: Arc::clone(&cape.name),
            texture: Arc::clone(&cape.url),
            is_default: default_cape_id
                .is_some_and(|default_cape_id| default_cape_id == cape.id),
            is_equipped: cape.state
                == MinecraftCharacterExpressionState::Active,
        })
        .collect())
}

/// Gets the skins for the selected Minecraft profile.
/// Returns saved custom skins, bundled default skins, and the active Mojang skin
/// if it is not already represented by texture key and model variant.
/// Exactly one returned skin is marked as equipped.
#[tracing::instrument]
pub async fn get_available_skins() -> crate::Result<Vec<Skin>> {
    let state = State::get().await?;

    let selected_credentials = Credentials::get_default_credential(&state.pool)
        .await?
        .ok_or(ErrorKind::NoCredentialsError)?;

    let profile = selected_credentials
        .online_profile_fresh()
        .await
        .ok_or_else(|| ErrorKind::OnlineMinecraftProfileUnavailable {
            user_name: selected_credentials.offline_profile.name.clone(),
        })?;

    let current_skin = profile.current_skin()?;
    let current_cape_id = profile.current_cape().map(|cape| cape.id);

    let current_skin_texture_key = current_skin.texture_key();
    let mut found_equipped_skin = false;
    let mut available_skins = Vec::new();

    for custom_skin in CustomMinecraftSkin::get_all(profile.id, &state.pool)
        .await?
        .collect::<Vec<_>>()
        .await
    {
        let is_equipped = !found_equipped_skin
            && custom_skin.texture_key == *current_skin_texture_key
            && custom_skin.variant == current_skin.variant;

        found_equipped_skin |= is_equipped;

        available_skins.push(Skin {
            name: None,
            variant: custom_skin.variant,
            cape_id: custom_skin.cape_id,
            texture: png_util::blob_to_data_url(
                custom_skin.texture_blob(&state.pool).await?,
            )
            .or_else(|| {
                png_util::blob_to_data_url(include_bytes!(
                    "minecraft_skins/assets/default/MissingNo.png"
                ))
            })
            .unwrap(),
            source: SkinSource::Custom,
            is_equipped,
            texture_key: custom_skin.texture_key.into(),
        });
    }

    for default_skin in assets::DEFAULT_SKINS.iter() {
        let is_equipped = !found_equipped_skin
            && default_skin.texture_key == current_skin_texture_key
            && default_skin.variant == current_skin.variant;

        found_equipped_skin |= is_equipped;

        available_skins.push(Skin {
            texture_key: Arc::clone(&default_skin.texture_key),
            name: default_skin.name.as_ref().cloned(),
            variant: default_skin.variant,
            cape_id: None,
            texture: Arc::clone(&default_skin.texture),
            source: SkinSource::Default,
            is_equipped,
        });
    }

    // Keep the active Mojang skin visible even if the app has never saved it.
    if !found_equipped_skin {
        available_skins.push(Skin {
            texture_key: current_skin_texture_key,
            name: current_skin.name.as_deref().map(Arc::from),
            variant: current_skin.variant,
            cape_id: current_cape_id,
            texture: Arc::clone(&current_skin.url),
            source: SkinSource::CustomExternal,
            is_equipped: true,
        });
    }

    Ok(available_skins)
}

/// Adds a custom skin to the app database and equips it for the current profile.
#[tracing::instrument(skip(texture_blob))]
pub async fn add_and_equip_custom_skin(
    texture_blob: Bytes,
    variant: MinecraftSkinVariant,
    cape_override: Option<Cape>,
) -> crate::Result<()> {
    let (skin_width, skin_height) = png_util::dimensions(&texture_blob)?;
    if skin_width != 64 || ![32, 64].contains(&skin_height) {
        return Err(ErrorKind::InvalidSkinTexture)?;
    }

    let cape_override = cape_override.map(|cape| cape.id);
    let state = State::get().await?;

    let selected_credentials = Credentials::get_default_credential(&state.pool)
        .await?
        .ok_or(ErrorKind::NoCredentialsError)?;

    let previous_profile = selected_credentials
        .online_profile_fresh()
        .await
        .ok_or_else(|| ErrorKind::OnlineMinecraftProfileUnavailable {
            user_name: selected_credentials.offline_profile.name.clone(),
        })?;

    preserve_current_profile_skin(&state, &previous_profile).await?;

    // Mojang only gives us the new texture key after accepting the uploaded skin.
    // Use the profile from that response when possible, and fetch it only if that
    // response cannot be read.
    let profile = mojang_api::MinecraftSkinOperation::equip(
        &selected_credentials,
        stream::iter([Ok::<_, String>(Bytes::clone(&texture_blob))]),
        variant,
    )
    .await?;

    let profile = match profile {
        Some(profile) => profile,
        None => selected_credentials
            .refresh_online_profile()
            .await
            .ok_or_else(|| ErrorKind::OnlineMinecraftProfileUnavailable {
                user_name: selected_credentials.offline_profile.name.clone(),
            })?,
    };

    if let Err(error) = CustomMinecraftSkin::add(
        profile.id,
        &profile.current_skin()?.texture_key(),
        &texture_blob,
        variant,
        cape_override,
        &state.pool,
    )
    .await
    {
        refresh_profile_cache(&selected_credentials).await;
        return Err(error);
    }

    if let Err(error) =
        sync_cape(&state, &selected_credentials, &profile, cape_override).await
    {
        refresh_profile_cache(&selected_credentials).await;
        return Err(error);
    }

    Ok(())
}

/// Sets the default cape for the selected Minecraft profile. If `None`, the
/// default cape is removed.
///
/// Saved skins without their own cape use this cape. If the equipped skin uses
/// the default cape, the equipped cape is changed too. If there is no default
/// cape in that case, the equipped cape is removed.
#[tracing::instrument]
pub async fn set_default_cape(cape: Option<Cape>) -> crate::Result<()> {
    let state = State::get().await?;

    let selected_credentials = Credentials::get_default_credential(&state.pool)
        .await?
        .ok_or(ErrorKind::NoCredentialsError)?;

    let profile = selected_credentials
        .online_profile_fresh()
        .await
        .ok_or_else(|| ErrorKind::OnlineMinecraftProfileUnavailable {
            user_name: selected_credentials.offline_profile.name.clone(),
        })?;

    let current_skin_uses_default_cape =
        current_skin_follows_default_cape(&state, &profile).await?;

    if let Some(cape) = cape {
        // Change the equipped cape too when the current skin follows the default cape.
        if current_skin_uses_default_cape {
            if let Err(error) = mojang_api::MinecraftCapeOperation::equip(
                &selected_credentials,
                cape.id,
            )
            .await
            {
                refresh_profile_cache(&selected_credentials).await;
                return Err(error);
            }
        }

        if let Err(error) =
            DefaultMinecraftCape::set(profile.id, cape.id, &state.pool).await
        {
            refresh_profile_cache(&selected_credentials).await;
            return Err(error);
        }
    } else {
        if current_skin_uses_default_cape {
            if let Err(error) = mojang_api::MinecraftCapeOperation::unequip_any(
                &selected_credentials,
            )
            .await
            {
                refresh_profile_cache(&selected_credentials).await;
                return Err(error);
            }
        }

        if let Err(error) =
            DefaultMinecraftCape::remove(profile.id, &state.pool).await
        {
            refresh_profile_cache(&selected_credentials).await;
            return Err(error);
        }
    }

    Ok(())
}

/// Equips the given skin for the currently selected Minecraft profile. If the skin is already
/// equipped, it will be re-equipped.
///
/// This does not check whether a custom skin exists in the app database.
#[tracing::instrument]
pub async fn equip_skin(skin: Skin) -> crate::Result<()> {
    let state = State::get().await?;

    let selected_credentials = Credentials::get_default_credential(&state.pool)
        .await?
        .ok_or(ErrorKind::NoCredentialsError)?;

    let profile = selected_credentials
        .online_profile_fresh()
        .await
        .ok_or_else(|| ErrorKind::OnlineMinecraftProfileUnavailable {
            user_name: selected_credentials.offline_profile.name.clone(),
        })?;

    preserve_current_profile_skin(&state, &profile).await?;

    let _ = mojang_api::MinecraftSkinOperation::equip(
        &selected_credentials,
        png_util::url_to_data_stream(&skin.texture).await?,
        skin.variant,
    )
    .await?;

    if let Err(error) =
        sync_cape(&state, &selected_credentials, &profile, skin.cape_id).await
    {
        refresh_profile_cache(&selected_credentials).await;
        return Err(error);
    }

    Ok(())
}

/// Removes a custom skin from the app database.
///
/// The player will continue to be equipped with the same skin and cape as before, even if
/// the currently selected skin is the one being removed. This gives frontend code more options
/// to decide between unequipping strategies: falling back to other custom skin, to a default
/// skin, letting the user choose another skin, etc.
#[tracing::instrument]
pub async fn remove_custom_skin(skin: Skin) -> crate::Result<()> {
    let state = State::get().await?;

    let selected_credentials = Credentials::get_default_credential(&state.pool)
        .await?
        .ok_or(ErrorKind::NoCredentialsError)?;

    let profile = selected_credentials
        .online_profile_fresh()
        .await
        .ok_or_else(|| ErrorKind::OnlineMinecraftProfileUnavailable {
            user_name: selected_credentials.offline_profile.name.clone(),
        })?;

    CustomMinecraftSkin {
        texture_key: skin.texture_key.to_string(),
        variant: skin.variant,
        cape_id: skin.cape_id,
    }
    .remove(profile.id, &state.pool)
    .await?;

    Ok(())
}

/// Unequips the currently equipped skin for the currently selected Minecraft profile, resetting
/// it to one of the default skins. The cape will be set to the default cape, or unequipped if
/// no default cape is set.
#[tracing::instrument]
pub async fn unequip_skin() -> crate::Result<()> {
    let state = State::get().await?;

    let selected_credentials = Credentials::get_default_credential(&state.pool)
        .await?
        .ok_or(ErrorKind::NoCredentialsError)?;

    let profile = selected_credentials
        .online_profile_fresh()
        .await
        .ok_or_else(|| ErrorKind::OnlineMinecraftProfileUnavailable {
            user_name: selected_credentials.offline_profile.name.clone(),
        })?;

    preserve_current_profile_skin(&state, &profile).await?;

    mojang_api::MinecraftSkinOperation::unequip_any(&selected_credentials)
        .await?;

    if let Err(error) =
        sync_cape(&state, &selected_credentials, &profile, None).await
    {
        refresh_profile_cache(&selected_credentials).await;
        return Err(error);
    }

    Ok(())
}

/// Normalizes the texture of a Minecraft skin to the modern 64x64 format, handling
/// legacy 64x32 skins as the vanilla game client does. This function prioritizes
/// PNG encoding speed over compression density, so the resulting textures are better
/// suited for display purposes, not persistent storage or transmission.
///
/// The normalized texture is returned as PNG bytes.
#[tracing::instrument]
pub async fn normalize_skin_texture(
    texture: &UrlOrBlob,
) -> crate::Result<Bytes> {
    png_util::normalize_skin_texture(texture).await
}

/// Reads and validates a skin texture file from the given path.
/// Returns the file content as bytes if it's a valid skin texture (PNG with 64x64 or 64x32 dimensions).
#[tracing::instrument]
pub async fn get_dragged_skin_data(
    path: &std::path::Path,
) -> crate::Result<Bytes> {
    if let Some(extension) = path.extension() {
        if extension.to_string_lossy().to_lowercase() != "png" {
            return Err(ErrorKind::InvalidSkinTexture.into());
        }
    } else {
        return Err(ErrorKind::InvalidSkinTexture.into());
    }

    tracing::debug!("Reading file: {:?}", path);

    if !path.exists() {
        tracing::error!("File does not exist: {:?}", path);
        return Err(ErrorKind::InvalidSkinTexture.into());
    }

    let data = match tokio::fs::read(path).await {
        Ok(data) => {
            tracing::debug!(
                "File read successfully, size: {} bytes",
                data.len()
            );
            data
        }
        Err(err) => {
            tracing::error!("Failed to read file: {}", err);
            return Err(err.into());
        }
    };

    let url_or_blob = UrlOrBlob::Blob(data.clone().into());

    match normalize_skin_texture(&url_or_blob).await {
        Ok(_) => Ok(data.into()),
        Err(err) => {
            tracing::error!("Failed to normalize skin texture: {}", err);
            Err(ErrorKind::InvalidSkinTexture.into())
        }
    }
}

async fn preserve_current_profile_skin(
    state: &State,
    profile: &MinecraftProfile,
) -> crate::Result<()> {
    let current_skin = profile.current_skin()?;
    let current_skin_texture_key = current_skin.texture_key();

    if assets::DEFAULT_SKINS.iter().any(|default_skin| {
        default_skin.texture_key == current_skin_texture_key
            && default_skin.variant == current_skin.variant
    }) {
        return Ok(());
    }

    if CustomMinecraftSkin::get_by_texture_and_variant(
        profile.id,
        &current_skin_texture_key,
        current_skin.variant,
        &state.pool,
    )
    .await?
    .is_some()
    {
        return Ok(());
    }

    let current_cape_id = profile.current_cape().map(|cape| cape.id);

    let texture = png_util::url_to_data_stream(&current_skin.url)
        .await?
        .try_fold(Vec::new(), |mut texture, chunk| async move {
            texture.extend_from_slice(&chunk);
            Ok(texture)
        })
        .await?;

    CustomMinecraftSkin::add(
        profile.id,
        &current_skin_texture_key,
        &texture,
        current_skin.variant,
        current_cape_id,
        &state.pool,
    )
    .await?;

    Ok(())
}

async fn refresh_profile_cache(selected_credentials: &Credentials) {
    let _ = selected_credentials.refresh_online_profile().await;
}

async fn current_skin_follows_default_cape(
    state: &State,
    profile: &MinecraftProfile,
) -> crate::Result<bool> {
    let current_skin = profile.current_skin()?;
    let current_skin_texture_key = current_skin.texture_key();

    if assets::DEFAULT_SKINS.iter().any(|default_skin| {
        default_skin.texture_key == current_skin_texture_key
            && default_skin.variant == current_skin.variant
    }) {
        return Ok(true);
    }

    Ok(CustomMinecraftSkin::get_by_texture_and_variant(
        profile.id,
        &current_skin_texture_key,
        current_skin.variant,
        &state.pool,
    )
    .await?
    .is_some_and(|skin| skin.cape_id.is_none()))
}

/// Sets the equipped cape to the selected cape, the app default cape, or no cape.
async fn sync_cape(
    state: &State,
    selected_credentials: &Credentials,
    profile: &MinecraftProfile,
    cape_override: Option<Uuid>,
) -> crate::Result<()> {
    let current_cape_id = profile.current_cape().map(|cape| cape.id);
    let target_cape_id = match cape_override {
        Some(cape_id) => Some(cape_id),
        None => DefaultMinecraftCape::get(profile.id, &state.pool)
            .await?
            .map(|cape| cape.id),
    };

    if current_cape_id != target_cape_id {
        match target_cape_id {
            Some(cape_id) => {
                mojang_api::MinecraftCapeOperation::equip(
                    selected_credentials,
                    cape_id,
                )
                .await?
            }
            None => {
                mojang_api::MinecraftCapeOperation::unequip_any(
                    selected_credentials,
                )
                .await?
            }
        }
    }

    Ok(())
}

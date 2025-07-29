//! Theseus skin management interface

use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

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
    /// An opaque identifier for the skin texture, which can be used to identify it.
    pub texture_key: Arc<str>,
    /// The name of the skin, if available.
    pub name: Option<Arc<str>>,
    /// The variant of the skin model.
    pub variant: MinecraftSkinVariant,
    /// The UUID of the cape that this skin uses, if any.
    ///
    /// If `None`, the skin does not have an explicit cape set, and the default cape for
    /// this player, if any, should be used.
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

/// Retrieves the available capes for the currently selected Minecraft profile. At most one cape
/// can be equipped at a time. Also, at most one cape can be set as the default cape.
#[tracing::instrument]
pub async fn get_available_capes() -> crate::Result<Vec<Cape>> {
    let state = State::get().await?;

    let selected_credentials = Credentials::get_default_credential(&state.pool)
        .await?
        .ok_or(ErrorKind::NoCredentialsError)?;

    let profile =
        selected_credentials.online_profile().await.ok_or_else(|| {
            ErrorKind::OnlineMinecraftProfileUnavailable {
                user_name: selected_credentials.offline_profile.name.clone(),
            }
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

/// Retrieves the available skins for the currently selected Minecraft profile. At the moment,
/// this includes custom skins stored in the app database, default Mojang skins, and the currently
/// equipped skin, if different from the previous skins. Exactly one of the returned skins is
/// marked as equipped.
#[tracing::instrument]
pub async fn get_available_skins() -> crate::Result<Vec<Skin>> {
    let state = State::get().await?;

    let selected_credentials = Credentials::get_default_credential(&state.pool)
        .await?
        .ok_or(ErrorKind::NoCredentialsError)?;

    let profile =
        selected_credentials.online_profile().await.ok_or_else(|| {
            ErrorKind::OnlineMinecraftProfileUnavailable {
                user_name: selected_credentials.offline_profile.name.clone(),
            }
        })?;

    let current_skin = profile.current_skin()?;
    let current_cape_id = profile.current_cape().map(|cape| cape.id);
    let default_cape_id = DefaultMinecraftCape::get(profile.id, &state.pool)
        .await?
        .map(|cape| cape.id);

    // Keep track of whether we have found the currently equipped skin, to potentially avoid marking
    // several skins as equipped, and know if the equipped skin was found (see below)
    let found_equipped_skin = Arc::new(AtomicBool::new(false));

    let custom_skins = CustomMinecraftSkin::get_all(profile.id, &state.pool)
        .await?
        .then(|custom_skin| {
            let found_equipped_skin = Arc::clone(&found_equipped_skin);
            let state = Arc::clone(&state);
            async move {
                // Several custom skins may reuse the same texture for different cape or skin model
                // variations, so check all attributes for correctness
                let is_equipped = !found_equipped_skin.load(Ordering::Acquire)
                    && custom_skin.texture_key == *current_skin.texture_key()
                    && custom_skin.variant == current_skin.variant
                    && custom_skin.cape_id
                        == if custom_skin.cape_id.is_some() {
                            current_cape_id
                        } else {
                            default_cape_id
                        };

                found_equipped_skin.fetch_or(is_equipped, Ordering::AcqRel);

                Ok::<_, crate::Error>(Skin {
                    name: None,
                    variant: custom_skin.variant,
                    cape_id: custom_skin.cape_id,
                    texture: png_util::blob_to_data_url(
                        custom_skin.texture_blob(&state.pool).await?,
                    )
                    .or_else(|| {
                        // Fall back to a placeholder texture if the DB somehow contains corrupt data
                        png_util::blob_to_data_url(include_bytes!(
                            "minecraft_skins/assets/default/MissingNo.png"
                        ))
                    })
                    .unwrap(),
                    source: SkinSource::Custom,
                    is_equipped,
                    texture_key: custom_skin.texture_key.into(),
                })
            }
        });

    let default_skins =
        stream::iter(assets::DEFAULT_SKINS.iter().map(|default_skin| {
            let is_equipped = !found_equipped_skin.load(Ordering::Acquire)
                && default_skin.texture_key == current_skin.texture_key()
                && default_skin.variant == current_skin.variant;

            found_equipped_skin.fetch_or(is_equipped, Ordering::AcqRel);

            Ok::<_, crate::Error>(Skin {
                texture_key: Arc::clone(&default_skin.texture_key),
                name: default_skin.name.as_ref().cloned(),
                variant: default_skin.variant,
                cape_id: None,
                texture: Arc::clone(&default_skin.texture),
                source: SkinSource::Default,
                is_equipped,
            })
        }));

    let mut available_skins = custom_skins
        .chain(default_skins)
        .try_collect::<Vec<_>>()
        .await?;

    // If the currently equipped skin does not match any of the skins we know about,
    // add it to the list of available skins as a custom external skin, set by an
    // external service (e.g., the Minecraft launcher or website). This way we guarantee
    // that the currently equipped skin is always returned as available
    if !found_equipped_skin.load(Ordering::Acquire) {
        available_skins.push(Skin {
            texture_key: current_skin.texture_key(),
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

/// Adds a custom skin to the app database and equips it for the currently selected
/// Minecraft profile.
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

    // We have to equip the skin first, as it's the Mojang API backend who knows
    // how to compute the texture key we require, which we can then read from the
    // updated player profile
    mojang_api::MinecraftSkinOperation::equip(
        &selected_credentials,
        stream::iter([Ok::<_, String>(Bytes::clone(&texture_blob))]),
        variant,
    )
    .await?;

    let profile =
        selected_credentials.online_profile().await.ok_or_else(|| {
            ErrorKind::OnlineMinecraftProfileUnavailable {
                user_name: selected_credentials.offline_profile.name.clone(),
            }
        })?;

    sync_cape(&state, &selected_credentials, &profile, cape_override).await?;

    CustomMinecraftSkin::add(
        profile.id,
        &profile.current_skin()?.texture_key(),
        &texture_blob,
        variant,
        cape_override,
        &state.pool,
    )
    .await?;

    Ok(())
}

/// Sets the default cape for the currently selected Minecraft profile. If `None`,
/// the default cape will be removed.
///
/// This cape will be used by any custom skin that does not have a cape override
/// set. If the currently equipped skin does not have a cape override set, the equipped
/// cape will also be changed to the new default cape. When neither the equipped skin
/// defines a cape override nor the default cape is set, the player will have no
/// cape equipped.
#[tracing::instrument]
pub async fn set_default_cape(cape: Option<Cape>) -> crate::Result<()> {
    let state = State::get().await?;

    let selected_credentials = Credentials::get_default_credential(&state.pool)
        .await?
        .ok_or(ErrorKind::NoCredentialsError)?;

    let profile =
        selected_credentials.online_profile().await.ok_or_else(|| {
            ErrorKind::OnlineMinecraftProfileUnavailable {
                user_name: selected_credentials.offline_profile.name.clone(),
            }
        })?;
    let current_skin = get_available_skins()
        .await?
        .into_iter()
        .find(|skin| skin.is_equipped)
        .unwrap();

    if let Some(cape) = cape {
        // Synchronize the equipped cape with the new default cape, if the current skin uses
        // the default cape
        if current_skin.cape_id.is_none() {
            mojang_api::MinecraftCapeOperation::equip(
                &selected_credentials,
                cape.id,
            )
            .await?;
        }

        DefaultMinecraftCape::set(profile.id, cape.id, &state.pool).await?;
    } else {
        if current_skin.cape_id.is_none() {
            mojang_api::MinecraftCapeOperation::unequip_any(
                &selected_credentials,
            )
            .await?;
        }

        DefaultMinecraftCape::remove(profile.id, &state.pool).await?;
    }

    Ok(())
}

/// Equips the given skin for the currently selected Minecraft profile. If the skin is already
/// equipped, it will be re-equipped.
///
/// This function does not check that the passed skin, if custom, exists in the app database,
/// giving the caller complete freedom to equip any skin at any time.
#[tracing::instrument]
pub async fn equip_skin(skin: Skin) -> crate::Result<()> {
    let state = State::get().await?;

    let selected_credentials = Credentials::get_default_credential(&state.pool)
        .await?
        .ok_or(ErrorKind::NoCredentialsError)?;

    let profile =
        selected_credentials.online_profile().await.ok_or_else(|| {
            ErrorKind::OnlineMinecraftProfileUnavailable {
                user_name: selected_credentials.offline_profile.name.clone(),
            }
        })?;

    mojang_api::MinecraftSkinOperation::equip(
        &selected_credentials,
        png_util::url_to_data_stream(&skin.texture).await?,
        skin.variant,
    )
    .await?;

    sync_cape(&state, &selected_credentials, &profile, skin.cape_id).await?;

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

    CustomMinecraftSkin {
        texture_key: skin.texture_key.to_string(),
        variant: skin.variant,
        cape_id: skin.cape_id,
    }
    .remove(
        selected_credentials.maybe_online_profile().await.id,
        &state.pool,
    )
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

    let profile =
        selected_credentials.online_profile().await.ok_or_else(|| {
            ErrorKind::OnlineMinecraftProfileUnavailable {
                user_name: selected_credentials.offline_profile.name.clone(),
            }
        })?;

    mojang_api::MinecraftSkinOperation::unequip_any(&selected_credentials)
        .await?;

    sync_cape(&state, &selected_credentials, &profile, None).await?;

    Ok(())
}

/// Normalizes the texture of a Minecraft skin to the modern 64x64 format, handling
/// legacy 64x32 skins as the vanilla game client does. This function prioritizes
/// PNG encoding speed over compression density, so the resulting textures are better
/// suited for display purposes, not persistent storage or transmission.
///
/// The normalized, processed is returned texture as a byte array in PNG format.
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

/// Synchronizes the equipped cape with the selected cape if necessary, taking into
/// account the currently equipped cape, the default cape for the player, and if a
/// cape override is provided.
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

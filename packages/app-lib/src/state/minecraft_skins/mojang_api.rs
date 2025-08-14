use std::{error::Error, sync::Arc, time::Instant};

use bytes::Bytes;
use futures::TryStream;
use reqwest::{Body, multipart::Part};
use serde_json::json;
use uuid::Uuid;

use super::MinecraftSkinVariant;
use crate::{
    ErrorKind,
    data::Credentials,
    state::{MinecraftProfile, PROFILE_CACHE, ProfileCacheEntry},
    util::fetch::REQWEST_CLIENT,
};

/// Provides operations for interacting with capes on a Minecraft player profile.
pub struct MinecraftCapeOperation;

impl MinecraftCapeOperation {
    pub async fn equip(
        credentials: &Credentials,
        cape_id: Uuid,
    ) -> crate::Result<()> {
        update_profile_cache_from_response(
            REQWEST_CLIENT
                .put("https://api.minecraftservices.com/minecraft/profile/capes/active")
                .header("Content-Type", "application/json; charset=utf-8")
                .header("Accept", "application/json")
                .bearer_auth(&credentials.access_token)
                .json(&json!({
                    "capeId": cape_id.hyphenated(),
                }))
                .send()
                .await
                .and_then(|response| response.error_for_status())?
        )
        .await;

        Ok(())
    }

    pub async fn unequip_any(credentials: &Credentials) -> crate::Result<()> {
        update_profile_cache_from_response(
            REQWEST_CLIENT
                .delete("https://api.minecraftservices.com/minecraft/profile/capes/active")
                .header("Accept", "application/json")
                .bearer_auth(&credentials.access_token)
                .send()
                .await
                .and_then(|response| response.error_for_status())?
        )
        .await;

        Ok(())
    }
}

/// Provides operations for interacting with skins on a Minecraft player profile.
pub struct MinecraftSkinOperation;

impl MinecraftSkinOperation {
    pub async fn equip<TextureStream>(
        credentials: &Credentials,
        texture: TextureStream,
        variant: MinecraftSkinVariant,
    ) -> crate::Result<()>
    where
        TextureStream: TryStream + Send + 'static,
        TextureStream::Error: Into<Box<dyn Error + Send + Sync>>,
        Bytes: From<TextureStream::Ok>,
    {
        let form = reqwest::multipart::Form::new()
            .text(
                "variant",
                match variant {
                    MinecraftSkinVariant::Slim => "slim",
                    MinecraftSkinVariant::Classic => "classic",
                    _ => {
                        return Err(ErrorKind::OtherError(
                            "Cannot equip skin of unknown model variant".into(),
                        )
                        .into());
                    }
                },
            )
            .part(
                "file",
                Part::stream(Body::wrap_stream(texture))
                    .mime_str("image/png")?
                    .file_name("skin.png"),
            );

        update_profile_cache_from_response(
            REQWEST_CLIENT
                .post(
                    "https://api.minecraftservices.com/minecraft/profile/skins",
                )
                .header("Accept", "application/json")
                .bearer_auth(&credentials.access_token)
                .multipart(form)
                .send()
                .await
                .and_then(|response| response.error_for_status())?,
        )
        .await;

        Ok(())
    }

    pub async fn unequip_any(credentials: &Credentials) -> crate::Result<()> {
        update_profile_cache_from_response(
            REQWEST_CLIENT
                .delete("https://api.minecraftservices.com/minecraft/profile/skins/active")
                .header("Accept", "application/json")
                .bearer_auth(&credentials.access_token)
                .send()
                .await
                .and_then(|response| response.error_for_status())?
        )
        .await;

        Ok(())
    }
}

async fn update_profile_cache_from_response(response: reqwest::Response) {
    let Some(mut profile) = response.json::<MinecraftProfile>().await.ok()
    else {
        tracing::warn!(
            "Failed to parse player profile from skin or cape operation response, not updating profile cache"
        );
        return;
    };

    profile.fetch_time = Some(Instant::now());

    PROFILE_CACHE
        .lock()
        .await
        .insert(profile.id, ProfileCacheEntry::Hit(Arc::new(profile)));
}

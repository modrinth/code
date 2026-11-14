use std::path::PathBuf;
use std::sync::{Arc, LazyLock};

use base64::Engine;
use bytes::Bytes;
use futures::TryStreamExt;
use modrinth_skin_rendering::{
    ArmModel, Camera, Lighting, Mat4, Model, ModelConventions, Player,
    RenderOptions, Scene, SkinTexture, Texture, Vec3,
    cache::{CacheKey, DirCache},
    ears::{Ears, EarsQuad, TexturePair},
    render, render_face,
};
use serde::Serialize;
use tokio::sync::{OnceCell, Semaphore};
use url::Url;

use super::{MinecraftSkinVariant, Skin, UrlOrBlob, png_util};
use crate::State;

const CACHE_FOLDER_NAME: &str = "skin-renders";
const CACHE_LIMIT_BYTES: u64 = 64 * 1024 * 1024;
const PREVIEW_WIDTH: u32 = 360;
const PREVIEW_HEIGHT: u32 = 504;
const HEAD_SIZE: u32 = 64;

/// Part of the cache key. Bump when preview framing or lighting changes.
const PREVIEW_STYLE: &str = "preview-1";
const HEAD_STYLE: &str = "head-1";

static RENDER_WORKERS: Semaphore = Semaphore::const_new(2);
static CACHE_PRUNED: OnceCell<()> = OnceCell::const_new();

static CLASSIC_MODEL: LazyLock<Model> = LazyLock::new(|| {
    Model::from_gltf(include_bytes!(
        "../../../../assets/models/classic-player.gltf"
    ))
    .expect("bundled classic player model is valid")
});
static SLIM_MODEL: LazyLock<Model> = LazyLock::new(|| {
    Model::from_gltf(include_bytes!(
        "../../../../assets/models/slim-player.gltf"
    ))
    .expect("bundled slim player model is valid")
});

#[derive(Serialize, Debug, Clone)]
pub struct SkinEarsTexture {
    pub width: u32,
    pub height: u32,
    /// Base64.
    pub rgba: String,
}

#[derive(Serialize, Debug, Clone)]
pub struct SkinEarsTexturePair {
    pub base: SkinEarsTexture,
    pub emissive: Option<SkinEarsTexture>,
}

#[derive(Serialize, Debug, Clone)]
pub struct SkinEars {
    pub skin: SkinEarsTexturePair,
    pub displaced_skin: Option<SkinEarsTexturePair>,
    pub wing: Option<SkinEarsTexturePair>,
    pub cape: Option<SkinEarsTexturePair>,
    pub quads: Vec<EarsQuad>,
}

impl From<&Texture> for SkinEarsTexture {
    fn from(texture: &Texture) -> Self {
        Self {
            width: texture.width(),
            height: texture.height(),
            rgba: base64::engine::general_purpose::STANDARD
                .encode(texture.as_rgba8()),
        }
    }
}

impl From<&TexturePair> for SkinEarsTexturePair {
    fn from(pair: &TexturePair) -> Self {
        Self {
            base: (&pair.base).into(),
            emissive: pair.emissive.as_ref().map(Into::into),
        }
    }
}

/// Returns the path of the cached preview image, rendering it first if needed.
#[tracing::instrument(skip(skin), fields(texture_key = %skin.texture_key))]
pub async fn render_skin_preview(
    skin: &Skin,
    cape_texture: Option<&Url>,
) -> crate::Result<PathBuf> {
    let key = CacheKey::builder()
        .field(PREVIEW_STYLE)
        .field(skin.texture_key.as_bytes())
        .field(format!("{:?}", skin.variant))
        .field(cape_texture.map_or("", Url::as_str))
        .finish();

    let cache = cache().await?;
    if let Some(path) = cache.lookup(&key) {
        return Ok(path);
    }

    let texture = fetch_texture(&skin.texture).await?;
    let cape = match cape_texture {
        Some(url) => Some(fetch_texture(url).await?),
        None => None,
    };
    let variant = skin.variant;

    render_to_cache(cache, key, move || {
        let skin = SkinTexture::from_png(&texture)?;
        let cape = cape.map(|cape| Texture::from_png(&cape)).transpose()?;
        let model = match variant {
            MinecraftSkinVariant::Classic => &*CLASSIC_MODEL,
            MinecraftSkinVariant::Slim => &*SLIM_MODEL,
            MinecraftSkinVariant::Unknown => match skin.detect_arm_model() {
                ArmModel::Classic => &*CLASSIC_MODEL,
                ArmModel::Slim => &*SLIM_MODEL,
            },
        };
        Ok(render_preview(model, &skin, cape.as_ref()))
    })
    .await
}

/// Returns the path of the cached head image, rendering it first if needed.
#[tracing::instrument(skip(skin), fields(texture_key = %skin.texture_key))]
pub async fn render_skin_head(skin: &Skin) -> crate::Result<PathBuf> {
    let key = CacheKey::builder()
        .field(HEAD_STYLE)
        .field(skin.texture_key.as_bytes())
        .finish();

    let cache = cache().await?;
    if let Some(path) = cache.lookup(&key) {
        return Ok(path);
    }

    let texture = fetch_texture(&skin.texture).await?;
    render_to_cache(cache, key, move || {
        Ok(render_face(&SkinTexture::from_png(&texture)?, HEAD_SIZE))
    })
    .await
}

/// `None` if the texture has no Ears data.
#[tracing::instrument(skip(texture))]
pub async fn get_skin_ears(
    texture: &UrlOrBlob,
    slim: bool,
) -> crate::Result<Option<SkinEars>> {
    let texture = match texture {
        UrlOrBlob::Url(url) => fetch_texture(url).await?,
        UrlOrBlob::Blob(blob) => Bytes::clone(blob),
    };

    tokio::task::spawn_blocking(move || {
        let skin = SkinTexture::from_png(&texture)?;
        Ok(Ears::build(&skin, slim).map(|ears| SkinEars {
            skin: (&ears.skin).into(),
            displaced_skin: ears.displaced_skin.as_ref().map(Into::into),
            wing: ears.wing.as_ref().map(Into::into),
            cape: ears.cape.as_ref().map(Into::into),
            quads: ears.quads,
        }))
    })
    .await?
}

fn render_preview(
    model: &Model,
    skin: &SkinTexture,
    cape: Option<&Texture>,
) -> Texture {
    let conventions = ModelConventions::default();
    let placement =
        Mat4::translation(0.0, 0.3, 1.95) * Mat4::scale(0.8, 0.8, 0.8);
    let head = placement.transform_point(
        model
            .node_position(&conventions.head)
            .unwrap_or(Vec3::new(0.0, 1.5, 0.0)),
    );

    let mut scene =
        Scene::build(model, &conventions, &Player::new(skin).with_cape(cape));
    scene.transform(&placement);

    render(
        &scene,
        &RenderOptions {
            width: PREVIEW_WIDTH,
            height: PREVIEW_HEIGHT,
            supersampling: 3,
            camera: Camera {
                position: Vec3::new(-1.3, 1.0, 6.3),
                target: head - Vec3::new(0.0, 0.3, 0.0),
                up: Vec3::Y,
                fov_y_degrees: 20.0,
                near: 0.4,
                far: 1000.0,
            },
            lighting: Lighting {
                ambient: 2.0 / std::f32::consts::PI,
                directional: 1.2 / std::f32::consts::PI,
                direction: Vec3::new(2.0, 4.0, 3.0),
            },
        },
    )
}

async fn cache() -> crate::Result<Arc<DirCache>> {
    let state = State::get().await?;
    let cache = Arc::new(DirCache::new(
        state.directories.caches_dir().join(CACHE_FOLDER_NAME),
        "png",
    ));

    CACHE_PRUNED
        .get_or_init(|| async {
            let cache = Arc::clone(&cache);
            let pruned = tokio::task::spawn_blocking(move || {
                cache.prune(CACHE_LIMIT_BYTES)
            })
            .await;
            if let Ok(Err(error)) = pruned {
                tracing::warn!("Failed to prune skin render cache: {error}");
            }
        })
        .await;

    Ok(cache)
}

async fn render_to_cache(
    cache: Arc<DirCache>,
    key: CacheKey,
    draw: impl FnOnce() -> crate::Result<Texture> + Send + 'static,
) -> crate::Result<PathBuf> {
    let permit = RENDER_WORKERS.acquire().await?;
    tokio::task::spawn_blocking(move || {
        let _permit = permit;
        if let Some(path) = cache.lookup(&key) {
            return Ok(path);
        }
        Ok(cache.store(&key, &draw()?.to_png()?)?)
    })
    .await?
}

async fn fetch_texture(url: &Url) -> crate::Result<Bytes> {
    let texture = png_util::url_to_data_stream(url)
        .await?
        .try_fold(Vec::new(), |mut texture, chunk| async move {
            texture.extend_from_slice(&chunk);
            Ok(texture)
        })
        .await?;
    Ok(texture.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_skin(name: &str) -> SkinTexture {
        let png: &[u8] = match name {
            "legacy" => include_bytes!("assets/test/legacy.png"),
            "notch" => include_bytes!("assets/test/notch.png"),
            _ => include_bytes!("assets/test/transparent.png"),
        };
        SkinTexture::from_png(png).unwrap()
    }

    fn opaque_pixels(image: &Texture) -> usize {
        image
            .pixels()
            .iter()
            .filter(|pixel| pixel[3] == 255)
            .count()
    }

    #[test]
    fn bundled_models_have_every_body_part_the_renderer_looks_for() {
        let conventions = ModelConventions::default();
        for model in [&*CLASSIC_MODEL, &*SLIM_MODEL] {
            for part in modrinth_skin_rendering::BodyPart::ALL {
                assert!(
                    model.base_mesh_bounds(part, &conventions).is_some(),
                    "missing {part:?}"
                );
            }
        }

        let arm_width = |model: &Model| {
            let bounds = model
                .base_mesh_bounds(
                    modrinth_skin_rendering::BodyPart::RightArm,
                    &conventions,
                )
                .unwrap();
            ((bounds.max.x - bounds.min.x) * 16.0).round()
        };
        assert_eq!(arm_width(&CLASSIC_MODEL), 4.0);
        assert_eq!(arm_width(&SLIM_MODEL), 3.0);
    }

    #[test]
    fn previews_show_the_player_centered_on_a_transparent_background() {
        for name in ["legacy", "notch", "transparent"] {
            let image = render_preview(&CLASSIC_MODEL, &test_skin(name), None);
            assert_eq!(
                (image.width(), image.height()),
                (PREVIEW_WIDTH, PREVIEW_HEIGHT)
            );
            assert_eq!(image.get(0, 0)[3], 0, "{name}");
            assert_eq!(image.get(PREVIEW_WIDTH - 1, 0)[3], 0, "{name}");
            assert_eq!(image.get(180, 250)[3], 255, "{name}");

            let coverage = opaque_pixels(&image) as f32
                / (PREVIEW_WIDTH * PREVIEW_HEIGHT) as f32;
            assert!((0.3..0.7).contains(&coverage), "{name}: {coverage}");
        }
    }

    #[test]
    fn capes_add_to_the_preview_only_when_given() {
        let skin = test_skin("notch");
        let cape =
            Texture::from_pixels(64, 32, vec![[200, 30, 30, 255]; 64 * 32])
                .unwrap();
        let without = render_preview(&CLASSIC_MODEL, &skin, None);
        let with = render_preview(&CLASSIC_MODEL, &skin, Some(&cape));
        assert!(opaque_pixels(&with) > opaque_pixels(&without));
    }
}

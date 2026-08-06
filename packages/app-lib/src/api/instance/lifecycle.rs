use crate::event::InstancePayloadType;
use crate::event::emit::emit_instance;
use crate::state::instances::adapters::sqlite::instance_rows;
use crate::state::{
    CreateInstance, EditInstance, InstanceIconBackground, InstanceIconRecipe,
    InstanceLink, InstanceMetadata, ModLoader, State,
};
use crate::util::io;
use bytes::Bytes;
use image::imageops::FilterType;
use image::{DynamicImage, ImageFormat, ImageReader, Rgba, RgbaImage};
use std::io::Cursor;
use std::path::Path;

const GENERATED_ICON_SIZE: u32 = 256;
const MAX_ICON_RECIPE_ID_LENGTH: usize = 64;
const MAX_SYMBOL_BYTES: usize = 4 * 1024 * 1024;
const MAX_SYMBOL_DIMENSION: u32 = 4096;

#[tracing::instrument]
#[allow(clippy::too_many_arguments)]
pub(crate) async fn create(
    name: String,
    game_version: String,
    modloader: ModLoader,
    loader_version: Option<String>,
    icon_path: Option<String>,
    link: InstanceLink,
) -> crate::Result<InstanceMetadata> {
    let state = State::get().await?;
    let instance = crate::state::create_instance(
        CreateInstance {
            name,
            path: None,
            game_version,
            loader: modloader,
            loader_version,
            icon_path,
            link,
        },
        &state,
    )
    .await?;

    let result = async {
        emit_instance(&instance.id, InstancePayloadType::Created).await?;

        crate::state::get_instance(&instance.id, &state.pool)
            .await?
            .ok_or_else(|| {
                crate::ErrorKind::InputError(
                    "Created instance could not be loaded".to_string(),
                )
                .into()
            })
    }
    .await;

    if result.is_err() {
        let _ = crate::state::remove_instance(&instance.id, &state).await;
    } else if let Err(error) =
        crate::onboarding_checklist::mark_created_instance().await
    {
        tracing::warn!(
            "Failed to mark instance creation in onboarding checklist: {error}"
        );
    }

    result
}

pub async fn edit(
    instance_id: &str,
    patch: EditInstance,
) -> crate::Result<InstanceMetadata> {
    let state = State::get().await?;
    crate::state::edit_instance(instance_id, patch, &state.pool).await?;

    let instance = crate::state::get_instance(instance_id, &state.pool)
        .await?
        .ok_or_else(|| {
            crate::ErrorKind::InputError("Unknown instance".to_string())
                .as_error()
        })?;

    emit_instance(&instance.instance.id, InstancePayloadType::Edited).await?;

    Ok(instance)
}

pub async fn edit_icon(
    instance_id: &str,
    icon_path: Option<&Path>,
) -> crate::Result<()> {
    let state = State::get().await?;
    let instance =
        instance_rows::get_instance_display_info(instance_id, &state.pool)
            .await?
            .ok_or_else(|| {
                crate::ErrorKind::InputError("Unknown instance".to_string())
            })?;
    let icon_path = if let Some(icon) = icon_path {
        let bytes = io::read(icon).await?;
        let file = crate::util::fetch::write_cached_icon(
            &icon.to_string_lossy(),
            &state.directories.caches_dir(),
            bytes::Bytes::from(bytes),
            &state.io_semaphore,
        )
        .await?;
        Some(file.to_string_lossy().to_string())
    } else {
        None
    };

    crate::state::edit_instance(
        instance_id,
        EditInstance {
            icon_path: Some(icon_path.clone()),
            icon_recipe: Some(None),
            ..EditInstance::default()
        },
        &state.pool,
    )
    .await?;

    if let Err(error) = super::shared::sync_shared_instance_icon(
        instance_id,
        icon_path.as_deref(),
        &state,
    )
    .await
    {
        tracing::warn!(
            instance_id,
            error = %error,
            "Failed to sync shared instance icon"
        );
    }

    emit_instance(&instance.id, InstancePayloadType::Edited).await?;

    Ok(())
}

pub async fn edit_generated_icon(
    instance_id: &str,
    recipe: InstanceIconRecipe,
    symbol_bytes: Vec<u8>,
) -> crate::Result<String> {
    let state = State::get().await?;
    let instance =
        instance_rows::get_instance_display_info(instance_id, &state.pool)
            .await?
            .ok_or_else(|| {
                crate::ErrorKind::InputError("Unknown instance".to_string())
            })?;
    let icon_path = cache_generated_icon(recipe.clone(), symbol_bytes).await?;

    crate::state::edit_instance(
        instance_id,
        EditInstance {
            icon_path: Some(Some(icon_path.clone())),
            icon_recipe: Some(Some(recipe)),
            ..EditInstance::default()
        },
        &state.pool,
    )
    .await?;

    if let Err(error) = super::shared::sync_shared_instance_icon(
        instance_id,
        Some(&icon_path),
        &state,
    )
    .await
    {
        tracing::warn!(
            instance_id,
            error = %error,
            "Failed to sync shared instance icon"
        );
    }

    emit_instance(&instance.id, InstancePayloadType::Edited).await?;

    Ok(icon_path)
}

pub async fn cache_generated_icon(
    recipe: InstanceIconRecipe,
    symbol_bytes: Vec<u8>,
) -> crate::Result<String> {
    let background_color = validate_icon_recipe(&recipe)?;
    let state = State::get().await?;
    let icon_bytes = tokio::task::spawn_blocking(move || {
        render_generated_icon(background_color, &symbol_bytes)
    })
    .await??;
    let file = crate::util::fetch::write_cached_icon(
        "generated-instance-icon.png",
        &state.directories.caches_dir(),
        Bytes::from(icon_bytes),
        &state.io_semaphore,
    )
    .await?;

    Ok(file.to_string_lossy().to_string())
}

pub async fn get_recent_icon_recipes() -> crate::Result<Vec<InstanceIconRecipe>>
{
    let state = State::get().await?;
    instance_rows::get_recent_instance_icon_recipes(&state.pool).await
}

fn validate_icon_recipe(recipe: &InstanceIconRecipe) -> crate::Result<[u8; 3]> {
    let background_color = match &recipe.background {
        InstanceIconBackground::Color { value } => {
            parse_background_color(value)?
        }
    };
    validate_icon_recipe_id("symbol", &recipe.symbol)?;
    Ok(background_color)
}

fn parse_background_color(value: &str) -> crate::Result<[u8; 3]> {
    if value.len() != 7 || !value.starts_with('#') {
        return Err(crate::ErrorKind::InputError(
            "Instance icon background must be a hexadecimal color".to_string(),
        )
        .into());
    }

    let color = u32::from_str_radix(&value[1..], 16).map_err(|_| {
        crate::ErrorKind::InputError(
            "Instance icon background must be a hexadecimal color".to_string(),
        )
    })?;

    Ok([
        ((color >> 16) & 0xff) as u8,
        ((color >> 8) & 0xff) as u8,
        (color & 0xff) as u8,
    ])
}

fn validate_icon_recipe_id(kind: &str, value: &str) -> crate::Result<()> {
    if value.is_empty()
        || value.len() > MAX_ICON_RECIPE_ID_LENGTH
        || !value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_'
        })
    {
        return Err(crate::ErrorKind::InputError(format!(
            "Instance icon {kind} ID is invalid"
        ))
        .into());
    }

    Ok(())
}

fn render_generated_icon(
    background_color: [u8; 3],
    symbol_bytes: &[u8],
) -> crate::Result<Vec<u8>> {
    if symbol_bytes.is_empty() || symbol_bytes.len() > MAX_SYMBOL_BYTES {
        return Err(crate::ErrorKind::InputError(
            "Instance icon symbol must be a PNG smaller than 4 MiB".to_string(),
        )
        .into());
    }

    let reader =
        ImageReader::with_format(Cursor::new(symbol_bytes), ImageFormat::Png);
    let (width, height) =
        reader.into_dimensions().map_err(image_input_error)?;
    if width == 0
        || height == 0
        || width > MAX_SYMBOL_DIMENSION
        || height > MAX_SYMBOL_DIMENSION
    {
        return Err(crate::ErrorKind::InputError(format!(
            "Instance icon symbol dimensions must be between 1 and {MAX_SYMBOL_DIMENSION} pixels"
        ))
        .into());
    }

    let symbol =
        image::load_from_memory_with_format(symbol_bytes, ImageFormat::Png)
            .map_err(image_input_error)?
            .resize_exact(
                GENERATED_ICON_SIZE,
                GENERATED_ICON_SIZE,
                FilterType::Lanczos3,
            )
            .to_rgba8();
    let mut icon = RgbaImage::from_pixel(
        GENERATED_ICON_SIZE,
        GENERATED_ICON_SIZE,
        Rgba([
            background_color[0],
            background_color[1],
            background_color[2],
            255,
        ]),
    );
    image::imageops::overlay(&mut icon, &symbol, 0, 0);
    for pixel in icon.pixels_mut() {
        pixel[3] = 255;
    }

    let mut encoded = Cursor::new(Vec::new());
    DynamicImage::ImageRgba8(icon)
        .write_to(&mut encoded, ImageFormat::Png)
        .map_err(image_input_error)?;

    Ok(encoded.into_inner())
}

fn image_input_error(error: image::ImageError) -> crate::Error {
    crate::ErrorKind::InputError(format!(
        "Invalid instance icon symbol: {error}"
    ))
    .into()
}

#[cfg(test)]
mod tests {
    use super::{
        GENERATED_ICON_SIZE, InstanceIconBackground, InstanceIconRecipe,
        render_generated_icon, validate_icon_recipe,
    };
    use image::{DynamicImage, ImageFormat, Rgba, RgbaImage};
    use std::io::Cursor;

    fn png(pixel: Rgba<u8>) -> Vec<u8> {
        let mut bytes = Cursor::new(Vec::new());
        DynamicImage::ImageRgba8(RgbaImage::from_pixel(1, 1, pixel))
            .write_to(&mut bytes, ImageFormat::Png)
            .unwrap();
        bytes.into_inner()
    }

    #[test]
    fn generated_icon_renders_background_and_symbol() {
        let bytes = render_generated_icon(
            [10, 20, 30],
            &png(Rgba([200, 100, 50, 128])),
        )
        .unwrap();
        let icon =
            image::load_from_memory_with_format(&bytes, ImageFormat::Png)
                .unwrap()
                .to_rgba8();

        assert_eq!(
            icon.dimensions(),
            (GENERATED_ICON_SIZE, GENERATED_ICON_SIZE)
        );
        assert_eq!(icon.get_pixel(0, 0), &Rgba([105, 60, 40, 255]));
    }

    #[test]
    fn generated_icon_rejects_invalid_symbol_data() {
        assert!(render_generated_icon([0, 0, 0], b"not a png").is_err());
    }

    #[test]
    fn generated_icon_recipe_validates_color_and_symbol_id() {
        assert_eq!(
            validate_icon_recipe(&InstanceIconRecipe {
                background: InstanceIconBackground::Color {
                    value: "#c78aff".to_string(),
                },
                symbol: "dusk_block".to_string(),
            })
            .unwrap(),
            [199, 138, 255]
        );
        assert!(
            validate_icon_recipe(&InstanceIconRecipe {
                background: InstanceIconBackground::Color {
                    value: "purple".to_string(),
                },
                symbol: "dusk_block".to_string(),
            })
            .is_err()
        );
        assert!(
            validate_icon_recipe(&InstanceIconRecipe {
                background: InstanceIconBackground::Color {
                    value: "#c78aff".to_string(),
                },
                symbol: "dusk-block".to_string(),
            })
            .is_err()
        );
    }
}

#[tracing::instrument]
pub async fn remove(instance_id: &str) -> crate::Result<()> {
    let state = State::get().await?;
    let instance =
        instance_rows::get_instance_display_info(instance_id, &state.pool)
            .await?;
    crate::state::remove_instance(instance_id, &state).await?;

    if let Some(instance) = instance {
        emit_instance(&instance.id, InstancePayloadType::Removed).await?;
    }

    Ok(())
}

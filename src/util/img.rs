use crate::database;
use crate::database::models::image_item;
use crate::database::redis::RedisPool;
use crate::models::images::ImageContext;
use crate::routes::ApiError;
use color_thief::ColorFormat;
use image::imageops::FilterType;
use image::{EncodableLayout, ImageError};

pub fn get_color_from_img(data: &[u8]) -> Result<Option<u32>, ImageError> {
    let image = image::load_from_memory(data)?
        .resize(256, 256, FilterType::Nearest)
        .crop_imm(128, 128, 64, 64);
    let color = color_thief::get_palette(image.to_rgb8().as_bytes(), ColorFormat::Rgb, 10, 2)
        .ok()
        .and_then(|x| x.get(0).copied())
        .map(|x| (x.r as u32) << 16 | (x.g as u32) << 8 | (x.b as u32));

    Ok(color)
}

// check changes to associated images
// if they no longer exist in the String list, delete them
// Eg: if description is modified and no longer contains a link to an iamge
pub async fn delete_unused_images(
    context: ImageContext,
    reference_strings: Vec<&str>,
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    redis: &RedisPool,
) -> Result<(), ApiError> {
    let uploaded_images = database::models::Image::get_many_contexted(context, transaction).await?;

    for image in uploaded_images {
        let mut should_delete = true;
        for reference in &reference_strings {
            if image.url.contains(reference) {
                should_delete = false;
                break;
            }
        }

        if should_delete {
            image_item::Image::remove(image.id, transaction, redis).await?;
            image_item::Image::clear_cache(image.id, redis).await?;
        }
    }

    Ok(())
}

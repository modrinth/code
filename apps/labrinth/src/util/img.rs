use crate::database;
use crate::database::models::image_item;
use crate::database::redis::RedisPool;
use crate::file_hosting::FileHost;
use crate::models::images::ImageContext;
use crate::routes::ApiError;
use color_thief::ColorFormat;
use hex::ToHex;
use image::imageops::FilterType;
use image::{
    DynamicImage, EncodableLayout, GenericImageView, ImageError, ImageFormat,
};
use sha1::Digest;
use std::io::Cursor;
use webp::Encoder;

pub fn get_color_from_img(data: &[u8]) -> Result<Option<u32>, ImageError> {
    let image = image::load_from_memory(data)?
        .resize(256, 256, FilterType::Nearest)
        .crop_imm(128, 128, 64, 64);
    let color = color_thief::get_palette(
        image.to_rgb8().as_bytes(),
        ColorFormat::Rgb,
        10,
        2,
    )
    .ok()
    .and_then(|x| x.first().copied())
    .map(|x| ((x.r as u32) << 16) | ((x.g as u32) << 8) | (x.b as u32));

    Ok(color)
}

pub struct UploadImageResult {
    pub url: String,
    pub url_path: String,

    pub raw_url: String,
    pub raw_url_path: String,

    pub color: Option<u32>,
}

pub async fn upload_image_optimized(
    upload_folder: &str,
    bytes: bytes::Bytes,
    file_extension: &str,
    target_width: Option<u32>,
    min_aspect_ratio: Option<f32>,
    file_host: &dyn FileHost,
) -> Result<UploadImageResult, ApiError> {
    let content_type = crate::util::ext::get_image_content_type(file_extension)
        .ok_or_else(|| {
            ApiError::InvalidInput(format!(
                "Invalid format for image: {file_extension}"
            ))
        })?;

    let cdn_url = dotenvy::var("CDN_URL")?;

    let hash = sha1::Sha1::digest(&bytes).encode_hex::<String>();
    let (processed_image, processed_image_ext) = process_image(
        bytes.clone(),
        content_type,
        target_width,
        min_aspect_ratio,
    )?;
    let color = get_color_from_img(&bytes)?;

    // Only upload the processed image if it's smaller than the original
    let processed_upload_data = if processed_image.len() < bytes.len() {
        Some(
            file_host
                .upload_file(
                    content_type,
                    &format!(
                        "{}/{}_{}.{}",
                        upload_folder,
                        hash,
                        target_width.unwrap_or(0),
                        processed_image_ext
                    ),
                    processed_image,
                )
                .await?,
        )
    } else {
        None
    };

    let upload_data = file_host
        .upload_file(
            content_type,
            &format!("{upload_folder}/{hash}.{file_extension}"),
            bytes,
        )
        .await?;

    let url = format!("{}/{}", cdn_url, upload_data.file_name);
    Ok(UploadImageResult {
        url: processed_upload_data
            .clone()
            .map(|x| format!("{}/{}", cdn_url, x.file_name))
            .unwrap_or_else(|| url.clone()),
        url_path: processed_upload_data
            .map(|x| x.file_name)
            .unwrap_or_else(|| upload_data.file_name.clone()),

        raw_url: url,
        raw_url_path: upload_data.file_name,
        color,
    })
}

fn process_image(
    image_bytes: bytes::Bytes,
    content_type: &str,
    target_width: Option<u32>,
    min_aspect_ratio: Option<f32>,
) -> Result<(bytes::Bytes, String), ImageError> {
    if content_type.to_lowercase() == "image/gif" {
        return Ok((image_bytes.clone(), "gif".to_string()));
    }

    let mut img = image::load_from_memory(&image_bytes)?;

    let webp_bytes = convert_to_webp(&img)?;
    img = image::load_from_memory(&webp_bytes)?;

    // Resize the image
    let (orig_width, orig_height) = img.dimensions();
    let aspect_ratio = orig_width as f32 / orig_height as f32;

    if let Some(target_width) = target_width {
        if img.width() > target_width {
            let new_height =
                (target_width as f32 / aspect_ratio).round() as u32;
            img = img.resize(target_width, new_height, FilterType::Lanczos3);
        }
    }

    if let Some(min_aspect_ratio) = min_aspect_ratio {
        // Crop if necessary
        if aspect_ratio < min_aspect_ratio {
            let crop_height =
                (img.width() as f32 / min_aspect_ratio).round() as u32;
            let y_offset = (img.height() - crop_height) / 2;
            img = img.crop_imm(0, y_offset, img.width(), crop_height);
        }
    }

    // Optimize and compress
    let mut output = Vec::new();
    img.write_to(&mut Cursor::new(&mut output), ImageFormat::WebP)?;

    Ok((bytes::Bytes::from(output), "webp".to_string()))
}

fn convert_to_webp(img: &DynamicImage) -> Result<Vec<u8>, ImageError> {
    let rgba = img.to_rgba8();
    let encoder = Encoder::from_rgba(&rgba, img.width(), img.height());
    let webp = encoder.encode(75.0); // Quality factor: 0-100, 75 is a good balance
    Ok(webp.to_vec())
}

pub async fn delete_old_images(
    image_url: Option<String>,
    raw_image_url: Option<String>,
    file_host: &dyn FileHost,
) -> Result<(), ApiError> {
    let cdn_url = dotenvy::var("CDN_URL")?;
    let cdn_url_start = format!("{cdn_url}/");
    if let Some(image_url) = image_url {
        let name = image_url.split(&cdn_url_start).nth(1);

        if let Some(icon_path) = name {
            file_host.delete_file_version("", icon_path).await?;
        }
    }

    if let Some(raw_image_url) = raw_image_url {
        let name = raw_image_url.split(&cdn_url_start).nth(1);

        if let Some(icon_path) = name {
            file_host.delete_file_version("", icon_path).await?;
        }
    }

    Ok(())
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
    let uploaded_images =
        database::models::DBImage::get_many_contexted(context, transaction)
            .await?;

    for image in uploaded_images {
        let mut should_delete = true;
        for reference in &reference_strings {
            if image.url.contains(reference) {
                should_delete = false;
                break;
            }
        }

        if should_delete {
            image_item::DBImage::remove(image.id, transaction, redis).await?;
            image_item::DBImage::clear_cache(image.id, redis).await?;
        }
    }

    Ok(())
}

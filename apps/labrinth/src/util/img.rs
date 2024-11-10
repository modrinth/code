use crate::database;
use crate::database::models::image_item;
use crate::database::redis::RedisPool;
use crate::file_hosting::FileHost;
use crate::models::images::ImageContext;
use crate::routes::ApiError;
use color_thief::ColorFormat;
use image::codecs::gif::GifDecoder;
use image::codecs::png::PngDecoder;
use image::codecs::webp::WebPDecoder;
use image::error::{UnsupportedError, UnsupportedErrorKind};
use image::imageops::FilterType;
use image::{
    AnimationDecoder, DynamicImage, EncodableLayout, Frame, GenericImageView,
    ImageError, ImageFormat,
};
use std::io::Cursor;
use std::ops::Div;
use webp::{AnimEncoder, AnimFrame, Encoder, WebPConfig};

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
    .map(|x| (x.r as u32) << 16 | (x.g as u32) << 8 | (x.b as u32));

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
                "Invalid format for image: {}",
                file_extension
            ))
        })?;

    let cdn_url = dotenvy::var("CDN_URL")?;

    let hash = sha1::Sha1::from(&bytes).hexdigest();
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
            &format!("{}/{}.{}", upload_folder, hash, file_extension),
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
    match content_type {
        "image/gif" => {
            if target_width.is_none() && min_aspect_ratio.is_none() {
                process_animated_image(
                    image_bytes,
                    content_type,
                    target_width,
                    min_aspect_ratio,
                )
            } else {
                // Skip animated image processing for GIFs that won't be modified
                // But not before checking if it's indeed a GIF
                let format = image::guess_format(&image_bytes)?;
                if format == ImageFormat::Gif {
                    Ok((image_bytes.clone(), "gif".to_string()))
                } else {
                    Err(ImageError::Unsupported(
                        UnsupportedError::from_format_and_kind(
                            ImageFormat::Gif.into(),
                            UnsupportedErrorKind::GenericFeature(
                                "Attempted to process an invalid GIF!".to_owned()
                            )
                        )
                    ))
                }
            }
        },
        "image/png" => {
            let decoder = PngDecoder::new(Cursor::new(image_bytes.clone()))?;
            if decoder.is_apng()? {
                process_animated_image(
                    image_bytes,
                    content_type,
                    target_width,
                    min_aspect_ratio,
                )
            } else {
                process_static_image(
                    image_bytes,
                    target_width,
                    min_aspect_ratio,
                )
            }
        }
        "image/webp" => process_animated_image(
            image_bytes,
            content_type,
            target_width,
            min_aspect_ratio,
        ),
        _ => process_static_image(image_bytes, target_width, min_aspect_ratio),
    }
}

fn process_animated_image(
    image_bytes: bytes::Bytes,
    content_type: &str,
    target_width: Option<u32>,
    min_aspect_ratio: Option<f32>,
) -> Result<(bytes::Bytes, String), ImageError> {
    let dimensions =
        image::load_from_memory(&image_bytes.clone())?.dimensions();
    let frames2: Vec<Frame> = match content_type {
        "image/gif" => GifDecoder::new(Cursor::new(image_bytes))?
            .into_frames()
            .collect_frames()?,
        "image/png" => PngDecoder::new(Cursor::new(image_bytes))?
            .apng()?
            .into_frames()
            .collect_frames()?,
        "image/webp" => WebPDecoder::new(Cursor::new(image_bytes))?
            .into_frames()
            .collect_frames()?,
        _ => unimplemented!(),
    };

    // Resize the image
    let (orig_width, orig_height) = dimensions;
    let og_aspect_ratio = orig_width as f32 / orig_height as f32;
    let mut width = orig_width;
    let mut height = orig_height;
    let mut crop_image = false;

    if let Some(target_width) = target_width {
        if dimensions.0 > target_width {
            width = target_width;
            height = (target_width as f32 / og_aspect_ratio).round() as u32;
        }
    }

    if let Some(min_aspect_ratio) = min_aspect_ratio {
        // Crop if necessary
        if og_aspect_ratio < min_aspect_ratio {
            crop_image = true;
        }
    }

    let mut config = WebPConfig::new().unwrap();
    config.quality = 75f32;
    config.method = 6;
    let mut encoder = AnimEncoder::new(width, height, &config);
    encoder.set_loop_count(0);

    let mut frames = vec![];
    frames2.iter().for_each(|frame| {
        let mut img = image::imageops::resize(
            frame.buffer(),
            width,
            height,
            FilterType::Lanczos3,
        );
        if crop_image {
            let crop_height =
                (width as f32 / min_aspect_ratio.unwrap()).round() as u32;
            let y_offset = (height - crop_height) / 2;
            img = image::imageops::crop_imm(
                &img,
                0,
                y_offset,
                img.width(),
                crop_height,
            )
            .to_image();
        }
        frames.push((img.clone(), frame.delay()));
    });

    let mut t = 0;
    for f in &frames {
        encoder.add_frame(AnimFrame::from_rgba(&f.0, width, height, t));
        t += f.1.numer_denom_ms().0.div(f.1.numer_denom_ms().1) as i32;
    }

    let webp_bytes = encoder.encode();

    Ok((bytes::Bytes::from(webp_bytes.to_vec()), "webp".to_string()))
}

fn process_static_image(
    image_bytes: bytes::Bytes,
    target_width: Option<u32>,
    min_aspect_ratio: Option<f32>,
) -> Result<(bytes::Bytes, String), ImageError> {
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
        database::models::Image::get_many_contexted(context, transaction)
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
            image_item::Image::remove(image.id, transaction, redis).await?;
            image_item::Image::clear_cache(image.id, redis).await?;
        }
    }

    Ok(())
}

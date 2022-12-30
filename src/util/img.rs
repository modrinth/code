use color_thief::ColorFormat;
use image::imageops::FilterType;
use image::{EncodableLayout, ImageError};

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
    .and_then(|x| x.get(0).copied())
    .map(|x| (x.r as u32) << 16 | (x.g as u32) << 8 | (x.b as u32));

    Ok(color)
}

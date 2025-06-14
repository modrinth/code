//! Miscellaneous PNG utilities for Minecraft skins.

use std::sync::Arc;

use base64::Engine;
use bytemuck::{AnyBitPattern, NoUninit};
use bytes::Bytes;
use data_url::DataUrl;
use futures::{Stream, TryStreamExt, future::Either, stream};
use tokio_util::{compat::FuturesAsyncReadCompatExt, io::SyncIoBridge};
use url::Url;

use crate::{
    ErrorKind, minecraft_skins::UrlOrBlob, util::fetch::REQWEST_CLIENT,
};

pub async fn url_to_data_stream(
    url: &Url,
) -> crate::Result<impl Stream<Item = Result<Bytes, reqwest::Error>> + use<>> {
    if url.scheme() == "data" {
        let data = DataUrl::process(url.as_str())?.decode_to_vec()?.0.into();

        Ok(Either::Left(stream::once(async { Ok(data) })))
    } else {
        let response = REQWEST_CLIENT
            .get(url.as_str())
            .header("Accept", "image/png")
            .send()
            .await
            .and_then(|response| response.error_for_status())?;

        Ok(Either::Right(response.bytes_stream()))
    }
}

pub fn blob_to_data_url(png_data: impl AsRef<[u8]>) -> Option<Arc<Url>> {
    let png_data = png_data.as_ref();

    is_png(png_data).then(|| {
        Url::parse(&format!(
            "data:image/png;base64,{}",
            base64::engine::general_purpose::STANDARD.encode(png_data)
        ))
        .unwrap()
        .into()
    })
}

pub fn is_png(png_data: &[u8]) -> bool {
    /// The initial 8 bytes of a PNG file, used to identify it as such.
    ///
    /// Reference: <https://www.w3.org/TR/png-3/#3PNGsignature>
    const PNG_SIGNATURE: &[u8] =
        &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];

    png_data.starts_with(PNG_SIGNATURE)
}

pub fn dimensions(png_data: &[u8]) -> crate::Result<(u32, u32)> {
    if !is_png(png_data) {
        Err(ErrorKind::InvalidPng)?;
    }

    // Read the width and height fields from the IHDR chunk, which the
    // PNG specification mandates to be the first in the file, just after
    // the 8 signature bytes. See:
    // https://www.w3.org/TR/png-3/#5DataRep
    // https://www.w3.org/TR/png-3/#11IHDR
    let width = u32::from_be_bytes(
        png_data
            .get(16..20)
            .ok_or(ErrorKind::InvalidPng)?
            .try_into()
            .unwrap(),
    );
    let height = u32::from_be_bytes(
        png_data
            .get(20..24)
            .ok_or(ErrorKind::InvalidPng)?
            .try_into()
            .unwrap(),
    );

    Ok((width, height))
}

/// Normalizes the texture of a Minecraft skin to the modern 64x64 format, handling
/// legacy 64x32 skins as the vanilla game client does. This function prioritizes
/// PNG encoding speed over compression density, so the resulting textures are better
/// suited for display purposes, not persistent storage or transmission.
///
/// The normalized, processed is returned texture as a byte array in PNG format.
pub async fn normalize_skin_texture(
    texture: &UrlOrBlob,
) -> crate::Result<Bytes> {
    let texture_stream = SyncIoBridge::new(Box::pin(
        match texture {
            UrlOrBlob::Url(url) => Either::Left(
                url_to_data_stream(url)
                    .await?
                    .map_err(std::io::Error::other)
                    .into_async_read(),
            ),
            UrlOrBlob::Blob(blob) => Either::Right(
                stream::once({
                    let blob = Bytes::clone(blob);
                    async { Ok(blob) }
                })
                .into_async_read(),
            ),
        }
        .compat(),
    ));

    tokio::task::spawn_blocking(|| {
        let mut png_reader = {
            let mut decoder = png::Decoder::new(texture_stream);
            decoder.set_transformations(
                png::Transformations::normalize_to_color8(),
            );
            decoder.read_info()
        }?;

        // The code below assumes that the skin texture has valid dimensions.
        // This also serves as a way to bail out early for obviously invalid or
        // adversarial textures
        if png_reader.info().width != 64
            || ![64, 32].contains(&png_reader.info().height)
        {
            Err(ErrorKind::InvalidSkinTexture)?;
        }

        let is_legacy_skin = png_reader.info().height == 32;

        let mut texture_buf = if is_legacy_skin {
            // Legacy skins have half the height, so duplicate the rows to
            // turn them into a 64x64 texture
            vec![0; png_reader.output_buffer_size() * 2]
        } else {
            // Modern skins are left as-is
            vec![0; png_reader.output_buffer_size()]
        };

        let texture_buf_color_type = png_reader.output_color_type().0;
        png_reader.next_frame(&mut texture_buf)?;

        if is_legacy_skin {
            convert_legacy_skin_texture(
                &mut texture_buf,
                texture_buf_color_type,
                png_reader.info(),
            )?;
        }

        let mut encoded_png = vec![];

        let mut png_encoder = png::Encoder::new(&mut encoded_png, 64, 64);
        png_encoder.set_color(texture_buf_color_type);
        png_encoder.set_depth(png::BitDepth::Eight);
        png_encoder.set_filter(png::FilterType::NoFilter);
        png_encoder.set_compression(png::Compression::Fast);

        // Keeping color space information properly set, to handle the occasional
        // strange PNG with non-sRGB chromacities and/or different grayscale spaces
        // that keeps most people wondering, is what sets a carefully crafted image
        // manipulation routine apart :)
        if let Some(source_chromacities) =
            png_reader.info().source_chromaticities.as_ref().copied()
        {
            png_encoder.set_source_chromaticities(source_chromacities);
        }
        if let Some(source_gamma) =
            png_reader.info().source_gamma.as_ref().copied()
        {
            png_encoder.set_source_gamma(source_gamma);
        }
        if let Some(source_srgb) = png_reader.info().srgb.as_ref().copied() {
            png_encoder.set_source_srgb(source_srgb);
        }

        let mut png_writer = png_encoder.write_header()?;
        png_writer.write_image_data(&texture_buf)?;
        png_writer.finish()?;

        Ok(encoded_png.into())
    })
    .await?
}

/// Converts a legacy skin texture (32x64 pixels) within a 64x64 buffer to the
/// native 64x64 format used by modern Minecraft clients.
///
/// See also 25w16a's `SkinTextureDownloader#processLegacySkin` method.
#[inline]
fn convert_legacy_skin_texture(
    texture_buf: &mut [u8],
    texture_color_type: png::ColorType,
    texture_info: &png::Info,
) -> crate::Result<()> {
    /// The skin faces the game client copies around, in order, when converting a
    /// legacy skin to the native 64x64 format.
    const FACE_COPY_PARAMETERS: &[(
        usize,
        usize,
        isize,
        isize,
        usize,
        usize,
    )] = &[
        (4, 16, 16, 32, 4, 4),
        (8, 16, 16, 32, 4, 4),
        (0, 20, 24, 32, 4, 12),
        (4, 20, 16, 32, 4, 12),
        (8, 20, 8, 32, 4, 12),
        (12, 20, 16, 32, 4, 12),
        (44, 16, -8, 32, 4, 4),
        (48, 16, -8, 32, 4, 4),
        (40, 20, 0, 32, 4, 12),
        (44, 20, -8, 32, 4, 12),
        (48, 20, -16, 32, 4, 12),
        (52, 20, -8, 32, 4, 12),
    ];

    for (x, y, off_x, off_y, width, height) in FACE_COPY_PARAMETERS {
        macro_rules! do_copy {
            ($pixel_type:ty) => {
                copy_rect_mirror_horizontally::<$pixel_type>(
                    // This cast should never fail because all pixels have a depth of 8 bits
                    // after the transformations applied during decoding
                    ::bytemuck::try_cast_slice_mut(texture_buf).map_err(|_| ErrorKind::InvalidPng)?,
                    &texture_info,
                    *x,
                    *y,
                    *off_x,
                    *off_y,
                    *width,
                    *height,
                )
            };
        }

        match texture_color_type.samples() {
            1 => do_copy!(rgb::Gray<u8>),
            2 => do_copy!(rgb::GrayAlpha<u8>),
            3 => do_copy!(rgb::Rgb<u8>),
            4 => do_copy!(rgb::Rgba<u8>),
            _ => Err(ErrorKind::InvalidPng)?, // Cannot happen by PNG spec after transformations
        };
    }

    Ok(())
}

/// Copies a `width` pixels wide, `height` pixels tall rectangle of pixels within `texture_buf`
/// whose top-left corner is at coordinates `(x, y)` to a destination rectangle whose top-left
/// corner is at coordinates `(x + off_x, y + off_y)`, while mirroring (i.e., flipping) the
/// pixels horizontally.
///
/// Equivalent to Mojang's Blaze3D `NativeImage#copyRect(int, int, int, int, int, int,
/// boolean, boolean)` method, but with the last two parameters fixed to `true` and `false`,
/// respectively.
#[allow(clippy::too_many_arguments)]
fn copy_rect_mirror_horizontally<PixelType: NoUninit + AnyBitPattern>(
    texture_buf: &mut [PixelType],
    texture_info: &png::Info,
    x: usize,
    y: usize,
    off_x: isize,
    off_y: isize,
    width: usize,
    height: usize,
) {
    for row in 0..height {
        for col in 0..width {
            let src_x = x + col;
            let src_y = y + row;
            let dst_x = (x as isize + off_x) as usize + (width - 1 - col);
            let dst_y = (y as isize + off_y) as usize + row;

            texture_buf[dst_x + dst_y * texture_info.width as usize] =
                texture_buf[src_x + src_y * texture_info.width as usize];
        }
    }
}

#[cfg(test)]
#[tokio::test]
async fn normalize_skin_texture_works() {
    let legacy_png_data = &include_bytes!("assets/default/MissingNo.png")[..];
    let expected_normalized_png_data =
        &include_bytes!("assets/test/MissingNo_normalized.png")[..];

    let normalized_png_data =
        normalize_skin_texture(&UrlOrBlob::Blob(legacy_png_data.into()))
            .await
            .expect("Failed to normalize skin texture");

    let decode_to_pixels = |png_data: &[u8]| {
        let decoder = png::Decoder::new(png_data);
        let mut reader = decoder.read_info().expect("Failed to read PNG info");
        let mut buffer = vec![0; reader.output_buffer_size()];
        reader
            .next_frame(&mut buffer)
            .expect("Failed to decode PNG");
        (buffer, reader.info().clone())
    };

    let (normalized_pixels, normalized_info) =
        decode_to_pixels(&normalized_png_data);
    let (expected_pixels, expected_info) =
        decode_to_pixels(expected_normalized_png_data);

    // Check that dimensions match
    assert_eq!(normalized_info.width, expected_info.width);
    assert_eq!(normalized_info.height, expected_info.height);
    assert_eq!(normalized_info.color_type, expected_info.color_type);

    // Check that pixel data matches
    assert_eq!(
        normalized_pixels, expected_pixels,
        "Pixel data doesn't match"
    );
}

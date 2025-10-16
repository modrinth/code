//! Miscellaneous PNG utilities for Minecraft skins.

use std::io::{BufRead, Cursor, Seek};
use std::sync::Arc;

use base64::Engine;
use bytes::Bytes;
use data_url::DataUrl;
use futures::{Stream, TryStreamExt, future::Either, stream};
use itertools::Itertools;
use rgb::Rgba;
use tokio::io::AsyncReadExt;
use tokio_util::compat::FuturesAsyncReadCompatExt;
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

/// Normalizes the texture of a Minecraft skin to the modern 64x64 format, handling legacy 64x32
/// skins, doing "Notch transparency hack" and making inner parts opaque as the vanilla game client
/// does. This function prioritizes PNG encoding speed over compression density, so the resulting
/// textures are better suited for display purposes, not persistent storage or transmission.
///
/// The normalized, processed is returned texture as a byte array in PNG format.
pub async fn normalize_skin_texture(
    texture: &UrlOrBlob,
) -> crate::Result<Bytes> {
    let mut texture_data = Vec::with_capacity(8192);
    Box::pin(
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
    )
    .read_to_end(&mut texture_data)
    .await?;

    let mut png_reader = {
        let mut decoder = png::Decoder::new(Cursor::new(texture_data));
        decoder
            .set_transformations(png::Transformations::normalize_to_color8());
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
    let mut texture_buf =
        get_skin_texture_buffer(&mut png_reader, is_legacy_skin)?;
    if is_legacy_skin {
        convert_legacy_skin_texture(&mut texture_buf, png_reader.info());
        do_notch_transparency_hack(&mut texture_buf, png_reader.info());
    }
    make_inner_parts_opaque(&mut texture_buf, png_reader.info());

    let mut encoded_png = vec![];

    let mut png_encoder = png::Encoder::new(&mut encoded_png, 64, 64);
    png_encoder.set_color(png::ColorType::Rgba);
    png_encoder.set_depth(png::BitDepth::Eight);
    png_encoder.set_filter(png::Filter::NoFilter);
    png_encoder.set_compression(png::Compression::Fast);

    // Keeping color space information properly set, to handle the occasional
    // strange PNG with non-sRGB chromaticities and/or different grayscale spaces
    // that keeps most people wondering, is what sets a carefully crafted image
    // manipulation routine apart :)
    if let Some(source_chromaticities) =
        png_reader.info().source_chromaticities.as_ref().copied()
    {
        png_encoder.set_source_chromaticities(source_chromaticities);
    }
    if let Some(source_gamma) = png_reader.info().source_gamma.as_ref().copied()
    {
        png_encoder.set_source_gamma(source_gamma);
    }
    if let Some(source_srgb) = png_reader.info().srgb.as_ref().copied() {
        png_encoder.set_source_srgb(source_srgb);
    }

    let png_buf = bytemuck::try_cast_slice(&texture_buf)
        .map_err(|_| ErrorKind::InvalidPng)?;
    let mut png_writer = png_encoder.write_header()?;
    png_writer.write_image_data(png_buf)?;
    png_writer.finish()?;

    Ok(encoded_png.into())
}

/// Reads a skin texture and returns a 64x64 buffer in RGBA format.
fn get_skin_texture_buffer<R: BufRead + Seek>(
    png_reader: &mut png::Reader<R>,
    is_legacy_skin: bool,
) -> crate::Result<Vec<Rgba<u8>>> {
    let output_buffer_size = png_reader
        .output_buffer_size()
        .expect("Reasonable skin texture size verified already");
    let mut png_buf = if is_legacy_skin {
        // Legacy skins have half the height, so duplicate the rows to
        // turn them into a 64x64 texture
        vec![0; output_buffer_size * 2]
    } else {
        // Modern skins are left as-is
        vec![0; output_buffer_size]
    };
    png_reader.next_frame(&mut png_buf)?;

    let mut texture_buf = match png_reader.output_color_type().0 {
        png::ColorType::Grayscale => png_buf
            .iter()
            .map(|&value| Rgba {
                r: value,
                g: value,
                b: value,
                a: 255,
            })
            .collect_vec(),
        png::ColorType::GrayscaleAlpha => png_buf
            .chunks_exact(2)
            .map(|chunk| Rgba {
                r: chunk[0],
                g: chunk[0],
                b: chunk[0],
                a: chunk[1],
            })
            .collect_vec(),
        png::ColorType::Rgb => png_buf
            .chunks_exact(3)
            .map(|chunk| Rgba {
                r: chunk[0],
                g: chunk[1],
                b: chunk[2],
                a: 255,
            })
            .collect_vec(),
        png::ColorType::Rgba => bytemuck::try_cast_vec(png_buf)
            .map_err(|_| ErrorKind::InvalidPng)?,
        _ => Err(ErrorKind::InvalidPng)?, // Cannot happen by PNG spec after transformations
    };

    // Make the added bottom half of the expanded legacy skin buffer transparent
    if is_legacy_skin {
        set_alpha(&mut texture_buf, png_reader.info(), 0, 32, 64, 64, 0);
    }

    Ok(texture_buf)
}

/// Converts a legacy skin texture (32x64 pixels) within a 64x64 buffer to the
/// native 64x64 format used by modern Minecraft clients.
///
/// See also 25w16a's `SkinTextureDownloader#processLegacySkin` method.
#[inline]
fn convert_legacy_skin_texture(
    texture_buf: &mut [Rgba<u8, u8>],
    texture_info: &png::Info,
) {
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
        copy_rect_mirror_horizontally(
            texture_buf,
            texture_info,
            *x,
            *y,
            *off_x,
            *off_y,
            *width,
            *height,
        )
    }
}

/// Makes outer head layer transparent if every pixel has alpha greater or equal to 128.
///
/// See also 25w16a's `SkinTextureDownloader#doNotchTransparencyHack` method.
fn do_notch_transparency_hack(
    texture_buf: &mut [Rgba<u8, u8>],
    texture_info: &png::Info,
) {
    // The skin part the game client makes transparent
    let (x1, y1, x2, y2) = (32, 0, 64, 32);

    for y in y1..y2 {
        for x in x1..x2 {
            if texture_buf[x + y * texture_info.width as usize].a < 128 {
                return;
            }
        }
    }

    set_alpha(texture_buf, texture_info, x1, y1, x2, y2, 0);
}

/// Makes inner parts of a skin texture opaque.
///
/// See also 25w16a's `SkinTextureDownloader#processLegacySkin` method.
#[inline]
fn make_inner_parts_opaque(
    texture_buf: &mut [Rgba<u8, u8>],
    texture_info: &png::Info,
) {
    /// The skin parts the game client makes opaque.
    const OPAQUE_PART_PARAMETERS: &[(usize, usize, usize, usize)] =
        &[(0, 0, 32, 16), (0, 16, 64, 32), (16, 48, 48, 64)];

    for (x1, y1, x2, y2) in OPAQUE_PART_PARAMETERS {
        set_alpha(texture_buf, texture_info, *x1, *y1, *x2, *y2, 255);
    }
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
fn copy_rect_mirror_horizontally(
    texture_buf: &mut [Rgba<u8, u8>],
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

/// Sets alpha for every pixel of a rectangle within `texture_buf`
/// whose top-left corner is at `(x1, y1)` and bottom-right corner is at `(x2 - 1, y2 - 1)`.
fn set_alpha(
    texture_buf: &mut [Rgba<u8, u8>],
    texture_info: &png::Info,
    x1: usize,
    y1: usize,
    x2: usize,
    y2: usize,
    alpha: u8,
) {
    for y in y1..y2 {
        for x in x1..x2 {
            texture_buf[x + y * texture_info.width as usize].a = alpha;
        }
    }
}

#[cfg(test)]
#[tokio::test]
async fn normalize_skin_texture_works() {
    let decode_to_pixels = |png_data: &[u8]| {
        let decoder = png::Decoder::new(Cursor::new(png_data));
        let mut reader = decoder.read_info().expect("Failed to read PNG info");
        let mut buffer =
            vec![0; reader.output_buffer_size().expect("Skin size too large")];
        reader
            .next_frame(&mut buffer)
            .expect("Failed to decode PNG");
        (buffer, reader.info().clone())
    };

    let test_data = [
        (
            "legacy",
            &include_bytes!("assets/test/legacy.png")[..],
            &include_bytes!("assets/test/legacy_normalized.png")[..],
        ),
        (
            "notch",
            &include_bytes!("assets/test/notch.png")[..],
            &include_bytes!("assets/test/notch_normalized.png")[..],
        ),
        (
            "transparent",
            &include_bytes!("assets/test/transparent.png")[..],
            &include_bytes!("assets/test/transparent_normalized.png")[..],
        ),
    ];

    for (skin_name, original_png_data, expected_normalized_png_data) in
        test_data
    {
        let normalized_png_data =
            normalize_skin_texture(&UrlOrBlob::Blob(original_png_data.into()))
                .await
                .expect("Failed to normalize skin texture");

        let (normalized_pixels, normalized_info) =
            decode_to_pixels(&normalized_png_data);
        let (expected_pixels, expected_info) =
            decode_to_pixels(expected_normalized_png_data);

        // Check that dimensions match
        assert_eq!(
            normalized_info.width, expected_info.width,
            "Widths don't match for {skin_name}"
        );
        assert_eq!(
            normalized_info.height, expected_info.height,
            "Heights don't match for {skin_name}"
        );
        assert_eq!(
            normalized_info.color_type, expected_info.color_type,
            "Color types don't match for {skin_name}"
        );

        // Check that pixel data matches
        assert_eq!(
            normalized_pixels, expected_pixels,
            "Pixel data doesn't match for {skin_name}"
        );
    }
}

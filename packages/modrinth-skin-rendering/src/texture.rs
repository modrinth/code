use crate::error::{Error, Result};

pub type Rgba = [u8; 4];

/// A straight-alpha sRGB image.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Texture {
    width: u32,
    height: u32,
    pixels: Vec<Rgba>,
}

impl Texture {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            pixels: vec![[0; 4]; width as usize * height as usize],
        }
    }

    pub fn from_pixels(
        width: u32,
        height: u32,
        pixels: Vec<Rgba>,
    ) -> Result<Self> {
        if pixels.len() != width as usize * height as usize {
            return Err(Error::PixelBufferSize { width, height });
        }
        Ok(Self {
            width,
            height,
            pixels,
        })
    }

    pub fn from_rgba8(width: u32, height: u32, bytes: &[u8]) -> Result<Self> {
        if bytes.len() != width as usize * height as usize * 4 {
            return Err(Error::PixelBufferSize { width, height });
        }
        let pixels = bytes
            .chunks_exact(4)
            .map(|pixel| [pixel[0], pixel[1], pixel[2], pixel[3]])
            .collect();
        Ok(Self {
            width,
            height,
            pixels,
        })
    }

    pub const fn width(&self) -> u32 {
        self.width
    }

    pub const fn height(&self) -> u32 {
        self.height
    }

    pub fn pixels(&self) -> &[Rgba] {
        &self.pixels
    }

    pub fn pixels_mut(&mut self) -> &mut [Rgba] {
        &mut self.pixels
    }

    pub fn as_rgba8(&self) -> &[u8] {
        self.pixels.as_flattened()
    }

    pub fn get(&self, x: u32, y: u32) -> Rgba {
        self.pixels[self.index(x, y)]
    }

    pub fn set(&mut self, x: u32, y: u32, pixel: Rgba) {
        let index = self.index(x, y);
        self.pixels[index] = pixel;
    }

    /// Nearest texel. Coordinates outside `0..1` clamp to the edge.
    pub fn sample(&self, u: f32, v: f32) -> Rgba {
        let x = ((u * self.width as f32).floor() as i64)
            .clamp(0, i64::from(self.width) - 1);
        let y = ((v * self.height as f32).floor() as i64)
            .clamp(0, i64::from(self.height) - 1);
        self.pixels[y as usize * self.width as usize + x as usize]
    }

    fn index(&self, x: u32, y: u32) -> usize {
        assert!(x < self.width && y < self.height);
        y as usize * self.width as usize + x as usize
    }

    #[cfg(feature = "png")]
    pub fn from_png(data: &[u8]) -> Result<Self> {
        Self::from_png_with_limit(data, 4096)
    }

    #[cfg(feature = "png")]
    pub(crate) fn from_png_with_limit(
        data: &[u8],
        max_dimension: u32,
    ) -> Result<Self> {
        let mut decoder = png::Decoder::new(std::io::Cursor::new(data));
        decoder
            .set_transformations(png::Transformations::normalize_to_color8());
        let mut reader = decoder.read_info()?;
        let (width, height) = (reader.info().width, reader.info().height);
        if width > max_dimension || height > max_dimension {
            return Err(Error::ImageTooLarge);
        }

        let mut buffer =
            vec![0; reader.output_buffer_size().ok_or(Error::ImageTooLarge)?];
        reader.next_frame(&mut buffer)?;
        let pixels = match reader.output_color_type().0 {
            png::ColorType::Grayscale => {
                buffer.iter().map(|&v| [v, v, v, 255]).collect()
            }
            png::ColorType::GrayscaleAlpha => buffer
                .chunks_exact(2)
                .map(|c| [c[0], c[0], c[0], c[1]])
                .collect(),
            png::ColorType::Rgb => buffer
                .chunks_exact(3)
                .map(|c| [c[0], c[1], c[2], 255])
                .collect(),
            png::ColorType::Rgba => buffer
                .chunks_exact(4)
                .map(|c| [c[0], c[1], c[2], c[3]])
                .collect(),
            png::ColorType::Indexed => {
                return Err(Error::PixelBufferSize { width, height });
            }
        };
        Self::from_pixels(width, height, pixels)
    }

    #[cfg(feature = "png")]
    pub fn to_png(&self) -> Result<Vec<u8>> {
        let mut encoded = Vec::new();
        let mut encoder =
            png::Encoder::new(&mut encoded, self.width, self.height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header()?;
        writer.write_image_data(self.as_rgba8())?;
        writer.finish()?;
        Ok(encoded)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum ArmModel {
    Classic,
    Slim,
}

pub const SKIN_SIZE: u32 = 64;
const LEGACY_SKIN_HEIGHT: u32 = 32;

/// A skin as authored: 64x64, or the legacy 64x32 layout.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SkinTexture {
    raw: Texture,
}

impl SkinTexture {
    pub fn new(raw: Texture) -> Result<Self> {
        if raw.width() != SKIN_SIZE
            || ![SKIN_SIZE, LEGACY_SKIN_HEIGHT].contains(&raw.height())
        {
            return Err(Error::SkinDimensions {
                width: raw.width(),
                height: raw.height(),
            });
        }
        Ok(Self { raw })
    }

    #[cfg(feature = "png")]
    pub fn from_png(data: &[u8]) -> Result<Self> {
        Self::new(Texture::from_png_with_limit(data, SKIN_SIZE)?)
    }

    pub const fn raw(&self) -> &Texture {
        &self.raw
    }

    pub const fn is_legacy(&self) -> bool {
        self.raw.height() == LEGACY_SKIN_HEIGHT
    }

    /// The 64x64 texture the vanilla client draws. See [`normalize_skin_pixels`].
    pub fn normalized(&self) -> Texture {
        let mut texture = Texture::new(SKIN_SIZE, SKIN_SIZE);
        texture.pixels_mut()[..self.raw.pixels().len()]
            .copy_from_slice(self.raw.pixels());
        normalize_skin_pixels(texture.pixels_mut(), self.is_legacy());
        texture
    }

    /// Slim if the arm columns that only classic skins use are transparent.
    pub fn detect_arm_model(&self) -> ArmModel {
        let painted = (20..32)
            .flat_map(|y| (54..56).map(move |x| (x, y)))
            .any(|(x, y)| self.raw.get(x, y)[3] != 0);
        if painted {
            ArmModel::Classic
        } else {
            ArmModel::Slim
        }
    }
}

/// Converts legacy skins and makes the base layer opaque, as
/// `SkinTextureDownloader#processLegacySkin` does in Minecraft 25w16a.
///
/// `pixels` is a 64x64 buffer. A legacy skin goes in the top half and the bottom half is
/// overwritten.
pub fn normalize_skin_pixels(pixels: &mut [Rgba], legacy: bool) {
    assert_eq!(pixels.len(), (SKIN_SIZE * SKIN_SIZE) as usize);

    if legacy {
        set_alpha(pixels, 0, 32, 64, 64, 0);
        convert_legacy_skin(pixels);
        apply_notch_transparency_hack(pixels);
    }
    for (x1, y1, x2, y2) in [(0, 0, 32, 16), (0, 16, 64, 32), (16, 48, 48, 64)]
    {
        set_alpha(pixels, x1, y1, x2, y2, 255);
    }
}

fn convert_legacy_skin(pixels: &mut [Rgba]) {
    const FACE_COPIES: [(usize, usize, isize, isize, usize, usize); 12] = [
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

    for (x, y, off_x, off_y, width, height) in FACE_COPIES {
        for row in 0..height {
            for col in 0..width {
                let dst_x = (x as isize + off_x) as usize + (width - 1 - col);
                let dst_y = (y as isize + off_y) as usize + row;
                pixels[dst_x + dst_y * SKIN_SIZE as usize] =
                    pixels[x + col + (y + row) * SKIN_SIZE as usize];
            }
        }
    }
}

fn apply_notch_transparency_hack(pixels: &mut [Rgba]) {
    let (x1, y1, x2, y2) = (32, 0, 64, 32);
    let has_transparency = (y1..y2)
        .flat_map(|y| (x1..x2).map(move |x| (x, y)))
        .any(|(x, y)| pixels[x + y * SKIN_SIZE as usize][3] < 128);
    if !has_transparency {
        set_alpha(pixels, x1, y1, x2, y2, 0);
    }
}

fn set_alpha(
    pixels: &mut [Rgba],
    x1: usize,
    y1: usize,
    x2: usize,
    y2: usize,
    alpha: u8,
) {
    for y in y1..y2 {
        for x in x1..x2 {
            pixels[x + y * SKIN_SIZE as usize][3] = alpha;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn filled(width: u32, height: u32, pixel: Rgba) -> Texture {
        Texture::from_pixels(
            width,
            height,
            vec![pixel; (width * height) as usize],
        )
        .unwrap()
    }

    #[test]
    fn rejects_textures_that_are_not_skin_sized() {
        assert!(SkinTexture::new(Texture::new(64, 64)).is_ok());
        assert!(SkinTexture::new(Texture::new(64, 32)).is_ok());
        assert!(SkinTexture::new(Texture::new(32, 32)).is_err());
        assert!(SkinTexture::new(Texture::new(128, 128)).is_err());
    }

    #[test]
    fn normalizing_makes_the_base_layer_opaque_and_keeps_overlays() {
        let skin = SkinTexture::new(filled(64, 64, [10, 20, 30, 40])).unwrap();
        let normalized = skin.normalized();
        assert_eq!(normalized.get(8, 8), [10, 20, 30, 255]);
        assert_eq!(normalized.get(20, 50), [10, 20, 30, 255]);
        assert_eq!(normalized.get(40, 8), [10, 20, 30, 40]);
        assert_eq!(normalized.get(0, 40), [10, 20, 30, 40]);
    }

    #[test]
    fn legacy_skins_gain_mirrored_left_limbs_and_lose_an_opaque_hat() {
        let mut raw = filled(64, 32, [0, 0, 0, 255]);
        raw.set(4, 20, [200, 0, 0, 255]);
        let skin = SkinTexture::new(raw).unwrap();
        assert!(skin.is_legacy());

        let normalized = skin.normalized();
        assert_eq!(normalized.width(), 64);
        assert_eq!(normalized.height(), 64);
        assert_eq!(normalized.get(23, 52), [200, 0, 0, 255]);
        assert_eq!(normalized.get(40, 8)[3], 0);
        assert_eq!(normalized.get(0, 40)[3], 0);
    }

    #[test]
    fn a_legacy_hat_with_transparency_is_kept() {
        let mut raw = filled(64, 32, [5, 5, 5, 255]);
        raw.set(33, 1, [0, 0, 0, 0]);
        let normalized = SkinTexture::new(raw).unwrap().normalized();
        assert_eq!(normalized.get(40, 8), [5, 5, 5, 255]);
    }

    #[test]
    fn detects_slim_arms_from_the_unpainted_arm_columns() {
        let classic = SkinTexture::new(filled(64, 64, [1, 1, 1, 255])).unwrap();
        assert_eq!(classic.detect_arm_model(), ArmModel::Classic);

        let mut raw = filled(64, 64, [1, 1, 1, 255]);
        for y in 20..32 {
            for x in 54..56 {
                raw.set(x, y, [0, 0, 0, 0]);
            }
        }
        let slim = SkinTexture::new(raw).unwrap();
        assert_eq!(slim.detect_arm_model(), ArmModel::Slim);
    }

    #[test]
    fn sampling_clamps_to_the_texture_edges() {
        let mut texture = Texture::new(2, 2);
        texture.set(0, 0, [1, 0, 0, 255]);
        texture.set(1, 1, [2, 0, 0, 255]);
        assert_eq!(texture.sample(-0.5, -0.5), [1, 0, 0, 255]);
        assert_eq!(texture.sample(1.5, 1.5), [2, 0, 0, 255]);
        assert_eq!(texture.sample(0.75, 0.75), [2, 0, 0, 255]);
    }

    #[cfg(feature = "png")]
    #[test]
    fn png_round_trips() {
        let mut texture = Texture::new(3, 2);
        texture.set(2, 1, [9, 8, 7, 6]);
        let decoded = Texture::from_png(&texture.to_png().unwrap()).unwrap();
        assert_eq!(decoded, texture);
    }
}

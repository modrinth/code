#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("pixel buffer length does not match {width}x{height}")]
    PixelBufferSize { width: u32, height: u32 },
    #[error("skin textures must be 64x64 or 64x32, got {width}x{height}")]
    SkinDimensions { width: u32, height: u32 },
    #[error("invalid PNG: {0}")]
    PngDecode(#[from] png::DecodingError),
    #[error("could not encode PNG: {0}")]
    PngEncode(#[from] png::EncodingError),
    #[error("image is too large to decode")]
    ImageTooLarge,
    #[error("invalid glTF: {0}")]
    Gltf(#[from] gltf::Error),
    #[error("invalid glTF model: {0}")]
    InvalidModel(&'static str),
}

pub type Result<T> = std::result::Result<T, Error>;

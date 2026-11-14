#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("pixel buffer length does not match {width}x{height}")]
    PixelBufferSize { width: u32, height: u32 },
    #[error("skin textures must be 64x64 or 64x32, got {width}x{height}")]
    SkinDimensions { width: u32, height: u32 },
    #[cfg(feature = "png")]
    #[error("invalid PNG: {0}")]
    PngDecode(#[from] png::DecodingError),
    #[cfg(feature = "png")]
    #[error("could not encode PNG: {0}")]
    PngEncode(#[from] png::EncodingError),
    #[cfg(feature = "png")]
    #[error("image is too large to decode")]
    ImageTooLarge,
    #[cfg(feature = "gltf")]
    #[error("invalid glTF JSON: {0}")]
    GltfJson(#[from] serde_json::Error),
    #[cfg(feature = "gltf")]
    #[error("invalid glTF: {0}")]
    Gltf(String),
}

pub type Result<T> = std::result::Result<T, Error>;

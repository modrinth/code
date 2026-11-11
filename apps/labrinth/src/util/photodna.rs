use std::ffi::{CString, c_char, c_void};
use std::io::Cursor;
use std::path::Path;
use std::sync::{Arc, LazyLock, Mutex};
use std::time::Duration;

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use bytes::Bytes;
use eyre::{Result, eyre};
use image::codecs::{gif::GifDecoder, png::PngDecoder, webp::WebPDecoder};
use image::{
    AnimationDecoder, DynamicImage, ImageDecoder, ImageFormat, ImageReader,
    Limits, RgbaImage,
};
use reqwest::header::{HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};
use tokio::sync::{Semaphore, mpsc};

use crate::env::ENV;
use crate::routes::ApiError;
use crate::util::error::Context as _;

const MAX_FRAMES: usize = 256;
const MAX_HASHES_PER_REQUEST: usize = 5;
const MAX_PIXELS: u64 = 64 * 1024 * 1024;
const MAX_DECODED_BYTES: u64 = 128 * 1024 * 1024;
const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);
const SDK_VERSION: &str = "1.05";
const HASH_SIZE: usize = 924;
const HASH_FORMAT_EDGE_V2: u32 = 0x80;
const PIXEL_FORMAT_RGBA: u32 = 0x100;

static SERVICE: LazyLock<Result<Option<PhotoDna>>> = LazyLock::new(|| {
    let key = ENV.PHOTODNA_API_KEY.trim();
    if key.is_empty() || key.eq_ignore_ascii_case("none") {
        return Ok(None);
    }
    let mut headers = HeaderMap::new();
    let mut key =
        HeaderValue::from_str(key).wrap_err("invalid `PHOTODNA_API_KEY`")?;
    key.set_sensitive(true);
    headers.insert("Ocp-Apim-Subscription-Key", key);
    let client = reqwest::Client::builder()
        .default_headers(headers)
        .redirect(reqwest::redirect::Policy::none())
        .timeout(REQUEST_TIMEOUT)
        .build()
        .wrap_err("initializing PhotoDNA HTTP client")?;
    if ENV.PHOTODNA_SDK_ROOT.is_empty() {
        return Err(eyre!("`PHOTODNA_SDK_ROOT` is not set"));
    }
    let library_dir = Path::new(&ENV.PHOTODNA_SDK_ROOT).join("clientlibrary");
    let generator = Generator::load(&library_dir)
        .wrap_err("initializing PhotoDNA hash generator")?;
    Ok(Some(PhotoDna {
        generator: Arc::new(Mutex::new(generator)),
        client,
        capacity: Arc::new(Semaphore::new(4)),
    }))
});

struct PhotoDna {
    generator: Arc<Mutex<Generator>>,
    client: reqwest::Client,
    capacity: Arc<Semaphore>,
}

type GeneratorInit = unsafe extern "C" fn(
    library_dir: *const c_char,
    max_threads: i32,
) -> *mut c_void;
type EdgeHash = unsafe extern "C" fn(
    instance: *mut c_void,
    image_data: *const u8,
    hash: *mut u8,
    width: i32,
    height: i32,
    stride: i32,
    options: u32,
) -> i32;

/// The PhotoDNA Edge Hash Generator SDK, loaded at runtime so builds don't need the proprietary SDK.
struct Generator {
    instance: *mut c_void,
    edge_hash: EdgeHash,
    _library: libloading::Library,
}

// SAFETY: the SDK instance is only accessed through a `Mutex`.
unsafe impl Send for Generator {}

impl Generator {
    fn load(library_dir: &Path) -> Result<Self> {
        let file_name = match (std::env::consts::OS, std::env::consts::ARCH) {
            ("linux", "x86_64") => {
                format!("libEdgeHashGenerator.so.{SDK_VERSION}")
            }
            ("linux", "aarch64") => {
                format!("libEdgeHashGenerator-arm64.so.{SDK_VERSION}")
            }
            ("macos", "x86_64") => {
                format!("libEdgeHashGenerator-macos.so.{SDK_VERSION}")
            }
            ("macos", "aarch64") => {
                format!("libEdgeHashGenerator-arm64-macos.so.{SDK_VERSION}")
            }
            (os, arch) => {
                return Err(eyre!("PhotoDNA is not supported on {os} {arch}"));
            }
        };
        let library_dir_c =
            CString::new(library_dir.to_string_lossy().into_owned())
                .wrap_err("invalid `PHOTODNA_SDK_ROOT`")?;
        // SAFETY: the signatures match `PhotoDnaEdgeHashGenerator.h` from SDK 1.05,
        // and `_library` keeps the copied function pointers valid.
        unsafe {
            let library = libloading::Library::new(library_dir.join(file_name))
                .wrap_err("loading PhotoDNA library")?;
            let init = *library
                .get::<GeneratorInit>(b"EdgeHashGeneratorInit\0")
                .wrap_err("loading `EdgeHashGeneratorInit`")?;
            let edge_hash = *library
                .get::<EdgeHash>(b"PhotoDnaEdgeHash\0")
                .wrap_err("loading `PhotoDnaEdgeHash`")?;
            let instance = init(library_dir_c.as_ptr(), 1);
            if instance.is_null() {
                return Err(eyre!("PhotoDNA library failed to initialize"));
            }
            Ok(Self {
                instance,
                edge_hash,
                _library: library,
            })
        }
    }

    fn hash(&self, image: &RgbaImage) -> Result<[u8; HASH_SIZE]> {
        let mut hash = [0; HASH_SIZE];
        // SAFETY: `image` holds `width * height` tightly packed RGBA pixels
        // (stride 0), and `hash` fits a binary Edge V2 hash.
        let code = unsafe {
            (self.edge_hash)(
                self.instance,
                image.as_raw().as_ptr(),
                hash.as_mut_ptr(),
                image.width() as i32,
                image.height() as i32,
                0,
                HASH_FORMAT_EDGE_V2 | PIXEL_FORMAT_RGBA,
            )
        };
        if code < 0 {
            return Err(eyre!("PhotoDNA hashing failed with code {code}"));
        }
        Ok(hash)
    }
}

/// A confirmed PhotoDNA match that prevents an image from being uploaded.
#[derive(Debug, thiserror::Error)]
#[error("image upload is not permitted")]
pub struct ImageNotPermitted;

/// Screens the original image, rejecting only confirmed PhotoDNA matches.
/// Missing configuration and screening failures allow the upload to continue.
pub async fn check_image(bytes: Bytes) -> Result<(), ApiError> {
    let service = match SERVICE.as_ref() {
        Ok(Some(service)) => service,
        Ok(None) => {
            tracing::debug!(
                "skipping PhotoDNA check because no subscription key is configured"
            );
            return Ok(());
        }
        Err(error) => {
            tracing::debug!(error = %error, "PhotoDNA initialization failed, allowing upload");
            return Ok(());
        }
    };
    let permit =
        service.capacity.clone().try_acquire_owned().map_err(|_| {
            ApiError::RateLimit(eyre!("image screening capacity exhausted"))
        })?;
    let permit = Arc::new(permit);
    let worker_permit = permit.clone();
    let generator = service.generator.clone();
    let (sender, mut receiver) = mpsc::channel(1);
    let _worker = tokio::task::spawn_blocking(move || {
        let _permit = worker_permit;
        let result = image_hashes(
            bytes,
            |image| {
                if sender.is_closed() {
                    return Err(eyre!("image screening stopped"));
                }
                let pixels = image.into_rgba8();
                let generator = generator
                    .lock()
                    .map_err(|_| eyre!("image screening is unavailable"))?;
                let hash = generator
                    .hash(&pixels)
                    .wrap_err("generating image hash")?;
                Ok(STANDARD.encode(hash))
            },
            |hashes| {
                sender
                    .blocking_send(hashes)
                    .wrap_err("sending image hashes for screening")
            },
        );
        if let Err(error) = result {
            tracing::debug!(error = %error, "PhotoDNA image hashing stopped");
        }
    });
    let matched = tokio::time::timeout(Duration::from_secs(60), async {
        while let Ok(Some(hashes)) =
            tokio::time::timeout(REQUEST_TIMEOUT, receiver.recv()).await
        {
            if check_hashes(&service.client, &ENV.PHOTODNA_API_URL, &hashes)
                .await
            {
                return true;
            }
        }
        false
    })
    .await
    .unwrap_or(false);
    tracing::debug!(matched, "PhotoDNA upload check finished");
    if matched {
        Err(ApiError::Request(eyre::Report::new(ImageNotPermitted)))
    } else {
        Ok(())
    }
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct MatchRequest<'a> {
    data_representation: &'a str,
    value: &'a str,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct MatchResponse {
    match_results: Vec<MatchResult>,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct MatchResult {
    status: MatchStatus,
    #[serde(default)]
    is_match: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct MatchStatus {
    code: u32,
}

impl MatchResponse {
    fn is_match(&self) -> bool {
        self.match_results
            .iter()
            .any(|result| result.status.code == 3000 && result.is_match)
    }
}

async fn check_hashes(
    client: &reqwest::Client,
    url: &str,
    hashes: &[String],
) -> bool {
    let url = format!("{}/MatchHash", url.trim_end_matches('/'));
    let body: Vec<_> = hashes
        .iter()
        .map(|hash| MatchRequest {
            data_representation: "PreHashV2",
            value: hash,
        })
        .collect();
    for attempt in 0..2 {
        let response = client.post(&url).json(&body).send().await;
        let retry = match &response {
            Ok(response) => {
                response.status().is_server_error()
                    || response.status()
                        == reqwest::StatusCode::TOO_MANY_REQUESTS
            }
            Err(error) => error.is_timeout() || error.is_connect(),
        };
        if retry && attempt == 0 {
            tokio::time::sleep(Duration::from_millis(250)).await;
            continue;
        }
        let http_status = response
            .as_ref()
            .ok()
            .map(|response| response.status().as_u16());
        let Ok(response) =
            response.and_then(reqwest::Response::error_for_status)
        else {
            tracing::debug!(
                ?http_status,
                "PhotoDNA HTTP request failed, allowing upload"
            );
            return false;
        };
        return match response.json::<MatchResponse>().await {
            Ok(response) => {
                let matched = response.is_match();
                tracing::debug!(
                    status_codes = ?response.match_results.iter().map(|result| result.status.code).collect::<Vec<_>>(),
                    matched,
                    "PhotoDNA API response received"
                );
                matched
            }
            Err(_) => {
                tracing::debug!(
                    "invalid PhotoDNA API response, allowing upload"
                );
                false
            }
        };
    }
    false
}

fn image_hashes(
    bytes: Bytes,
    mut hash: impl FnMut(DynamicImage) -> Result<String>,
    mut send: impl FnMut(Vec<String>) -> Result<()>,
) -> Result<()> {
    let mut hashes = Vec::with_capacity(MAX_HASHES_PER_REQUEST);
    let result = visit_image_frames(bytes, |image| {
        hashes.push(hash(image)?);
        if hashes.len() == MAX_HASHES_PER_REQUEST {
            send(std::mem::take(&mut hashes))?;
        }
        Ok(())
    });
    if !hashes.is_empty() {
        send(hashes)?;
    }
    result
}

fn visit_image_frames(
    bytes: Bytes,
    mut visit: impl FnMut(DynamicImage) -> Result<()>,
) -> Result<()> {
    let mut limits = Limits::default();
    limits.max_image_width = Some(16384);
    limits.max_image_height = Some(16384);
    limits.max_alloc = Some(MAX_DECODED_BYTES);
    let mut reader = ImageReader::new(Cursor::new(&bytes))
        .with_guessed_format()
        .wrap_err("reading uploaded image format")?;
    reader.limits(limits.clone());
    let mut pixels = 0;
    let mut frame_count = 0;
    let mut add_image = |image: DynamicImage| {
        pixels += u64::from(image.width()) * u64::from(image.height());
        if frame_count >= MAX_FRAMES || pixels > MAX_PIXELS {
            return Err(eyre!("image exceeds screening limits"));
        }
        visit(image)?;
        frame_count += 1;
        Ok(())
    };
    let frames = match reader.format() {
        Some(ImageFormat::Gif) => {
            let mut decoder = GifDecoder::new(Cursor::new(&bytes))
                .wrap_err("decoding uploaded GIF")?;
            limit_decoder(&mut decoder, limits)?;
            Some(decoder.into_frames())
        }
        Some(ImageFormat::Png) => {
            let mut decoder = PngDecoder::new(Cursor::new(&bytes))
                .wrap_err("decoding uploaded PNG")?;
            limit_decoder(&mut decoder, limits.clone())?;
            if decoder
                .is_apng()
                .wrap_err("reading uploaded PNG animation")?
            {
                // APNG may have a separate default image outside the animation.
                let mut default_image = ImageReader::with_format(
                    Cursor::new(&bytes),
                    ImageFormat::Png,
                );
                default_image.limits(limits);
                add_image(
                    default_image
                        .decode()
                        .wrap_err("decoding uploaded image")?,
                )?;
                Some(
                    decoder
                        .apng()
                        .wrap_err("decoding uploaded APNG")?
                        .into_frames(),
                )
            } else {
                None
            }
        }
        Some(ImageFormat::WebP) => {
            let mut decoder = WebPDecoder::new(Cursor::new(&bytes))
                .wrap_err("decoding uploaded WebP")?;
            limit_decoder(&mut decoder, limits)?;
            if decoder.has_animation() {
                Some(decoder.into_frames())
            } else {
                None
            }
        }
        _ => None,
    };
    if let Some(frames) = frames {
        for frame in frames {
            let frame = frame.wrap_err("decoding uploaded animation frame")?;
            add_image(DynamicImage::ImageRgba8(frame.into_buffer()))?;
        }
    } else {
        add_image(reader.decode().wrap_err("decoding uploaded image")?)?;
    }
    if frame_count == 0 {
        return Err(eyre!("image contains no frames"));
    }
    Ok(())
}

fn limit_decoder(
    decoder: &mut impl ImageDecoder,
    limits: Limits,
) -> Result<()> {
    let (width, height) = decoder.dimensions();
    if u64::from(width) * u64::from(height) > MAX_DECODED_BYTES / 4 {
        return Err(eyre!("image exceeds screening limits"));
    }
    decoder
        .set_limits(limits)
        .wrap_err("limiting uploaded image")
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{Frame, RgbaImage};
    use serde_json::json;

    #[test]
    fn match_request_uses_prehash_v2() {
        let body = serde_json::to_value([MatchRequest {
            data_representation: "PreHashV2",
            value: "hash",
        }])
        .unwrap();
        assert_eq!(
            body,
            json!([{"DataRepresentation": "PreHashV2", "Value": "hash"}])
        );
    }

    #[test]
    fn only_successful_matches_are_blocked() {
        for (body, matched) in [
            (
                json!({"MatchResults": [{"Status": {"Code": 3000}, "IsMatch": false}]}),
                false,
            ),
            (
                json!({"MatchResults": [{"Status": {"Code": 3000}, "IsMatch": true}]}),
                true,
            ),
            (
                json!({"MatchResults": [{"Status": {"Code": 4000}, "IsMatch": false}]}),
                false,
            ),
            (
                json!({"MatchResults": [{"Status": {"Code": 4000}, "IsMatch": true}]}),
                false,
            ),
            (json!({"MatchResults": []}), false),
            (json!({"MatchResults": [{"Status": {"Code": 3000}}]}), false),
            (json!({}), false),
            (
                json!({"MatchResults": [
                    {"Status": {"Code": 3000}, "IsMatch": false},
                    {"Status": {"Code": 3000}, "IsMatch": false}
                ]}),
                false,
            ),
            (
                json!({"MatchResults": [
                    {"Status": {"Code": 4000}},
                    {"Status": {"Code": 3000}, "IsMatch": true}
                ]}),
                true,
            ),
        ] {
            let result = serde_json::from_value::<MatchResponse>(body)
                .map(|response| response.is_match())
                .unwrap_or(false);
            assert_eq!(result, matched);
        }
    }

    fn animated_gif(frames: usize) -> Bytes {
        let mut output = Vec::new();
        let mut encoder = image::codecs::gif::GifEncoder::new(&mut output);
        for index in 0..frames {
            let pixels = RgbaImage::from_pixel(
                50,
                50,
                image::Rgba([index as u8, 0, 0, 255]),
            );
            encoder.encode_frame(Frame::new(pixels)).unwrap();
        }
        drop(encoder);
        Bytes::from(output)
    }

    #[test]
    fn screens_later_animation_frames() {
        let mut batches = Vec::new();
        image_hashes(
            animated_gif(7),
            |image| Ok(image.to_rgba8().get_pixel(0, 0)[0].to_string()),
            |hashes| {
                batches.push(hashes);
                Ok(())
            },
        )
        .unwrap();
        assert_eq!(batches, [vec!["0", "1", "2", "3", "4"], vec!["5", "6"]]);
    }

    #[test]
    fn stops_hashing_when_screening_fails() {
        let mut frames = 0;
        let mut batches = Vec::new();
        let result = image_hashes(
            animated_gif(3),
            |_| {
                frames += 1;
                if frames == 2 {
                    Err(eyre!("image cannot be screened"))
                } else {
                    Ok("hash".to_string())
                }
            },
            |hashes| {
                batches.push(hashes);
                Ok(())
            },
        );
        assert!(result.is_err());
        assert_eq!(frames, 2);
        assert_eq!(batches, [vec!["hash"]]);
    }

    #[test]
    fn preserves_hashes_when_animation_exceeds_limits() {
        let mut batches = Vec::new();
        let result = image_hashes(
            animated_gif(MAX_FRAMES + 1),
            |_| Ok("hash".to_string()),
            |hashes| {
                batches.push(hashes);
                Ok(())
            },
        );
        assert!(result.is_err());
        assert_eq!(batches.iter().map(Vec::len).sum::<usize>(), MAX_FRAMES);
        assert!(
            batches
                .iter()
                .all(|batch| batch.len() <= MAX_HASHES_PER_REQUEST)
        );
        assert_eq!(batches.last().unwrap().len(), 1);
    }

    #[test]
    fn rejects_invalid_image_data() {
        assert!(
            image_hashes(
                Bytes::from_static(b"invalid image"),
                |_| panic!("invalid data must never be hashed"),
                |_| panic!("invalid data must never be sent"),
            )
            .is_err()
        );
    }

    #[actix_rt::test]
    async fn only_confirmed_http_matches_block_uploads() {
        use actix_web::{App, HttpResponse, HttpServer, web};

        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let server = HttpServer::new(|| App::new()
            .route("/match/MatchHash", web::post().to(|body: web::Json<serde_json::Value>| async move {
                assert_eq!(body.as_array().unwrap().len(), MAX_HASHES_PER_REQUEST);
                for (index, request) in body.as_array().unwrap().iter().enumerate() {
                    assert_eq!(request["DataRepresentation"], "PreHashV2");
                    assert_eq!(request["Value"], index.to_string());
                }
                HttpResponse::Ok().json(json!({"MatchResults": [
                    {"Status": {"Code": 3000}, "IsMatch": false},
                    {"Status": {"Code": 3000}, "IsMatch": false},
                    {"Status": {"Code": 3000}, "IsMatch": false},
                    {"Status": {"Code": 3000}, "IsMatch": false},
                    {"Status": {"Code": 3000}, "IsMatch": true}
                ]}))
            }))
            .route("/clear/MatchHash", web::post().to(|| async {
                HttpResponse::Ok().json(json!({"MatchResults": [{"Status": {"Code": 3000}, "IsMatch": false}]}))
            }))
            .route("/invalid/MatchHash", web::post().to(|| async { HttpResponse::Ok().body("invalid") }))
            .route("/unavailable/MatchHash", web::post().to(|| async { HttpResponse::ServiceUnavailable().finish() }))
            .route("/unauthorized/MatchHash", web::post().to(|| async { HttpResponse::Unauthorized().finish() }))
        ).workers(1).listen(listener).unwrap().run();
        let handle = server.handle();
        actix_rt::spawn(server);
        let client = reqwest::Client::builder()
            .timeout(REQUEST_TIMEOUT)
            .build()
            .unwrap();
        let hashes: Vec<_> = (0..MAX_HASHES_PER_REQUEST)
            .map(|index| index.to_string())
            .collect();
        for (path, allowed) in [
            ("match", false),
            ("clear", true),
            ("invalid", true),
            ("unavailable", true),
            ("unauthorized", true),
        ] {
            assert_eq!(
                !check_hashes(
                    &client,
                    &format!("http://{address}/{path}"),
                    &hashes
                )
                .await,
                allowed
            );
        }
        handle.stop(true).await;
    }
}

//! Theseus error type
use crate::data::ModLoader;
use crate::state::MinecraftAuthenticationError;
use crate::util;
use crate::util::io::IOError;
use crate::util::jre::JREError;
use async_tungstenite::tungstenite;
use data_url::DataUrlError;
use hickory_resolver::net::NetError;
use serde::{Deserialize, Serialize};
use sqlx::migrate::MigrateError;
use std::path::StripPrefixError;
use std::time::Duration;
use zip::result::ZipError;

#[derive(Serialize, Deserialize, Debug, thiserror::Error)]
#[error("{description}")]
pub struct LabrinthError {
    pub error: String,
    pub description: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub route: Option<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SharedInstanceUnavailableReason {
    Deleted,
    AccessRevoked,
    Quarantined,
}

impl std::fmt::Display for SharedInstanceUnavailableReason {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Deleted => write!(fmt, "deleted"),
            Self::AccessRevoked => write!(fmt, "access_revoked"),
            Self::Quarantined => write!(fmt, "quarantined"),
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ErrorKind {
    #[error("pack sync inputs changed during preparation")]
    PackSyncChanged,

    #[error("filesystem error: {0}")]
    FSError(String),

    #[error("serialization error (INI): {0}")]
    INIError(#[from] serde_ini::de::Error),

    #[error("serialization error (JSON): {0}")]
    JSONError(#[from] serde_json::Error),

    #[error("serialization error (NBT): {0}")]
    NBTError(#[from] quartz_nbt::io::NbtIoError),

    #[error("NBT data structure error: {0}")]
    NBTReprError(#[from] quartz_nbt::NbtReprError),

    #[error("serialization error (websocket): {0}")]
    WebsocketSerializationError(
        #[from] ariadne::networking::serialization::SerializationError,
    ),

    #[error("error parsing UUID: {0}")]
    UUIDError(#[from] uuid::Error),

    #[error("error parsing URL: {0}")]
    URLError(#[from] url::ParseError),

    #[error("unable to read {0} from any source")]
    NoValueFor(String),

    #[error("metadata error: {0}")]
    MetadataError(#[from] anyhow::Error),

    #[error("I/O error: {0}")]
    IOError(#[from] util::io::IOError),

    #[error("I/O (std) error: {0}")]
    StdIOError(#[from] std::io::Error),

    #[error("launcher error: {0}")]
    LauncherError(String),

    #[error("error fetching URL: {0}")]
    FetchError(#[from] reqwest::Error),

    #[error("too many API errors, try again in {0} minutes")]
    ApiIsDownError(u32),

    #[error("too many requests, retry in {}", format_duration(*.retry_in))]
    RateLimited { retry_in: Duration },

    #[error("websocket error: {0}")]
    WSError(#[from] async_tungstenite::tungstenite::Error),

    #[error("websocket closed before {0} could be received")]
    WSClosedError(String),

    #[error("incorrect SHA1 hash for download: {0} != {1}")]
    HashError(String, String),

    #[error("regex error: {0}")]
    RegexError(#[from] regex::Error),

    #[error("paths stored in the database need to be valid UTF-8: {0}")]
    UTFError(std::path::PathBuf),

    #[error("invalid input: {0}")]
    InputError(String),

    #[error("shared instance unavailable: {0}")]
    SharedInstanceUnavailable(SharedInstanceUnavailableReason),

    #[error("shared instances api request failed")]
    SharedInstancesApiError,

    #[error("you are not allowed to share {0}")]
    SharedInstanceFileNotAllowed(String),

    #[error("join handle error: {0}")]
    JoinError(#[from] tokio::task::JoinError),

    #[error("recv error: {0}")]
    RecvError(#[from] tokio::sync::oneshot::error::RecvError),

    #[error("error acquiring semaphore: {0}")]
    AcquireError(#[from] tokio::sync::AcquireError),

    #[error("instance {0} is not managed by the app")]
    UnmanagedInstanceError(String),

    #[error("not logged in")]
    NotLoggedIn,

    #[error("JRE error: {0}")]
    JREError(#[from] crate::util::jre::JREError),

    #[error("error parsing date: {0}")]
    ChronoParseError(#[from] chrono::ParseError),

    #[error("event error: {0}")]
    EventError(#[from] crate::event::EventError),

    #[error("zip error: {0}")]
    ZipError(#[from] ZipError),

    #[error("file watching error: {0}")]
    NotifyError(#[from] notify::Error),

    #[error("error stripping prefix: {0}")]
    StripPrefixError(#[from] std::path::StripPrefixError),

    #[cfg(feature = "tauri")]
    #[error("Tauri error: {0}")]
    TauriError(#[from] tauri::Error),

    #[error("error interacting with database: {0}")]
    Sqlx(#[from] sqlx::Error),

    #[error("error while applying migrations: {0}")]
    SqlxMigrate(#[from] sqlx::migrate::MigrateError),

    #[error("error resolving DNS: {0}")]
    DNSError(#[from] NetError),

    #[error("an online profile for {user_name} is not available")]
    OnlineMinecraftProfileUnavailable { user_name: String },

    #[error("invalid data URL: {0}")]
    InvalidDataUrl(#[from] DataUrlError),

    #[error("invalid data URL: {0}")]
    InvalidDataUrlBase64(#[from] data_url::forgiving_base64::InvalidBase64),

    #[error("invalid PNG")]
    InvalidPng,

    #[error("invalid PNG: {0}")]
    PngDecodingError(#[from] png::DecodingError),

    #[error("PNG encoding error: {0}")]
    PngEncodingError(#[from] png::EncodingError),

    #[error(
        "a skin texture must have a dimension of either 64x64 or 64x32 pixels"
    )]
    InvalidSkinTexture,

    #[cfg(windows)]
    #[error("Windows error: {0}")]
    WindowsError(#[from] windows_core::Error),

    #[error("zbus error: {0}")]
    ZbusError(#[from] zbus::Error),

    #[error("deserialization error: {0}")]
    DeserializationError(#[from] serde::de::value::Error),

    #[error("Discord IPC error: {0}")]
    DiscordRichPresenceError(#[from] discord_rich_presence::error::Error),

    #[error("login canceled")]
    LoginCanceled,

    #[error("no loader version selected for {}", .loader.as_str())]
    NoLoaderVersion { loader: ModLoader },

    #[error("failed to fetch player profile")]
    NoMinecraftProfile,

    #[error("moving app directory")]
    DirectoryMoveFailed,

    #[error(
        "not enough space: the destination needs at least {required} bytes plus working space for the app-directory move"
    )]
    NotEnoughSpace { required: u64 },

    #[error("{0}")]
    ContentStoreError(String),

    #[error("{0}")]
    InvalidPack(InvalidPack),
}

#[derive(Debug, derive_more::Display)]
pub enum InvalidPack {
    #[display("only resource packs and data packs can be synced")]
    UnsupportedType,
    #[display("data pack syncing is currently disabled")]
    DataPacksDisabled,
    #[display("invalid pack file path")]
    InvalidPath,
    #[display("the pack is not registered")]
    NotRegistered,
    #[display("the pack's compatibility could not be verified")]
    UnverifiedCompatibility,
    #[display("choose a valid pack zip file")]
    NotZip,
    #[display("choose a pack zip containing pack.mcmeta")]
    MissingMetadata,
    #[display("the pack metadata is too large")]
    MetadataTooLarge,
    #[display("this file is not a pack of the selected type")]
    WrongType,
}

fn format_duration(duration: Duration) -> String {
    let seconds = duration
        .as_secs()
        .saturating_add(u64::from(duration.subsec_nanos() > 0));
    let plural = |unit: u64| if unit == 1 { "" } else { "s" };

    if seconds <= 59 {
        return format!("{seconds} second{}", plural(seconds));
    }

    let minutes = seconds / 60;
    let rem_seconds = seconds % 60;

    if rem_seconds == 0 {
        return format!("{minutes} minutes");
    }

    format!(
        "{minutes} minute{} and {rem_seconds} second{}",
        plural(minutes),
        plural(rem_seconds)
    )
}

pub type Error = anyhow::Error;

impl ErrorKind {
    fn code(&self) -> &'static str {
        match self {
            Self::SharedInstanceUnavailable(reason) => match reason {
                SharedInstanceUnavailableReason::Deleted => {
                    "shared_instance_deleted"
                }
                SharedInstanceUnavailableReason::AccessRevoked => {
                    "shared_instance_access_revoked"
                }
                SharedInstanceUnavailableReason::Quarantined => {
                    "shared_instance_quarantined"
                }
            },
            Self::SharedInstancesApiError => "shared_instances_api_error",
            Self::SharedInstanceFileNotAllowed(_) => {
                "shared_instance_file_not_allowed"
            }
            Self::HashError(_, _) => "hash_error",
            Self::LoginCanceled => "login_canceled",
            Self::NotLoggedIn => "not_logged_in",
            Self::NoLoaderVersion { .. } => "no_loader_version",
            Self::NoMinecraftProfile => "minecraft_auth_no_profile",
            Self::LauncherError(_) => "launcher_error",
            Self::JREError(_) => "java_error",
            Self::NoValueFor(_) | Self::MetadataError(_) => "metadata_error",
            Self::FetchError(_)
            | Self::ApiIsDownError(_)
            | Self::WSError(_)
            | Self::WSClosedError(_)
            | Self::DNSError(_)
            | Self::RateLimited { .. } => "network_error",
            Self::ZipError(_) => "archive_error",
            Self::DeserializationError(_) | Self::StripPrefixError(_) => {
                "path_error"
            }
            Self::FSError(_)
            | Self::IOError(_)
            | Self::StdIOError(_)
            | Self::UTFError(_) => "filesystem_error",
            Self::INIError(_) | Self::JSONError(_) => "parse_error",
            Self::Sqlx(_) | Self::SqlxMigrate(_) => "database_error",
            _ => "internal_error",
        }
    }
}

/// `downcast_ref` stops at the outermost kind, so this also yields kinds
/// further down the chain.
pub fn error_kinds(error: &Error) -> impl Iterator<Item = &ErrorKind> {
    error
        .downcast_ref::<ErrorKind>()
        .into_iter()
        .chain(error.chain().filter_map(|cause| cause.downcast_ref()))
}

pub fn error_code(error: &Error) -> &'static str {
    let permission_denied = error.chain().any(|cause| {
        cause.downcast_ref::<std::io::Error>().is_some_and(|error| {
            error.kind() == std::io::ErrorKind::PermissionDenied
        })
    });

    if error_kinds(error)
        .any(|kind| matches!(kind, ErrorKind::DirectoryMoveFailed))
    {
        if error_kinds(error)
            .any(|kind| matches!(kind, ErrorKind::NotEnoughSpace { .. }))
        {
            "directory_move_no_space"
        } else if permission_denied {
            "directory_move_permission_denied"
        } else {
            "directory_move_failed"
        }
    } else if permission_denied {
        "permission_denied"
    } else if let Some(kind) = error.downcast_ref::<ErrorKind>() {
        kind.code()
    } else if let Some(error) =
        error.downcast_ref::<MinecraftAuthenticationError>()
    {
        error.code()
    } else if error.is::<LabrinthError>() {
        "api_error"
    } else if error.is::<reqwest::Error>()
        || error.is::<tungstenite::Error>()
        || error.is::<NetError>()
    {
        "network_error"
    } else if error.is::<JREError>() {
        "java_error"
    } else if error.is::<ZipError>() {
        "archive_error"
    } else if error.is::<serde::de::value::Error>()
        || error.is::<StripPrefixError>()
    {
        "path_error"
    } else if error.is::<IOError>() || error.is::<std::io::Error>() {
        "filesystem_error"
    } else if error.is::<serde_ini::de::Error>()
        || error.is::<serde_json::Error>()
    {
        "parse_error"
    } else if error.is::<sqlx::Error>() || error.is::<MigrateError>() {
        "database_error"
    } else {
        "internal_error"
    }
}

pub type Result<T> = core::result::Result<T, Error>;

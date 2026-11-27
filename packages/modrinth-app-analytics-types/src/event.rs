use serde::{Deserialize, Serialize};
use smol_str::SmolStr;

use crate::enums::{
    AppArch, AppLoader, AppOs, MinecraftAuthErrorCode, MinecraftAuthStep,
};

#[derive(Debug, Clone, Serialize, Deserialize, strum::IntoStaticStr)]
pub enum AppEventType {
    /// The app was opened.
    AppStart {
        arch: AppArch,
        os: AppOs,
        app_version: SmolStr,
    },
    /// An instance was launched.
    InstanceStart {
        #[serde(flatten)]
        info: InstanceInfo,
    },
    /// An instance was created.
    InstanceCreated {
        #[serde(flatten)]
        info: InstanceInfo,
    },
    /// The user signed in to a Microsoft account.
    MicrosoftSignIn {
        /// Number of Microsoft accounts signed in after this sign-in, so `1`
        /// for the first account.
        nth_account: u32,
        /// Whether the account wasn't already signed in before.
        is_new_account: bool,
        /// Time from starting to sign in to being signed in.
        duration_ms: u64,
    },
    /// Signing in to a Microsoft account failed.
    MicrosoftSignInFailed {
        #[serde(flatten)]
        reason: MicrosoftSignInFailureReason,
        /// Time from starting to sign in to the failure.
        duration_ms: u64,
    },
    /// The user removed a Microsoft account.
    MicrosoftSignOut {
        num_accounts_left: u32,
        /// Whether the removed account was the one in use.
        was_active_account: bool,
    },
    /// The user signed in to their Modrinth account.
    ModrinthSignIn {
        is_sign_up: bool,
        /// Time from opening the sign-in page to being signed in.
        duration_ms: u64,
    },
    /// Signing in to a Modrinth account failed.
    ModrinthSignInFailed {
        is_sign_up: bool,
        reason: ModrinthSignInFailureReason,
        /// Time from opening the sign-in page to the failure.
        duration_ms: u64,
    },
    /// The user signed out of their Modrinth account.
    ModrinthSignOut {
        /// Whether the app signed out because refreshing the session failed,
        /// rather than the user signing out.
        is_refresh_failed: bool,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceInfo {
    pub loader: AppLoader,
    /// `None` for vanilla instances.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub loader_version: Option<SmolStr>,
    pub game_version: SmolStr,
}

/// Why signing in to a Microsoft account failed.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(tag = "reason", rename_all = "snake_case")]
pub enum MicrosoftSignInFailureReason {
    /// The user closed the sign-in window.
    Cancelled,
    /// The user didn't finish signing in before the sign-in window timed out.
    TimedOut,
    /// The Microsoft account doesn't own Minecraft.
    NoMinecraftProfile,
    /// Authenticating with Microsoft, Xbox or Minecraft services failed.
    Auth {
        code: MinecraftAuthErrorCode,
        /// `None` when the error didn't happen during a request.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        step: Option<MinecraftAuthStep>,
    },
    /// Any other error, such as a database or window error.
    Other,
}

/// Why signing in to a Modrinth account failed.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModrinthSignInFailureReason {
    /// The user cancelled signing in.
    Cancelled,
    /// The local server that receives the sign-in reply couldn't start.
    ReplyListenerFailed,
    /// The sign-in page couldn't be opened in the browser.
    BrowserOpenFailed,
    /// The sign-in reply couldn't be decrypted or was malformed.
    InvalidReply,
    /// Fetching the signed-in user from the Modrinth API failed.
    FetchUserFailed,
    /// Any other error, such as a database error.
    Other,
}

impl AppEventType {
    /// Returns the name stored in the `event_type` column.
    #[must_use]
    pub fn name(&self) -> &'static str {
        self.into()
    }
}

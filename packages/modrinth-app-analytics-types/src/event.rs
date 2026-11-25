use serde::{Deserialize, Serialize};
use smol_str::SmolStr;

use crate::enums::{AppArch, AppLoader, AppOs};

#[derive(
    Debug, Clone, PartialEq, Eq, Serialize, Deserialize, strum::IntoStaticStr,
)]
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
    /// The user signed in to a Microsoft account.
    MicrosoftSignIn {},
    /// Signing in to a Microsoft account failed.
    MicrosoftSignInFailed {},
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstanceInfo {
    pub loader: AppLoader,
    /// `None` for vanilla instances.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub loader_version: Option<SmolStr>,
    pub game_version: SmolStr,
}

impl AppEventType {
    /// Returns the name stored in the `event_type` column.
    #[must_use]
    pub fn name(&self) -> &'static str {
        self.into()
    }
}

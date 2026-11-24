use serde::{Deserialize, Serialize};

use crate::iso3166_1_alpha_2;

/// Loader of a Minecraft instance.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
    strum::IntoStaticStr,
)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum AppLoader {
    Vanilla,
    Forge,
    Fabric,
    Quilt,
    NeoForge,
}

impl AppLoader {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        self.into()
    }
}

/// OS the app is running on.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
    strum::IntoStaticStr,
    strum::EnumString,
)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum AppOs {
    Windows,
    Macos,
    Linux,
}

impl AppOs {
    /// Returns the operating system this binary was built for, if it is one
    /// the app supports.
    #[must_use]
    pub fn current() -> Option<Self> {
        std::env::consts::OS.parse().ok()
    }

    #[must_use]
    pub fn as_str(self) -> &'static str {
        self.into()
    }
}

/// CPU architecture the app runs on, named as in [`std::env::consts::ARCH`].
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
    strum::IntoStaticStr,
    strum::EnumString,
)]
pub enum AppArch {
    #[serde(rename = "x86")]
    #[strum(serialize = "x86")]
    X86,
    #[serde(rename = "x86_64")]
    #[strum(serialize = "x86_64")]
    X86_64,
    #[serde(rename = "arm")]
    #[strum(serialize = "arm")]
    Arm,
    #[serde(rename = "aarch64")]
    #[strum(serialize = "aarch64")]
    Aarch64,
}

impl AppArch {
    /// Returns the architecture this binary was built for, if it is one the
    /// app supports.
    #[must_use]
    pub fn current() -> Option<Self> {
        std::env::consts::ARCH.parse().ok()
    }

    #[must_use]
    pub fn as_str(self) -> &'static str {
        self.into()
    }
}

/// ISO 3166-1 alpha-2 code, plus `XX` for unknown and `T1` for Tor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Country([u8; 2]);

impl Country {
    /// Country used when the request has no valid country header.
    pub const UNKNOWN: Self = Self(*b"XX");
    pub const TOR: Self = Self(*b"T1");

    /// Parses a two-character code of uppercase ASCII letters and digits.
    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        let &[b0, b1] = value.as_bytes() else {
            return None;
        };

        iso3166_1_alpha_2::SORTED_CODES
            .iter()
            .chain([Country::UNKNOWN.as_str(), Country::TOR.as_str()].iter())
            .any(|code| code == &value)
            .then_some(Self([b0, b1]))
    }

    #[must_use]
    pub const fn as_bytes(&self) -> [u8; 2] {
        self.0
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        // should always be valid UTF-8
        std::str::from_utf8(&self.0).unwrap_or_default()
    }
}

//! Platform-related code
use daedalus::minecraft::{Os, OsRule};
use regex::Regex;

// OS detection
pub trait OsExt {
    /// Get the OS of the current system
    fn native() -> Self;

    /// Gets the OS + Arch of the current system
    fn native_arch() -> Self;
}

impl OsExt for Os {
    fn native_arch() -> Self {
        if std::env::consts::OS == "windows" {
            if std::env::consts::ARCH == "aarch64" {
                Os::WindowsArm64
            } else {
                Os::Windows
            }
        } else if std::env::consts::OS == "linux" {
            if std::env::consts::ARCH == "aarch64" {
                Os::LinuxArm64
            } else if std::env::consts::ARCH == "arm" {
                Os::LinuxArm32
            } else {
                Os::Linux
            }
        } else if std::env::consts::OS == "macos" {
            if std::env::consts::ARCH == "aarch64" {
                Os::OsxArm64
            } else {
                Os::Osx
            }
        } else {
            Os::Unknown
        }
    }

    fn native() -> Self {
        if std::env::consts::OS == "windows" {
            if std::env::consts::ARCH == "aarch64" {
                Os::WindowsArm64
            } else {
                Os::Windows
            }
        } else if std::env::consts::OS == "linux" {
            if std::env::consts::ARCH == "aarch64" {
                Os::LinuxArm64
            } else if std::env::consts::ARCH == "arm" {
                Os::LinuxArm32
            } else {
                Os::Linux
            }
        } else if std::env::consts::OS == "macos" {
            if std::env::consts::ARCH == "aarch64" {
                Os::OsxArm64
            } else {
                Os::Osx
            }
        } else {
            Os::Unknown
        };

        match std::env::consts::OS {
            "windows" => Self::Windows,
            "macos" => Self::Osx,
            "linux" => Self::Linux,
            _ => Self::Unknown,
        }
    }
}

// Bit width
#[cfg(target_pointer_width = "64")]
pub const ARCH_WIDTH: &str = "64";

#[cfg(target_pointer_width = "32")]
pub const ARCH_WIDTH: &str = "32";

// Platform rule handling
pub fn os_rule(rule: &OsRule) -> bool {
    let mut rule_match = true;

    if let Some(ref arch) = rule.arch {
        rule_match &= !matches!(arch.as_str(), "x86" | "arm");
    }

    if let Some(name) = &rule.name {
        rule_match &= &Os::native() == name || &Os::native_arch() == name;
    }

    if let Some(version) = &rule.version {
        if let Ok(regex) = Regex::new(version.as_str()) {
            rule_match &=
                regex.is_match(&sys_info::os_release().unwrap_or_default());
        }
    }

    rule_match
}

pub fn classpath_separator() -> &'static str {
    match Os::native_arch() {
        Os::Osx
        | Os::OsxArm64
        | Os::Linux
        | Os::LinuxArm32
        | Os::LinuxArm64
        | Os::Unknown => ":",
        Os::Windows | Os::WindowsArm64 => ";",
    }
}

//! Platform-related code
use daedalus::minecraft::{Os, OsRule};
use regex::Regex;

// OS detection
pub trait OsExt {
    /// Get the OS of the current system
    fn native() -> Self;

    /// Gets the OS + Arch of the current system
    fn native_arch(java_arch: &str) -> Self;

    /// Gets the OS from an OS + Arch
    fn get_os(&self) -> Self;
}

impl OsExt for Os {
    fn native() -> Self {
        match std::env::consts::OS {
            "windows" => Self::Windows,
            "macos" => Self::Osx,
            "linux" => Self::Linux,
            _ => Self::Unknown,
        }
    }

    fn native_arch(java_arch: &str) -> Self {
        if std::env::consts::OS == "windows" {
            if java_arch == "aarch64" {
                Os::WindowsArm64
            } else {
                Os::Windows
            }
        } else if std::env::consts::OS == "linux" {
            if java_arch == "aarch64" {
                Os::LinuxArm64
            } else if java_arch == "arm" {
                Os::LinuxArm32
            } else {
                Os::Linux
            }
        } else if std::env::consts::OS == "macos" {
            if java_arch == "aarch64" {
                Os::OsxArm64
            } else {
                Os::Osx
            }
        } else {
            Os::Unknown
        }
    }

    fn get_os(&self) -> Self {
        match self {
            Os::OsxArm64 => Os::Osx,
            Os::LinuxArm32 => Os::Linux,
            Os::LinuxArm64 => Os::Linux,
            Os::WindowsArm64 => Os::Windows,
            _ => self.clone(),
        }
    }
}

// Bit width
#[cfg(target_pointer_width = "64")]
pub const ARCH_WIDTH: &str = "64";

#[cfg(target_pointer_width = "32")]
pub const ARCH_WIDTH: &str = "32";

// Platform rule handling
pub fn os_rule(
    rule: &OsRule,
    java_arch: &str,
    // Minecraft updated over 1.18.2 (supports MacOS Natively)
    minecraft_updated: bool,
) -> bool {
    let mut rule_match = true;

    if let Some(ref arch) = rule.arch {
        rule_match &= !matches!(arch.as_str(), "x86" | "arm");
    }

    if let Some(name) = &rule.name {
        if minecraft_updated
            && (name != &Os::LinuxArm64 || name != &Os::LinuxArm32)
        {
            rule_match &= Os::native() == name.get_os()
                || &Os::native_arch(java_arch) == name;
        } else {
            rule_match &= &Os::native_arch(java_arch) == name;
        }
    }

    if let Some(version) = &rule.version {
        if let Ok(regex) = Regex::new(version.as_str()) {
            rule_match &=
                regex.is_match(&sys_info::os_release().unwrap_or_default());
        }
    }

    rule_match
}

pub fn classpath_separator(java_arch: &str) -> &'static str {
    match Os::native_arch(java_arch) {
        Os::Osx
        | Os::OsxArm64
        | Os::Linux
        | Os::LinuxArm32
        | Os::LinuxArm64
        | Os::Unknown => ":",
        Os::Windows | Os::WindowsArm64 => ";",
    }
}

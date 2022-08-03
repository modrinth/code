//! Platform-related code
use daedalus::minecraft::{Os, OsRule};
use regex::Regex;

// OS detection
pub trait OsExt {
    /// Get the OS of the current system
    fn native() -> Self;
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
        rule_match &= match arch.as_str() {
            "x86" => cfg!(any(target_arch = "x86", target_arch = "x86_64")),
            "arm" => cfg!(target_arch = "arm"),
            _ => true,
        };
    }

    if let Some(name) = &rule.name {
        rule_match &= &Os::native() == name;
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
    match Os::native() {
        Os::Osx | Os::Linux | Os::Unknown => ":",
        Os::Windows => ";",
    }
}

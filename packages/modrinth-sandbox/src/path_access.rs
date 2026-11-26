use std::path::{Component, Path, PathBuf};

use anyhow::{Result, anyhow, bail};

/// Types which Windows or a common runtime executes, installs or imports when
/// they are opened.
#[cfg(windows)]
const EXECUTABLE_EXTENSIONS: &[&str] = &[
    "appinstaller",
    "application",
    "appref-ms",
    "appx",
    "appxbundle",
    "bat",
    "chm",
    "cmd",
    "com",
    "cpl",
    "diagcab",
    "dll",
    "drv",
    "exe",
    "gadget",
    "hlp",
    "hta",
    "inf",
    "ins",
    "isp",
    "jar",
    "jnlp",
    "js",
    "jse",
    "library-ms",
    "lnk",
    "msc",
    "msi",
    "msix",
    "msixbundle",
    "msp",
    "mst",
    "ocx",
    "pif",
    "ps1",
    "ps1xml",
    "psc1",
    "psd1",
    "psm1",
    "py",
    "pyc",
    "pyw",
    "pyz",
    "reg",
    "scf",
    "scr",
    "sct",
    "search-ms",
    "settingcontent-ms",
    "sh",
    "shb",
    "shs",
    "sys",
    "theme",
    "themepack",
    "url",
    "vb",
    "vbe",
    "vbs",
    "website",
    "ws",
    "wsc",
    "wsf",
    "wsh",
    "xbap",
    "xll",
];

/// Whether a path is unsafe to access on behalf of the sandboxed process.
/// These paths aren't necessarily blocked, just rejected in various places
/// e.g. default file dialog locations, open uri calls, linux request-access
///
/// This must not open the path, since it could be a network path
#[cfg(windows)]
pub fn is_risky_path(path: &Path) -> bool {
    // Only allows paths such as `C:\folder`. Accessing a network path could send
    // the user's credentials to a malicious server.
    let is_local = path.is_absolute()
        && matches!(
            path.components().next(),
            Some(Component::Prefix(prefix))
                if matches!(prefix.kind(), std::path::Prefix::Disk(_))
        );
    if !is_local {
        return true;
    }
    let has_risky_name = path.components().any(|component| match component {
        Component::Normal(name) => is_risky_name(name),
        _ => false,
    });
    has_risky_name || has_executable_extension(path)
}

#[cfg(not(windows))]
pub fn is_risky_path(_path: &Path) -> bool {
    false
}

#[cfg(windows)]
fn is_risky_name(name: &std::ffi::OsStr) -> bool {
    let Some(name) = name.to_str() else {
        return true;
    };
    // These can't be in a name, but can be used to refer to alternate data
    // streams or as wildcards in a file dialog
    if name.contains(|char: char| {
        char.is_control()
            || matches!(char, '<' | '>' | ':' | '"' | '|' | '?' | '*')
    }) {
        return true;
    }

    // Windows treats these as devices in every folder, with any extension
    let stem = name.split('.').next().unwrap_or(name);
    let stem = stem.trim_end_matches(' ').to_ascii_uppercase();
    if matches!(
        stem.as_str(),
        "CON" | "PRN" | "AUX" | "NUL" | "CONIN$" | "CONOUT$"
    ) {
        return true;
    }
    let port = stem.strip_prefix("COM").or(stem.strip_prefix("LPT"));
    let mut port = port.unwrap_or_default().chars();
    matches!(
        (port.next(), port.next()),
        (Some('0'..='9' | '\u{b9}' | '\u{b2}' | '\u{b3}'), None)
    )
}

#[cfg(windows)]
pub fn has_executable_extension(path: &Path) -> bool {
    let Some(name) = path.file_name() else {
        return false;
    };
    let Some(name) = name.to_str() else {
        return true;
    };
    // Windows ignores these
    let name = name.trim_end_matches(['.', ' ']);
    let Some((_, extension)) = name.rsplit_once('.') else {
        return false;
    };
    EXECUTABLE_EXTENSIONS
        .iter()
        .any(|executable| executable.eq_ignore_ascii_case(extension))
}

/// Which access the sandbox has to a path.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum PathPermission {
    ReadOnly,
    ReadWrite,
}

/// Whether paths can be exposed to the sandbox.
///
/// Rules also apply to everything inside of their path. If multiple rules apply to
/// a path, the last one takes precedence.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum PathRule {
    /// Can't be exposed at all, including by paths that the sandbox is explicitly
    /// given access to when it starts.
    HardBlock(PathBuf),
    /// Like [`PathRule::HardBlock`], but doesn't affect whether the path can be read.
    HardBlockWrite(PathBuf),
    /// Can't be exposed after the sandbox has started. The sandbox can still be given
    /// access to this path, or paths inside of it, when it starts.
    SoftBlock(PathBuf),
    /// Like [`PathRule::SoftBlock`], but doesn't affect whether the path can be read.
    SoftBlockWrite(PathBuf),
    /// Doesn't give the sandbox access to the path by itself,
    /// just permits access if selected by the user
    Unblock(PathBuf),
    /// Like [`PathRule::Unblock`], but doesn't affect whether the path can be written to.
    UnblockRead(PathBuf),
}

impl PathRule {
    fn path(&self) -> &Path {
        match self {
            Self::HardBlock(path)
            | Self::HardBlockWrite(path)
            | Self::SoftBlock(path)
            | Self::SoftBlockWrite(path)
            | Self::Unblock(path)
            | Self::UnblockRead(path) => path,
        }
    }

    fn read_level(&self) -> Option<BlockLevel> {
        match self {
            Self::HardBlock(_) => Some(BlockLevel::Hard),
            Self::SoftBlock(_) => Some(BlockLevel::Soft),
            Self::Unblock(_) | Self::UnblockRead(_) => {
                Some(BlockLevel::Unblocked)
            }
            Self::HardBlockWrite(_) | Self::SoftBlockWrite(_) => None,
        }
    }

    fn write_level(&self) -> Option<BlockLevel> {
        match self {
            Self::HardBlock(_) | Self::HardBlockWrite(_) => {
                Some(BlockLevel::Hard)
            }
            Self::SoftBlock(_) | Self::SoftBlockWrite(_) => {
                Some(BlockLevel::Soft)
            }
            Self::Unblock(_) => Some(BlockLevel::Unblocked),
            Self::UnblockRead(_) => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum BlockLevel {
    Unblocked,
    Soft,
    Hard,
}

#[derive(Debug)]
struct ResolvedRule {
    path: PathBuf,
    /// The path before resolving links, if it's different. Replacing one of those
    /// links would replace the path.
    original_path: Option<PathBuf>,
    rule: PathRule,
}

#[derive(Debug)]
pub struct PathRules {
    rules: Vec<ResolvedRule>,
}

impl PathRules {
    pub fn new(rules: impl IntoIterator<Item = PathRule>) -> Self {
        let mut path_rules = Self { rules: Vec::new() };
        for rule in rules {
            path_rules.push(rule);
        }
        path_rules
    }

    /// Whether reading a path is blocked
    ///
    /// This is used alongside is_risky_path for the same purposes,
    /// which is why this allows paths which contain a blocked path
    /// unlike calculate_expose_permission
    ///
    /// This doesn't depend on whether the path itself exists, so that the result
    /// can't be used to find out which files exist.
    pub fn is_blocked(&self, path: &Path) -> bool {
        let Some(path) = resolve_existing(path).map(normalize_path) else {
            return true;
        };
        self.rules
            .iter()
            .rev()
            .filter(|rule| path.starts_with(&rule.path))
            .find_map(|rule| rule.rule.read_level())
            .is_some_and(|level| level > BlockLevel::Unblocked)
    }

    /// Calculates which access to a path can be given to the sandbox after it has
    /// started.
    ///
    /// Fails if the path is blocked, or if it contains a blocked path.
    pub fn calculate_expose_permission(
        &self,
        path: &Path,
    ) -> Result<PathPermission> {
        self.calculate_permission(path, BlockLevel::Unblocked)
    }

    /// Calculates which access to a path can be given to the sandbox when it starts.
    ///
    /// Fails if the path is hard blocked, or if it contains a blocked path.
    pub fn calculate_bind_permission(
        &self,
        path: &Path,
    ) -> Result<PathPermission> {
        self.calculate_permission(path, BlockLevel::Soft)
    }

    /// `max_level` is the highest level at which access is still given.
    fn calculate_permission(
        &self,
        path: &Path,
        max_level: BlockLevel,
    ) -> Result<PathPermission> {
        let resolved = resolve_existing(path)
            .map(normalize_path)
            .ok_or_else(|| anyhow!("unable to resolve path {path:?}"))?;

        let given = normalize_path(path.to_path_buf());

        let mut read_level = BlockLevel::Unblocked;
        let mut read_rule = None;
        let mut write_level = BlockLevel::Unblocked;
        let mut contains_write_blocked = false;
        for ResolvedRule {
            path: rule_path,
            original_path,
            rule,
        } in &self.rules
        {
            let contains_rule = rule_path.starts_with(&resolved)
                || original_path.as_ref().is_some_and(|original| {
                    original.starts_with(&resolved)
                        || original.starts_with(&given)
                });
            if resolved.starts_with(rule_path) {
                if let Some(level) = rule.read_level() {
                    read_level = level;
                    read_rule = Some(rule);
                }
                if let Some(level) = rule.write_level() {
                    write_level = level;
                }
            } else if contains_rule {
                if rule.read_level() > Some(BlockLevel::Unblocked) {
                    bail!(
                        "refusing to expose {path:?} since it contains a blocked path: {rule:?}"
                    );
                }
                if rule.write_level() > Some(BlockLevel::Unblocked) {
                    contains_write_blocked = true;
                }
            }
        }

        if let Some(rule) = read_rule
            && read_level > max_level
        {
            bail!("refusing to expose {path:?} since it's blocked: {rule:?}");
        }
        if write_level > max_level || contains_write_blocked {
            Ok(PathPermission::ReadOnly)
        } else {
            Ok(PathPermission::ReadWrite)
        }
    }

    /// Adds a rule for a path that the sandbox has access to when it starts.
    ///
    /// Returns the permission that it should be bound with, which is read-only if
    /// writing is hard blocked or if it contains a path that must not be written to.
    pub fn add_bind(
        &mut self,
        path: &Path,
        permission: PathPermission,
    ) -> Result<PathPermission> {
        // Backends ignore paths that can't be resolved
        if resolve_existing(path).is_none() {
            return Ok(permission);
        }
        let permission = permission.min(self.calculate_bind_permission(path)?);
        self.push(match permission {
            PathPermission::ReadOnly => PathRule::UnblockRead(path.to_owned()),
            PathPermission::ReadWrite => PathRule::Unblock(path.to_owned()),
        });
        Ok(permission)
    }

    fn push(&mut self, rule: PathRule) {
        // An empty path would apply to everything
        if rule.path().as_os_str().is_empty() {
            return;
        }
        let original_path = normalize_path(rule.path().to_path_buf());
        let path = resolve_existing(rule.path())
            .map(normalize_path)
            .unwrap_or_else(|| original_path.clone());
        self.rules.push(ResolvedRule {
            original_path: (original_path != path).then_some(original_path),
            path,
            rule,
        });
    }
}

/// Makes paths comparable, since most filesystems on Windows and macOS ignore case.
pub fn normalize_path(path: PathBuf) -> PathBuf {
    let mut os_string = path.into_os_string();
    if cfg!(any(windows, target_os = "macos")) {
        os_string.make_ascii_lowercase();
    }

    #[cfg(windows)]
    {
        let encoded_bytes = os_string.as_os_str().as_encoded_bytes();
        if let Some(rest) = encoded_bytes.strip_prefix(b"\\\\?\\") {
            return PathBuf::from(unsafe {
                std::ffi::OsStr::from_encoded_bytes_unchecked(rest)
            });
        }
    }

    PathBuf::from(os_string)
}

/// Resolves links in the part of the path that exists.
pub fn resolve_existing(path: &Path) -> Option<PathBuf> {
    for ancestor in path.ancestors() {
        let Ok(resolved) = ancestor.canonicalize() else {
            continue;
        };
        let rest = path.strip_prefix(ancestor).ok()?;
        if rest.as_os_str().is_empty() {
            return Some(resolved);
        }
        return rest
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
            .then(|| resolved.join(rest));
    }
    None
}

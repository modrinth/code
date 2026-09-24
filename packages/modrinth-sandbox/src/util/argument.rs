use std::{
    borrow::Cow,
    ffi::{OsStr, OsString},
    path::{Path, PathBuf},
};

/// Wrapper over Cow<'static OsStr> implementing conversions from str/String, OsStr/OsString and Path/PathBuf
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SandboxArg(Cow<'static, OsStr>);

impl SandboxArg {
    pub fn as_os_str(&self) -> &OsStr {
        &self.0
    }

    pub fn into_os_string(self) -> OsString {
        self.0.into_owned()
    }
}

// OsStr conversions
impl From<Cow<'static, OsStr>> for SandboxArg {
    fn from(value: Cow<'static, OsStr>) -> Self {
        SandboxArg(value)
    }
}

impl From<&'static OsStr> for SandboxArg {
    fn from(value: &'static OsStr) -> Self {
        SandboxArg(Cow::Borrowed(value))
    }
}

impl From<OsString> for SandboxArg {
    fn from(value: OsString) -> Self {
        SandboxArg(Cow::Owned(value))
    }
}

// str conversions
impl From<Cow<'static, str>> for SandboxArg {
    fn from(value: Cow<'static, str>) -> Self {
        SandboxArg(match value {
            Cow::Borrowed(value) => Cow::Borrowed(OsStr::new(value)),
            Cow::Owned(value) => Cow::Owned(value.into()),
        })
    }
}

impl From<&'static str> for SandboxArg {
    fn from(value: &'static str) -> Self {
        SandboxArg(Cow::Borrowed(OsStr::new(value)))
    }
}

impl From<String> for SandboxArg {
    fn from(value: String) -> Self {
        SandboxArg(Cow::Owned(value.into()))
    }
}

// Path convesions
impl From<Cow<'static, Path>> for SandboxArg {
    fn from(value: Cow<'static, Path>) -> Self {
        SandboxArg(match value {
            Cow::Borrowed(value) => Cow::Borrowed(value.as_os_str()),
            Cow::Owned(value) => Cow::Owned(value.into_os_string()),
        })
    }
}

impl From<&'static Path> for SandboxArg {
    fn from(value: &'static Path) -> Self {
        SandboxArg(Cow::Borrowed(value.as_os_str()))
    }
}

impl From<PathBuf> for SandboxArg {
    fn from(value: PathBuf) -> Self {
        SandboxArg(Cow::Owned(value.into_os_string()))
    }
}

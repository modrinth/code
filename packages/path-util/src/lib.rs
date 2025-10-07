use itertools::Itertools;
use serde::{
    Deserialize, Deserializer, Serialize, Serializer,
    de::value::StringDeserializer,
};
use typed_path::{
    Utf8Component, Utf8TypedPathBuf, Utf8UnixComponent, Utf8UnixPathBuf,
};

#[derive(
    Eq, PartialEq, Hash, Debug, Clone, derive_more::Display, derive_more::Deref,
)]
#[repr(transparent)]
pub struct SafeRelativeUtf8UnixPathBuf(Utf8UnixPathBuf);

impl<'de> Deserialize<'de> for SafeRelativeUtf8UnixPathBuf {
    fn deserialize<D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Self, D::Error> {
        // When parsed successfully, the path is guaranteed to be free from leading backslashes
        // and Windows prefixes (e.g., `C:`)
        let Utf8TypedPathBuf::Unix(path) =
            Utf8TypedPathBuf::from(String::deserialize(deserializer)?)
        else {
            return Err(serde::de::Error::custom(
                "File path must be a Unix-style relative path",
            ));
        };

        let mut path_components = path.components().peekable();

        if path_components.peek().is_none() {
            return Err(serde::de::Error::custom("File path cannot be empty"));
        }

        // All components should be normal: a file or directory name, not `/`, or `..`,
        // and not refer to any reserved Windows device name. Also, at this point we may have
        // a pseudo-Unix path like `my\directory`, which we should reject by filtering out
        // backslashes to guarantee consistent cross-platform behavior when interpreting component
        // separators
        if !path_components.all(|component| {
            (component.is_normal() || component.is_current())
                && !component.as_str().contains('\\')
                && !is_reserved_windows_device_name(&component)
        }) {
            return Err(serde::de::Error::custom(
                "File path cannot contain any special component, prefix, reserved Windows device name, or backslashes",
            ));
        }

        Ok(Self(path))
    }
}

impl Serialize for SafeRelativeUtf8UnixPathBuf {
    fn serialize<S: Serializer>(
        &self,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let mut path_components = self.0.components().peekable();

        if path_components.peek().is_none() {
            return Err(serde::ser::Error::custom("File path cannot be empty"));
        }

        if !path_components.all(|component| {
            (component.is_normal() || component.is_current())
                && !component.as_str().contains('\\')
                && !is_reserved_windows_device_name(&component)
        }) {
            return Err(serde::ser::Error::custom(
                "File path cannot contain any special component, prefix, reserved Windows device name, or backslashes",
            ));
        }

        // Iterating over components does basic normalization by e.g. removing redundant
        // slashes and collapsing `.` components, so do that to produce a cleaner output
        // friendlier to the strict deserialization algorithm above
        self.0.components().join("/").serialize(serializer)
    }
}

impl TryFrom<String> for SafeRelativeUtf8UnixPathBuf {
    type Error = serde::de::value::Error;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        Self::deserialize(StringDeserializer::new(s))
    }
}

fn is_reserved_windows_device_name(component: &Utf8UnixComponent) -> bool {
    let file_name = component.as_str().to_ascii_uppercase();

    // Windows reserves some special DOS device names in every directory, which may be optionally
    // followed by an extension or alternate data stream name and be case insensitive. Trying to
    // write, read, or delete these files is usually not that useful even for malware, since they
    // mostly refer to console and printer devices, but it's best to avoid them entirely anyway.
    // References:
    // https://learn.microsoft.com/en-us/windows/win32/fileio/naming-a-file#naming-conventions
    // https://devblogs.microsoft.com/oldnewthing/20031022-00/?p=42073
    // https://github.com/wine-mirror/wine/blob/01269452e0fbb1f081d506bd64996590a553e2b9/dlls/ntdll/path.c#L66
    const RESERVED_WINDOWS_DEVICE_NAMES: &[&str] = &[
        "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5",
        "COM6", "COM7", "COM8", "COM9", "COM¹", "COM²", "COM³", "LPT1", "LPT2",
        "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9", "LPT¹", "LPT²",
        "LPT³", "CONIN$", "CONOUT$",
    ];

    RESERVED_WINDOWS_DEVICE_NAMES.iter().any(|name| {
        file_name.starts_with(name)
            && (file_name[name.len()..].is_empty()
                || file_name[name.len()..].starts_with('.')
                || file_name[name.len()..].starts_with(':'))
    })
}

#[test]
fn safe_relative_path_deserialization_contract() {
    let valid_paths = [
        "file.txt",
        "directory/file.txt",
        "my-directory/file.name.with.dots.tar.gz",
        "my_directory/123_456-789.file",
        "./my/file.txt",
        "my/./file.txt",
    ];
    for path in valid_paths {
        SafeRelativeUtf8UnixPathBuf::try_from(path.to_string())
            .expect("Path should be considered valid");
    }

    let invalid_paths = [
        "",                        // Empty path
        "/absolute/file.txt",      // Absolute path
        "C:/absolute/file.txt",    // Absolute path with common Windows prefix
        "//server/share/file.txt", // Absolute path with Windows UNC prefix
        "directory/../file.txt",   // Path with `..` component
        "CON.txt",                 // Reserved Windows device name
        "NUL/file.txt",            // Reserved Windows device name "directory"
        "COM1.txt:ads",            // Reserved Windows device name with ADS name
        "file\\name.txt",          // Backslash in file name
        "my\\directory/file.txt",  // Backslash in directory name
    ];
    for path in invalid_paths {
        SafeRelativeUtf8UnixPathBuf::try_from(path.to_string())
            .expect_err("Path should be considered invalid");
    }
}

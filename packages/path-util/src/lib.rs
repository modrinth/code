use itertools::Itertools;
use serde::{
    Deserialize, Deserializer, Serialize, Serializer,
    de::value::StringDeserializer,
};
use typed_path::{Utf8Component, Utf8TypedPathBuf, Utf8UnixPathBuf};

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

        // At this point, we may have a pseudo-Unix path like `my\directory`, which we should reject
        // to guarantee consistent cross-platform behavior when interpreting component separators
        if path.as_str().contains('\\') {
            return Err(serde::de::Error::custom(
                "File path must not contain backslashes",
            ));
        }

        let mut path_components = path.components().peekable();

        if path_components.peek().is_none() {
            return Err(serde::de::Error::custom("File path cannot be empty"));
        }

        // All components should be normal: a file or directory name, not `/`, `.`, or `..`
        if path_components.any(|component| !component.is_normal()) {
            return Err(serde::de::Error::custom(
                "File path cannot contain any special component or prefix",
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

        if path_components.any(|component| !component.is_normal()) {
            return Err(serde::ser::Error::custom(
                "File path cannot contain any special component or prefix",
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

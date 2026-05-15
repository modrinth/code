use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct SafePath(String);

impl SafePath {
    pub fn new(path: &str) -> Option<Self> {
        if path.is_empty() || path.contains("..") || path.contains('/') || path.contains('\\') {
            return None;
        }
        Some(Self(path.to_string()))
    }

    pub fn to_path(&self, base: &std::path::Path) -> PathBuf {
        base.join(&self.0)
    }
}

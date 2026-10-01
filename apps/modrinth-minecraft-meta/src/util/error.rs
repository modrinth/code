use serde::{Deserialize, Serialize};
use tracing::warn;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorVec(Vec<Box<str>>);

impl ErrorVec {
    #[must_use]
    pub const fn new() -> Self {
        Self(Vec::new())
    }

    pub fn push(&mut self, err: &anyhow::Error) {
        warn!("{err:?}");
        self.0.push(format!("{err:#}").into_boxed_str());
    }
}

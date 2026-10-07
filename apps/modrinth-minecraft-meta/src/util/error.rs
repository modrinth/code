use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use tracing::warn;

#[derive(Debug)]
pub struct ErrorAccumulator {
    imp: Mutex<ErrorVec>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorVec(Vec<Box<str>>);

impl ErrorAccumulator {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            imp: Mutex::new(ErrorVec(Vec::new())),
        }
    }

    pub fn push(&self, err: &anyhow::Error) {
        warn!("error: {err:?}");
        let err = format!("{err:#}").into_boxed_str();
        let mut imp = self.imp.lock().expect("should not be poisoned");
        imp.0.push(err);
    }

    pub fn finish(self) -> ErrorVec {
        self.imp.into_inner().expect("should not be poisoned")
    }
}

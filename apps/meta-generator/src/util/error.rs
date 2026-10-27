use std::sync::Mutex;

use anyhow::anyhow;
use tracing::warn;

#[derive(Debug)]
pub struct ErrorAccumulator {
    imp: Mutex<Vec<String>>,
}

impl ErrorAccumulator {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            imp: Mutex::new(Vec::new()),
        }
    }

    pub fn push(&self, err: &anyhow::Error) {
        warn!("error: {err:?}");
        let err = format!("{err:#}");
        let mut imp = self.imp.lock().expect("should not be poisoned");
        imp.push(err);
    }

    pub fn push_mut(&mut self, err: &anyhow::Error) {
        warn!("error: {err:?}");
        let err = format!("{err:#}");
        let imp = self.imp.get_mut().expect("should not be poisoned");
        imp.push(err);
    }

    pub fn len(&self) -> usize {
        self.imp.lock().expect("should not be poisoned").len()
    }

    pub fn finish_into_messages(self) -> Vec<String> {
        self.imp.into_inner().expect("should not be poisoned")
    }

    pub fn finish_into_result(self) -> anyhow::Result<()> {
        let mut messages = self
            .imp
            .into_inner()
            .expect("should not be poisoned")
            .into_iter();
        let first = match messages.next() {
            Some(message) => message,
            None => return Ok(()),
        };
        let mut err = anyhow!(first);
        for message in messages {
            err = err.context(message);
        }
        Err(err)
    }
}

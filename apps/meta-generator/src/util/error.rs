use std::sync::Mutex;

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

	pub fn len(&self) -> usize {
		self.imp.lock().expect("should not be poisoned").len()
	}

	pub fn finish(self) -> Vec<String> {
		self.imp.into_inner().expect("should not be poisoned")
	}
}

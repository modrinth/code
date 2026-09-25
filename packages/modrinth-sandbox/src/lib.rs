mod backend;
mod minecraft;
mod util;

use std::sync::Arc;

use derive_more::Debug;
use eyre::Result;

pub use backend::{SandboxChild, SandboxChildTrait, SandboxCommand, SandboxExitStatus};
pub use minecraft::*;
pub use util::argument::SandboxArg;

/// Creates the environment and initializes the required resources to perform
/// sandboxing.
///
/// On most platforms, sandboxing is done using operating system primitives, in
/// which case creating the environment will always succeed. On other platforms,
/// sandboxing may require extra dependencies to be installed in the app
/// environment - if those are not fulfilled, this returns [`Err`].
pub async fn create_env() -> Result<SandboxEnv> {
    backend::init_env().await.map(|inner| SandboxEnv {
        inner: Arc::from(inner),
    })
}

/// Allows spawning processes in a sandboxed environment, configured by
/// [`SandboxCommand`].
#[derive(Debug, Clone)]
#[debug("{:?}", &self.inner)]
pub struct SandboxEnv {
    inner: Arc<dyn backend::SandboxEnv>,
}

impl SandboxEnv {
    pub async fn spawn(&self, command: SandboxCommand) -> Result<SandboxChild> {
        self.inner.spawn(command).await
    }
}

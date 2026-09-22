mod backend;
mod util;

use std::{path::PathBuf, sync::Arc};

use derive_more::Debug;
use eyre::Result;

use crate::backend::SandboxChild;

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

#[derive(Debug)]
pub struct SandboxCommand {
    pub executable: String,
    pub args: Vec<String>,
    /// Host paths mounted read-only at the same absolute path in the sandbox.
    pub read_only_paths: Vec<PathBuf>,
    /// Host paths mounted read-write at the same absolute path in the sandbox.
    pub writable_paths: Vec<PathBuf>,
    pub working_directory: Option<PathBuf>,
    pub allow_network: bool,
}

/// Allows spawning processes in a sandboxed environment, configured by
/// [`Sandbox`].
#[derive(Debug, Clone)]
#[debug("{:?}", &self.inner)]
pub struct SandboxEnv {
    inner: Arc<dyn backend::SandboxEnv>,
}

impl SandboxEnv {
    pub async fn spawn(
        &self,
        sandbox: SandboxCommand,
    ) -> Result<Box<dyn SandboxChild>> {
        self.inner.spawn(sandbox).await
    }
}

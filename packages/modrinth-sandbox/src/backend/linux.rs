use std::process::ExitStatus;

use eyre::Result;

#[derive(Debug)]
pub struct SandboxChild {
    child: tokio::process::Child,
}

impl SandboxChild {
    pub(super) fn new(child: tokio::process::Child) -> Self {
        Self { child }
    }

    pub fn id(&self) -> Option<u32> {
        self.child.id()
    }

    pub fn take_stdout(&mut self) -> Option<tokio::process::ChildStdout> {
        self.child.stdout.take()
    }

    pub fn take_stderr(&mut self) -> Option<tokio::process::ChildStderr> {
        self.child.stderr.take()
    }

    pub fn try_wait(&mut self) -> Result<Option<ExitStatus>> {
        Ok(self.child.try_wait()?)
    }

    pub async fn wait(&mut self) -> Result<ExitStatus> {
        Ok(self.child.wait().await?)
    }

    pub async fn kill(&mut self) -> Result<()> {
        Ok(self.child.kill().await?)
    }
}

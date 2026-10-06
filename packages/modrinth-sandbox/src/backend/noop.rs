use std::{
    io::{PipeReader, PipeWriter},
    process::Stdio,
};

use async_trait::async_trait;
use eyre::{Context, Result, eyre};

use crate::{
    SandboxCommand, SandboxExitStatus, SandboxStdio,
    backend::{Backend, SandboxChildOp, SandboxEnv},
};

#[derive(Debug)]
pub struct NoSandbox;

#[async_trait]
impl Backend for NoSandbox {
    async fn init(
        _make_helper: crate::helper::MakeHelper,
    ) -> Result<Box<dyn SandboxEnv>> {
        Ok(Box::new(Self))
    }
}

#[async_trait]
impl SandboxEnv for NoSandbox {
    async fn spawn(
        &self,
        command: SandboxCommand,
    ) -> Result<crate::SandboxChild> {
        spawn(command).await
    }
}

async fn spawn(mut command: SandboxCommand) -> Result<crate::SandboxChild> {
    for directory in &command.ensure_dirs_exist {
        tokio::fs::create_dir_all(directory)
            .await
            .wrap_err_with(|| eyre!("creating directory {directory:?}"))?;
    }

    let environment = command.take_environment();
    let mut builder = tokio::process::Command::new(&command.executable);
    builder.args(command.args.iter().map(|arg| arg.as_os_str()));
    builder.env_clear();
    builder.envs(
        environment
            .iter()
            .map(|(key, value)| (key.as_os_str(), value.as_os_str())),
    );
    if let Some(directory) = command.working_directory {
        builder.current_dir(directory);
    }

    let (stdin, stdin_writer) = input_stdio(command.stdin)?;
    let (stdout, stdout_reader) = output_stdio(command.stdout)?;
    let (stderr, stderr_reader) = output_stdio(command.stderr)?;
    builder.stdin(stdin).stdout(stdout).stderr(stderr);
    let child = builder.spawn().wrap_err("spawning unsandboxed process")?;
    // Command retains the child-side pipe handles after spawning.
    drop(builder);

    Ok(crate::SandboxChild {
        stdin: stdin_writer,
        stdout: stdout_reader,
        stderr: stderr_reader,
        imp: StdChild { child }.into(),
    })
}

fn input_stdio(mode: SandboxStdio) -> Result<(Stdio, Option<PipeWriter>)> {
    match mode {
        SandboxStdio::Null => Ok((Stdio::null(), None)),
        SandboxStdio::Inherit => Ok((Stdio::inherit(), None)),
        SandboxStdio::Pipe => {
            let (reader, writer) =
                std::io::pipe().wrap_err("creating unsandboxed stdin pipe")?;
            Ok((reader.into(), Some(writer)))
        }
    }
}

fn output_stdio(mode: SandboxStdio) -> Result<(Stdio, Option<PipeReader>)> {
    match mode {
        SandboxStdio::Null => Ok((Stdio::null(), None)),
        SandboxStdio::Inherit => Ok((Stdio::inherit(), None)),
        SandboxStdio::Pipe => {
            let (reader, writer) =
                std::io::pipe().wrap_err("creating unsandboxed output pipe")?;
            Ok((writer.into(), Some(reader)))
        }
    }
}

#[derive(Debug)]
pub struct StdChild {
    child: tokio::process::Child,
}

#[async_trait]
impl SandboxChildOp for StdChild {
    fn id(&self) -> Option<u32> {
        self.child.id()
    }

    fn try_wait(&mut self) -> Result<Option<SandboxExitStatus>> {
        self.child
            .try_wait()
            .map(|status| status.map(Into::into))
            .wrap_err("checking unsandboxed child status")
    }

    async fn wait(&mut self) -> Result<SandboxExitStatus> {
        self.child
            .wait()
            .await
            .map(Into::into)
            .wrap_err("waiting for unsandboxed child")
    }

    async fn kill(&mut self) -> Result<()> {
        if self.child.try_wait()?.is_some() {
            return Ok(());
        }
        self.child
            .kill()
            .await
            .wrap_err("killing unsandboxed child")
    }
}

#[cfg(all(test, unix))]
mod tests {
    use std::{
        collections::{BTreeMap, BTreeSet},
        io::{Read, Write},
    };

    use super::*;

    fn command(script: &'static str) -> SandboxCommand {
        SandboxCommand {
            executable: "/bin/sh".into(),
            args: vec!["-c".into(), script.into()],
            ensure_dirs_exist: Vec::new(),
            read_only_paths: Vec::new(),
            read_write_paths: Vec::new(),
            working_directory: None,
            passthrough_environment: BTreeSet::new(),
            extra_environment: BTreeMap::new(),
            allow_network: false,
            is_jvm: false,
            die_with_parent: false,
            stdin: SandboxStdio::Pipe,
            stdout: SandboxStdio::Pipe,
            stderr: SandboxStdio::Pipe,
            app_container_name: "test".into(),
            app_container_description: "test".into(),
        }
    }

    #[tokio::test]
    async fn forwards_pipes_and_environment() -> Result<()> {
        let mut command = command(
            "IFS= read -r line; printf '%s:%s' \"$TEST_VALUE\" \"$line\"; printf error >&2; exit 7",
        );
        command
            .extra_environment
            .insert("TEST_VALUE".into(), "value".into());
        let mut child = NoSandbox.spawn(command).await?;
        assert!(child.id().is_some());
        child.stdin.take().unwrap().write_all(b"input\n")?;
        let status = child.wait().await?;
        assert_eq!(status.code(), Some(7));
        assert!(!status.success());
        assert_eq!(child.wait().await?.code(), Some(7));
        assert_eq!(child.try_wait()?.unwrap().code(), Some(7));
        child.kill().await?;
        let mut stdout = String::new();
        child.stdout.take().unwrap().read_to_string(&mut stdout)?;
        assert_eq!(stdout, "value:input");
        let mut stderr = String::new();
        child.stderr.take().unwrap().read_to_string(&mut stderr)?;
        assert_eq!(stderr, "error");
        Ok(())
    }

    #[tokio::test]
    async fn kills_running_child() -> Result<()> {
        let mut child = NoSandbox.spawn(command("while :; do :; done")).await?;
        assert!(child.try_wait()?.is_none());
        child.kill().await?;
        assert!(!child.wait().await?.success());
        child.kill().await?;
        Ok(())
    }

    #[tokio::test]
    async fn reports_spawn_failure() {
        let mut command = command("");
        command.executable = "/nonexistent/modrinth-sandbox-noop".into();
        assert!(NoSandbox.spawn(command).await.is_err());
    }
}

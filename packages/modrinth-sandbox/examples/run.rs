use std::{
    ffi::OsString,
    io::{BufRead, BufReader, PipeReader, Write},
    path::PathBuf,
};

use anyhow::{Context, Result, ensure};
use modrinth_sandbox::{SandboxArg, SandboxCommand, SandboxEnv, SandboxStdio};
use tracing::info;

#[derive(Debug, clap::Parser)]
struct Cli {
    #[arg(long, value_name = "PATH")]
    read: Vec<PathBuf>,
    #[arg(long, value_name = "PATH")]
    write: Vec<PathBuf>,
    #[arg(long, value_name = "PATH")]
    cwd: Option<PathBuf>,
    executable: PathBuf,
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    args: Vec<OsString>,
}

#[tokio::main]
async fn main() -> Result<()> {
    if modrinth_sandbox::helper::run_default()? {
        return Ok(());
    }
    let cli = <Cli as clap::Parser>::parse();
    tracing_subscriber::fmt().init();

    let env = SandboxEnv::new()
        .await
        .context("creating sandbox environment")?;
    info!("using environment {env:?}");

    let mut command = SandboxCommand::default();
    command.executable = cli.executable;
    command.args = cli.args.into_iter().map(SandboxArg::from).collect();
    command.read_only_paths = cli.read;
    command.read_write_paths = cli.write;
    command.working_directory = cli.cwd;
    command.allow_network = true;
    command.die_with_parent = true;
    command.stdin = SandboxStdio::Null;
    command.stdout = SandboxStdio::Pipe;
    command.stderr = SandboxStdio::Pipe;
    command.app_container_name = "ModrinthMinecraftSandbox".into();
    command.app_container_description =
        "Sandbox for Minecraft instances created by modrinth-sandbox".into();

    let mut child = env
        .spawn(command)
        .await
        .context("spawning process in sandbox")?;

    let stdout = forward_pipe(
        child.stdout.take().expect("we set `stdout` to `Pipe`"),
        || std::io::stdout().lock(),
    );
    let stderr = forward_pipe(
        child.stderr.take().expect("we set `stderr` to `Pipe`"),
        || std::io::stderr().lock(),
    );

    let status = child
        .wait()
        .await
        .context("waiting for process in sandbox")?;

    stdout
        .await
        .context("joining stdout reader task")?
        .context("forwarding sandboxed process stdout")?;
    stderr
        .await
        .context("joining stderr reader task")?
        .context("forwarding sandboxed process stderr")?;

    ensure!(status.success(), "sandboxed process exited with {status}");

    Ok(())
}

fn forward_pipe<W>(
    pipe: PipeReader,
    make_output: fn() -> W,
) -> tokio::task::JoinHandle<std::io::Result<()>>
where
    W: Write + 'static,
{
    tokio::task::spawn_blocking(move || {
        let mut output = make_output();
        let mut reader = BufReader::new(pipe);
        let mut line = String::new();

        while reader.read_line(&mut line)? != 0 {
            write!(output, "child: {line}")?;
            if !line.ends_with('\n') {
                writeln!(output)?;
            }
            output.flush()?;
            line.clear();
        }

        Ok(())
    })
}

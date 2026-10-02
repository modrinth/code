use std::{
    collections::{BTreeMap, BTreeSet},
    ffi::OsString,
    io::{BufRead, BufReader, PipeReader, Write},
    path::PathBuf,
};

use eyre::{Result, WrapErr, ensure};
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
        .wrap_err("creating sandbox environment")?;
    info!("using environment {env:?}");

    let mut child = env
        .spawn(SandboxCommand {
            executable: cli.executable,
            args: cli.args.into_iter().map(SandboxArg::from).collect(),
            ensure_dirs_exist: Vec::new(),
            read_only_paths: cli.read,
            read_write_paths: cli.write,
            working_directory: cli.cwd,
            passthrough_environment: BTreeSet::new(),
            extra_environment: BTreeMap::new(),
            allow_network: true,
            is_jvm: false,
            die_with_parent: true,
            stdin: SandboxStdio::Null,
            stdout: SandboxStdio::Pipe,
            stderr: SandboxStdio::Pipe,
            app_container_name: "ModrinthMinecraftSandbox".into(),
            app_container_description:
                "Sandbox for Minecraft instances created by modrinth-sandbox"
                    .into(),
        })
        .await
        .wrap_err("spawning process in sandbox")?;

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
        .wrap_err("waiting for process in sandbox")?;

    stdout
        .await
        .wrap_err("joining stdout reader task")?
        .wrap_err("forwarding sandboxed process stdout")?;
    stderr
        .await
        .wrap_err("joining stderr reader task")?
        .wrap_err("forwarding sandboxed process stderr")?;

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

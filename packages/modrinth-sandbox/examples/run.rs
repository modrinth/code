use std::path::PathBuf;

use eyre::{Context, Result};
use modrinth_sandbox::{SandboxCommand, SandboxOutput};
use tracing::info;

#[derive(Debug, clap::Parser)]
struct Cli {
    #[arg(long, value_name = "PATH")]
    read_only_path: Vec<PathBuf>,
    #[arg(long, value_name = "PATH")]
    writable_path: Vec<PathBuf>,
    #[arg(long, value_name = "PATH")]
    working_directory: Option<PathBuf>,
    executable: String,
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    args: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = <Cli as clap::Parser>::parse();
    tracing_subscriber::fmt().init();

    let env = modrinth_sandbox::create_env()
        .await
        .wrap_err("creating sandbox environment")?;
    info!("using environment: {env:?}");

    let mut child = env
        .spawn(SandboxCommand {
            executable: cli.executable,
            args: cli.args,
            read_only_paths: cli
                .read_only_path
                .into_iter()
                .map(|path| (path.clone(), path))
                .collect(),
            read_write_paths: cli
                .writable_path
                .into_iter()
                .map(|path| (path.clone(), path))
                .collect(),
            working_directory: cli.working_directory,
            passthrough_environment: Vec::new(),
            extra_environment: Vec::new(),
            output: SandboxOutput::Inherit,
            system_runtime: true,
            network: true,
            graphics: true,
            audio: true,
        })
        .await
        .context("spawning process in sandbox")?;

    let status = child
        .wait()
        .await
        .context("waiting for process in sandbox")?;
    eyre::ensure!(status.success(), "sandboxed process exited with {status}");

    Ok(())
}

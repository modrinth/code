use std::path::PathBuf;

use eyre::{Context, Result};
use modrinth_sandbox::SandboxCommand;
use tracing::info;

#[derive(Debug, clap::Parser)]
struct Cli {
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
            read_only_paths: Vec::new(),
            writable_paths: cli.writable_path,
            working_directory: cli.working_directory,
            allow_network: true,
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

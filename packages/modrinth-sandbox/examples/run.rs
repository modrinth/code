use std::{
    collections::{BTreeMap, HashSet},
    ffi::OsString,
    path::PathBuf,
};

use eyre::{Context, Result};
use modrinth_sandbox::{SandboxArg, SandboxCommand};
use tracing::info;

#[derive(Debug, clap::Parser)]
struct Cli {
    #[arg(long, value_name = "PATH")]
    read_only_path: Vec<PathBuf>,
    #[arg(long, value_name = "PATH")]
    writable_path: Vec<PathBuf>,
    #[arg(long, value_name = "PATH")]
    working_directory: Option<PathBuf>,
    executable: PathBuf,
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    args: Vec<OsString>,
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
            args: cli.args.into_iter().map(SandboxArg::from).collect(),
            ensure_dirs_exist: Vec::new(),
            read_only_paths: cli.read_only_path,
            read_write_paths: cli.writable_path,
            working_directory: cli.working_directory,
            passthrough_environment: HashSet::new(),
            extra_environment: BTreeMap::new(),
            network: true,
            is_jvm: false,
            die_with_parent: true,
        })
        .context("spawning process in sandbox")?;

    let status = child
        .wait()
        .await
        .context("waiting for process in sandbox")?;
    eyre::ensure!(status.success(), "sandboxed process exited with {status}");

    Ok(())
}

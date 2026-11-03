use std::path::PathBuf;

use tracing_subscriber::EnvFilter;

#[derive(Debug, clap::Parser)]
struct Cli {
    #[arg(long, value_name = "PATH")]
    mountpoint: PathBuf,
    #[arg(long, value_name = "PATH")]
    bind: Vec<PathBuf>,
}

fn main() {
    let cli = <Cli as clap::Parser>::parse();
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    modrinth_sandbox::start_file_sharing_test(cli.mountpoint, cli.bind).unwrap();
}

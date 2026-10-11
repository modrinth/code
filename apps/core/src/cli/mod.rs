//! Terminal command surface for installing and operating a local Copal Core.

pub(crate) mod admin;
mod install;
mod interactive;
mod metadata;
mod service;
mod update;

use clap::{Args, CommandFactory, Parser, Subcommand, ValueEnum};
use color_eyre::eyre::{bail, Result};

/// Copal installs, runs, diagnoses, and updates a self-hosted Core.
#[derive(Debug, Parser)]
#[command(name = "copal", version, arg_required_else_help = false)]
pub(crate) struct Cli {
    /// Emit machine-readable output where the command supports it.
    #[arg(long, global = true, default_value = "table")]
    format: OutputFormat,
    /// Reduce non-essential output.
    #[arg(short, long, global = true)]
    quiet: bool,
    /// Include diagnostic detail in failures.
    #[arg(short, long, global = true, action = clap::ArgAction::Count)]
    verbose: u8,
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub(crate) enum OutputFormat {
    Table,
    Json,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Start the Core HTTP server. This is the default command.
    Run,
    /// Validate Core configuration, migrations, and database connectivity.
    Check,
    /// Apply pending database migrations and exit.
    Migrate,
    /// Print the Copal version.
    Version,
    /// Install Copal as a local system service.
    Install(install::InstallArgs),
    /// Inspect or control the installed Copal service.
    Service(service::ServiceArgs),
    /// Show a concise operational status report.
    Status,
    /// Collect local installation and service diagnostics.
    Diagnose,
    /// Check for or install a Copal release.
    Update(update::UpdateArgs),
    /// Generate shell completions.
    Completion(CompletionArgs),
    /// Open the prompt-assisted operational menu.
    Menu,
    /// Emergency instance controls for a running local Core.
    Instance(InstanceArgs),
    /// Remove locally installed service integration without deleting Core data.
    Uninstall(UninstallArgs),
}

#[derive(Debug, Args)]
struct CompletionArgs {
    #[arg(value_enum)]
    shell: Shell,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum Shell {
    Bash,
    Elvish,
    Fish,
    PowerShell,
    Zsh,
}

#[derive(Debug, Args)]
pub(crate) struct InstanceArgs {
    #[command(subcommand)]
    command: InstanceCommand,
}

#[derive(Debug, Subcommand)]
pub(crate) enum InstanceCommand {
    /// List instances known to the locally running Core.
    List,
    /// Show one instance's status.
    Status { id: String },
    /// Start one or more instances.
    Start {
        id: Option<String>,
        /// Select one or more instances interactively.
        #[arg(long)]
        select: bool,
        /// Confirm a non-interactive start.
        #[arg(short = 'y', long)]
        yes: bool,
    },
    /// Request a graceful stop for one instance.
    Stop {
        id: Option<String>,
        /// Select one or more instances interactively.
        #[arg(long)]
        select: bool,
        /// Confirm a non-interactive stop.
        #[arg(short = 'y', long)]
        yes: bool,
    },
    /// Restart one or more instances.
    Restart {
        id: Option<String>,
        /// Select one or more instances interactively.
        #[arg(long)]
        select: bool,
        /// Confirm a non-interactive restart.
        #[arg(short = 'y', long)]
        yes: bool,
    },
    /// Force-kill a running instance. Use only when a graceful stop fails.
    Kill {
        id: String,
        #[arg(short = 'y', long)]
        yes: bool,
    },
    /// Re-download server files for an offline instance.
    Repair {
        id: String,
        #[arg(short = 'y', long)]
        yes: bool,
    },
    /// Access an instance console.
    Console {
        id: String,
        /// Send one command to a running instance without attaching.
        #[arg(long)]
        send: Option<String>,
    },
}

#[derive(Debug, Args)]
pub(crate) struct UninstallArgs {
    /// Also remove the Core data directory. This is irreversible.
    #[arg(long)]
    purge_data: bool,
    /// Confirm a destructive uninstall in non-interactive use.
    #[arg(short = 'y', long)]
    yes: bool,
}

pub(crate) async fn execute(cli: Cli) -> Result<()> {
    match cli.command.unwrap_or(Command::Run) {
        Command::Run => {
            crate::init_tracing();
            crate::run_server().await
        }
        Command::Check => check().await,
        Command::Migrate => migrate().await,
        Command::Version => {
            println!("copal {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        Command::Install(args) => install::execute(args).await,
        Command::Service(args) => service::execute(args, cli.format).await,
        Command::Status => service::status(cli.format).await,
        Command::Diagnose => service::diagnose(cli.format).await,
        Command::Update(args) => update::execute(args).await,
        Command::Completion(args) => completion(args.shell),
        Command::Menu => interactive::menu().await,
        Command::Instance(args) => {
            interactive::instance(args.command, cli.format).await
        }
        Command::Uninstall(args) => install::uninstall(args).await,
    }
}

async fn migrate() -> Result<()> {
    crate::init_tracing();
    let config = crate::config::Config::from_env()?;
    tokio::fs::create_dir_all(&config.data_dir).await?;
    let db_path = config.data_dir.join("data.db");
    let pool = crate::infrastructure::db::connect(&db_path).await?;
    crate::infrastructure::db::migrate(&pool).await?;
    println!("Migrations applied successfully.");
    Ok(())
}

async fn check() -> Result<()> {
    crate::init_tracing();
    let config = crate::config::Config::from_env()?;
    tokio::fs::create_dir_all(&config.data_dir).await?;
    let db_path = config.data_dir.join("data.db");
    let pool = crate::infrastructure::db::connect(&db_path).await?;
    crate::infrastructure::db::migrate(&pool).await?;
    let owner: Option<String> =
        sqlx::query_scalar("SELECT user_id FROM core_owner WHERE id = 1")
            .fetch_optional(&pool)
            .await?;
    println!("Config  : OK (data_dir = {})", config.data_dir.display());
    println!("Database: OK ({})", db_path.display());
    println!(
        "Owner   : {}",
        owner
            .as_deref()
            .unwrap_or("none, start Core to get a pairing code")
    );
    Ok(())
}

fn completion(shell: Shell) -> Result<()> {
    let mut command = Cli::command();
    match shell {
        Shell::Bash => clap_complete::generate(
            clap_complete::Shell::Bash,
            &mut command,
            "copal",
            &mut std::io::stdout(),
        ),
        Shell::Elvish => clap_complete::generate(
            clap_complete::Shell::Elvish,
            &mut command,
            "copal",
            &mut std::io::stdout(),
        ),
        Shell::Fish => clap_complete::generate(
            clap_complete::Shell::Fish,
            &mut command,
            "copal",
            &mut std::io::stdout(),
        ),
        Shell::PowerShell => clap_complete::generate(
            clap_complete::Shell::PowerShell,
            &mut command,
            "copal",
            &mut std::io::stdout(),
        ),
        Shell::Zsh => clap_complete::generate(
            clap_complete::Shell::Zsh,
            &mut command,
            "copal",
            &mut std::io::stdout(),
        ),
    };
    Ok(())
}

pub(crate) fn require_confirmation(yes: bool, prompt: &str) -> Result<()> {
    if yes {
        return Ok(());
    }
    if !std::io::IsTerminal::is_terminal(&std::io::stdin()) {
        bail!("{prompt} Re-run with --yes to confirm in non-interactive mode.");
    }
    if inquire::Confirm::new(prompt).with_default(false).prompt()? {
        Ok(())
    } else {
        bail!("Operation cancelled.")
    }
}

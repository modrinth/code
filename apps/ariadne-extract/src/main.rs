mod error;
mod extractor;

use crate::extractor::Extractor;
use clap::Parser;
use error::Result;
use miette::GraphicalReportHandler;
use std::fs;
use std::fs::File;
use std::path::PathBuf;
use std::process::exit;

#[derive(Debug, Clone, Parser)]
struct Args {
    /// The path to output the extracted translation data to
    #[clap(short, long)]
    out_file: PathBuf,

    /// Include translations from tests. For simplicity, `cfg` above a module declarations are
    /// skipped.
    #[clap(short = 't', long)]
    include_tests: bool,

    /// The paths to the directories containing Cargo.toml that should be extracted to the specified file
    packages: Vec<PathBuf>,
}

fn main() {
    if let Err(err) = run(Args::parse()) {
        eprintln!("Failed to run extractor:\n{err}");
        exit(1);
    }
}

fn run(args: Args) -> Result<()> {
    let mut extractor = Extractor::new(args.include_tests);
    for package in args.packages {
        extractor.process_package(&package)?;
    }
    if !extractor.errors().is_empty() {
        for error in extractor.errors() {
            let mut error_message = String::new();
            GraphicalReportHandler::new()
                .render_report(&mut error_message, error)
                .unwrap();
            eprintln!("{}", error.render());
        }
        exit(extractor.errors().len().try_into().unwrap_or(i32::MAX));
    }

    if let Some(parent) = args.out_file.parent() {
        fs::create_dir_all(parent)?;
    }
    let writer = File::create(args.out_file)?;
    serde_json::to_writer_pretty(writer, &extractor.output())?;
    Ok(())
}

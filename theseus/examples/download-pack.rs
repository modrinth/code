/* TODO: Rewrite after porting modpacks
use std::{path::PathBuf, time::Instant};

use argh::FromArgs;
use theseus::modpack::{fetch_modpack, pack::ModpackSide};

#[derive(FromArgs)]
/// Simple modpack download
struct ModpackDownloader {
    /// where to download to
    #[argh(positional)]
    url: String,

    /// where to put the resulting pack
    #[argh(option, short = 'o')]
    output: Option<PathBuf>,

    /// the sha1 hash, if you want it checked
    #[argh(option, short = 'c')]
    hash: Option<String>,

    /// use verbose logging
    #[argh(switch, short = 'v')]
    verbose: bool,
}

// Simple logging helper
fn debug(msg: &str, verbose: bool) {
    if verbose {
        println!("{}", msg);
    }
}

#[tokio::main]
pub async fn main() {
    let args = argh::from_env::<ModpackDownloader>();
    let dest = args.output.unwrap_or(PathBuf::from("./pack-download/"));

    debug(
        &format!(
            "Downloading pack {} to {}",
            args.url,
            dest.to_str().unwrap_or("?")
        ),
        args.verbose,
    );

    let start = Instant::now();
    fetch_modpack(&args.url, args.hash.as_deref(), &dest, ModpackSide::Client)
        .await
        .unwrap();
    let end = start.elapsed();

    println!("Download completed in {} seconds", end.as_secs_f32());
    debug("Done!", args.verbose);
}
*/

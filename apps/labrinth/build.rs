use std::path::Path;

use chrono::{Local, Utc};
use dotenv_build::Config;

fn main() {
    println!("cargo:rerun-if-changed=migrations");
    println!("cargo::rerun-if-env-changed=GITHUB_SHA");

    let git_hash = std::env::var("GITHUB_SHA")
        .ok()
        .or_else(|| {
            std::process::Command::new("git")
                .args(["rev-parse", "HEAD"])
                .output()
                .ok()
                .filter(|output| output.status.success())
                .map(|output| {
                    String::from_utf8_lossy(&output.stdout).trim().to_owned()
                })
        })
        .unwrap_or_else(|| "unknown".to_owned());
    let short_git_hash = &git_hash[..git_hash.len().min(7)];

    println!("cargo::rustc-env=GIT_HASH={git_hash}");
    println!(
        "cargo::rustc-env=LABRINTH_VERSION={}-{short_git_hash}",
        Utc::now().format("%Y.%m.%d")
    );

    println!(
        "cargo::rustc-env=COMPILATION_DATE={}",
        format_args!(
            "{date} ({tz})",
            date = Local::now().format("%F @ %I:%M %p"),
            tz = iana_time_zone::get_timezone().unwrap_or_default()
        )
    );
    println!(
        "cargo::rustc-env=COMPILATION_PROFILE={}",
        std::env::var("PROFILE").as_deref().unwrap_or("unknown")
    );

    dotenv_build::output(Config {
        filename: Path::new(".build.env"),
        recursive_search: true,
        fail_if_missing_dotenv: false,
    })
    .unwrap();
}

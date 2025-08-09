use std::path::Path;

use chrono::Local;
use dotenv_build::Config;

fn main() {
    println!("cargo:rerun-if-changed=migrations");

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

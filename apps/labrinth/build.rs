use std::path::Path;
use std::process::Command;

use chrono::Local;
use dotenv_build::Config;

fn main() {
    let output = Command::new("git")
        .arg("rev-parse")
        .arg("HEAD")
        .output()
        .expect("`git` invocation to succeed");

    let git_hash = String::from_utf8(output.stdout)
        .expect("valid UTF-8 output from `git` invocation");

    println!("cargo::rerun-if-changed=.git/HEAD");
    println!("cargo::rustc-env=GIT_HASH={}", git_hash.trim());

    let timedate_fmt = Local::now().format("%F @ %I:%M %p");
    let timezone_fmt = iana_time_zone::get_timezone()
        .map(|tz| format!(" ({tz})"))
        .unwrap_or_default();

    let comptime = timedate_fmt.to_string() + timezone_fmt.as_str();

    println!("cargo::rustc-env=COMPILATION_DATE={comptime}");

    // trick to get compilation profile
    let profile = std::env::var("OUT_DIR")
        .expect("OUT_DIR to be set")
        .split(std::path::MAIN_SEPARATOR)
        .nth_back(3)
        .unwrap_or("unknown")
        .to_string();

    println!("cargo::rustc-env=COMPILATION_PROFILE={profile}");

    dotenv_build::output(Config {
        filename: Path::new(".build.env"),
        recursive_search: true,
        fail_if_missing_dotenv: false,
    })
    .unwrap();

    println!("cargo:rerun-if-changed=migrations");
}

use std::ffi::OsString;
use std::path::PathBuf;
use std::process::{Command, exit};
use std::{env, fs};

#[allow(dead_code)]
mod app_event_bindings;

fn main() {
    println!("cargo::rerun-if-changed=.env");
    println!("cargo::rerun-if-changed=.env.local");
    println!("cargo::rerun-if-changed=java/gradle");
    println!("cargo::rerun-if-changed=java/src");
    println!("cargo::rerun-if-changed=java/build.gradle.kts");
    println!("cargo::rerun-if-changed=java/settings.gradle.kts");
    println!("cargo::rerun-if-changed=java/gradle.properties");

    check_app_event_bindings();
    set_env();
    build_java_jars();
}

fn check_app_event_bindings() {
    if env::var_os("CARGO_FEATURE_EXPORT_TS").is_some() {
        return;
    }

    let manifest_dir =
        PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let output = manifest_dir
        .join("../..")
        .join("apps/app-frontend/src/generated/app-events");

    for input in app_event_bindings::tracked_inputs(&manifest_dir) {
        println!("cargo::rerun-if-changed={}", input.display());
    }
    println!(
        "cargo::rerun-if-changed={}",
        output.join(app_event_bindings::MANIFEST_FILE).display()
    );
    println!("cargo::rerun-if-changed={}", output.display());

    if let Err(error) =
        app_event_bindings::validate_manifest(&manifest_dir, &output)
    {
        println!(
            "cargo::error=App event TypeScript bindings are out of date: {error}"
        );
        println!(
            "cargo::error=Run `cargo export-app-events` from the workspace root and commit the generated files"
        );
        exit(1);
    }
}

fn set_env() {
    let variables = dotenvy::dotenv_iter()
        .or_else(|_| dotenvy::from_path_iter(".env.local"));

    for (var_name, var_value) in variables.into_iter().flatten().flatten() {
        if var_name == "DATABASE_URL" {
            // The sqlx database URL is a build-time detail that should not be exposed to the crate
            continue;
        }

        println!("cargo::rustc-env={var_name}={var_value}");
    }
}

fn build_java_jars() {
    let out_dir =
        dunce::canonicalize(PathBuf::from(env::var_os("OUT_DIR").unwrap()))
            .unwrap();

    println!(
        "cargo::rustc-env=JAVA_JARS_DIR={}",
        out_dir.join("java/libs").display()
    );

    let gradle_path = fs::canonicalize(
        #[cfg(target_os = "windows")]
        "java\\gradlew.bat",
        #[cfg(not(target_os = "windows"))]
        "java/gradlew",
    )
    .unwrap();

    let mut build_dir_str = OsString::from("-Dorg.gradle.project.buildDir=");
    build_dir_str.push(out_dir.join("java"));
    let exit_status = Command::new(gradle_path)
        .arg(build_dir_str)
        .arg("build")
        .arg("--no-daemon")
        .arg("--console=rich")
        .current_dir(dunce::canonicalize("java").unwrap())
        .status()
        .expect("Failed to wait on Gradle build");

    if !exit_status.success() {
        println!("cargo::error=Gradle build failed with {exit_status}");
        exit(exit_status.code().unwrap_or(1));
    }
}

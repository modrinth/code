use std::fs;
use std::process::{Command, exit};

fn main() {
    println!("cargo::rerun-if-changed=java/build/libs/theseus.jar");
    println!("cargo::rerun-if-changed=java/gradle");
    println!("cargo::rerun-if-changed=java/src");
    println!("cargo::rerun-if-changed=java/build.gradle.kts");
    println!("cargo::rerun-if-changed=java/settings.gradle.kts");
    println!("cargo::rerun-if-changed=java/gradle.properties");

    let gradle_path = fs::canonicalize(
        #[cfg(target_os = "windows")]
        "java\\gradlew.bat",
        #[cfg(not(target_os = "windows"))]
        "java/gradlew",
    )
    .unwrap();
    let exit_status = Command::new(gradle_path)
        .arg("build")
        .arg("--no-daemon")
        .current_dir(dunce::canonicalize("java").unwrap())
        .status()
        .expect("Failed to wait on Gradle build");
    if !exit_status.success() {
        println!("cargo::error=Gradle build failed with {exit_status}");
        exit(exit_status.code().unwrap_or(1));
    }

    // I wish we could copy theseus.jar to OUT_DIR, but I don't know how we'd tell Tauri where to
    // bundle it from if we did that
}

use std::ffi::OsString;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::path::{Path, PathBuf};
use std::process::{Command, exit};
use std::{env, fs, io};

const JAR_NAME: &str = "SandboxAgent.jar";

/// Everything that affects the contents of the jar.
const INPUTS: &[&str] = &[
    "agent/build.gradle",
    "agent/gradle",
    "agent/settings.gradle",
    "agent/src",
];

fn main() {
    for input in INPUTS {
        println!("cargo::rerun-if-changed={input}");
    }

    let out_dir =
        dunce::canonicalize(PathBuf::from(env::var_os("OUT_DIR").unwrap()))
            .unwrap();
    let jar = out_dir.join(JAR_NAME);

    let cached_jar = cached_jar_path(&out_dir);
    if let Some(cached_jar) = &cached_jar
        && fs::copy(cached_jar, &jar).is_ok()
    {
        return;
    }

    build_agent(&out_dir);
    fs::copy(out_dir.join("agent/libs").join(JAR_NAME), &jar)
        .expect("Failed to copy agent jar");

    if let Some(cached_jar) = &cached_jar
        && let Err(err) = store_cached_jar(&jar, cached_jar)
    {
        println!("cargo::warning=Failed to cache agent jar: {err}");
    }
}

/// Cargo uses a different `OUT_DIR` for each combination of profile, target,
/// features, etc., none of which affect the jar. To avoid rebuilding it for each
/// of them, it's cached in the target directory based on a hash of its inputs.
fn cached_jar_path(out_dir: &Path) -> Option<PathBuf> {
    let target_dir = target_dir(out_dir)?;

    let mut hasher = DefaultHasher::new();
    for input in INPUTS {
        hash_path(Path::new(input), &mut hasher)
            .expect("Failed to hash agent inputs");
    }

    Some(
        target_dir
            .join("sandbox-agent")
            .join(format!("{:016x}.jar", hasher.finish())),
    )
}

/// `OUT_DIR` is `<target dir>/[target triple]/<profile>/build/<package>/out`.
fn target_dir(out_dir: &Path) -> Option<&Path> {
    let build_dir = out_dir.ancestors().nth(2)?;
    if build_dir.file_name()? != "build" {
        return None;
    }
    let dir = build_dir.ancestors().nth(2)?;
    if dir.file_name() == env::var_os("TARGET").as_deref() {
        dir.parent()
    } else {
        Some(dir)
    }
}

fn hash_path(path: &Path, hasher: &mut impl Hasher) -> io::Result<()> {
    path.file_name().hash(hasher);

    let is_dir = path.is_dir();
    is_dir.hash(hasher);
    if !is_dir {
        fs::read(path)?.hash(hasher);
        return Ok(());
    }

    let mut entries = fs::read_dir(path)?
        .map(|entry| Ok(entry?.path()))
        .collect::<io::Result<Vec<_>>>()?;
    entries.sort();
    entries.len().hash(hasher);
    for entry in entries {
        hash_path(&entry, hasher)?;
    }
    Ok(())
}

fn store_cached_jar(jar: &Path, cached_jar: &Path) -> io::Result<()> {
    fs::create_dir_all(cached_jar.parent().unwrap())?;

    // Builds of different profiles can run at the same time, so this has to be
    // renamed into place
    let temp_jar =
        cached_jar.with_extension(format!("{}.tmp", std::process::id()));
    fs::copy(jar, &temp_jar)?;
    fs::rename(&temp_jar, cached_jar)
}

fn build_agent(out_dir: &Path) {
    let gradle_path = dunce::canonicalize(
        #[cfg(target_os = "windows")]
        "agent\\gradlew.bat",
        #[cfg(not(target_os = "windows"))]
        "agent/gradlew",
    )
    .unwrap();

    let mut build_dir_str = OsString::from("-Dorg.gradle.project.buildDir=");
    build_dir_str.push(out_dir.join("agent"));
    let exit_status = Command::new(gradle_path)
        .arg(build_dir_str)
        .arg("shadowJar")
        .arg("--no-daemon")
        .arg("--console=rich")
        .current_dir(dunce::canonicalize("agent").unwrap())
        .status()
        .expect("Failed to wait on Gradle build");

    if !exit_status.success() {
        println!("cargo::error=Gradle build failed with {exit_status}");
        exit(exit_status.code().unwrap_or(1));
    }
}

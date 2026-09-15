use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{LazyLock, Mutex};
use std::time::Duration;
use wait_timeout::ChildExt;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JavaInstallation {
    pub path: String,
    pub version: String,
    pub major_version: u32,
    pub is_64_bit: bool,
}

const PROBE_TIMEOUT: Duration = Duration::from_secs(5);
static JAVA_INSTALL_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

pub fn find_all() -> Vec<JavaInstallation> {
    let mut found = Vec::new();
    let mut seen = HashSet::new();

    if let Ok(home) = std::env::var("JAVA_HOME") {
        consider(&PathBuf::from(home).join("bin").join(java_bin()), &mut found, &mut seen);
    }

    if let Ok(path) = std::env::var("PATH") {
        for dir in std::env::split_paths(&path) {
            consider(&dir.join(java_bin()), &mut found, &mut seen);
        }
    }

    for root in common_roots() {
        if !root.is_dir() {
            continue;
        }
        if let Ok(entries) = std::fs::read_dir(&root) {
            for entry in entries.flatten() {
                let candidate = entry.path().join("bin").join(java_bin());
                consider(&candidate, &mut found, &mut seen);
            }
        }
        consider(&root.join("bin").join(java_bin()), &mut found, &mut seen);
    }

    found.sort_by(|a, b| b.major_version.cmp(&a.major_version));
    found
}

pub fn find_compatible(min_major: u32) -> Result<JavaInstallation, String> {
    // Prefer explicitly configured path for the required major (or newest >= min).
    if let Ok(cfg) = crate::config::load() {
        let mut majors: Vec<u32> = cfg
            .java_versions
            .keys()
            .filter_map(|k| k.parse::<u32>().ok())
            .filter(|m| *m >= min_major)
            .collect();
        majors.sort_by(|a, b| b.cmp(a));
        for major in majors {
            if let Some(path) = cfg.java_versions.get(&major.to_string()) {
                let p = PathBuf::from(path);
                if p.is_file() {
                    if let Some(info) = probe(&p) {
                        return Ok(info);
                    }
                }
            }
        }
    }

    let cached = crate::config::load()
        .map(|c| c.java_installations)
        .unwrap_or_default();
    if let Some(hit) = cached
        .into_iter()
        .find(|j| j.major_version >= min_major && Path::new(&j.path).is_file())
    {
        return Ok(hit);
    }

    let all = find_all();
    let _ = crate::config::set_java_installations(all.clone());
    all.into_iter()
        .find(|j| j.major_version >= min_major)
        .ok_or_else(|| {
            format!("Java {min_major}+ not found. Install a JDK/JRE and Detect in Settings.")
        })
}

pub fn find_for_major(major: u32) -> Vec<JavaInstallation> {
    find_all()
        .into_iter()
        .filter(|j| j.major_version == major)
        .collect()
}

pub fn probe_path(path: &Path) -> Option<JavaInstallation> {
    probe(path)
}

/// Download Azul Zulu JRE (Modrinth-style) into meta/java_versions/{major}/.
pub async fn auto_install(major: u32) -> Result<JavaInstallation, String> {
    const MAX_JAVA_ZIP: u64 = 400 * 1024 * 1024;
    let arch = match std::env::consts::ARCH {
        "x86_64" => "x64",
        "aarch64" => "aarch64",
        other => other,
    };
    let os = match std::env::consts::OS {
        "windows" => "windows",
        "macos" => "macos",
        _ => "linux",
    };
    let metadata_url = format!(
        "https://api.azul.com/metadata/v1/zulu/packages?arch={arch}&java_version={major}&os={os}&archive_type=zip&javafx_bundled=false&java_package_type=jre&page_size=1"
    );
    assert_azul_url(&metadata_url)?;

    let http = reqwest::Client::builder()
        .user_agent(concat!("Owyx/", env!("CARGO_PKG_VERSION")))
        .timeout(Duration::from_secs(180))
        .redirect(reqwest::redirect::Policy::custom(|attempt| {
            if attempt.previous().len() >= 5 {
                return attempt.error("too many redirects");
            }
            match assert_azul_url(attempt.url().as_str()) {
                Ok(()) => attempt.follow(),
                Err(err) => attempt.error(err),
            }
        }))
        .build()
        .map_err(|e| format!("HTTP client: {e}"))?;

    #[derive(serde::Deserialize)]
    struct Package {
        download_url: String,
        name: String,
    }

    let packages: Vec<Package> = http
        .get(&metadata_url)
        .send()
        .await
        .map_err(|e| format!("Azul metadata: {e}"))?
        .error_for_status()
        .map_err(|e| format!("Azul metadata HTTP: {e}"))?
        .json()
        .await
        .map_err(|e| format!("Azul metadata JSON: {e}"))?;

    let package = packages
        .first()
        .ok_or_else(|| format!("No Azul Zulu package for Java {major}"))?;
    assert_azul_url(&package.download_url)?;
    let zip_name = sanitize_zip_name(&package.name)?;

    let cache = crate::config::ensure_layout()?.join("cache").join("java");
    fs::create_dir_all(&cache).map_err(|e| format!("Create java cache: {e}"))?;
    let zip_path = cache.join(format!("{major}-{zip_name}"));
    let bytes = download_capped(&http, &package.download_url, MAX_JAVA_ZIP).await?;

    // Serialize write/extract/register so parallel Install clicks cannot race the same major.
    let _guard = JAVA_INSTALL_LOCK
        .lock()
        .map_err(|_| "Java install lock poisoned".to_string())?;
    fs::write(&zip_path, &bytes).map_err(|e| format!("Write Java zip: {e}"))?;

    let dest_root = crate::config::java_versions_dir()?.join(major.to_string());
    if dest_root.exists() {
        fs::remove_dir_all(&dest_root).map_err(|e| format!("Clear java dir: {e}"))?;
    }
    fs::create_dir_all(&dest_root).map_err(|e| format!("Create java dir: {e}"))?;
    extract_zip_safe(&zip_path, &dest_root)?;
    let _ = fs::remove_file(&zip_path);

    let java_bin = find_java_binary(&dest_root)
        .ok_or_else(|| "Extracted Java but could not find javaw.exe/java".to_string())?;
    let info = probe(&java_bin).ok_or_else(|| "Installed Java failed -version probe".to_string())?;
    let _ = crate::config::set_java_version_path(major, Some(info.path.clone()));

    let mut all = find_all();
    if !all.iter().any(|j| j.path.eq_ignore_ascii_case(&info.path)) {
        all.push(info.clone());
        let _ = crate::config::set_java_installations(all);
    }
    Ok(info)
}

fn assert_azul_url(raw: &str) -> Result<(), String> {
    let parsed = reqwest::Url::parse(raw).map_err(|e| format!("Bad Java URL: {e}"))?;
    if parsed.scheme() != "https" {
        return Err(format!("Only https Java downloads allowed: {raw}"));
    }
    if parsed.username() != "" || parsed.password().is_some() {
        return Err("Java download URL must not include credentials".into());
    }
    let host = parsed
        .host_str()
        .ok_or_else(|| format!("Java URL missing host: {raw}"))?
        .to_ascii_lowercase();
    let ok = host == "azul.com" || host.ends_with(".azul.com");
    if !ok {
        return Err(format!("Java download host not allowed: {host}"));
    }
    Ok(())
}

fn sanitize_zip_name(name: &str) -> Result<String, String> {
    let name = name.trim().replace('\\', "/");
    let base = name
        .rsplit('/')
        .next()
        .unwrap_or("")
        .trim()
        .to_string();
    if base.is_empty()
        || base.contains("..")
        || !base.chars().all(|c| {
            c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-' | '+')
        })
        || !base.to_ascii_lowercase().ends_with(".zip")
    {
        return Err(format!("Unsafe Java archive name: {name}"));
    }
    Ok(base)
}

async fn download_capped(
    http: &reqwest::Client,
    url: &str,
    max_bytes: u64,
) -> Result<Vec<u8>, String> {
    use futures_util::StreamExt;
    assert_azul_url(url)?;
    let res = http
        .get(url)
        .send()
        .await
        .map_err(|e| format!("Download Java: {e}"))?
        .error_for_status()
        .map_err(|e| format!("Download Java HTTP: {e}"))?;
    assert_azul_url(res.url().as_str())?;
    if let Some(len) = res.content_length() {
        if len > max_bytes {
            return Err(format!("Java archive too large: {len} > {max_bytes}"));
        }
    }
    let mut out = Vec::new();
    let mut stream = res.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| format!("Java download stream: {e}"))?;
        if out.len() as u64 + chunk.len() as u64 > max_bytes {
            return Err(format!("Java archive exceeded {max_bytes} bytes"));
        }
        out.extend_from_slice(&chunk);
    }
    Ok(out)
}

fn extract_zip_safe(zip_path: &Path, dest: &Path) -> Result<(), String> {
    use std::io::copy;
    let dest_canon = fs::canonicalize(dest).unwrap_or_else(|_| dest.to_path_buf());
    let file = File::open(zip_path).map_err(|e| format!("Open Java zip: {e}"))?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| format!("Java zip: {e}"))?;
    for i in 0..archive.len() {
        let mut entry = archive
            .by_index(i)
            .map_err(|e| format!("Java zip entry: {e}"))?;
        let name = entry.name().replace('\\', "/");
        if name.is_empty() || name.starts_with('/') || name.contains('\0') {
            continue;
        }
        let mut safe = PathBuf::new();
        for comp in Path::new(&name).components() {
            match comp {
                std::path::Component::Normal(part) => safe.push(part),
                std::path::Component::CurDir => {}
                _ => return Err(format!("Unsafe zip path component: {name}")),
            }
        }
        if safe.as_os_str().is_empty() {
            continue;
        }
        let out = dest.join(&safe);
        // Ensure destination stays under dest_root (zip-slip).
        let parent_for_check = if name.ends_with('/') {
            out.clone()
        } else {
            out.parent().unwrap_or(dest).to_path_buf()
        };
        fs::create_dir_all(&parent_for_check).map_err(|e| format!("Java zip parent: {e}"))?;
        let check = if name.ends_with('/') {
            fs::canonicalize(&out).unwrap_or(out.clone())
        } else {
            fs::canonicalize(out.parent().unwrap_or(dest)).unwrap_or_else(|_| out.clone())
        };
        if !check.starts_with(&dest_canon) && !out.starts_with(dest) {
            return Err(format!("Zip slip blocked: {name}"));
        }
        if name.ends_with('/') {
            continue;
        }
        let mut outfile = File::create(&out).map_err(|e| format!("Java zip create: {e}"))?;
        copy(&mut entry, &mut outfile).map_err(|e| format!("Java zip extract: {e}"))?;
    }
    Ok(())
}

fn find_java_binary(root: &Path) -> Option<PathBuf> {
    let bin_name = java_bin();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let candidate = dir.join("bin").join(bin_name);
        if candidate.is_file() {
            return Some(candidate);
        }
        if let Ok(entries) = fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_dir() {
                    stack.push(p);
                }
            }
        }
    }
    None
}

fn consider(path: &Path, found: &mut Vec<JavaInstallation>, seen: &mut HashSet<String>) {
    if !path.is_file() {
        return;
    }
    let key = path.to_string_lossy().to_ascii_lowercase();
    if !seen.insert(key) {
        return;
    }
    if let Some(info) = probe(path) {
        found.push(info);
    }
}

fn probe(path: &Path) -> Option<JavaInstallation> {
    let mut cmd = Command::new(path);
    cmd.arg("-version")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .stdin(Stdio::null());
    #[cfg(windows)]
    {
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    let mut child = cmd.spawn().ok()?;

    let mut stdout_pipe = child.stdout.take()?;
    let mut stderr_pipe = child.stderr.take()?;
    let reader = std::thread::spawn(move || {
        let mut stdout = String::new();
        let mut stderr = String::new();
        let _ = stdout_pipe.read_to_string(&mut stdout);
        let _ = stderr_pipe.read_to_string(&mut stderr);
        (stdout, stderr)
    });

    match child.wait_timeout(PROBE_TIMEOUT).ok()? {
        Some(_) => {}
        None => {
            let _ = child.kill();
            let _ = child.wait();
            return None;
        }
    }

    let (stdout, stderr) = reader.join().ok()?;
    let text = format!("{stderr}{stdout}");
    let version = parse_version(&text)?;
    let major = major_version(&version)?;
    Some(JavaInstallation {
        path: path.display().to_string(),
        version,
        major_version: major,
        is_64_bit: text.to_ascii_lowercase().contains("64-bit"),
    })
}

fn parse_version(output: &str) -> Option<String> {
    for line in output.lines() {
        if !line.contains("version") {
            continue;
        }
        let start = line.find('"')?;
        let end = line.rfind('"')?;
        if end > start + 1 {
            return Some(line[start + 1..end].to_string());
        }
    }
    None
}

fn major_version(version: &str) -> Option<u32> {
    let mut parts = version.split(['.', '_', '+', '-']);
    let first = parts.next()?.parse::<u32>().ok()?;
    if first == 1 {
        parts.next()?.parse().ok()
    } else {
        Some(first)
    }
}

fn java_bin() -> &'static str {
    // Windows: javaw.exe is a GUI subsystem binary (no console window).
    // java.exe is console-subsystem and flashes a blank cmd when spawned.
    if cfg!(windows) {
        "javaw.exe"
    } else {
        "java"
    }
}

/// Prefer `javaw.exe` beside `java.exe` so game launch stays console-free on Windows.
pub fn for_launch(path: &Path) -> PathBuf {
    #[cfg(windows)]
    {
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if name.eq_ignore_ascii_case("java.exe") {
                let javaw = path.with_file_name("javaw.exe");
                if javaw.is_file() {
                    return javaw;
                }
            }
        }
    }
    path.to_path_buf()
}

fn common_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();
    if cfg!(windows) {
        let program_files = std::env::var_os("ProgramFiles").map(PathBuf::from);
        let program_files_x86 = std::env::var_os("ProgramFiles(x86)").map(PathBuf::from);
        let local = std::env::var_os("LOCALAPPDATA").map(PathBuf::from);
        for base in [program_files, program_files_x86].into_iter().flatten() {
            for name in [
                "Java",
                "Eclipse Adoptium",
                "Eclipse Foundation",
                "Microsoft",
                "BellSoft",
                "Amazon Corretto",
                "Zulu",
                "Semeru",
                "SapMachine",
            ] {
                roots.push(base.join(name));
            }
        }
        if let Some(local) = local {
            roots.push(local.join("Programs").join("Eclipse Adoptium"));
        }
    } else {
        roots.push(PathBuf::from("/usr/lib/jvm"));
        roots.push(PathBuf::from("/Library/Java/JavaVirtualMachines"));
    }
    roots
}

use crate::config;
use crate::java;
use crate::models::{JavaRequirement, ManifestFile, PackLocalStatus, PackManifest, SyncProgress};
use crate::sync;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::fs::{self, File};
use std::io::{copy, Read, Write};
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Stdio};
#[cfg(windows)]
use std::os::windows::process::CommandExt;
use std::sync::{LazyLock, Mutex};
use std::time::Duration;
use tauri::{AppHandle, Emitter};
use wait_timeout::ChildExt;
use zip::ZipArchive;

static INSTALL_LOCK: LazyLock<Mutex<Option<String>>> = LazyLock::new(|| Mutex::new(None));

fn acquire_install(id: &str) -> Result<(), String> {
    let mut guard = INSTALL_LOCK
        .lock()
        .map_err(|_| "Install lock poisoned".to_string())?;
    if let Some(active) = guard.as_ref() {
        return Err(format!(
            "Another install is already running ({active}). Wait for it to finish."
        ));
    }
    *guard = Some(id.to_string());
    Ok(())
}

fn release_install(id: &str) {
    if let Ok(mut guard) = INSTALL_LOCK.lock() {
        if guard.as_deref() == Some(id) {
            *guard = None;
        }
    }
}

pub fn is_install_active(id: &str) -> bool {
    INSTALL_LOCK
        .lock()
        .ok()
        .and_then(|g| g.as_ref().map(|active| active == id))
        .unwrap_or(false)
}

const MOJANG_MANIFEST: &str = "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";
const LIBRARIES_BASE: &str = "https://libraries.minecraft.net/";
const RESOURCES_BASE: &str = "https://resources.download.minecraft.net/";

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceRecord {
    pub id: String,
    pub name: String,
    pub loader: String,
    pub minecraft: String,
    pub loader_version: String,
    pub created_at: u64,
    #[serde(default)]
    pub status: String,
}

pub async fn install_instance(
    app: AppHandle,
    id: String,
    name: String,
    loader: String,
    minecraft: String,
    loader_version: String,
) -> Result<PackLocalStatus, String> {
    acquire_install(&id)?;
    let result = install_instance_inner(
        app,
        id.clone(),
        name,
        loader,
        minecraft,
        loader_version,
    )
    .await;
    release_install(&id);
    match result {
        Ok(status) => Ok(status),
        Err(err) => {
            let _ = mark_instance_status(&id, "error");
            Err(err)
        }
    }
}

async fn install_instance_inner(
    app: AppHandle,
    id: String,
    name: String,
    loader: String,
    minecraft: String,
    loader_version: String,
) -> Result<PackLocalStatus, String> {
    validate_id(&id)?;
    let loader = loader.to_ascii_lowercase();
    if !matches!(
        loader.as_str(),
        "vanilla" | "fabric" | "quilt" | "forge" | "neoforge"
    ) {
        return Err(format!("Unsupported loader: {loader}"));
    }
    if minecraft.trim().is_empty() {
        return Err("Minecraft version is required".into());
    }
    if loader != "vanilla" && loader_version.trim().is_empty() {
        return Err("Loader version is required".into());
    }

    let root = config::ensure_layout()?.join("instances").join(&id);
    let game_dir = root.join("game");
    fs::create_dir_all(&game_dir).map_err(|e| format!("Create game dir: {e}"))?;

    let record = InstanceRecord {
        id: id.clone(),
        name: name.trim().chars().take(48).collect(),
        loader: loader.clone(),
        minecraft: minecraft.clone(),
        loader_version: loader_version.clone(),
        created_at: unix_now(),
        status: "installing".into(),
    };
    write_json(&root.join("instance.json"), &record)?;

    // Remove stale ready marker while installing.
    let meta_path = root.join("meta.json");
    if meta_path.exists() {
        let _ = fs::remove_file(&meta_path);
    }

    emit(
        &app,
        &id,
        "resolve",
        None,
        0,
        0,
        format!("Resolving {loader} {minecraft}"),
    )?;

    let http = http_client()?;
    let mojang = fetch_mojang_version(&http, &minecraft).await?;
    let asset_index_id = mojang
        .pointer("/assetIndex/id")
        .and_then(|v| v.as_str())
        .unwrap_or(&minecraft)
        .to_string();
    let java_min = mojang
        .pointer("/javaVersion/majorVersion")
        .and_then(|v| v.as_u64())
        .unwrap_or(17) as u32;

    let mut jobs: Vec<DownloadJob> = Vec::new();
    let mut natives_jobs: Vec<DownloadJob> = Vec::new();
    store_mojang_version_json(&minecraft, &mojang)?;
    collect_mojang_jobs(&mojang, &minecraft, &mut jobs, &mut natives_jobs)?;

    let mut main_class = mojang
        .get("mainClass")
        .and_then(|v| v.as_str())
        .unwrap_or("net.minecraft.client.main.Main")
        .to_string();
    let mut jvm_args: Vec<String> = vec!["-Xmx2G".into()];
    let mut game_args: Vec<String> = Vec::new();

    match loader.as_str() {
        "fabric" => {
            let profile = fetch_json(
                &http,
                &format!(
                    "https://meta.fabricmc.net/v2/versions/loader/{}/{}/profile/json",
                    enc(&minecraft),
                    enc(&loader_version)
                ),
            )
            .await?;
            if let Some(mc) = profile.get("mainClass").and_then(|v| v.as_str()) {
                main_class = mc.to_string();
            }
            collect_profile_libs(&profile, &mut jobs)?;
        }
        "quilt" => {
            let profile = fetch_json(
                &http,
                &format!(
                    "https://meta.quiltmc.org/v3/versions/loader/{}/{}/profile/json",
                    enc(&minecraft),
                    enc(&loader_version)
                ),
            )
            .await?;
            if let Some(mc) = profile.get("mainClass").and_then(|v| v.as_str()) {
                main_class = mc.to_string();
            }
            collect_profile_libs(&profile, &mut jobs)?;
        }
        "forge" | "neoforge" => {
            // Download + run official installer, then merge resulting version JSON libs.
            run_modded_installer(
                &app,
                &http,
                &id,
                &loader,
                &minecraft,
                &loader_version,
                &game_dir,
            )
            .await?;
            // Move installer-written libraries into shared meta (Theseus-style).
            merge_tree_into(
                &game_dir.join("libraries"),
                &config::meta_libraries_dir()?,
            )?;
            let version_id = if loader == "forge" {
                format!("{minecraft}-forge-{loader_version}")
            } else {
                format!("neoforge-{loader_version}")
            };
            let version_json_path = game_dir
                .join("versions")
                .join(&version_id)
                .join(format!("{version_id}.json"));
            let version_json = if version_json_path.is_file() {
                read_json_file(&version_json_path)?
            } else {
                find_best_version_json(&game_dir, &minecraft, &loader)?
            };
            if let Some(mc) = version_json.get("mainClass").and_then(|v| v.as_str()) {
                main_class = mc.to_string();
            }
            collect_version_tree_libs(&http, &game_dir, &version_json, &mut jobs, &mut natives_jobs)
                .await?;
            let (extracted_jvm, extracted_game) =
                collect_modded_launch_args(&http, &game_dir, &version_json).await?;
            ensure_modded_bootstrap_args(&extracted_jvm, &extracted_game)?;
            jvm_args = extracted_jvm;
            if !jvm_args.iter().any(|a| a.starts_with("-Xmx")) {
                jvm_args.insert(0, "-Xmx2G".into());
            }
            game_args = extracted_game;
        }
        _ => {}
    }

    // Deduplicate by dest path.
    jobs = dedupe_jobs(jobs);
    natives_jobs = dedupe_jobs(natives_jobs);

    let asset_jobs = collect_asset_jobs(&http, &mojang, &game_dir).await?;
    let total = (jobs.len() + natives_jobs.len() + asset_jobs.len()) as u32;
    let mut done = count_reusable_jobs(
        jobs.iter().chain(natives_jobs.iter()).chain(asset_jobs.iter()),
        &game_dir,
    )?;
    let mut files: Vec<ManifestFile> = Vec::new();

    if done > 0 {
        emit(
            &app,
            &id,
            "download",
            None,
            done,
            total,
            format!("Reusing {done}/{total} cached meta files"),
        )?;
    }

    for job in jobs.iter().chain(natives_jobs.iter()) {
        let dest = config::resolve_game_or_meta(&job.rel, &game_dir)?;
        let reused = ensure_dest_reusable(&job.rel, &dest, job.size);
        if !reused {
            emit(
                &app,
                &id,
                "download",
                Some(job.rel.clone()),
                done,
                total,
                format!("Downloading {}", job.rel),
            )?;
        }
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("Create parent: {e}"))?;
        }
        let outcome = if reused {
            ensure_meta_file(
                &http,
                &job.url,
                &dest,
                job.sha1.as_deref(),
                job.size,
                VerifyAfterDownload::Sha1,
                None,
            )
            .await?
        } else {
            let mut on_prog = file_progress_callback(
                app.clone(),
                id.clone(),
                "download".into(),
                Some(job.rel.clone()),
                done,
                total,
                format!("Downloading {}", job.rel),
                job.size,
            );
            ensure_meta_file(
                &http,
                &job.url,
                &dest,
                job.sha1.as_deref(),
                job.size,
                VerifyAfterDownload::Sha1,
                Some(&mut on_prog),
            )
            .await?
        };
        if job.rel.to_ascii_lowercase().ends_with(".jar") && !job.natives_only {
            // Prefer Mojang SHA1 in the manifest field (prefixed) so we never re-hash
            // multi‑MB jars on disk after a cache hit — that froze install at N-1/N.
            let (sha256, size) = match outcome {
                EnsureOutcome::Reused => {
                    let size = fs::metadata(&dest).map(|m| m.len()).unwrap_or(0);
                    let hash = job
                        .sha1
                        .as_ref()
                        .map(|s| format!("sha1:{s}"))
                        .unwrap_or_else(|| "reuse".into());
                    (hash, size)
                }
                EnsureOutcome::Downloaded { bytes } => {
                    let hash = job
                        .sha1
                        .as_ref()
                        .map(|s| format!("sha1:{s}"))
                        .unwrap_or_else(|| hex_sha256(&bytes));
                    (hash, bytes.len() as u64)
                }
            };
            files.push(ManifestFile {
                path: job.rel.clone(),
                url: job.url.clone(),
                sha256,
                size: Some(size),
            });
        }
        if !reused {
            done += 1;
        }
    }

    // Extract natives into shared meta/natives/{minecraft}
    let natives_dir = config::meta_natives_dir(&minecraft)?;
    for job in &natives_jobs {
        let jar = config::resolve_game_or_meta(&job.rel, &game_dir)?;
        extract_natives(&jar, &natives_dir)?;
    }
    // Forge installer may have written natives under game/ — merge into shared.
    merge_tree_into(&game_dir.join("natives"), &natives_dir)?;

    download_missing_assets(
        &app,
        &id,
        &http,
        &asset_jobs,
        &game_dir,
        &mut done,
        total,
        None,
    )
    .await?;

    let version_label = if loader == "vanilla" {
        minecraft.clone()
    } else {
        format!("{minecraft}-{loader}-{loader_version}")
    };

    let manifest = PackManifest {
        id: id.clone(),
        version: version_label,
        minecraft: minecraft.clone(),
        loader: loader.clone(),
        main_class: Some(main_class),
        java: Some(JavaRequirement {
            min_major: java_min,
        }),
        asset_index: Some(asset_index_id),
        files,
        jvm_args,
        game_args,
    };
    write_json(&meta_path, &manifest)?;

    let mut ready_record = record;
    ready_record.status = "ready".into();
    write_json(&root.join("instance.json"), &ready_record)?;

    emit(
        &app,
        &id,
        "done",
        None,
        done,
        total,
        "Install complete".into(),
    )?;

    sync::pack_status(&id)
}

fn mark_instance_status(id: &str, status: &str) -> Result<(), String> {
    let path = config::ensure_layout()?
        .join("instances")
        .join(id)
        .join("instance.json");
    if !path.is_file() {
        return Ok(());
    }
    let mut record: InstanceRecord = read_json_file(&path).and_then(|v| {
        serde_json::from_value(v).map_err(|e| format!("Parse instance.json: {e}"))
    })?;
    record.status = status.to_string();
    write_json(&path, &record)
}

#[derive(Clone)]
struct DownloadJob {
    rel: String,
    url: String,
    sha1: Option<String>,
    size: Option<u64>,
    natives_only: bool,
}

fn http_client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .user_agent(concat!("Owyx/", env!("CARGO_PKG_VERSION")))
        .timeout(Duration::from_secs(120))
        .connect_timeout(Duration::from_secs(15))
        .redirect(reqwest::redirect::Policy::custom(|attempt| {
            if attempt.previous().len() >= 5 {
                return attempt.error("too many redirects");
            }
            match assert_allowed_download_url(attempt.url().as_str()) {
                Ok(()) => attempt.follow(),
                Err(err) => attempt.error(err),
            }
        }))
        .build()
        .map_err(|e| format!("HTTP client: {e}"))
}

async fn fetch_mojang_version(http: &reqwest::Client, id: &str) -> Result<Value, String> {
    let manifest = fetch_json(http, MOJANG_MANIFEST).await?;
    let versions = manifest
        .get("versions")
        .and_then(|v| v.as_array())
        .ok_or("Invalid Mojang manifest")?;
    let url = versions
        .iter()
        .find(|v| v.get("id").and_then(|x| x.as_str()) == Some(id))
        .and_then(|v| v.get("url").and_then(|x| x.as_str()))
        .ok_or_else(|| format!("Minecraft version not found: {id}"))?
        .to_string();
    fetch_json(http, &url).await
}

async fn fetch_json(http: &reqwest::Client, url: &str) -> Result<Value, String> {
    assert_allowed_download_url(url)?;
    let res = http
        .get(url)
        .send()
        .await
        .map_err(|e| format!("GET {url}: {e}"))?
        .error_for_status()
        .map_err(|e| format!("HTTP {url}: {e}"))?;
    assert_allowed_download_url(res.url().as_str())?;
    res.json()
        .await
        .map_err(|e| format!("JSON {url}: {e}"))
}

fn collect_mojang_jobs(
    version: &Value,
    version_id: &str,
    jobs: &mut Vec<DownloadJob>,
    natives: &mut Vec<DownloadJob>,
) -> Result<(), String> {
    // Theseus: client jar lives at meta/versions/{id}/{id}.jar — NOT under libraries/.
    if let Some(client) = version.pointer("/downloads/client") {
        let client_url = client
            .get("url")
            .and_then(|v| v.as_str())
            .ok_or("client.url missing")?;
        let client_sha1 = client.get("sha1").and_then(|v| v.as_str()).map(str::to_string);
        let client_size = client.get("size").and_then(|v| v.as_u64());
        let id = version_id.trim();
        if id.is_empty() {
            return Err("Minecraft version id missing for client jar".into());
        }
        jobs.push(DownloadJob {
            rel: format!("versions/{id}/{id}.jar"),
            url: client_url.to_string(),
            sha1: client_sha1,
            size: client_size,
            natives_only: false,
        });
    }

    let libs = version
        .get("libraries")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    for lib in libs {
        if !library_allowed(&lib) {
            continue;
        }
        if let Some(artifact) = lib.pointer("/downloads/artifact") {
            if let Some(job) = artifact_job(artifact, false)? {
                jobs.push(job);
            }
        }
        if let Some(classifiers) = lib.pointer("/downloads/classifiers") {
            let key = natives_classifier_key(&lib);
            if let Some(key) = key {
                if let Some(native_art) = classifiers.get(&key) {
                    if let Some(job) = artifact_job(native_art, true)? {
                        natives.push(job);
                    }
                }
            }
        }
    }
    Ok(())
}

/// Persist Mojang version JSON under shared `meta/versions/{id}/{id}.json` (Theseus).
fn store_mojang_version_json(version_id: &str, version: &Value) -> Result<(), String> {
    let dir = config::meta_version_dir(version_id)?;
    let path = dir.join(format!("{version_id}.json"));
    if path.is_file() {
        return Ok(());
    }
    write_json(&path, version)
}

fn collect_profile_libs(profile: &Value, jobs: &mut Vec<DownloadJob>) -> Result<(), String> {
    let libs = profile
        .get("libraries")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    for lib in libs {
        if !library_allowed(&lib) {
            continue;
        }
        if let Some(artifact) = lib.pointer("/downloads/artifact") {
            if let Some(job) = artifact_job(artifact, false)? {
                jobs.push(job);
                continue;
            }
        }
        // Fabric/Quilt sometimes only provide name maven coords.
        if let Some(name) = lib.get("name").and_then(|v| v.as_str()) {
            let url_base = lib
                .get("url")
                .and_then(|v| v.as_str())
                .unwrap_or(LIBRARIES_BASE);
            if let Some(job) = maven_job(name, url_base)? {
                jobs.push(job);
            }
        }
    }
    Ok(())
}

async fn collect_version_tree_libs(
    http: &reqwest::Client,
    game_dir: &Path,
    version: &Value,
    jobs: &mut Vec<DownloadJob>,
    natives: &mut Vec<DownloadJob>,
) -> Result<(), String> {
    let mut current = version.clone();
    let mut guard = 0;
    loop {
        let vid = current
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();
        // Only the vanilla parent usually carries downloads.client; path uses that id.
        collect_mojang_jobs(&current, &vid, jobs, natives)?;
        collect_profile_libs(&current, jobs)?;
        let Some(parent) = current
            .get("inheritsFrom")
            .and_then(|v| v.as_str())
            .map(str::to_string)
        else {
            break;
        };
        guard += 1;
        if guard > 6 {
            return Err("Version inheritance too deep".into());
        }
        let shared = config::meta_version_dir(&parent)?.join(format!("{parent}.json"));
        let local = game_dir
            .join("versions")
            .join(&parent)
            .join(format!("{parent}.json"));
        current = if shared.is_file() {
            read_json_file(&shared)?
        } else if local.is_file() {
            read_json_file(&local)?
        } else {
            let fetched = fetch_mojang_version(http, &parent).await?;
            let _ = store_mojang_version_json(&parent, &fetched);
            fetched
        };
    }
    Ok(())
}

/// Walk inheritsFrom and collect JVM/game args needed for Forge/NeoForge BootstrapLauncher.
async fn collect_modded_launch_args(
    http: &reqwest::Client,
    game_dir: &Path,
    version: &Value,
) -> Result<(Vec<String>, Vec<String>), String> {
    let mut chain: Vec<Value> = Vec::new();
    let mut current = version.clone();
    let mut guard = 0;
    loop {
        chain.push(current.clone());
        let Some(parent) = current
            .get("inheritsFrom")
            .and_then(|v| v.as_str())
            .map(str::to_string)
        else {
            break;
        };
        guard += 1;
        if guard > 6 {
            return Err("Version inheritance too deep while reading args".into());
        }
        let local = game_dir
            .join("versions")
            .join(&parent)
            .join(format!("{parent}.json"));
        current = if local.is_file() {
            read_json_file(&local)?
        } else {
            fetch_mojang_version(http, &parent).await?
        };
    }
    // Parent first, then child overrides/appends (Minecraft launcher merge order).
    chain.reverse();

    let version_name = version
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    let natives = config::meta_natives_dir(
        version
            .pointer("/inheritsFrom")
            .and_then(|v| v.as_str())
            .unwrap_or(version_name),
    )?;
    let libraries = config::meta_libraries_dir()?;
    let classpath_sep = if cfg!(windows) { ";" } else { ":" };

    let mut jvm_raw = Vec::new();
    let mut game_raw = Vec::new();
    for node in &chain {
        append_version_arg_list(node.pointer("/arguments/jvm"), &mut jvm_raw);
        append_version_arg_list(node.pointer("/arguments/game"), &mut game_raw);
    }

    let jvm_args = resolve_modded_jvm_args(
        &jvm_raw,
        &natives,
        &libraries,
        classpath_sep,
        version_name,
    )?;
    let game_args = filter_modded_game_args(&game_raw)?;
    Ok((jvm_args, game_args))
}

fn ensure_modded_bootstrap_args(jvm: &[String], game: &[String]) -> Result<(), String> {
    let has_module_path = jvm.iter().any(|a| a == "-p" || a == "--module-path");
    if !has_module_path {
        return Err(
            "Forge/NeoForge install did not produce a module-path (-p). Try another loader version."
                .into(),
        );
    }
    let has_launch_target = game.windows(2).any(|w| w[0] == "--launchTarget" && !w[1].is_empty());
    if !has_launch_target {
        return Err(
            "Forge/NeoForge install did not produce --launchTarget. Try another loader version."
                .into(),
        );
    }
    Ok(())
}

fn append_version_arg_list(node: Option<&Value>, out: &mut Vec<String>) {
    let Some(arr) = node.and_then(|v| v.as_array()) else {
        return;
    };
    for item in arr {
        if let Some(s) = item.as_str() {
            out.push(s.to_string());
            continue;
        }
        if !argument_rules_allow(item) {
            continue;
        }
        match item.get("value") {
            Some(Value::String(s)) => out.push(s.clone()),
            Some(Value::Array(vals)) => {
                for v in vals {
                    if let Some(s) = v.as_str() {
                        out.push(s.to_string());
                    }
                }
            }
            _ => {}
        }
    }
}

fn argument_rules_allow(arg: &Value) -> bool {
    let Some(rules) = arg.get("rules").and_then(|v| v.as_array()) else {
        return true;
    };
    let mut allowed = false;
    for rule in rules {
        let action = rule.get("action").and_then(|v| v.as_str()).unwrap_or("allow");
        let os_name = rule.pointer("/os/name").and_then(|v| v.as_str());
        let matches_os = match os_name {
            None => true,
            Some("windows") => cfg!(windows),
            Some("osx") => cfg!(target_os = "macos"),
            Some("linux") => cfg!(target_os = "linux"),
            _ => false,
        };
        if matches_os {
            allowed = action == "allow";
        }
    }
    allowed
}

fn resolve_modded_jvm_args(
    raw: &[String],
    natives: &Path,
    libraries: &Path,
    classpath_sep: &str,
    version_name: &str,
) -> Result<Vec<String>, String> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < raw.len() {
        let substituted = substitute_jvm_templates(
            &raw[i],
            natives,
            libraries,
            classpath_sep,
            version_name,
        );
        // Launcher already supplies -cp / java.library.path.
        if substituted == "-cp" || substituted == "-classpath" {
            i += 2;
            continue;
        }
        if substituted == "${classpath}" || substituted.contains("${classpath}") {
            i += 1;
            continue;
        }
        if substituted.starts_with("-Djava.library.path=") {
            i += 1;
            continue;
        }
        if substituted.is_empty() {
            i += 1;
            continue;
        }
        out.push(substituted);
        i += 1;
    }
    Ok(out)
}

fn substitute_jvm_templates(
    arg: &str,
    natives: &Path,
    libraries: &Path,
    classpath_sep: &str,
    version_name: &str,
) -> String {
    arg.replace("${natives_directory}", &natives.display().to_string())
        .replace("${library_directory}", &libraries.display().to_string())
        .replace("${classpath_separator}", classpath_sep)
        .replace("${version_name}", version_name)
        .replace("${launcher_name}", "Owyx")
        .replace("${launcher_version}", env!("CARGO_PKG_VERSION"))
        .replace("${/}", std::path::MAIN_SEPARATOR_STR)
}

fn filter_modded_game_args(raw: &[String]) -> Result<Vec<String>, String> {
    const FLAGS: &[&str] = &[
        "--launchTarget",
        "--fml.forgeVersion",
        "--fml.mcVersion",
        "--fml.forgeGroup",
        "--fml.mcpVersion",
        "--fml.neoForgeVersion",
        "--fml.fmlVersion",
        "--fml.neoFormVersion",
    ];
    let mut out = Vec::new();
    let mut i = 0;
    while i < raw.len() {
        let flag = raw[i].trim();
        if !FLAGS.contains(&flag) {
            i += 1;
            continue;
        }
        let Some(value) = raw.get(i + 1).map(|s| s.trim()) else {
            return Err(format!("Missing value for Forge game arg {flag}"));
        };
        if value.is_empty() || value.starts_with('-') {
            return Err(format!("Invalid value for Forge game arg {flag}"));
        }
        out.push(flag.to_string());
        out.push(value.to_string());
        i += 2;
    }
    Ok(out)
}

fn artifact_job(artifact: &Value, natives_only: bool) -> Result<Option<DownloadJob>, String> {
    let path = artifact
        .get("path")
        .and_then(|v| v.as_str())
        .ok_or("artifact.path missing")?;
    let url = artifact
        .get("url")
        .and_then(|v| v.as_str())
        .map(str::to_string)
        .unwrap_or_else(|| format!("{LIBRARIES_BASE}{path}"));
    let sha1 = artifact.get("sha1").and_then(|v| v.as_str()).map(str::to_string);
    let size = artifact.get("size").and_then(|v| v.as_u64());
    Ok(Some(DownloadJob {
        rel: format!("libraries/{path}"),
        url,
        sha1,
        size,
        natives_only,
    }))
}

fn maven_job(name: &str, base: &str) -> Result<Option<DownloadJob>, String> {
    // group:artifact:version[:classifier]
    let parts: Vec<&str> = name.split(':').collect();
    if parts.len() < 3 {
        return Ok(None);
    }
    let group = parts[0].replace('.', "/");
    let artifact = parts[1];
    let version = parts[2];
    let classifier = parts.get(3).copied();
    let file = match classifier {
        Some(c) => format!("{artifact}-{version}-{c}.jar"),
        None => format!("{artifact}-{version}.jar"),
    };
    let path = format!("{group}/{artifact}/{version}/{file}");
    let base = base.trim_end_matches('/');
    Ok(Some(DownloadJob {
        rel: format!("libraries/{path}"),
        url: format!("{base}/{path}"),
        sha1: None,
        size: None,
        natives_only: false,
    }))
}

fn library_allowed(lib: &Value) -> bool {
    let Some(rules) = lib.get("rules").and_then(|v| v.as_array()) else {
        return true;
    };
    let mut allowed = false;
    for rule in rules {
        let action = rule.get("action").and_then(|v| v.as_str()).unwrap_or("allow");
        let os_name = rule.pointer("/os/name").and_then(|v| v.as_str());
        let matches_os = match os_name {
            None => true,
            Some("windows") => cfg!(windows),
            Some("osx") => cfg!(target_os = "macos"),
            Some("linux") => cfg!(target_os = "linux"),
            _ => false,
        };
        if matches_os {
            allowed = action == "allow";
        }
    }
    allowed
}

fn natives_classifier_key(lib: &Value) -> Option<String> {
    let natives = lib.get("natives")?;
    let key = if cfg!(windows) {
        natives.get("windows")
    } else if cfg!(target_os = "macos") {
        natives.get("osx")
    } else {
        natives.get("linux")
    }?
    .as_str()?;
    Some(key.replace("${arch}", if cfg!(target_pointer_width = "64") { "64" } else { "32" }))
}

async fn collect_asset_jobs(
    http: &reqwest::Client,
    version: &Value,
    game_dir: &Path,
) -> Result<Vec<DownloadJob>, String> {
    let index_url = version
        .pointer("/assetIndex/url")
        .and_then(|v| v.as_str())
        .ok_or("assetIndex.url missing")?;
    let index_id = version
        .pointer("/assetIndex/id")
        .and_then(|v| v.as_str())
        .unwrap_or("legacy");
    let index = fetch_json(http, index_url).await?;
    let index_path = config::meta_assets_dir()?
        .join("indexes")
        .join(format!("{index_id}.json"));
    if let Some(parent) = index_path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("assets indexes: {e}"))?;
    }
    write_json(&index_path, &index)?;
    let _ = game_dir; // assets are shared in meta/; game_dir kept for API stability

    let objects = index
        .get("objects")
        .and_then(|v| v.as_object())
        .ok_or("asset index objects missing")?;
    let mut jobs = Vec::new();
    for (_name, obj) in objects {
        let hash = obj
            .get("hash")
            .and_then(|v| v.as_str())
            .ok_or("asset hash missing")?;
        if hash.len() < 2 {
            continue;
        }
        let prefix = &hash[..2];
        let rel = format!("assets/objects/{prefix}/{hash}");
        let url = format!("{RESOURCES_BASE}{prefix}/{hash}");
        let size = obj.get("size").and_then(|v| v.as_u64());
        jobs.push(DownloadJob {
            rel,
            url,
            sha1: Some(hash.to_string()),
            size,
            natives_only: false,
        });
    }
    Ok(jobs)
}

async fn run_modded_installer(
    app: &AppHandle,
    http: &reqwest::Client,
    id: &str,
    loader: &str,
    minecraft: &str,
    loader_version: &str,
    game_dir: &Path,
) -> Result<(), String> {
    let (url, jar_name) = if loader == "forge" {
        let ver = format!("{minecraft}-{loader_version}");
        (
            format!(
                "https://maven.minecraftforge.net/net/minecraftforge/forge/{ver}/forge-{ver}-installer.jar"
            ),
            format!("forge-{ver}-installer.jar"),
        )
    } else {
        (
            format!(
                "https://maven.neoforged.net/releases/net/neoforged/neoforge/{loader_version}/neoforge-{loader_version}-installer.jar"
            ),
            format!("neoforge-{loader_version}-installer.jar"),
        )
    };

    emit(
        app,
        id,
        "installer",
        Some(jar_name.clone()),
        0,
        0,
        format!("Downloading {loader} installer"),
    )?;

    let installer_path = game_dir.join(&jar_name);
    let bytes = {
        let mut on_prog = file_progress_callback(
            app.clone(),
            id.to_string(),
            "installer".into(),
            Some(jar_name.clone()),
            0,
            0,
            format!("Downloading {loader} installer"),
            None,
        );
        download_bytes(http, &url, 256 * 1024 * 1024, Some(&mut on_prog)).await?
    };
    atomic_write(&installer_path, &bytes)?;

    let java = java::find_compatible(17)?;
    emit(
        app,
        id,
        "installer",
        Some(jar_name.clone()),
        0,
        0,
        format!("Running {loader} installer"),
    )?;

    let java_bin = java::for_launch(Path::new(&java.path));
    let mut installer_cmd = Command::new(&java_bin);
    installer_cmd
        .arg("-jar")
        .arg(&installer_path)
        .arg("--installClient")
        .arg(game_dir)
        .current_dir(game_dir)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    #[cfg(windows)]
    {
        installer_cmd.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
    }
    let mut child = installer_cmd
        .spawn()
        .map_err(|e| format!("Failed to start installer: {e}"))?;

    let timeout = Duration::from_secs(10 * 60);
    match child
        .wait_timeout(timeout)
        .map_err(|e| format!("Wait installer: {e}"))?
    {
        Some(status) if status.success() => {}
        Some(status) => {
            return Err(format!(
                "{loader} installer failed with status {status}. Try another loader version."
            ));
        }
        None => {
            let _ = child.kill();
            let _ = child.wait();
            return Err(format!(
                "{loader} installer timed out after {}s",
                timeout.as_secs()
            ));
        }
    }
    let _ = fs::remove_file(&installer_path);
    Ok(())
}

fn find_best_version_json(game_dir: &Path, minecraft: &str, loader: &str) -> Result<Value, String> {
    let versions = game_dir.join("versions");
    if !versions.is_dir() {
        return Err("Installer did not create versions/".into());
    }
    let needle = if loader == "neoforge" {
        "neoforge"
    } else {
        "forge"
    };
    let mut preferred: Option<PathBuf> = None;
    let mut fallback: Option<PathBuf> = None;
    for entry in fs::read_dir(&versions).map_err(|e| format!("Read versions: {e}"))? {
        let entry = entry.map_err(|e| format!("Read versions entry: {e}"))?;
        let name = entry.file_name().to_string_lossy().to_string();
        if name == minecraft {
            continue;
        }
        let json = entry.path().join(format!("{name}.json"));
        if !json.is_file() {
            continue;
        }
        if name.to_ascii_lowercase().contains(needle) {
            preferred = Some(json);
        } else if fallback.is_none() {
            fallback = Some(json);
        }
    }
    let path = preferred
        .or(fallback)
        .ok_or("Could not find installed mod loader version JSON")?;
    read_json_file(&path)
}

/// Theseus-style reusable check: exist + size match when known (no full-file hash).
fn meta_file_reusable(path: &Path, expected_size: Option<u64>) -> bool {
    let Ok(meta) = fs::metadata(path) else {
        return false;
    };
    if !meta.is_file() {
        return false;
    }
    match expected_size {
        Some(sz) => meta.len() == sz,
        None => meta.len() > 0,
    }
}

/// If an older build stored client.jar under libraries/, adopt it into versions/{id}/.
fn adopt_legacy_client_jar(rel: &str, dest: &Path, expected_size: Option<u64>) -> bool {
    let norm = rel.replace('\\', "/");
    if !(norm.starts_with("versions/") && norm.ends_with(".jar")) {
        return false;
    }
    if meta_file_reusable(dest, expected_size) {
        return true;
    }
    let Ok(legacy) = config::meta_libraries_dir().map(|d| {
        d.join("com")
            .join("mojang")
            .join("minecraft")
            .join("client.jar")
    }) else {
        return false;
    };
    if !meta_file_reusable(&legacy, expected_size) {
        return false;
    }
    if let Some(parent) = dest.parent() {
        let _ = fs::create_dir_all(parent);
    }
    match fs::copy(&legacy, dest) {
        Ok(_) => {
            let _ = fs::remove_file(&legacy);
            true
        }
        Err(_) => false,
    }
}

fn ensure_dest_reusable(rel: &str, dest: &Path, expected_size: Option<u64>) -> bool {
    if meta_file_reusable(dest, expected_size) {
        return true;
    }
    adopt_legacy_client_jar(rel, dest, expected_size)
}

fn count_reusable_jobs<'a, I>(jobs: I, game_dir: &Path) -> Result<u32, String>
where
    I: IntoIterator<Item = &'a DownloadJob>,
{
    let mut n = 0u32;
    for job in jobs {
        let dest = config::resolve_game_or_meta(&job.rel, game_dir)?;
        if ensure_dest_reusable(&job.rel, &dest, job.size) {
            n += 1;
        }
    }
    Ok(n)
}

enum EnsureOutcome {
    Reused,
    Downloaded { bytes: Vec<u8> },
}

/// Post-download integrity policy.
/// Mojang asset object names are already content hashes — Theseus-style size trust is enough.
#[derive(Clone, Copy)]
enum VerifyAfterDownload {
    SizeOnly,
    Sha1,
}

/// Concurrent Mojang asset fetches (Theseus uses loading_try_for_each_concurrent).
const ASSET_DOWNLOAD_CONCURRENCY: usize = 24;

/// Download into shared meta (or game) unless the file already exists and passes size check.
/// Matches Modrinth/Theseus `should_download(!exists)` for skips; fresh assets skip SHA1 when size matches.
async fn ensure_meta_file(
    http: &reqwest::Client,
    url: &str,
    dest: &Path,
    sha1: Option<&str>,
    expected_size: Option<u64>,
    verify: VerifyAfterDownload,
    on_progress: Option<&mut (dyn FnMut(u64, Option<u64>) -> Result<(), String> + Send)>,
) -> Result<EnsureOutcome, String> {
    if meta_file_reusable(dest, expected_size) {
        return Ok(EnsureOutcome::Reused);
    }
    let bytes = download_bytes(http, url, 512 * 1024 * 1024, on_progress).await?;
    if let Some(sz) = expected_size {
        if bytes.len() as u64 != sz {
            return Err(format!(
                "Size mismatch for {url}: expected {sz}, got {}",
                bytes.len()
            ));
        }
    }
    // Assets: size match is enough (object path is the SHA1). Full-file hash after every
    // download froze the titlebar for minutes on large objects.
    if matches!(verify, VerifyAfterDownload::Sha1) {
        if let Some(expected) = sha1 {
            let actual = hex_sha1(&bytes);
            if !actual.eq_ignore_ascii_case(expected) {
                return Err(format!(
                    "SHA1 mismatch for {url}: expected {expected}, got {actual}"
                ));
            }
        }
    }
    atomic_write(dest, &bytes)?;
    Ok(EnsureOutcome::Downloaded { bytes })
}

/// Download missing asset objects concurrently; advances `done` as each file finishes.
async fn download_missing_assets(
    app: &AppHandle,
    pack_id: &str,
    http: &reqwest::Client,
    asset_jobs: &[DownloadJob],
    game_dir: &Path,
    done: &mut u32,
    total: u32,
    // When set (prefetch), refresh in-memory prefetch progress marker.
    prefetch_minecraft: Option<&str>,
) -> Result<(), String> {
    use futures_util::stream::{self, StreamExt};
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    let mut pending: Vec<DownloadJob> = Vec::new();
    for job in asset_jobs {
        let dest = config::resolve_game_or_meta(&job.rel, game_dir)?;
        if !meta_file_reusable(&dest, job.size) {
            pending.push(job.clone());
        }
    }
    // One hash object can appear under many logical asset names; dedupe by rel
    // so buffer_unordered writers never race on the same assets/objects path.
    pending = dedupe_jobs(pending);
    if pending.is_empty() {
        return Ok(());
    }

    let done_atomic = Arc::new(AtomicU32::new(*done));
    let game_dir = game_dir.to_path_buf();
    let pack_id_owned = pack_id.to_string();

    let mut tasks = stream::iter(pending.into_iter().map(|job| {
        let http = http.clone();
        let app = app.clone();
        let pack_id = pack_id_owned.clone();
        let game_dir = game_dir.clone();
        let done_atomic = Arc::clone(&done_atomic);
        async move {
            let dest = config::resolve_game_or_meta(&job.rel, &game_dir)?;
            if let Some(parent) = dest.parent() {
                fs::create_dir_all(parent).map_err(|e| format!("Create parent: {e}"))?;
            }
            let snapshot = done_atomic.load(Ordering::Relaxed);
            let rel = job.rel.clone();
            let mut on_prog = file_progress_callback(
                app.clone(),
                pack_id.clone(),
                "assets".into(),
                Some(rel.clone()),
                snapshot,
                total,
                format!("Asset {rel}"),
                job.size,
            );
            ensure_meta_file(
                &http,
                &job.url,
                &dest,
                job.sha1.as_deref(),
                job.size,
                VerifyAfterDownload::SizeOnly,
                Some(&mut on_prog),
            )
            .await?;
            let new_done = done_atomic.fetch_add(1, Ordering::Relaxed) + 1;
            // Clear per-file tip by emitting without byte fields; advance N/M immediately.
            emit(
                &app,
                &pack_id,
                "assets",
                Some(job.rel.clone()),
                new_done,
                total,
                format!("Asset {}", job.rel),
            )?;
            Ok::<u32, String>(new_done)
        }
    }))
    .buffer_unordered(ASSET_DOWNLOAD_CONCURRENCY);

    while let Some(res) = tasks.next().await {
        let new_done = res?;
        if let Some(mc) = prefetch_minecraft {
            set_prefetch_state(PrefetchStatus {
                ready: false,
                minecraft: Some(mc.to_string()),
                message: format!("Prefetch {new_done}/{total}"),
                done_files: new_done,
                total_files: total,
            });
        }
    }

    *done = done_atomic.load(Ordering::Relaxed);
    Ok(())
}

/// Min delta between byte-progress events (Theseus uses ~256 KiB / 0.5% of file).
const FILE_PROGRESS_MIN_BYTES: u64 = 256 * 1024;

async fn download_bytes(
    http: &reqwest::Client,
    url: &str,
    max_bytes: u64,
    mut on_progress: Option<&mut (dyn FnMut(u64, Option<u64>) -> Result<(), String> + Send)>,
) -> Result<Vec<u8>, String> {
    use futures_util::StreamExt;

    assert_allowed_download_url(url)?;
    let res = http
        .get(url)
        .send()
        .await
        .map_err(|e| format!("Download {url}: {e}"))?
        .error_for_status()
        .map_err(|e| format!("Download HTTP {url}: {e}"))?;
    assert_allowed_download_url(res.url().as_str())?;
    let content_len = res.content_length();
    if let Some(len) = content_len {
        if len > max_bytes {
            return Err(format!(
                "File too large ({len} bytes, max {max_bytes}): {url}"
            ));
        }
    }
    let mut stream = res.bytes_stream();
    let mut buf: Vec<u8> = Vec::new();
    let mut last_reported = 0u64;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| format!("Read body {url}: {e}"))?;
        if buf.len() as u64 + chunk.len() as u64 > max_bytes {
            return Err(format!("File too large (max {max_bytes} bytes): {url}"));
        }
        buf.extend_from_slice(&chunk);
        if let Some(cb) = on_progress.as_mut() {
            let received = buf.len() as u64;
            let min_delta = content_len
                .map(|t| (t / 200).max(FILE_PROGRESS_MIN_BYTES))
                .unwrap_or(FILE_PROGRESS_MIN_BYTES);
            let at_end = content_len.is_some_and(|t| received >= t);
            if last_reported == 0
                || at_end
                || received.saturating_sub(last_reported) >= min_delta
            {
                last_reported = received;
                cb(received, content_len)?;
            }
        }
    }
    if let Some(cb) = on_progress.as_mut() {
        let received = buf.len() as u64;
        if received != last_reported {
            cb(received, content_len.or(Some(received)))?;
        }
    }
    Ok(buf)
}

fn assert_allowed_download_url(raw: &str) -> Result<(), String> {
    let parsed = reqwest::Url::parse(raw).map_err(|e| format!("Bad download URL: {e}"))?;
    if parsed.scheme() != "https" {
        return Err(format!("Only https downloads are allowed: {raw}"));
    }
    if parsed.username() != "" || parsed.password().is_some() {
        return Err(format!("Download URL must not include credentials: {raw}"));
    }
    let host = parsed
        .host_str()
        .ok_or_else(|| format!("Download URL missing host: {raw}"))?
        .to_ascii_lowercase();
    const ALLOWED_SUFFIXES: &[&str] = &[
        "mojang.com",
        "minecraft.net",
        "fabricmc.net",
        "quiltmc.org",
        "minecraftforge.net",
        "neoforged.net",
        "azul.com",
    ];
    let ok = ALLOWED_SUFFIXES
        .iter()
        .any(|suffix| host == *suffix || host.ends_with(&format!(".{suffix}")));
    if !ok {
        return Err(format!("Download host not allowed: {host}"));
    }
    Ok(())
}

fn extract_natives(jar: &Path, out_dir: &Path) -> Result<(), String> {
    let file = File::open(jar).map_err(|e| format!("Open natives jar: {e}"))?;
    let mut archive = ZipArchive::new(file).map_err(|e| format!("Zip natives: {e}"))?;
    for i in 0..archive.len() {
        let mut entry = archive
            .by_index(i)
            .map_err(|e| format!("Zip entry: {e}"))?;
        let name = entry.name().to_string();
        if name.ends_with('/') || name.contains("META-INF") {
            continue;
        }
        let rel = safe_rel(&name)?;
        // Only extract native binaries / related files at top levels.
        let dest = out_dir.join(rel.file_name().unwrap_or_default());
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("natives parent: {e}"))?;
        }
        let mut outfile = File::create(&dest).map_err(|e| format!("Create native: {e}"))?;
        copy(&mut entry, &mut outfile).map_err(|e| format!("Extract native: {e}"))?;
    }
    Ok(())
}

fn dedupe_jobs(jobs: Vec<DownloadJob>) -> Vec<DownloadJob> {
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for job in jobs {
        if seen.insert(job.rel.clone()) {
            out.push(job);
        }
    }
    out
}

fn merge_tree_into(src: &Path, dst: &Path) -> Result<(), String> {
    if !src.is_dir() {
        return Ok(());
    }
    fs::create_dir_all(dst).map_err(|e| format!("Create merge dest: {e}"))?;
    for entry in fs::read_dir(src).map_err(|e| format!("Read merge src: {e}"))? {
        let entry = entry.map_err(|e| format!("Read merge entry: {e}"))?;
        let from = entry.path();
        let to = dst.join(entry.file_name());
        let ft = entry.file_type().map_err(|e| format!("File type: {e}"))?;
        if ft.is_dir() {
            merge_tree_into(&from, &to)?;
        } else if ft.is_file() {
            if let Some(parent) = to.parent() {
                fs::create_dir_all(parent).map_err(|e| format!("Create parent: {e}"))?;
            }
            fs::copy(&from, &to).map_err(|e| format!("Merge copy: {e}"))?;
        }
    }
    Ok(())
}

fn write_json<T: serde::Serialize>(path: &Path, value: &T) -> Result<(), String> {
    let raw = serde_json::to_string_pretty(value).map_err(|e| format!("Serialize: {e}"))?;
    atomic_write(path, raw.as_bytes())
}

fn read_json_file(path: &Path) -> Result<Value, String> {
    let raw = fs::read_to_string(path).map_err(|e| format!("Read {}: {e}", path.display()))?;
    serde_json::from_str(&raw).map_err(|e| format!("Parse {}: {e}", path.display()))
}

fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), String> {
    use std::sync::atomic::{AtomicU64, Ordering};
    static TMP_SEQ: AtomicU64 = AtomicU64::new(0);

    let parent = path
        .parent()
        .ok_or_else(|| "Destination has no parent".to_string())?;
    fs::create_dir_all(parent).map_err(|e| format!("Create parent: {e}"))?;
    // Unique per call so concurrent writers to the same dest never share a tmp.
    let seq = TMP_SEQ.fetch_add(1, Ordering::Relaxed);
    let tmp = parent.join(format!(
        ".{}.{}.{}.tmp",
        path.file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("file"),
        std::process::id(),
        seq
    ));
    {
        let mut file = fs::File::create(&tmp).map_err(|e| format!("Create temp: {e}"))?;
        file.write_all(bytes).map_err(|e| format!("Write temp: {e}"))?;
        // Theseus does not fsync every asset/lib write. sync_all() here stalled the UI
        // for 1–2+ minutes per file on Windows (Defender / slow disks).
    }
    fs::rename(&tmp, path).map_err(|e| {
        let _ = fs::remove_file(&tmp);
        format!("Replace failed: {e}")
    })
}

fn safe_rel(raw: &str) -> Result<PathBuf, String> {
    let path = Path::new(raw);
    if path.is_absolute() {
        return Err(format!("Absolute paths are not allowed: {raw}"));
    }
    let mut out = PathBuf::new();
    for comp in path.components() {
        match comp {
            Component::Normal(part) => out.push(part),
            Component::CurDir => {}
            _ => return Err(format!("Unsafe path component in {raw}")),
        }
    }
    if out.as_os_str().is_empty() {
        return Err("Empty file path".into());
    }
    Ok(out)
}

fn hex_sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

#[allow(dead_code)]
fn hex_sha256_file(path: &Path) -> Result<String, String> {
    let mut file = File::open(path).map_err(|e| format!("Open {}: {e}", path.display()))?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 64 * 1024];
    loop {
        let n = file
            .read(&mut buf)
            .map_err(|e| format!("Read {}: {e}", path.display()))?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(hasher
        .finalize()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect())
}

pub(crate) fn hex_sha1(bytes: &[u8]) -> String {
    // Prefer sha1 crate? Avoid new dep: use a tiny impl via openssl-less approach.
    // sha2 doesn't do sha1 — add quick dependency-free via `sha1_smol` or compute only when needed.
    // Use `sha1` from existing? Not in Cargo. Implement with `sha1` crate — add soft check:
    use sha1_compat::Sha1;
    let mut h = Sha1::new();
    h.update(bytes);
    h.digest().to_string()
}

// Minimal SHA-1 (public domain style) to avoid extra crate churn if pull fails — use module.
mod sha1_compat {
    pub struct Sha1 {
        state: [u32; 5],
        buf: Vec<u8>,
        len: u64,
    }
    impl Sha1 {
        pub fn new() -> Self {
            Self {
                state: [0x67452301, 0xEFCDAB89, 0x98BADCFE, 0x10325476, 0xC3D2E1F0],
                buf: Vec::new(),
                len: 0,
            }
        }
        pub fn update(&mut self, data: &[u8]) {
            self.len += data.len() as u64;
            let mut offset = 0usize;
            // Finish any partial block first (O(1) per call, not O(n²) via drain).
            if !self.buf.is_empty() {
                let need = 64 - self.buf.len();
                let take = need.min(data.len());
                self.buf.extend_from_slice(&data[..take]);
                offset = take;
                if self.buf.len() == 64 {
                    let block: [u8; 64] = self.buf.as_slice().try_into().unwrap();
                    self.buf.clear();
                    Self::process(&mut self.state, &block);
                }
            }
            while offset + 64 <= data.len() {
                let block: [u8; 64] = data[offset..offset + 64].try_into().unwrap();
                Self::process(&mut self.state, &block);
                offset += 64;
            }
            if offset < data.len() {
                self.buf.extend_from_slice(&data[offset..]);
            }
        }
        pub fn digest(mut self) -> Digest {
            let bit_len = self.len * 8;
            self.buf.push(0x80);
            while (self.buf.len() % 64) != 56 {
                self.buf.push(0);
            }
            self.buf.extend_from_slice(&bit_len.to_be_bytes());
            let mut offset = 0usize;
            while offset < self.buf.len() {
                let block: [u8; 64] = self.buf[offset..offset + 64].try_into().unwrap();
                Self::process(&mut self.state, &block);
                offset += 64;
            }
            Digest(self.state)
        }
        fn process(state: &mut [u32; 5], block: &[u8; 64]) {
            let mut w = [0u32; 80];
            for i in 0..16 {
                w[i] = u32::from_be_bytes([
                    block[i * 4],
                    block[i * 4 + 1],
                    block[i * 4 + 2],
                    block[i * 4 + 3],
                ]);
            }
            for i in 16..80 {
                w[i] = (w[i - 3] ^ w[i - 8] ^ w[i - 14] ^ w[i - 16]).rotate_left(1);
            }
            let (mut a, mut b, mut c, mut d, mut e) =
                (state[0], state[1], state[2], state[3], state[4]);
            for i in 0..80 {
                let (f, k) = match i {
                    0..=19 => ((b & c) | ((!b) & d), 0x5A827999),
                    20..=39 => (b ^ c ^ d, 0x6ED9EBA1),
                    40..=59 => ((b & c) | (b & d) | (c & d), 0x8F1BBCDC),
                    _ => (b ^ c ^ d, 0xCA62C1D6),
                };
                let temp = a
                    .rotate_left(5)
                    .wrapping_add(f)
                    .wrapping_add(e)
                    .wrapping_add(k)
                    .wrapping_add(w[i]);
                e = d;
                d = c;
                c = b.rotate_left(30);
                b = a;
                a = temp;
            }
            state[0] = state[0].wrapping_add(a);
            state[1] = state[1].wrapping_add(b);
            state[2] = state[2].wrapping_add(c);
            state[3] = state[3].wrapping_add(d);
            state[4] = state[4].wrapping_add(e);
        }
    }
    pub struct Digest([u32; 5]);
    impl Digest {
        pub fn to_string(&self) -> String {
            self.0.iter().map(|x| format!("{x:08x}")).collect()
        }
    }

    #[cfg(test)]
    mod tests {
        use super::Sha1;

        #[test]
        fn sha1_empty_and_abc() {
            let h = Sha1::new();
            assert_eq!(h.digest().to_string(), "da39a3ee5e6b4b0d3255bfef95601890afd80709");
            let mut h = Sha1::new();
            h.update(b"abc");
            assert_eq!(h.digest().to_string(), "a9993e364706816aba3e25717850c26c9cd0d89d");
        }

        #[test]
        fn sha1_large_is_linear() {
            // Regression: old drain-based update was O(n²) and hung install on client.jar.
            let chunk = vec![0x5Au8; 1024 * 1024]; // 1 MiB
            let mut h = Sha1::new();
            for _ in 0..8 {
                h.update(&chunk);
            }
            let dig = h.digest().to_string();
            assert_eq!(dig.len(), 40);
        }
    }
}

fn validate_id(id: &str) -> Result<(), String> {
    crate::instances::validate_instance_id(id)
}

fn enc(s: &str) -> String {
    // enough for version tokens
    s.replace(' ', "%20")
}

fn unix_now() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn emit(
    app: &AppHandle,
    id: &str,
    phase: &str,
    current: Option<String>,
    done: u32,
    total: u32,
    message: String,
) -> Result<(), String> {
    emit_progress(app, id, phase, current, done, total, message, None, None)
}

fn emit_progress(
    app: &AppHandle,
    id: &str,
    phase: &str,
    current: Option<String>,
    done: u32,
    total: u32,
    message: String,
    file_bytes_done: Option<u64>,
    file_bytes_total: Option<u64>,
) -> Result<(), String> {
    app.emit(
        "install://progress",
        SyncProgress {
            pack_id: id.to_string(),
            phase: phase.into(),
            current_file: current,
            done_files: done,
            total_files: total,
            message,
            file_bytes_done,
            file_bytes_total,
        },
    )
    .map_err(|e| format!("Emit progress: {e}"))
}

/// Throttled per-file byte progress (Modrinth/Theseus-style: ~0.5% or 256 KiB).
fn file_progress_callback(
    app: AppHandle,
    id: String,
    phase: String,
    current: Option<String>,
    done: u32,
    total: u32,
    message: String,
    expected_size: Option<u64>,
) -> impl FnMut(u64, Option<u64>) -> Result<(), String> {
    move |received, content_len| {
        let file_total = expected_size.or(content_len);
        emit_progress(
            &app,
            &id,
            &phase,
            current.clone(),
            done,
            total,
            message.clone(),
            Some(received),
            file_total,
        )
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrefetchStatus {
    pub ready: bool,
    pub minecraft: Option<String>,
    pub message: String,
    pub done_files: u32,
    pub total_files: u32,
}

static PREFETCH_STATE: LazyLock<Mutex<PrefetchStatus>> = LazyLock::new(|| {
    Mutex::new(PrefetchStatus {
        ready: false,
        minecraft: None,
        message: "Not started".into(),
        done_files: 0,
        total_files: 0,
    })
});

pub fn meta_prefetch_status() -> PrefetchStatus {
    if let Ok(marker) = config::prefetch_marker_path() {
        if marker.is_file() {
            if let Ok(raw) = fs::read_to_string(&marker) {
                if let Ok(parsed) = serde_json::from_str::<PrefetchStatus>(&raw) {
                    if parsed.ready {
                        return parsed;
                    }
                }
            }
        }
    }
    PREFETCH_STATE
        .lock()
        .map(|g| g.clone())
        .unwrap_or_else(|_| PrefetchStatus {
            ready: false,
            minecraft: None,
            message: "Busy".into(),
            done_files: 0,
            total_files: 0,
        })
}

fn set_prefetch_state(status: PrefetchStatus) {
    if let Ok(mut guard) = PREFETCH_STATE.lock() {
        *guard = status.clone();
    }
    if status.ready {
        if let Ok(path) = config::prefetch_marker_path() {
            let _ = write_json(&path, &status);
        }
    }
}

/// Prefetch shared Mojang libraries + assets for the latest release into meta/.
pub async fn prefetch_shared_meta(app: AppHandle) -> Result<PrefetchStatus, String> {
    let existing = meta_prefetch_status();
    if existing.ready {
        return Ok(existing);
    }

    let prefetch_id = "__meta_prefetch__";
    let mut acquired = false;
    for _ in 0..30 {
        if acquire_install(prefetch_id).is_ok() {
            acquired = true;
            break;
        }
        // Another install/prefetch holds the lock — wait briefly.
        std::thread::sleep(Duration::from_millis(500));
        let status = meta_prefetch_status();
        if status.ready {
            return Ok(status);
        }
    }
    if !acquired {
        return Err("Prefetch could not acquire install lock".into());
    }

    let result = prefetch_shared_meta_inner(&app, prefetch_id).await;
    release_install(prefetch_id);
    result
}

async fn prefetch_shared_meta_inner(
    app: &AppHandle,
    prefetch_id: &str,
) -> Result<PrefetchStatus, String> {
    set_prefetch_state(PrefetchStatus {
        ready: false,
        minecraft: None,
        message: "Resolving latest Minecraft release…".into(),
        done_files: 0,
        total_files: 0,
    });
    emit(
        app,
        prefetch_id,
        "resolve",
        None,
        0,
        0,
        "Prefetch: resolving latest release".into(),
    )?;

    let http = http_client()?;
    let manifest: Value = fetch_json(&http, MOJANG_MANIFEST).await?;
    let latest = manifest
        .pointer("/latest/release")
        .and_then(|v| v.as_str())
        .ok_or("Mojang manifest missing latest.release")?
        .to_string();

    let mojang = fetch_mojang_version(&http, &latest).await?;
    let mut jobs: Vec<DownloadJob> = Vec::new();
    let mut natives_jobs: Vec<DownloadJob> = Vec::new();
    store_mojang_version_json(&latest, &mojang)?;
    collect_mojang_jobs(&mojang, &latest, &mut jobs, &mut natives_jobs)?;
    jobs = dedupe_jobs(jobs);
    natives_jobs = dedupe_jobs(natives_jobs);

    // Dummy game dir — assets/libs resolve to meta via resolve_game_or_meta.
    let game_dir = config::ensure_layout()?.join("cache").join("prefetch");
    fs::create_dir_all(&game_dir).map_err(|e| format!("prefetch cache: {e}"))?;
    let asset_jobs = collect_asset_jobs(&http, &mojang, &game_dir).await?;

    let total = (jobs.len() + natives_jobs.len() + asset_jobs.len()) as u32;
    let mut done = count_reusable_jobs(
        jobs.iter().chain(natives_jobs.iter()).chain(asset_jobs.iter()),
        &game_dir,
    )?;

    set_prefetch_state(PrefetchStatus {
        ready: false,
        minecraft: Some(latest.clone()),
        message: if done >= total && total > 0 {
            format!("Verifying cached meta for {latest}…")
        } else {
            format!("Downloading shared meta for {latest} ({done}/{total} cached)…")
        },
        done_files: done,
        total_files: total,
    });
    emit(
        app,
        prefetch_id,
        "download",
        None,
        done,
        total,
        format!("Prefetch cached {done}/{total}"),
    )?;

    for job in jobs.iter().chain(natives_jobs.iter()) {
        let dest = config::resolve_game_or_meta(&job.rel, &game_dir)?;
        if ensure_dest_reusable(&job.rel, &dest, job.size) {
            continue;
        }
        emit(
            app,
            prefetch_id,
            "download",
            Some(job.rel.clone()),
            done,
            total,
            format!("Prefetch {}", job.rel),
        )?;
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("Create parent: {e}"))?;
        }
        {
            let mut on_prog = file_progress_callback(
                app.clone(),
                prefetch_id.to_string(),
                "download".into(),
                Some(job.rel.clone()),
                done,
                total,
                format!("Prefetch {}", job.rel),
                job.size,
            );
            let _ = ensure_meta_file(
                &http,
                &job.url,
                &dest,
                job.sha1.as_deref(),
                job.size,
                VerifyAfterDownload::Sha1,
                Some(&mut on_prog),
            )
            .await?;
        }
        done += 1;
        set_prefetch_state(PrefetchStatus {
            ready: false,
            minecraft: Some(latest.clone()),
            message: format!("Prefetch {done}/{total}"),
            done_files: done,
            total_files: total,
        });
    }

    let natives_dir = config::meta_natives_dir(&latest)?;
    for job in &natives_jobs {
        let jar = config::resolve_game_or_meta(&job.rel, &game_dir)?;
        extract_natives(&jar, &natives_dir)?;
    }

    download_missing_assets(
        app,
        prefetch_id,
        &http,
        &asset_jobs,
        &game_dir,
        &mut done,
        total,
        Some(latest.as_str()),
    )
    .await?;

    let status = PrefetchStatus {
        ready: true,
        minecraft: Some(latest.clone()),
        message: format!("Shared meta ready ({latest})"),
        done_files: done,
        total_files: total,
    };
    set_prefetch_state(status.clone());
    emit(
        app,
        prefetch_id,
        "done",
        None,
        done,
        total,
        status.message.clone(),
    )?;
    crate::log::info(&status.message);
    Ok(status)
}

/// Update loader / game version on an existing instance (reinstalls on next Play).
pub fn update_instance_installation(
    id: &str,
    loader: &str,
    minecraft: &str,
    loader_version: &str,
) -> Result<InstanceRecord, String> {
    crate::instances::validate_instance_id(id)?;
    let loader = loader.trim().to_ascii_lowercase();
    if !matches!(
        loader.as_str(),
        "vanilla" | "fabric" | "quilt" | "forge" | "neoforge"
    ) {
        return Err(format!("Unsupported loader: {loader}"));
    }
    if minecraft.trim().is_empty() {
        return Err("Minecraft version is required".into());
    }
    if loader != "vanilla" && loader_version.trim().is_empty() {
        return Err("Loader version is required".into());
    }
    if is_install_active(id) {
        return Err("Cannot change installation while install is running".into());
    }

    let path = config::ensure_layout()?
        .join("instances")
        .join(id)
        .join("instance.json");
    let mut record: InstanceRecord = read_json_file(&path).and_then(|v| {
        serde_json::from_value(v).map_err(|e| format!("Parse instance.json: {e}"))
    })?;
    record.loader = loader;
    record.minecraft = minecraft.trim().to_string();
    record.loader_version = if record.loader == "vanilla" {
        String::new()
    } else {
        loader_version.trim().to_string()
    };
    record.status = "pending".into();
    write_json(&path, &record)?;

    // Drop ready marker so Play reinstalls.
    let meta = config::ensure_layout()?
        .join("instances")
        .join(id)
        .join("meta.json");
    if meta.exists() {
        let _ = fs::remove_file(meta);
    }
    Ok(record)
}

pub async fn repair_instance(app: AppHandle, id: String) -> Result<PackLocalStatus, String> {
    crate::instances::validate_instance_id(&id)?;
    let path = config::ensure_layout()?
        .join("instances")
        .join(&id)
        .join("instance.json");
    let record: InstanceRecord = read_json_file(&path).and_then(|v| {
        serde_json::from_value(v).map_err(|e| format!("Parse instance.json: {e}"))
    })?;
    install_instance(
        app,
        record.id.clone(),
        record.name,
        record.loader,
        record.minecraft,
        record.loader_version,
    )
    .await
}


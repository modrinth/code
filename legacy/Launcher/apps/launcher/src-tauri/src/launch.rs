use crate::config;
use crate::java::{self, JavaInstallation};
use crate::log as owyx_log;
use crate::models::PackManifest;
use crate::sync;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::{LazyLock, Mutex};
use tauri::{AppHandle, Emitter};

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

static RUNNING: LazyLock<Mutex<HashMap<String, Option<u32>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// Last launch log path per pack (kept after exit so Live Log can still show it).
static LIVE_LOGS: LazyLock<Mutex<HashMap<String, PathBuf>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

fn apply_no_window(_cmd: &mut Command) {
    #[cfg(windows)]
    {
        _cmd.creation_flags(CREATE_NO_WINDOW);
    }
}

pub fn live_log_path(pack_id: &str) -> Option<PathBuf> {
    LIVE_LOGS
        .lock()
        .ok()
        .and_then(|g| g.get(pack_id).cloned())
}

fn set_live_log(pack_id: &str, path: PathBuf) {
    if let Ok(mut guard) = LIVE_LOGS.lock() {
        guard.insert(pack_id.to_string(), path);
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchResult {
    pub pack_id: String,
    pub mode: String,
    pub pid: u32,
    pub java_path: String,
    pub java_version: String,
    pub log_path: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessFinishedEvent {
    pub pack_id: String,
    pub pid: u32,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct LaunchOverrides {
    pub memory_mb: Option<u32>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub jvm_args: Option<Vec<String>>,
    pub java_path: Option<String>,
    pub env_vars: Option<Vec<(String, String)>>,
    pub pre_launch: Option<String>,
    pub wrapper: Option<String>,
    pub post_exit: Option<String>,
    /// Join this server on launch (Theseus-style servers.dat + quickPlay).
    pub server_address: Option<String>,
    pub server_port: Option<u16>,
}

pub fn is_pack_running(pack_id: &str) -> bool {
    let Ok(mut guard) = RUNNING.lock() else {
        return false;
    };
    match guard.get(pack_id) {
        Some(None) => true, // reserved / launching
        Some(Some(pid)) if process_alive(*pid) => true,
        Some(Some(_)) => {
            guard.remove(pack_id);
            false
        }
        None => false,
    }
}

fn reserve_launch(pack_id: &str) -> Result<(), String> {
    let mut guard = RUNNING
        .lock()
        .map_err(|_| "Process map lock poisoned".to_string())?;
    match guard.get(pack_id) {
        Some(None) => return Err("Instance is already launching".into()),
        Some(Some(pid)) if process_alive(*pid) => {
            return Err("Instance is already running. Stop it first.".into());
        }
        Some(Some(_)) => {
            guard.remove(pack_id);
        }
        None => {}
    }
    guard.insert(pack_id.to_string(), None);
    Ok(())
}

fn commit_launch(pack_id: &str, pid: u32) {
    if let Ok(mut guard) = RUNNING.lock() {
        guard.insert(pack_id.to_string(), Some(pid));
    }
}

fn abort_launch(pack_id: &str) {
    if let Ok(mut guard) = RUNNING.lock() {
        if matches!(guard.get(pack_id), Some(None)) {
            guard.remove(pack_id);
        }
    }
}

fn clear_running(pack_id: &str, expected_pid: Option<u32>) {
    let Ok(mut guard) = RUNNING.lock() else {
        return;
    };
    match (guard.get(pack_id), expected_pid) {
        // Exit watcher: only clear when the committed PID still matches.
        (Some(Some(pid)), Some(expected)) if *pid == expected => {
            guard.remove(pack_id);
        }
        // Explicit clear without a PID — never touch a fresh launching placeholder.
        (Some(Some(_)), None) => {
            guard.remove(pack_id);
        }
        // Some(None) is a reserve_launch placeholder for a newer Play — leave it.
        _ => {}
    }
}

fn spawn_exit_watcher(
    app: AppHandle,
    pack_id: String,
    pid: u32,
    mut child: Child,
    post_exit: Option<(String, PathBuf)>,
) {
    std::thread::spawn(move || {
        let _ = child.wait();
        clear_running(&pack_id, Some(pid));
        let _ = app.emit(
            "process://finished",
            ProcessFinishedEvent {
                pack_id: pack_id.clone(),
                pid,
            },
        );
        if let Some((cmd, cwd)) = post_exit {
            let _ = run_hook_command(&cmd, &cwd);
        }
    });
}

pub fn stop_pack(pack_id: &str) -> Result<(), String> {
    validate_pack_id(pack_id)?;
    let pid = {
        let guard = RUNNING
            .lock()
            .map_err(|_| "Process map lock poisoned".to_string())?;
        match guard.get(pack_id) {
            Some(None) => return Err("Instance is still starting".into()),
            Some(Some(pid)) => *pid,
            None => return Err("No running process for this instance".into()),
        }
    };
    // Keep map entry until wait() clears it so UI stays consistent until exit.
    kill_pid(pid)?;
    Ok(())
}

fn process_alive(pid: u32) -> bool {
    #[cfg(windows)]
    {
        // Prefer OpenProcess over shelling out to tasklist (faster + reliable).
        // SAFETY: Win32 process query APIs with a valid PID and cleaned-up handle.
        unsafe {
            #[link(name = "kernel32")]
            extern "system" {
                fn OpenProcess(access: u32, inherit: i32, pid: u32) -> *mut core::ffi::c_void;
                fn CloseHandle(handle: *mut core::ffi::c_void) -> i32;
                fn GetExitCodeProcess(handle: *mut core::ffi::c_void, code: *mut u32) -> i32;
            }
            const PROCESS_QUERY_LIMITED_INFORMATION: u32 = 0x1000;
            const STILL_ACTIVE: u32 = 259;
            let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
            if handle.is_null() {
                return false;
            }
            let mut code = 0u32;
            let ok = GetExitCodeProcess(handle, &mut code);
            CloseHandle(handle);
            ok != 0 && code == STILL_ACTIVE
        }
    }
    #[cfg(not(windows))]
    {
        Path::new(&format!("/proc/{pid}")).exists()
    }
}

fn kill_pid(pid: u32) -> Result<(), String> {
    #[cfg(windows)]
    {
        let mut cmd = Command::new("taskkill");
        cmd.args(["/PID", &pid.to_string(), "/T", "/F"]);
        apply_no_window(&mut cmd);
        let status = cmd
            .status()
            .map_err(|e| format!("taskkill failed: {e}"))?;
        if !status.success() {
            return Err(format!("Failed to stop process {pid}"));
        }
        Ok(())
    }
    #[cfg(not(windows))]
    {
        let status = Command::new("kill")
            .args(["-9", &pid.to_string()])
            .status()
            .map_err(|e| format!("kill failed: {e}"))?;
        if !status.success() {
            return Err(format!("Failed to stop process {pid}"));
        }
        Ok(())
    }
}

pub fn launch_pack(
    app: AppHandle,
    pack_id: &str,
    nick: &str,
    overrides: LaunchOverrides,
) -> Result<LaunchResult, String> {
    validate_pack_id(pack_id)?;
    let nick = nick.trim();
    if nick.is_empty() {
        return Err("Nick is required to launch".into());
    }
    if !is_safe_nick(nick) {
        return Err("Nick may only contain letters, numbers, and _".into());
    }

    let status = sync::pack_status(pack_id)?;
    if !status.ready {
        return Err("Pack is not ready. Download/verify it before Play.".into());
    }

    let instance = config::ensure_layout()?.join("instances").join(pack_id);
    let meta_path = instance.join("meta.json");
    let raw = fs::read_to_string(&meta_path).map_err(|e| format!("Read meta.json: {e}"))?;
    let manifest: PackManifest =
        serde_json::from_str(&raw).map_err(|e| format!("Invalid meta.json: {e}"))?;

    let min_major = manifest
        .java
        .as_ref()
        .map(|j| j.min_major)
        .unwrap_or(17);
    let java = if let Some(path) = overrides
        .java_path
        .as_ref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
    {
        let p = PathBuf::from(path);
        java::probe_path(&p).ok_or_else(|| format!("Custom Java path is invalid: {path}"))?
    } else {
        java::find_compatible(min_major)?
    };

    let game_dir = instance.join("game");
    let jars = collect_manifest_jars(&game_dir, &manifest)?;
    let natives_dir = resolve_natives_dir(&game_dir, &manifest.minecraft)?;
    let assets_dir = resolve_assets_dir(&game_dir)?;
    let (mut log_file, log_path) = {
        let (path, file) = owyx_log::create_launch_log(pack_id)?;
        (file, path)
    };

    owyx_log::write_line(
        &mut log_file,
        &format!(
            "Owyx launch start pack={pack_id} nick={nick} java={} ({})",
            java.path, java.version
        ),
    )?;

    if jars.is_empty() {
        if pack_id.starts_with("demo-") {
            return launch_smoke(pack_id, &java, log_path, log_file);
        }
        return Err(
            "No .jar files in manifest. Sync a full Fabric client pack before Play.".into(),
        );
    }

    let loader = manifest.loader.to_ascii_lowercase();
    if !matches!(
        loader.as_str(),
        "fabric" | "vanilla" | "quilt" | "forge" | "neoforge"
    ) {
        return Err(format!("Loader '{loader}' is not supported yet"));
    }

    let main_class = sanitize_main_class(
        manifest
            .main_class
            .as_deref()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or(match loader.as_str() {
                "fabric" => "net.fabricmc.loader.impl.launch.knot.KnotClient",
                "quilt" => "org.quiltmc.loader.impl.launch.knot.KnotClient",
                "forge" | "neoforge" => "cpw.mods.bootstraplauncher.BootstrapLauncher",
                _ => "net.minecraft.client.main.Main",
            }),
    )?;

    let is_modded = matches!(loader.as_str(), "forge" | "neoforge");

    // Remote gameArgs ignored for Fabric/Quilt/Vanilla. Forge FML flags are allowlisted.
    let mut jvm_args = sanitize_jvm_args(&manifest.jvm_args, &game_dir, is_modded)?;
    #[cfg(target_os = "macos")]
    {
        if !jvm_args.iter().any(|a| a == "-XstartOnFirstThread") {
            jvm_args.insert(0, "-XstartOnFirstThread".into());
        }
    }
    if let Some(extra) = overrides.jvm_args.as_ref() {
        let sanitized_extra = sanitize_jvm_args(extra, &game_dir, false)?;
        jvm_args.extend(sanitized_extra);
    }
    // UI memory wins over any -Xmx/-Xms from manifest or advanced JVM args.
    apply_memory_override(&mut jvm_args, overrides.memory_mb);

    let mut extra_game_args = if is_modded {
        sanitize_modded_game_args(&manifest.game_args)?
    } else {
        Vec::new()
    };
    apply_window_override(&mut extra_game_args, overrides.width, overrides.height);

    let catalog_meta = crate::catalog::read_catalog_meta(pack_id);
    let join_addr = overrides
        .server_address
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .or_else(|| catalog_meta.as_ref().map(|m| m.address.clone()));
    let join_port = overrides
        .server_port
        .or_else(|| catalog_meta.as_ref().map(|m| m.port))
        .unwrap_or(25565);
    if let Some(addr) = join_addr.as_deref() {
        crate::catalog::write_servers_dat(&game_dir, &manifest.id, addr, join_port)?;
        if crate::catalog::supports_quick_play(&manifest.minecraft) {
            if let Ok(safe) = crate::catalog::sanitize_server_addr(addr) {
                extra_game_args.push("--quickPlayMultiplayer".into());
                extra_game_args.push(format!("{safe}:{join_port}"));
            }
        } else if let Ok(safe) = crate::catalog::sanitize_server_addr(addr) {
            extra_game_args.push("--server".into());
            extra_game_args.push(safe);
            extra_game_args.push("--port".into());
            extra_game_args.push(join_port.to_string());
        }
    }

    let classpath = join_classpath(&jars);
    let mut args: Vec<String> = Vec::new();
    args.extend(jvm_args);
    args.push(format!(
        "-Djava.library.path={}",
        natives_dir.display()
    ));
    args.push("-cp".into());
    args.push(classpath);
    args.push(main_class.clone());

    args.push("--username".into());
    args.push(nick.to_string());
    args.push("--version".into());
    args.push(sanitize_token(&manifest.minecraft, "minecraft version")?);
    args.push("--gameDir".into());
    args.push(game_dir.display().to_string());
    args.push("--assetsDir".into());
    args.push(assets_dir.display().to_string());
    args.push("--assetIndex".into());
    args.push(sanitize_token(
        manifest
            .asset_index
            .as_deref()
            .unwrap_or(&manifest.minecraft),
        "asset index",
    )?);
    args.push("--accessToken".into());
    args.push("0".into());
    args.push("--userType".into());
    args.push("legacy".into());
    args.extend(extra_game_args);

    owyx_log::write_line(
        &mut log_file,
        &format!("mode={loader} main={main_class} jars={}", jars.len()),
    )?;
    owyx_log::write_line(
        &mut log_file,
        &format!("cmd={} {}", java.path, args.join(" ")),
    )?;

    drop(log_file);
    let log_out = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .map_err(|e| format!("Reopen launch log: {e}"))?;
    let log_err = log_out
        .try_clone()
        .map_err(|e| format!("Clone launch log: {e}"))?;

    if let Some(pre) = overrides
        .pre_launch
        .as_ref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
    {
        run_hook_command(pre, &game_dir)?;
    }

    let (program, program_args) = if let Some(wrapper) = overrides
        .wrapper
        .as_ref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
    {
        let mut parts = shell_split(wrapper);
        if parts.is_empty() {
            return Err("Wrapper command is empty".into());
        }
        let prog = parts.remove(0);
        let java_launch = java::for_launch(Path::new(&java.path));
        parts.push(java_launch.display().to_string());
        parts.extend(args);
        (prog, parts)
    } else {
        (
            java::for_launch(Path::new(&java.path))
                .display()
                .to_string(),
            args,
        )
    };

    let mut cmd = Command::new(&program);
    cmd.args(&program_args)
        .current_dir(&game_dir)
        .stdin(Stdio::null())
        .stdout(Stdio::from(log_out))
        .stderr(Stdio::from(log_err));
    apply_no_window(&mut cmd);

    if let Some(vars) = overrides.env_vars.as_ref() {
        for (k, v) in vars {
            let key = k.trim();
            if key.is_empty() || key.contains('=') || key.contains('\0') {
                continue;
            }
            cmd.env(key, v);
        }
    }

    reserve_launch(pack_id)?;
    let child = match cmd.spawn() {
        Ok(child) => child,
        Err(e) => {
            abort_launch(pack_id);
            return Err(format!("Failed to start Minecraft process: {e}"));
        }
    };
    let pid = child.id();
    commit_launch(pack_id, pid);
    set_live_log(pack_id, log_path.clone());

    let post_exit = overrides
        .post_exit
        .as_ref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| (s.to_string(), game_dir.clone()));
    // Keep Child handle and wait on a background thread so RUNNING clears on exit
    // (closing the game with window X must update Stop → Play without pressing Stop).
    spawn_exit_watcher(app, pack_id.to_string(), pid, child, post_exit);

    Ok(LaunchResult {
        pack_id: pack_id.to_string(),
        mode: loader,
        pid,
        java_path: java.path,
        java_version: java.version,
        log_path: log_path.display().to_string(),
        message: format!("Game started (pid {pid})"),
    })
}

fn run_hook_command(command: &str, cwd: &Path) -> Result<(), String> {
    let parts = shell_split(command);
    if parts.is_empty() {
        return Ok(());
    }
    let mut cmd = Command::new(&parts[0]);
    if parts.len() > 1 {
        cmd.args(&parts[1..]);
    }
    cmd.current_dir(cwd)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    apply_no_window(&mut cmd);
    let status = cmd
        .status()
        .map_err(|e| format!("Hook command failed: {e}"))?;
    if !status.success() {
        return Err(format!("Hook command exited with {status}"));
    }
    Ok(())
}

fn shell_split(input: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut in_quotes = false;
    for ch in input.chars() {
        match ch {
            '"' => in_quotes = !in_quotes,
            c if c.is_whitespace() && !in_quotes => {
                if !cur.is_empty() {
                    out.push(std::mem::take(&mut cur));
                }
            }
            c => cur.push(c),
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

fn apply_memory_override(jvm_args: &mut Vec<String>, memory_mb: Option<u32>) {
    let Some(mb) = memory_mb else {
        return;
    };
    let mb = mb.clamp(512, 32768);
    jvm_args.retain(|a| !(a.starts_with("-Xmx") || a.starts_with("-Xms")));
    jvm_args.insert(0, format!("-Xmx{mb}M"));
}

fn apply_window_override(game_args: &mut Vec<String>, width: Option<u32>, height: Option<u32>) {
    let strip_pair = |args: &mut Vec<String>, flag: &str| {
        let mut i = 0;
        while i < args.len() {
            if args[i] == flag {
                args.remove(i);
                if i < args.len() {
                    args.remove(i);
                }
            } else {
                i += 1;
            }
        }
    };
    if let Some(w) = width.filter(|w| (640..=7680).contains(w)) {
        strip_pair(game_args, "--width");
        game_args.push("--width".into());
        game_args.push(w.to_string());
    }
    if let Some(h) = height.filter(|h| (480..=4320).contains(h)) {
        strip_pair(game_args, "--height");
        game_args.push("--height".into());
        game_args.push(h.to_string());
    }
}

fn launch_smoke(
    pack_id: &str,
    java: &JavaInstallation,
    log_path: PathBuf,
    mut log_file: File,
) -> Result<LaunchResult, String> {
    owyx_log::write_line(
        &mut log_file,
        "mode=smoke reason=no-jars (demo pack) running java -version",
    )?;
    drop(log_file);

    let log_out = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .map_err(|e| format!("Reopen launch log: {e}"))?;
    let log_err = log_out
        .try_clone()
        .map_err(|e| format!("Clone launch log: {e}"))?;

    let mut cmd = Command::new(java::for_launch(Path::new(&java.path)));
    cmd.arg("-version")
        .stdin(Stdio::null())
        .stdout(Stdio::from(log_out))
        .stderr(Stdio::from(log_err));
    apply_no_window(&mut cmd);
    let child = cmd
        .spawn()
        .map_err(|e| format!("Failed to start Java smoke process: {e}"))?;
    let pid = child.id();
    set_live_log(pack_id, log_path.clone());

    Ok(LaunchResult {
        pack_id: pack_id.to_string(),
        mode: "smoke".into(),
        pid,
        java_path: java.path.clone(),
        java_version: java.version.clone(),
        log_path: log_path.display().to_string(),
        message: format!(
            "Java OK (smoke). Full Fabric launch needs client jars in the pack. Log: {}",
            log_path.display()
        ),
    })
}

fn sanitize_main_class(value: &str) -> Result<String, String> {
    let value = value.trim();
    if value.is_empty() || value.len() > 256 {
        return Err("Invalid main class".into());
    }
    if !value
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '_' || c == '$')
    {
        return Err("Main class contains forbidden characters".into());
    }
    if value.contains("..") {
        return Err("Main class looks invalid".into());
    }
    Ok(value.to_string())
}

fn sanitize_jvm_args(
    args: &[String],
    game_dir: &Path,
    allow_modded: bool,
) -> Result<Vec<String>, String> {
    let mut out = Vec::with_capacity(args.len());
    let mut expect: Option<JvmValueKind> = None;
    for arg in args {
        if let Some(kind) = expect.take() {
            if !allow_modded {
                return Err("Modded JVM flags are only allowed for Forge/NeoForge".into());
            }
            out.push(sanitize_jvm_value(arg, kind, game_dir)?);
            continue;
        }
        let trimmed = arg.trim();
        if trimmed.is_empty() {
            return Err("Empty JVM argument".into());
        }
        if allow_modded {
            match trimmed {
                "-p" | "--module-path" => {
                    out.push(trimmed.to_string());
                    expect = Some(JvmValueKind::ModulePath);
                    continue;
                }
                "--add-modules" => {
                    out.push(trimmed.to_string());
                    expect = Some(JvmValueKind::AddModules);
                    continue;
                }
                "--add-opens" => {
                    out.push(trimmed.to_string());
                    expect = Some(JvmValueKind::AddOpensExports);
                    continue;
                }
                "--add-exports" => {
                    out.push(trimmed.to_string());
                    expect = Some(JvmValueKind::AddOpensExports);
                    continue;
                }
                _ => {}
            }
        }
        out.push(sanitize_jvm_arg(trimmed, game_dir, allow_modded)?);
    }
    if expect.is_some() {
        return Err("Incomplete JVM argument (missing value)".into());
    }
    Ok(out)
}

#[derive(Clone, Copy)]
enum JvmValueKind {
    ModulePath,
    AddModules,
    AddOpensExports,
}

fn sanitize_jvm_arg(arg: &str, game_dir: &Path, allow_modded: bool) -> Result<String, String> {
    let arg = arg.trim();
    if arg.is_empty() || arg.len() > 8192 {
        return Err("Invalid JVM argument length".into());
    }
    if arg.chars().any(|c| c.is_control() || c == '\0') {
        return Err("JVM argument contains forbidden characters".into());
    }
    if is_dangerous_jvm_arg(arg) {
        return Err(format!("JVM argument not allowed: {arg}"));
    }

    let memory = regex_is_memory_flag(arg);
    let xx_safe = matches!(
        arg,
        "-XX:+UseG1GC"
            | "-XX:+UseZGC"
            | "-XX:+UseShenandoahGC"
            | "-XX:+UnlockExperimentalVMOptions"
            | "-XX:+DisableExplicitGC"
            | "-XX:HeapDumpPath=MojangTricksIntelDriversForPerformance_javaw.exe_minecraft.exe.heapdump"
    );
    let mac_first = arg == "-XstartOnFirstThread";
    if memory || xx_safe || mac_first {
        return Ok(arg.to_string());
    }
    if allow_modded {
        if let Some(sanitized) = sanitize_allowed_d_prop(arg, game_dir)? {
            return Ok(sanitized);
        }
    }
    Err(format!("JVM argument not in allowlist: {arg}"))
}

fn sanitize_jvm_value(arg: &str, kind: JvmValueKind, game_dir: &Path) -> Result<String, String> {
    let arg = arg.trim();
    if arg.is_empty() || arg.len() > 8192 {
        return Err("Invalid JVM argument value length".into());
    }
    if arg.chars().any(|c| c.is_control() || c == '\0' || c == '\n' || c == '\r') {
        return Err("JVM argument value contains forbidden characters".into());
    }
    if is_dangerous_jvm_arg(arg) {
        return Err(format!("JVM argument value not allowed: {arg}"));
    }
    match kind {
        JvmValueKind::ModulePath => sanitize_module_path(arg, game_dir),
        JvmValueKind::AddModules => {
            if arg != "ALL-MODULE-PATH"
                && !arg
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == ',' || c == '_' || c == '-')
            {
                return Err("Invalid --add-modules value".into());
            }
            Ok(arg.to_string())
        }
        JvmValueKind::AddOpensExports => {
            if !arg.contains('/') || !arg.contains('=') {
                return Err("Invalid --add-opens/--add-exports value".into());
            }
            if !arg
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '/' | '=' | ',' | '-' | '_'))
            {
                return Err("Invalid --add-opens/--add-exports characters".into());
            }
            Ok(arg.to_string())
        }
    }
}

fn sanitize_module_path(raw: &str, game_dir: &Path) -> Result<String, String> {
    let libraries = canonicalize_dir(&config::meta_libraries_dir()?)?;
    let legacy_libraries = canonicalize_dir(&game_dir.join("libraries"))?;
    let sep = if cfg!(windows) { ';' } else { ':' };
    // On Windows paths also contain `:`, so split carefully: prefer `;` on Windows.
    let parts: Vec<&str> = if cfg!(windows) {
        raw.split(';').filter(|p| !p.is_empty()).collect()
    } else {
        raw.split(sep).filter(|p| !p.is_empty()).collect()
    };
    if parts.is_empty() {
        return Err("Empty module path".into());
    }
    let mut cleaned = Vec::with_capacity(parts.len());
    for part in parts {
        let path = Path::new(part);
        if !path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.eq_ignore_ascii_case("jar"))
            .unwrap_or(false)
        {
            return Err(format!("Module path entry is not a jar: {part}"));
        }
        let canon = path
            .canonicalize()
            .map_err(|e| format!("Module path resolve failed ({part}): {e}"))?;
        if !(path_under_root(&canon, &libraries) || path_under_root(&canon, &legacy_libraries)) {
            return Err(format!(
                "Module path jar outside meta/libraries or game/libraries: {}",
                canon.display()
            ));
        }
        cleaned.push(canon.display().to_string());
    }
    Ok(cleaned.join(&sep.to_string()))
}

fn canonicalize_dir(path: &Path) -> Result<PathBuf, String> {
    if !path.exists() {
        fs::create_dir_all(path).map_err(|e| format!("Create {}: {e}", path.display()))?;
    }
    path.canonicalize()
        .map_err(|e| format!("Canonicalize {}: {e}", path.display()))
}

fn path_under_root(path: &Path, root: &Path) -> bool {
    path.starts_with(root)
}

fn is_dangerous_jvm_arg(arg: &str) -> bool {
    let lower = arg.to_ascii_lowercase();
    lower.starts_with("-javaagent")
        || lower.starts_with("-agentpath")
        || lower.starts_with("-agentlib")
        || lower.starts_with("-xbootclasspath")
        || lower.contains("onerror")
        || lower.contains("onoutofmemoryerror")
        || lower.contains("java.security.manager")
        || lower.contains("${")
}

fn sanitize_allowed_d_prop(arg: &str, game_dir: &Path) -> Result<Option<String>, String> {
    let Some(rest) = arg.strip_prefix("-D") else {
        return Ok(None);
    };
    let Some((key, value)) = rest.split_once('=') else {
        return Ok(None);
    };
    let path_keys = matches!(
        key,
        "libraryDirectory"
            | "jna.tmpdir"
            | "org.lwjgl.system.SharedLibraryExtractPath"
            | "io.netty.native.workdir"
    );
    let key_ok = path_keys
        || matches!(
            key,
            "ignoreList"
                | "mergeModules"
                | "java.net.preferIPv6Addresses"
                | "minecraft.launcher.brand"
                | "minecraft.launcher.version"
        )
        || key.starts_with("forge.")
        || key.starts_with("fml.")
        || key.starts_with("neoforge.");
    if !key_ok {
        return Ok(None);
    }
    if value.len() > 4096 {
        return Err(format!("JVM -D{key} value too long"));
    }
    if value.chars().any(|c| c.is_control() || c == '\0') {
        return Err(format!("JVM -D{key} has forbidden characters"));
    }

    if path_keys {
        let libraries = canonicalize_dir(&config::meta_libraries_dir()?)?;
        let legacy_libraries = canonicalize_dir(&game_dir.join("libraries"))?;
        let natives_meta = canonicalize_dir(&config::meta_dir()?.join("natives"))?;
        let natives_legacy = canonicalize_dir(&game_dir.join("natives"))?;
        let path = Path::new(value);
        let canon = path
            .canonicalize()
            .map_err(|e| format!("JVM -D{key} resolve failed: {e}"))?;
        let ok = match key {
            "libraryDirectory" => {
                path_under_root(&canon, &libraries)
                    || canon == libraries
                    || path_under_root(&canon, &legacy_libraries)
                    || canon == legacy_libraries
            }
            _ => {
                path_under_root(&canon, &natives_meta)
                    || path_under_root(&canon, &natives_legacy)
                    || canon == natives_meta
                    || canon == natives_legacy
            }
        };
        if !ok {
            return Err(format!(
                "JVM -D{key} must stay under meta/libraries|natives or game/libraries|natives"
            ));
        }
        return Ok(Some(format!("-D{key}={}", canon.display())));
    }

    // Non-path forge/Mojang props: tight token charset (no paths).
    if !value
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_' | ',' | '=' | '/' | ':'))
    {
        return Err(format!("JVM -D{key} value has forbidden characters"));
    }
    Ok(Some(arg.to_string()))
}

fn sanitize_modded_game_args(args: &[String]) -> Result<Vec<String>, String> {
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
    if args.is_empty() {
        return Ok(Vec::new());
    }
    let mut out = Vec::with_capacity(args.len());
    let mut i = 0;
    while i < args.len() {
        let flag = args[i].trim();
        if !FLAGS.contains(&flag) {
            return Err(format!("Game argument not in allowlist: {flag}"));
        }
        let Some(value) = args.get(i + 1).map(|s| s.trim()) else {
            return Err(format!("Missing value for {flag}"));
        };
        if value.is_empty() || value.len() > 128 {
            return Err(format!("Invalid value for {flag}"));
        }
        if !value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '_' || c == '-')
        {
            return Err(format!("Invalid characters in {flag} value"));
        }
        out.push(flag.to_string());
        out.push(value.to_string());
        i += 2;
    }
    Ok(out)
}

fn regex_is_memory_flag(arg: &str) -> bool {
    // -Xmx2G / -Xms512M / -Xss1M
    let bytes = arg.as_bytes();
    if bytes.len() < 5 || !arg.starts_with("-X") {
        return false;
    }
    let kind = &arg[2..4];
    if kind != "mx" && kind != "ms" && kind != "ss" {
        return false;
    }
    let rest = &arg[4..];
    let (num, unit) = rest.split_at(rest.len().saturating_sub(1));
    if num.is_empty() || !num.chars().all(|c| c.is_ascii_digit()) {
        return false;
    }
    matches!(unit, "k" | "K" | "m" | "M" | "g" | "G")
}

fn resolve_natives_dir(game_dir: &Path, minecraft: &str) -> Result<PathBuf, String> {
    let shared = config::meta_natives_dir(minecraft)?;
    let legacy = game_dir.join("natives");
    // Prefer shared meta when it has content; else legacy game/natives for older installs.
    let shared_has = shared
        .read_dir()
        .map(|mut it| it.next().is_some())
        .unwrap_or(false);
    if shared_has {
        Ok(shared)
    } else if legacy.is_dir() {
        Ok(legacy)
    } else {
        Ok(shared)
    }
}

fn resolve_assets_dir(game_dir: &Path) -> Result<PathBuf, String> {
    let shared = config::meta_assets_dir()?;
    let legacy = game_dir.join("assets");
    let shared_objects = shared.join("objects");
    let shared_has = shared_objects
        .read_dir()
        .map(|mut it| it.next().is_some())
        .unwrap_or(false);
    if shared_has {
        Ok(shared)
    } else if legacy.is_dir() {
        Ok(legacy)
    } else {
        Ok(shared)
    }
}

fn collect_manifest_jars(
    game_dir: &Path,
    manifest: &PackManifest,
) -> Result<Vec<PathBuf>, String> {
    let mut jars = Vec::new();
    for file in &manifest.files {
        if !file
            .path
            .to_ascii_lowercase()
            .ends_with(".jar")
        {
            continue;
        }
        let path = resolve_manifest_file(game_dir, &file.path)?;
        if !path.is_file() {
            return Err(format!("Missing jar from manifest: {}", file.path));
        }
        // Theseus trusts install-time checks; re-hashing every jar on Play freezes launch.
        // Prefer size match when known; accept sha1:/reuse markers without full-file SHA256.
        if !jar_integrity_ok(&path, &file.sha256, file.size)? {
            return Err(format!("Jar integrity check failed: {}", file.path));
        }
        jars.push(path);
    }
    jars.sort();
    Ok(jars)
}

fn resolve_manifest_file(game_dir: &Path, rel: &str) -> Result<PathBuf, String> {
    let preferred = config::resolve_game_or_meta(rel, game_dir)?;
    if preferred.is_file() {
        return Ok(preferred);
    }
    // Legacy installs put client.jar under libraries/com/mojang/minecraft/.
    let legacy_client = "libraries/com/mojang/minecraft/client.jar";
    if rel.replace('\\', "/").starts_with("versions/") && rel.ends_with(".jar") {
        let old = config::resolve_game_or_meta(legacy_client, game_dir)?;
        if old.is_file() {
            return Ok(old);
        }
        let old_game = game_dir.join(safe_rel_path(legacy_client)?);
        if old_game.is_file() {
            return Ok(old_game);
        }
    }
    // Legacy installs kept libraries/assets/versions under game/.
    let legacy = game_dir.join(safe_rel_path(rel)?);
    if legacy.is_file() {
        return Ok(legacy);
    }
    Ok(preferred)
}

fn jar_integrity_ok(path: &Path, expected_hash: &str, expected_size: Option<u64>) -> Result<bool, String> {
    let meta = fs::metadata(path).map_err(|e| format!("Stat jar: {e}"))?;
    if let Some(sz) = expected_size {
        if meta.len() != sz {
            return Ok(false);
        }
    }
    let hash = expected_hash.trim();
    if hash.is_empty() || hash == "reuse" || hash.starts_with("sha1:") {
        return Ok(true);
    }
    // Older manifests stored full SHA256 — verify only then.
    file_sha256_matches(path, hash)
}

fn safe_rel_path(raw: &str) -> Result<PathBuf, String> {
    use std::path::{Component, Path};
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

fn file_sha256_matches(path: &Path, expected: &str) -> Result<bool, String> {
    use sha2::{Digest, Sha256};
    let mut file = fs::File::open(path).map_err(|e| format!("Read jar: {e}"))?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 64 * 1024];
    loop {
        let n = std::io::Read::read(&mut file, &mut buf).map_err(|e| format!("Read jar: {e}"))?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    let actual: String = hasher.finalize().iter().map(|b| format!("{b:02x}")).collect();
    Ok(actual.eq_ignore_ascii_case(expected))
}

fn sanitize_token(value: &str, label: &str) -> Result<String, String> {
    let value = value.trim();
    if value.is_empty() || value.len() > 64 {
        return Err(format!("Invalid {label}"));
    }
    if !value
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '_' || c == '-')
    {
        return Err(format!("Invalid {label} characters"));
    }
    Ok(value.to_string())
}

fn is_safe_nick(nick: &str) -> bool {
    !nick.is_empty()
        && nick.len() <= 16
        && nick
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_')
}

fn join_classpath(jars: &[PathBuf]) -> String {
    let sep = if cfg!(windows) { ';' } else { ':' };
    jars.iter()
        .map(|p| p.display().to_string())
        .collect::<Vec<_>>()
        .join(&sep.to_string())
}

fn validate_pack_id(pack_id: &str) -> Result<(), String> {
    crate::instances::validate_instance_id(pack_id).map_err(|_| "Invalid pack id".into())
}

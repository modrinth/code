mod api;
mod catalog;
mod config;
mod install;
mod instances;
mod java;
mod launch;
mod log;
mod models;
mod modrinth;
mod mrpack;
mod ms_auth;
mod owyx;
mod profiles;
mod skins;
mod sync;
mod versions;

use config::LauncherConfig;
use install::{InstanceRecord, PrefetchStatus};
use instances::{ContentInfo, DirEntryInfo, DiskInstance, InstanceLogInfo, LogFileInfo, WorldInfo};
use java::JavaInstallation;
use launch::{LaunchOverrides, LaunchResult};
use models::{PackLocalStatus, PackSummary};
use modrinth::ModrinthSearchResult;
use profiles::{Profile, ProfilesState};
use tauri::AppHandle;
use versions::{LoaderKind, MinecraftVersion};

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Owyx backend online · hello, {name}")
}

#[tauri::command]
fn get_config() -> Result<LauncherConfig, String> {
    config::load()
}

#[tauri::command]
fn get_data_dir() -> Result<String, String> {
    config::app_data_dir().map(|p| p.display().to_string())
}

#[tauri::command]
fn set_nick(nick: String) -> Result<LauncherConfig, String> {
    config::set_nick(nick)
}

#[tauri::command]
fn set_locale(locale: String) -> Result<LauncherConfig, String> {
    config::set_locale(locale)
}

#[tauri::command]
fn get_profiles() -> Result<ProfilesState, String> {
    profiles::load_state()
}

#[tauri::command]
fn save_profiles(
    profiles: Vec<Profile>,
    active_profile_id: Option<String>,
) -> Result<ProfilesState, String> {
    profiles::save_state(profiles, active_profile_id)
}

#[tauri::command]
fn check_updates() -> Result<LauncherConfig, String> {
    // Stub: record check time; real EXE updater comes later.
    config::set_updater_checked(None)
}

#[tauri::command]
async fn list_packs() -> Result<Vec<PackSummary>, String> {
    let cfg = config::load()?;
    sync::list_packs(cfg.update_base_url).await
}

#[tauri::command]
fn get_pack_status(pack_id: String) -> Result<PackLocalStatus, String> {
    sync::pack_status(&pack_id)
}

#[tauri::command]
async fn sync_pack(app: AppHandle, pack_id: String) -> Result<PackLocalStatus, String> {
    let cfg = config::load()?;
    sync::sync_pack(app, pack_id, cfg.update_base_url).await
}

#[tauri::command]
fn list_java() -> Result<Vec<JavaInstallation>, String> {
    let cfg = config::load()?;
    Ok(cfg
        .java_installations
        .into_iter()
        .filter(|j| std::path::Path::new(&j.path).is_file())
        .collect())
}

#[tauri::command]
async fn detect_java() -> Result<Vec<JavaInstallation>, String> {
    let found = tauri::async_runtime::spawn_blocking(java::find_all)
        .await
        .map_err(|e| format!("Java detect task failed: {e}"))?;
    config::set_java_installations(found.clone())?;
    Ok(found)
}

#[tauri::command]
async fn find_java_for_major(major: u32) -> Result<Vec<JavaInstallation>, String> {
    tauri::async_runtime::spawn_blocking(move || java::find_for_major(major))
        .await
        .map_err(|e| format!("Java filter task failed: {e}"))
}

#[tauri::command]
fn set_java_version(major: u32, path: Option<String>) -> Result<LauncherConfig, String> {
    config::set_java_version_path(major, path)
}

#[tauri::command]
async fn install_java(major: u32) -> Result<JavaInstallation, String> {
    java::auto_install(major).await
}

#[tauri::command]
fn meta_prefetch_status() -> PrefetchStatus {
    install::meta_prefetch_status()
}

#[tauri::command]
async fn prefetch_shared_meta(app: AppHandle) -> Result<PrefetchStatus, String> {
    install::prefetch_shared_meta(app).await
}

#[tauri::command]
fn update_instance_installation(
    id: String,
    loader: String,
    minecraft: String,
    loader_version: String,
) -> Result<InstanceRecord, String> {
    install::update_instance_installation(&id, &loader, &minecraft, &loader_version)
}

#[tauri::command]
async fn repair_instance(app: AppHandle, id: String) -> Result<PackLocalStatus, String> {
    install::repair_instance(app, id).await
}

#[tauri::command]
fn system_memory_mb() -> u32 {
    // Approximate physical RAM for the memory slider max. Fallback 16G.
    #[cfg(windows)]
    {
        // SAFETY: GlobalMemoryStatusEx is a well-defined Win32 call with a sized buffer.
        unsafe {
            #[repr(C)]
            struct MemoryStatusEx {
                dw_length: u32,
                dw_memory_load: u32,
                ull_total_phys: u64,
                ull_avail_phys: u64,
                ull_total_page_file: u64,
                ull_avail_page_file: u64,
                ull_total_virtual: u64,
                ull_avail_virtual: u64,
                ull_avail_extended_virtual: u64,
            }
            #[link(name = "kernel32")]
            extern "system" {
                fn GlobalMemoryStatusEx(lp_buffer: *mut MemoryStatusEx) -> i32;
            }
            let mut status = std::mem::zeroed::<MemoryStatusEx>();
            status.dw_length = std::mem::size_of::<MemoryStatusEx>() as u32;
            if GlobalMemoryStatusEx(&mut status) != 0 {
                let total = status.ull_total_phys / (1024 * 1024);
                return total.clamp(2048, 262144) as u32;
            }
        }
    }
    16384
}

#[tauri::command]
fn launch_pack(
    app: AppHandle,
    pack_id: String,
    overrides: Option<LaunchOverrides>,
) -> Result<LaunchResult, String> {
    let nick = profiles::active_nick()?;
    launch::launch_pack(app, &pack_id, &nick, overrides.unwrap_or_default())
}

#[tauri::command]
fn stop_pack(pack_id: String) -> Result<(), String> {
    launch::stop_pack(&pack_id)
}

#[tauri::command]
fn is_pack_running(pack_id: String) -> bool {
    launch::is_pack_running(&pack_id)
}

#[tauri::command]
fn list_loaders() -> Vec<LoaderKind> {
    versions::list_loaders()
}

#[tauri::command]
async fn list_minecraft_versions(
    loader: String,
    include_snapshots: bool,
) -> Result<Vec<MinecraftVersion>, String> {
    versions::list_minecraft_versions(&loader, include_snapshots).await
}

#[tauri::command]
async fn list_loader_versions(loader: String, minecraft: String) -> Result<Vec<String>, String> {
    versions::list_loader_versions(&loader, &minecraft).await
}

#[tauri::command]
async fn install_instance(
    app: AppHandle,
    id: String,
    name: String,
    loader: String,
    minecraft: String,
    loader_version: String,
) -> Result<PackLocalStatus, String> {
    install::install_instance(app, id, name, loader, minecraft, loader_version).await
}

#[tauri::command]
fn prepare_instance(
    name: String,
    loader: String,
    minecraft: String,
    loader_version: String,
    icon_path: Option<String>,
) -> Result<DiskInstance, String> {
    instances::prepare_instance(
        &name,
        &loader,
        &minecraft,
        &loader_version,
        icon_path.as_deref(),
    )
}

#[tauri::command]
fn set_instance_icon(id: String, path: Option<String>) -> Result<(), String> {
    instances::set_instance_icon(&id, path.as_deref())
}

#[tauri::command]
fn get_instance_icon(id: String) -> Result<Option<String>, String> {
    instances::instance_icon_data_url(&id)
}

#[tauri::command]
fn read_image_data_url(path: String) -> Result<String, String> {
    instances::image_as_data_url(std::path::Path::new(&path))
}

#[tauri::command]
fn list_instances() -> Result<Vec<DiskInstance>, String> {
    instances::list_instances()
}

#[tauri::command]
fn delete_instance(id: String) -> Result<(), String> {
    instances::delete_instance(&id)
}

#[tauri::command]
fn open_instance_dir(app: AppHandle, id: String, which: String) -> Result<(), String> {
    instances::open_instance_dir(app, &id, &which)
}

#[tauri::command]
async fn modrinth_search(
    query: String,
    limit: Option<u32>,
    offset: Option<u32>,
    loader: Option<String>,
    game_version: Option<String>,
    category: Option<String>,
    project_type: Option<String>,
    index: Option<String>,
) -> Result<ModrinthSearchResult, String> {
    modrinth::search_projects(
        query,
        limit,
        offset,
        loader,
        game_version,
        category,
        project_type,
        index,
    )
    .await
}

#[tauri::command]
async fn modrinth_project_versions(
    project_id: String,
    loader: Option<String>,
    game_version: Option<String>,
) -> Result<Vec<modrinth::ModrinthVersion>, String> {
    modrinth::list_project_versions(project_id, loader, game_version).await
}

#[tauri::command]
async fn modrinth_get_project(project_id: String) -> Result<modrinth::ModrinthProjectDetail, String> {
    modrinth::get_project(project_id).await
}

#[tauri::command]
async fn modrinth_install(
    instance_id: String,
    project_id: String,
    loader: String,
    game_version: String,
    include_optional_dependencies: Option<bool>,
) -> Result<ContentInfo, String> {
    modrinth::install_to_instance(
        instance_id,
        project_id,
        loader,
        game_version,
        include_optional_dependencies.unwrap_or(false),
    )
    .await
}

#[tauri::command]
async fn list_instance_dir(
    id: String,
    relative: Option<String>,
) -> Result<Vec<DirEntryInfo>, String> {
    let rel = relative.unwrap_or_default();
    tauri::async_runtime::spawn_blocking(move || instances::list_instance_dir(&id, &rel))
        .await
        .map_err(|e| format!("list_instance_dir join: {e}"))?
}

#[tauri::command]
async fn list_instance_logs(pack_id: String) -> Result<Vec<InstanceLogInfo>, String> {
    tauri::async_runtime::spawn_blocking(move || instances::list_instance_logs(&pack_id))
        .await
        .map_err(|e| format!("list_instance_logs join: {e}"))?
}

#[tauri::command]
async fn read_instance_log(
    pack_id: String,
    source: String,
    filename: String,
    max_bytes: Option<u64>,
) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        instances::read_instance_log(&pack_id, &source, &filename, max_bytes)
    })
    .await
    .map_err(|e| format!("read_instance_log join: {e}"))?
}

#[tauri::command]
async fn list_launch_logs(pack_id: Option<String>) -> Result<Vec<LogFileInfo>, String> {
    tauri::async_runtime::spawn_blocking(move || instances::list_launch_logs(pack_id))
        .await
        .map_err(|e| format!("list_launch_logs join: {e}"))?
}

#[tauri::command]
async fn read_launch_log(name: String, max_bytes: Option<u64>) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || instances::read_launch_log(&name, max_bytes))
        .await
        .map_err(|e| format!("read_launch_log join: {e}"))?
}

#[tauri::command]
fn list_worlds(id: String) -> Result<Vec<WorldInfo>, String> {
    instances::list_worlds(&id)
}

#[tauri::command]
fn delete_world(id: String, world_id: String) -> Result<(), String> {
    instances::delete_world(&id, &world_id)
}

#[tauri::command]
fn list_content(id: String) -> Result<Vec<ContentInfo>, String> {
    instances::list_content(&id)
}

#[tauri::command]
fn set_content_enabled(
    id: String,
    kind: String,
    file_name: String,
    enabled: bool,
) -> Result<(), String> {
    instances::set_content_enabled(&id, &kind, &file_name, enabled)
}

#[tauri::command]
fn delete_content(id: String, kind: String, file_name: String) -> Result<(), String> {
    instances::delete_content(&id, &kind, &file_name)
}

#[tauri::command]
fn add_content_from_path(
    id: String,
    path: String,
    kind: Option<String>,
) -> Result<ContentInfo, String> {
    instances::add_content_from_path(&id, &path, kind.as_deref())
}

#[tauri::command]
fn duplicate_instance(
    id: String,
    new_id: String,
    new_name: String,
) -> Result<DiskInstance, String> {
    instances::duplicate_instance(&id, &new_id, &new_name)
}

#[tauri::command]
async fn ms_auth_start() -> Result<ms_auth::MsAuthStatus, String> {
    ms_auth::start().await
}

#[tauri::command]
async fn ms_auth_poll() -> Result<ms_auth::MsAuthStatus, String> {
    ms_auth::poll().await
}

#[tauri::command]
fn ms_auth_cancel() -> Result<(), String> {
    ms_auth::cancel()
}

#[tauri::command]
fn list_skins() -> Result<Vec<skins::SkinEntry>, String> {
    skins::list_skins()
}

#[tauri::command]
fn import_skin(path: String, name: Option<String>) -> Result<skins::SkinEntry, String> {
    skins::import_skin(path, name)
}

#[tauri::command]
fn apply_local_skin(
    skin_id: String,
    instance_id: Option<String>,
) -> Result<skins::SkinEntry, String> {
    skins::apply_skin(skin_id, instance_id)
}

#[tauri::command]
fn remove_skin(skin_id: String) -> Result<(), String> {
    skins::remove_skin(skin_id)
}

#[tauri::command]
async fn import_mrpack(path: String) -> Result<mrpack::MrpackImportResult, String> {
    mrpack::import_mrpack_file(path).await
}

#[tauri::command]
async fn import_mrpack_modrinth(
    project_id: String,
    version_id: Option<String>,
) -> Result<mrpack::MrpackImportResult, String> {
    mrpack::import_mrpack_from_modrinth(project_id, version_id).await
}

#[tauri::command]
fn export_mrpack(instance_id: String, dest_path: String) -> Result<String, String> {
    mrpack::export_mrpack(instance_id, dest_path)
}

#[tauri::command]
async fn owyx_login(
    base_url: Option<String>,
    email: String,
    password: String,
) -> Result<owyx::OwyxSession, String> {
    owyx::login(base_url.as_deref(), &email, &password).await
}

#[tauri::command]
async fn owyx_me(nickname: String) -> Result<owyx::OwyxSession, String> {
    owyx::me(&nickname).await
}

#[tauri::command]
fn owyx_logout(nickname: String) -> Result<(), String> {
    owyx::logout(&nickname)
}

#[tauri::command]
async fn catalog_list() -> Result<catalog::CatalogLoadResult, String> {
    catalog::list_catalog(None).await
}

#[tauri::command]
async fn install_catalog_server(
    app: AppHandle,
    server_id: String,
    name: String,
    address: String,
    port: u16,
    pack_id: Option<String>,
    minecraft: String,
    loader: String,
    download_url: Option<String>,
    sha256: Option<String>,
    manifest_url: Option<String>,
    source_type: String,
) -> Result<catalog::CatalogInstallResult, String> {
    catalog::install_catalog_server(
        app,
        server_id,
        name,
        address,
        port,
        pack_id,
        minecraft,
        loader,
        download_url,
        sha256,
        manifest_url,
        source_type,
    )
    .await
}

#[tauri::command]
async fn owyx_admin_request(
    nickname: String,
    method: String,
    path: String,
    body: Option<serde_json::Value>,
) -> Result<serde_json::Value, String> {
    owyx::admin_request(&nickname, &method, &path, body).await
}

#[tauri::command]
async fn apply_owyx_skin(
    nickname: String,
    instance_id: Option<String>,
) -> Result<owyx::SkinApplyResult, String> {
    owyx::apply_skin(&nickname, instance_id).await
}

#[tauri::command]
async fn modrinth_install_version(
    instance_id: String,
    version_id: String,
    with_dependencies: Option<bool>,
    include_optional_dependencies: Option<bool>,
) -> Result<instances::ContentInfo, String> {
    modrinth::install_version_to_instance(
        instance_id,
        version_id,
        with_dependencies.unwrap_or(true),
        include_optional_dependencies.unwrap_or(false),
    )
    .await
}

fn fit_main_window(app: &tauri::App) {
    use tauri::{LogicalSize, Manager, Size};

    let Some(win) = app.get_webview_window("main") else {
        return;
    };
    let (w, h) = match win.current_monitor().ok().flatten() {
        Some(monitor) => {
            let size = monitor.size();
            let scale = monitor.scale_factor();
            let avail_w = size.width as f64 / scale;
            let avail_h = size.height as f64 / scale;
            let target_w = 1440_f64.min(avail_w * 0.92).max(1024.0);
            let target_h = 900_f64.min(avail_h * 0.88).max(640.0);
            (target_w, target_h)
        }
        None => (1440.0, 900.0),
    };
    let _ = win.set_size(Size::Logical(LogicalSize::new(w, h)));
    let _ = win.center();
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            fit_main_window(app);
            if let Err(e) = log::init_launcher_logging() {
                eprintln!("Failed to init launcher log: {e}");
            }
            // Prefetch is started once from the splash UI (`prefetch_shared_meta`).
            // Do not also spawn it here — dual callers fight INSTALL_LOCK and break Play.
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            get_config,
            get_data_dir,
            set_nick,
            set_locale,
            get_profiles,
            save_profiles,
            check_updates,
            list_packs,
            get_pack_status,
            sync_pack,
            list_java,
            detect_java,
            find_java_for_major,
            set_java_version,
            install_java,
            meta_prefetch_status,
            prefetch_shared_meta,
            update_instance_installation,
            repair_instance,
            system_memory_mb,
            launch_pack,
            stop_pack,
            is_pack_running,
            list_loaders,
            list_minecraft_versions,
            list_loader_versions,
            install_instance,
            prepare_instance,
            set_instance_icon,
            get_instance_icon,
            read_image_data_url,
            list_instances,
            delete_instance,
            open_instance_dir,
            modrinth_search,
            modrinth_project_versions,
            modrinth_get_project,
            modrinth_install,
            list_instance_dir,
            list_instance_logs,
            read_instance_log,
            list_launch_logs,
            read_launch_log,
            list_worlds,
            delete_world,
            list_content,
            set_content_enabled,
            delete_content,
            add_content_from_path,
            duplicate_instance,
            owyx_login,
            owyx_me,
            owyx_logout,
            catalog_list,
            install_catalog_server,
            owyx_admin_request,
            apply_owyx_skin,
            modrinth_install_version,
            ms_auth_start,
            ms_auth_poll,
            ms_auth_cancel,
            list_skins,
            import_skin,
            apply_local_skin,
            remove_skin,
            import_mrpack,
            import_mrpack_modrinth,
            export_mrpack
        ])
        .run(tauri::generate_context!())
        .expect("error while running Owyx");
}

//! Modrinth Labrinth API client scaffold.
//! Real marketplace polish comes later — this wires HTTPS search so Content can call it.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::time::Duration;

const MODRINTH_API: &str = "https://api.modrinth.com/v2";
const USER_AGENT: &str = concat!(
    "owyx/",
    env!("CARGO_PKG_VERSION"),
    " (github.com/ebluffy/Owyx)"
);
/// Hard cap for a single downloaded mod/pack file.
const MAX_FILE_BYTES: u64 = 512 * 1024 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModrinthProjectHit {
    pub project_id: String,
    pub slug: String,
    pub title: String,
    pub description: String,
    pub project_type: String,
    pub downloads: u64,
    #[serde(default)]
    pub icon_url: Option<String>,
    #[serde(default)]
    pub categories: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModrinthSearchResult {
    pub hits: Vec<ModrinthProjectHit>,
    pub offset: u32,
    pub limit: u32,
    pub total_hits: u32,
    /// True when this is a local stub (no network / scaffold mode).
    #[serde(default)]
    pub stub: bool,
    #[serde(default)]
    pub message: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ApiSearchResponse {
    hits: Vec<ApiHit>,
    offset: u32,
    limit: u32,
    total_hits: u32,
}

#[derive(Debug, Deserialize)]
struct ApiHit {
    project_id: String,
    slug: String,
    title: String,
    description: String,
    project_type: String,
    downloads: u64,
    #[serde(default)]
    icon_url: Option<String>,
    #[serde(default)]
    categories: Vec<String>,
}

fn http_client() -> Result<reqwest::Client, String> {
    let redirect_policy = reqwest::redirect::Policy::custom(|attempt| {
        if attempt.previous().len() >= 8 {
            return attempt.error("Too many Modrinth redirects");
        }
        if assert_modrinth_url(attempt.url().as_str()).is_ok() {
            attempt.follow()
        } else {
            attempt.error("Modrinth redirect left the allowed hosts")
        }
    });
    reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .timeout(Duration::from_secs(30))
        .connect_timeout(Duration::from_secs(10))
        .redirect(redirect_policy)
        .build()
        .map_err(|e| format!("HTTP client: {e}"))
}

fn assert_modrinth_url(raw: &str) -> Result<(), String> {
    let parsed = reqwest::Url::parse(raw).map_err(|e| format!("Bad URL: {e}"))?;
    if parsed.scheme() != "https" {
        return Err("Only https allowed for Modrinth API".into());
    }
    if parsed.username() != "" || parsed.password().is_some() {
        return Err("Modrinth URL must not include credentials".into());
    }
    let host = parsed
        .host_str()
        .ok_or_else(|| "Missing host".to_string())?
        .to_ascii_lowercase();
    if host != "api.modrinth.com" && !host.ends_with(".modrinth.com") {
        return Err(format!("Host not allowed: {host}"));
    }
    Ok(())
}

/// Search Modrinth projects (mods / resource packs / etc.).
/// Empty query returns an empty scaffold result without hitting the network.
pub async fn search_projects(
    query: String,
    limit: Option<u32>,
    offset: Option<u32>,
    loader: Option<String>,
    game_version: Option<String>,
    category: Option<String>,
    project_type: Option<String>,
    index: Option<String>,
) -> Result<ModrinthSearchResult, String> {
    let q = query.trim().to_string();
    let limit = limit.unwrap_or(20).clamp(1, 50);
    let offset = offset.unwrap_or(0);
    let ptype = project_type
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("mod");
    let allowed_types = ["mod", "resourcepack", "datapack", "shader", "modpack"];
    if !allowed_types.contains(&ptype) {
        return Err(format!("Unsupported project type: {ptype}"));
    }

    let mut url = format!(
        "{MODRINTH_API}/search?query={}&limit={limit}&offset={offset}",
        urlencoding_lite(&q)
    );
    if let Some(idx) = index
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        // Modrinth indices: relevance, downloads, follows, newest, updated
        let allowed = ["relevance", "downloads", "follows", "newest", "updated"];
        if allowed.contains(&idx) {
            url.push_str(&format!("&index={}", urlencoding_lite(idx)));
        }
    }
    let mut facets: Vec<String> = vec![format!(r#"["project_type:{ptype}"]"#)];
    if let Some(l) = loader
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty() && *s != "vanilla")
    {
        // Loader facet applies to mods; shaders use different categories.
        if ptype == "mod" || ptype == "modpack" {
            facets.push(format!(r#"["categories:{}"]"#, urlencoding_lite(l)));
        }
    }
    if let Some(v) = game_version
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        facets.push(format!(r#"["versions:{}"]"#, urlencoding_lite(v)));
    }
    if let Some(c) = category
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        facets.push(format!(r#"["categories:{}"]"#, urlencoding_lite(c)));
    }
    url.push_str(&format!("&facets=[{}]", facets.join(",")));
    assert_modrinth_url(&url)?;

    let http = http_client()?;
    let res = http
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Modrinth search: {e}"))?
        .error_for_status()
        .map_err(|e| format!("Modrinth HTTP: {e}"))?;
    assert_modrinth_url(res.url().as_str())?;

    let body: ApiSearchResponse = res
        .json()
        .await
        .map_err(|e| format!("Modrinth JSON: {e}"))?;

    Ok(ModrinthSearchResult {
        hits: body
            .hits
            .into_iter()
            .map(|h| ModrinthProjectHit {
                project_id: h.project_id,
                slug: h.slug,
                title: h.title,
                description: h.description,
                project_type: h.project_type,
                downloads: h.downloads,
                icon_url: h.icon_url,
                categories: h.categories,
            })
            .collect(),
        offset: body.offset,
        limit: body.limit,
        total_hits: body.total_hits,
        stub: false,
        message: None,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModrinthGalleryImage {
    pub url: String,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub featured: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModrinthProjectDetail {
    pub id: String,
    pub slug: String,
    pub title: String,
    pub description: String,
    pub body: String,
    pub project_type: String,
    pub downloads: u64,
    #[serde(default)]
    pub icon_url: Option<String>,
    #[serde(default)]
    pub categories: Vec<String>,
    #[serde(default)]
    pub loaders: Vec<String>,
    #[serde(default)]
    pub game_versions: Vec<String>,
    #[serde(default)]
    pub gallery: Vec<ModrinthGalleryImage>,
    #[serde(default)]
    pub project_url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ApiGalleryImage {
    url: String,
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    featured: bool,
}

#[derive(Debug, Deserialize)]
struct ApiProject {
    id: String,
    slug: String,
    title: String,
    description: String,
    #[serde(default)]
    body: String,
    project_type: String,
    downloads: u64,
    #[serde(default)]
    icon_url: Option<String>,
    #[serde(default)]
    categories: Vec<String>,
    #[serde(default)]
    loaders: Vec<String>,
    #[serde(default)]
    game_versions: Vec<String>,
    #[serde(default)]
    gallery: Vec<ApiGalleryImage>,
}

/// Fetch a single Modrinth project by id or slug (detail panel).
pub async fn get_project(project_id: String) -> Result<ModrinthProjectDetail, String> {
    let id = sanitize_project_id(&project_id)?;
    let url = format!("{MODRINTH_API}/project/{id}");
    assert_modrinth_url(&url)?;

    let http = http_client()?;
    let res = http
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Modrinth project: {e}"))?
        .error_for_status()
        .map_err(|e| format!("Modrinth HTTP: {e}"))?;
    assert_modrinth_url(res.url().as_str())?;
    let p: ApiProject = res
        .json()
        .await
        .map_err(|e| format!("Modrinth project JSON: {e}"))?;
    let project_url = Some(format!("https://modrinth.com/{}/{}", p.project_type, p.slug));
    Ok(ModrinthProjectDetail {
        id: p.id,
        slug: p.slug,
        title: p.title,
        description: p.description,
        body: p.body,
        project_type: p.project_type,
        downloads: p.downloads,
        icon_url: p.icon_url,
        categories: p.categories,
        loaders: p.loaders,
        game_versions: p.game_versions,
        gallery: p
            .gallery
            .into_iter()
            .map(|g| ModrinthGalleryImage {
                url: g.url,
                title: g.title,
                description: g.description,
                featured: g.featured,
            })
            .collect(),
        project_url,
    })
}

// ---------------------------------------------------------------------------
// Versions + install (MR-1): pick a loader/MC-compatible file and drop it into
// the instance content folder. Ported from Theseus' project-version selection.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModrinthVersionFile {
    pub url: String,
    pub filename: String,
    pub primary: bool,
    #[serde(default)]
    pub size: Option<u64>,
    #[serde(default)]
    pub sha512: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModrinthDependency {
    #[serde(default)]
    pub project_id: Option<String>,
    #[serde(default)]
    pub version_id: Option<String>,
    #[serde(default)]
    pub dependency_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModrinthVersion {
    pub id: String,
    pub name: String,
    pub version_number: String,
    pub version_type: String,
    pub loaders: Vec<String>,
    pub game_versions: Vec<String>,
    pub files: Vec<ModrinthVersionFile>,
    #[serde(default)]
    pub dependencies: Vec<ModrinthDependency>,
    #[serde(default)]
    pub project_id: Option<String>,
    #[serde(default)]
    pub changelog: Option<String>,
    #[serde(default)]
    pub date_published: Option<String>,
    #[serde(default)]
    pub downloads: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct ApiVersion {
    id: String,
    name: String,
    version_number: String,
    #[serde(default)]
    version_type: String,
    #[serde(default)]
    loaders: Vec<String>,
    #[serde(default)]
    game_versions: Vec<String>,
    #[serde(default)]
    files: Vec<ApiVersionFile>,
    #[serde(default)]
    dependencies: Vec<ApiDependency>,
    #[serde(default)]
    project_id: Option<String>,
    #[serde(default)]
    changelog: Option<String>,
    #[serde(default)]
    date_published: Option<String>,
    #[serde(default)]
    downloads: Option<u64>,
}

fn map_api_version(v: ApiVersion) -> ModrinthVersion {
    ModrinthVersion {
        id: v.id,
        name: v.name,
        version_number: v.version_number,
        version_type: v.version_type,
        loaders: v.loaders,
        game_versions: v.game_versions,
        files: v
            .files
            .into_iter()
            .map(|f| ModrinthVersionFile {
                url: f.url,
                filename: f.filename,
                primary: f.primary,
                size: f.size,
                sha512: f.hashes.and_then(|h| h.sha512),
            })
            .collect(),
        dependencies: v
            .dependencies
            .into_iter()
            .map(|d| ModrinthDependency {
                project_id: d.project_id,
                version_id: d.version_id,
                dependency_type: d.dependency_type,
            })
            .collect(),
        project_id: v.project_id,
        changelog: v.changelog,
        date_published: v.date_published,
        downloads: v.downloads,
    }
}

#[derive(Debug, Deserialize)]
struct ApiDependency {
    #[serde(default)]
    project_id: Option<String>,
    #[serde(default)]
    version_id: Option<String>,
    #[serde(default)]
    dependency_type: String,
}

#[derive(Debug, Deserialize)]
struct ApiVersionFile {
    url: String,
    filename: String,
    #[serde(default)]
    primary: bool,
    #[serde(default)]
    size: Option<u64>,
    #[serde(default)]
    hashes: Option<ApiHashes>,
}

#[derive(Debug, Deserialize)]
struct ApiHashes {
    #[serde(default)]
    sha512: Option<String>,
}

/// List a project's versions, optionally filtered by loader + Minecraft version (server-side).
pub async fn list_project_versions(
    project_id: String,
    loader: Option<String>,
    game_version: Option<String>,
) -> Result<Vec<ModrinthVersion>, String> {
    let id = sanitize_project_id(&project_id)?;
    let mut url = format!("{MODRINTH_API}/project/{id}/version");
    let mut params: Vec<String> = Vec::new();
    if let Some(l) = loader.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        // Modrinth expects a JSON array: loaders=["fabric"]
        params.push(format!("loaders=%5B%22{}%22%5D", urlencoding_lite(l)));
    }
    if let Some(v) = game_version
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        params.push(format!("game_versions=%5B%22{}%22%5D", urlencoding_lite(v)));
    }
    if !params.is_empty() {
        url.push('?');
        url.push_str(&params.join("&"));
    }
    assert_modrinth_url(&url)?;

    let http = http_client()?;
    let res = http
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Modrinth versions: {e}"))?
        .error_for_status()
        .map_err(|e| format!("Modrinth HTTP: {e}"))?;
    assert_modrinth_url(res.url().as_str())?;
    let raw: Vec<ApiVersion> = res
        .json()
        .await
        .map_err(|e| format!("Modrinth versions JSON: {e}"))?;
    Ok(raw.into_iter().map(map_api_version).collect())
}

/// Resolve the best compatible version and install its primary file into the instance.
/// Vanilla instances skip the loader filter (resource/data/shader packs still match by MC).
pub async fn install_to_instance(
    instance_id: String,
    project_id: String,
    loader: String,
    game_version: String,
    include_optional_dependencies: bool,
) -> Result<crate::instances::ContentInfo, String> {
    crate::instances::validate_instance_id(&instance_id)?;
    let loader = loader.trim().to_ascii_lowercase();
    let loader_filter = if loader.is_empty() || loader == "vanilla" {
        None
    } else {
        Some(loader.clone())
    };
    let mc = if game_version.trim().is_empty() {
        None
    } else {
        Some(game_version.trim().to_string())
    };

    // Try loader + MC first; fall back to MC-only so resource/data packs still resolve.
    let mut versions =
        list_project_versions(project_id.clone(), loader_filter.clone(), mc.clone()).await?;
    if versions.is_empty() && loader_filter.is_some() {
        versions = list_project_versions(project_id.clone(), None, mc.clone()).await?;
    }
    if versions.is_empty() {
        return Err("No compatible version for this loader / Minecraft version".into());
    }

    // Modrinth returns newest first. Use the same dependency-aware path as the
    // explicit version picker instead of silently skipping required projects.
    let version = versions.into_iter().next().ok_or("No version returned")?;
    install_version_inner(
        instance_id,
        version.id,
        true,
        include_optional_dependencies,
        0,
        &mut HashSet::new(),
        loader_filter,
        mc,
    )
    .await?
    .ok_or_else(|| "Selected version was already resolved as a dependency".to_string())
}

/// Install a specific Modrinth version and its selected dependency set.
/// Dependency planning follows Theseus' fail-fast behavior: a required project
/// must finish successfully before the requested file is written.
pub async fn install_version_to_instance(
    instance_id: String,
    version_id: String,
    with_dependencies: bool,
    include_optional_dependencies: bool,
) -> Result<crate::instances::ContentInfo, String> {
    crate::instances::validate_instance_id(&instance_id)?;
    let (prefer_loader, prefer_mc) = instance_loader_mc(&instance_id);
    let mut visited = HashSet::new();
    install_version_inner(
        instance_id,
        version_id,
        with_dependencies,
        include_optional_dependencies,
        0,
        &mut visited,
        prefer_loader,
        prefer_mc,
    )
    .await?
    .ok_or_else(|| "Selected version was already resolved as a dependency".to_string())
}

fn instance_loader_mc(id: &str) -> (Option<String>, Option<String>) {
    let Ok(list) = crate::instances::list_instances() else {
        return (None, None);
    };
    list.into_iter()
        .find(|i| i.id == id)
        .map(|i| {
            let loader = i.loader.trim().to_ascii_lowercase();
            let loader = if loader.is_empty() || loader == "vanilla" {
                None
            } else {
                Some(loader)
            };
            let mc = i.minecraft.trim();
            let mc = if mc.is_empty() {
                None
            } else {
                Some(mc.to_string())
            };
            (loader, mc)
        })
        .unwrap_or((None, None))
}

fn resolve_dep_filters(
    prefer_loader: Option<String>,
    prefer_mc: Option<String>,
    version_loaders: &[String],
    version_game_versions: &[String],
) -> (Option<String>, Option<String>) {
    (
        prefer_loader.or_else(|| version_loaders.first().cloned()),
        prefer_mc.or_else(|| version_game_versions.first().cloned()),
    )
}

pub async fn fetch_version_public(version_id: &str) -> Result<ModrinthVersion, String> {
    fetch_version(version_id).await
}

pub fn assert_download_url_public(url: &str) -> Result<(), String> {
    assert_modrinth_download_url(url)
}

pub async fn download_bytes_public(url: &str, max_bytes: u64) -> Result<Vec<u8>, String> {
    download_capped(url, max_bytes).await
}

async fn fetch_version(version_id: &str) -> Result<ModrinthVersion, String> {
    let id = sanitize_project_id(version_id)?;
    let url = format!("{MODRINTH_API}/version/{id}");
    assert_modrinth_url(&url)?;
    let http = http_client()?;
    let res = http
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Modrinth version: {e}"))?
        .error_for_status()
        .map_err(|e| format!("Modrinth HTTP: {e}"))?;
    assert_modrinth_url(res.url().as_str())?;
    let v: ApiVersion = res
        .json()
        .await
        .map_err(|e| format!("Modrinth version JSON: {e}"))?;
    Ok(map_api_version(v))
}

async fn install_version_inner(
    instance_id: String,
    version_id: String,
    with_dependencies: bool,
    include_optional_dependencies: bool,
    depth: u8,
    visited: &mut HashSet<String>,
    prefer_loader: Option<String>,
    prefer_mc: Option<String>,
) -> Result<Option<crate::instances::ContentInfo>, String> {
    if depth > 5 {
        return Err("Dependency tree too deep".into());
    }
    if !visited.insert(version_id.clone()) {
        return Ok(None);
    }
    let version = fetch_version(&version_id).await?;
    if with_dependencies {
        for dep in &version.dependencies {
            let required = dep.dependency_type.eq_ignore_ascii_case("required");
            let optional = dep.dependency_type.eq_ignore_ascii_case("optional");
            if !required && !(optional && include_optional_dependencies) {
                continue;
            }
            let dep_label = dep
                .project_id
                .as_deref()
                .or(dep.version_id.as_deref())
                .unwrap_or("unknown dependency");
            let result = if let Some(vid) = dep.version_id.as_deref().filter(|s| !s.is_empty()) {
                Box::pin(install_version_inner(
                    instance_id.clone(),
                    vid.to_string(),
                    true,
                    include_optional_dependencies,
                    depth + 1,
                    visited,
                    prefer_loader.clone(),
                    prefer_mc.clone(),
                ))
                .await
            } else if let Some(pid) = dep.project_id.as_deref().filter(|s| !s.is_empty()) {
                let (loader, mc) = resolve_dep_filters(
                    prefer_loader.clone(),
                    prefer_mc.clone(),
                    &version.loaders,
                    &version.game_versions,
                );
                let versions = list_project_versions(pid.to_string(), loader, mc).await?;
                if let Some(first) = versions.into_iter().next() {
                    Box::pin(install_version_inner(
                        instance_id.clone(),
                        first.id,
                        true,
                        include_optional_dependencies,
                        depth + 1,
                        visited,
                        prefer_loader.clone(),
                        prefer_mc.clone(),
                    ))
                    .await
                } else {
                    Err(format!("No compatible version for dependency {pid}"))
                }
            } else if required {
                Err("Required dependency has no project or version id".into())
            } else {
                Ok(None)
            };
            if let Err(err) = result {
                return Err(format!(
                    "{} dependency {dep_label} failed: {err}",
                    if required { "Required" } else { "Optional" }
                ));
            }
        }
    }

    let file = version
        .files
        .iter()
        .find(|f| f.primary)
        .cloned()
        .or_else(|| version.files.first().cloned())
        .ok_or("Selected version has no downloadable file")?;
    let existing =
        crate::instances::find_content_by_file_name(&instance_id, &file.filename)?;
    if let Some(ref existing) = existing {
        if let Some(on_disk) =
            crate::instances::read_content_file(&instance_id, &existing.kind, &existing.file_name)?
        {
            if existing_file_is_same_version(&on_disk, file.sha512.as_deref()) {
                if let Some(pid) = version.project_id.as_deref().filter(|s| !s.is_empty()) {
                    crate::instances::record_modrinth_project_id(
                        &instance_id,
                        &existing.file_name,
                        pid,
                    )?;
                }
                return Ok(Some(existing.clone()));
            }
        }
    }
    assert_modrinth_download_url(&file.url)?;
    let bytes = download_capped(&file.url, MAX_FILE_BYTES).await?;
    if let Some(expected) = file.sha512.as_deref() {
        verify_sha512(&bytes, expected)?;
    }
    let replace = existing.is_some();
    let project_id_for_index = version.project_id.clone();
    let instance_id_for_index = instance_id.clone();
    let file_name_for_index = file.filename.clone();
    let installed = tauri::async_runtime::spawn_blocking(move || {
        if replace {
            crate::instances::replace_content_from_bytes(&instance_id, &file.filename, bytes, None)
        } else {
            crate::instances::add_content_from_bytes(&instance_id, &file.filename, bytes, None)
        }
    })
    .await
    .map_err(|e| format!("install join: {e}"))??;
    if let Some(pid) = project_id_for_index.as_deref().filter(|s| !s.is_empty()) {
        crate::instances::record_modrinth_project_id(
            &instance_id_for_index,
            &installed.file_name,
            pid,
        )?;
    } else if !file_name_for_index.is_empty() {
        // No project id on version payload — skip index (hide-installed stays accurate for recorded files only).
    }
    Ok(Some(installed))
}

/// Skip a re-download only when the on-disk bytes match the Modrinth SHA-512.
/// Same filename + different hash (typical upgrade) replaces the old jar.
/// No hash from the API → do not skip; we cannot prove it is the same version.
fn existing_file_is_same_version(existing_bytes: &[u8], expected_sha512: Option<&str>) -> bool {
    match expected_sha512 {
        Some(expected) => verify_sha512(existing_bytes, expected).is_ok(),
        None => false,
    }
}

fn sanitize_project_id(raw: &str) -> Result<String, String> {
    let id = raw.trim();
    if id.is_empty() || id.len() > 64 {
        return Err("Invalid Modrinth project id".into());
    }
    if !id
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_'))
    {
        return Err("Invalid Modrinth project id characters".into());
    }
    Ok(id.to_string())
}

/// Modrinth serves version files from the CDN; only allow modrinth hosts over https.
fn assert_modrinth_download_url(raw: &str) -> Result<(), String> {
    assert_modrinth_url(raw).map_err(|err| err.replace("API", "downloads"))
}

async fn download_capped(url: &str, max_bytes: u64) -> Result<Vec<u8>, String> {
    use futures_util::StreamExt;
    assert_modrinth_download_url(url)?;
    let http = http_client()?;
    let res = http
        .get(url)
        .send()
        .await
        .map_err(|e| format!("Download: {e}"))?
        .error_for_status()
        .map_err(|e| format!("Download HTTP: {e}"))?;
    assert_modrinth_download_url(res.url().as_str())?;
    if let Some(len) = res.content_length() {
        if len > max_bytes {
            return Err(format!("File too large ({len} bytes)"));
        }
    }
    let mut out = Vec::new();
    let mut stream = res.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| format!("Download stream: {e}"))?;
        if out.len() as u64 + chunk.len() as u64 > max_bytes {
            return Err(format!("File exceeded {max_bytes} bytes"));
        }
        out.extend_from_slice(&chunk);
    }
    Ok(out)
}

fn verify_sha512(bytes: &[u8], expected: &str) -> Result<(), String> {
    use sha2::{Digest, Sha512};
    let actual: String = Sha512::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    if actual.eq_ignore_ascii_case(expected) {
        Ok(())
    } else {
        Err("Downloaded file failed SHA-512 verification".into())
    }
}

/// Minimal URL-encode for query strings (alphanumeric + a few safe chars pass through).
fn urlencoding_lite(s: &str) -> String {
    let mut out = String::with_capacity(s.len() * 3);
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char);
            }
            b' ' => out.push('+'),
            _ => {
                out.push('%');
                out.push(char::from_digit((b >> 4) as u32, 16).unwrap());
                out.push(char::from_digit((b & 0xf) as u32, 16).unwrap());
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn modrinth_urls_require_https_and_allowed_hosts() {
        assert!(assert_modrinth_url("https://api.modrinth.com/v2/search").is_ok());
        assert!(assert_modrinth_download_url(
            "https://cdn.modrinth.com/data/project/versions/file.jar"
        )
        .is_ok());
        assert!(assert_modrinth_url("http://api.modrinth.com/v2/search").is_err());
        assert!(assert_modrinth_url("https://modrinth.com.evil.test/file").is_err());
        assert!(assert_modrinth_url("https://user:pass@api.modrinth.com/v2").is_err());
    }

    #[test]
    fn sha512_verification_rejects_changed_content() {
        let expected = "ddaf35a193617abacc417349ae20413112e6fa4e89a97ea20a9eeee64b55d39a\
                        2192992a274fc1a836ba3c23a3feebbd454d4423643ce80e2a9ac94fa54ca49f";
        assert!(verify_sha512(b"abc", &expected.replace(' ', "")).is_ok());
        assert!(verify_sha512(b"abd", &expected.replace(' ', "")).is_err());
    }

    #[test]
    fn dep_filters_prefer_instance_over_version_first() {
        let (loader, mc) = resolve_dep_filters(
            Some("fabric".into()),
            Some("1.20.1".into()),
            &["neoforge".into()],
            &["1.21.1".into(), "1.20.1".into()],
        );
        assert_eq!(loader.as_deref(), Some("fabric"));
        assert_eq!(mc.as_deref(), Some("1.20.1"));
    }

    #[test]
    fn same_filename_skips_only_when_sha512_matches() {
        let expected = "ddaf35a193617abacc417349ae20413112e6fa4e89a97ea20a9eeee64b55d39a\
                        2192992a274fc1a836ba3c23a3feebbd454d4423643ce80e2a9ac94fa54ca49f";
        assert!(existing_file_is_same_version(b"abc", Some(&expected.replace(' ', ""))));
        assert!(!existing_file_is_same_version(b"abd", Some(&expected.replace(' ', ""))));
        assert!(!existing_file_is_same_version(b"abc", None));
    }
}

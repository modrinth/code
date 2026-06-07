use crate::api::Result;
#[cfg(target_os = "macos")]
use std::hash::{DefaultHasher, Hash, Hasher};
use std::path::{Path, PathBuf};
#[cfg(target_os = "windows")]
use std::process::Command;
use tauri::Runtime;

pub fn init<R: Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri::plugin::Builder::new("shortcuts")
        .invoke_handler(tauri::generate_handler![create_profile_shortcut])
        .build()
}

#[tauri::command]
pub async fn create_profile_shortcut(
    profile_name: String,
    profile_path: String,
    output_path: PathBuf,
) -> Result<PathBuf> {
    let launch_url = format!(
        "modrinth://launch/profile/{}",
        urlencoding::encode(&profile_path)
    );
    let output_path = shortcut_path_with_extension(output_path);
    let output_path_existed =
        tokio::fs::try_exists(&output_path).await.unwrap_or(false);

    if let Err(error) =
        create_shortcut(&profile_name, &launch_url, &output_path).await
    {
        cleanup_shortcut_artifact(&output_path, output_path_existed).await;
        return Err(error);
    }

    Ok(output_path)
}

#[cfg(target_os = "macos")]
async fn create_shortcut(
    profile_name: &str,
    launch_url: &str,
    output_path: &Path,
) -> Result<()> {
    let contents_dir = output_path.join("Contents");
    let macos_dir = contents_dir.join("MacOS");
    let resources_dir = contents_dir.join("Resources");
    tokio::fs::create_dir_all(&macos_dir).await?;
    tokio::fs::create_dir_all(&resources_dir).await?;

    let executable_path = macos_dir.join("launch");
    let target_path = std::env::current_exe()?;
    tokio::fs::write(
        &executable_path,
        format!(
            "#!/bin/sh\nexec {} {}\n",
            shell_quote(&target_path.to_string_lossy()),
            shell_quote(launch_url),
        ),
    )
    .await?;

    tokio::fs::write(
        resources_dir.join("icon.icns"),
        include_bytes!("../../icons/icon.icns"),
    )
    .await?;

    tokio::fs::write(
        contents_dir.join("Info.plist"),
        format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
            <!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \
            \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n\
            <plist version=\"1.0\">\n\
            <dict>\n\
            \t<key>CFBundleExecutable</key>\n\
            \t<string>launch</string>\n\
            \t<key>CFBundleIdentifier</key>\n\
            \t<string>{}</string>\n\
            \t<key>CFBundleIconFile</key>\n\
            \t<string>icon.icns</string>\n\
            \t<key>CFBundleName</key>\n\
            \t<string>{}</string>\n\
            \t<key>CFBundlePackageType</key>\n\
            \t<string>APPL</string>\n\
            </dict>\n\
            </plist>\n",
            macos_shortcut_identifier(launch_url),
            escape_xml(&format!("Launch {profile_name}")),
        ),
    )
    .await?;

    use std::os::unix::fs::PermissionsExt;

    let mut permissions =
        tokio::fs::metadata(&executable_path).await?.permissions();
    permissions.set_mode(0o755);
    tokio::fs::set_permissions(&executable_path, permissions).await?;

    Ok(())
}

#[cfg(target_os = "macos")]
fn macos_shortcut_identifier(launch_url: &str) -> String {
    let mut hasher = DefaultHasher::new();
    launch_url.hash(&mut hasher);

    format!("com.modrinth.instance-shortcut.{:x}", hasher.finish())
}

#[cfg(target_os = "windows")]
async fn create_shortcut(
    _profile_name: &str,
    launch_url: &str,
    output_path: &Path,
) -> Result<()> {
    let target_path = std::env::current_exe()?;
    let working_dir = target_path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_default();
    let script = r#"
$WshShell = New-Object -ComObject WScript.Shell
$Shortcut = $WshShell.CreateShortcut($env:MODRINTH_SHORTCUT_PATH)
$Shortcut.TargetPath = $env:MODRINTH_TARGET_PATH
$Shortcut.Arguments = $env:MODRINTH_SHORTCUT_ARGUMENTS
$Shortcut.WorkingDirectory = $env:MODRINTH_WORKING_DIRECTORY
$Shortcut.IconLocation = $env:MODRINTH_TARGET_PATH
$Shortcut.Save()
"#;

    let status = Command::new("powershell")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            script,
        ])
        .env("MODRINTH_SHORTCUT_PATH", output_path)
        .env("MODRINTH_TARGET_PATH", &target_path)
        .env("MODRINTH_SHORTCUT_ARGUMENTS", launch_url)
        .env("MODRINTH_WORKING_DIRECTORY", working_dir)
        .status()?;

    if !status.success() {
        return Err(std::io::Error::other(format!(
            "failed to create shortcut with exit status {status}"
        ))
        .into());
    }

    Ok(())
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
async fn create_shortcut(
    profile_name: &str,
    launch_url: &str,
    output_path: &Path,
) -> Result<()> {
    let target_path = std::env::current_exe()?;
    tokio::fs::write(
        output_path,
        format!(
            "[Desktop Entry]\n\
            Type=Application\n\
            Name={}\n\
            Exec={} {}\n\
            Icon=ModrinthApp\n\
            Terminal=false\n\
            Categories=Game;\n",
            escape_desktop_entry_value(&format!("Launch {profile_name}")),
            quote_desktop_exec_arg(&target_path.to_string_lossy()),
            quote_desktop_exec_arg(launch_url),
        ),
    )
    .await?;

    use std::os::unix::fs::PermissionsExt;

    let mut permissions = tokio::fs::metadata(output_path).await?.permissions();
    permissions.set_mode(0o755);
    tokio::fs::set_permissions(output_path, permissions).await?;

    Ok(())
}

fn shortcut_path_with_extension(mut path: PathBuf) -> PathBuf {
    let extension = shortcut_extension();

    if path
        .extension()
        .is_none_or(|current_extension| current_extension != extension)
    {
        path.set_extension(extension);
    }

    path
}

async fn cleanup_shortcut_artifact(path: &Path, existed: bool) {
    if existed {
        return;
    }

    let result = match tokio::fs::metadata(path).await {
        Ok(metadata) if metadata.is_dir() => {
            tokio::fs::remove_dir_all(path).await
        }
        _ => tokio::fs::remove_file(path).await,
    };

    if let Err(error) = result
        && error.kind() != std::io::ErrorKind::NotFound
    {
        tracing::warn!(
            "failed to clean up shortcut artifact {}: {}",
            path.display(),
            error
        );
    }
}

fn shortcut_extension() -> &'static str {
    #[cfg(target_os = "windows")]
    {
        "lnk"
    }

    #[cfg(target_os = "macos")]
    {
        "app"
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        "desktop"
    }
}

#[cfg(target_os = "macos")]
fn shell_quote(input: &str) -> String {
    format!("'{}'", input.replace('\'', "'\\''"))
}

#[cfg(target_os = "macos")]
fn escape_xml(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
fn escape_desktop_entry_value(input: &str) -> String {
    input
        .replace('\\', "\\\\")
        .replace('\n', "\\n")
        .replace('\r', "")
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
fn quote_desktop_exec_arg(input: &str) -> String {
    format!(
        "\"{}\"",
        input
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('$', "\\$")
            .replace('`', "\\`")
    )
}
